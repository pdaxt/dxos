use std::path::PathBuf;

use dxos_core::{ContentBlock, ConversationMessage, DxosError, Result, Session, TokenUsage};

use crate::compact::{should_compact, compact_session, CompactionConfig};
use crate::permissions::{PermissionOutcome, PermissionPolicy, PermissionPrompter};

#[derive(Debug, Clone)]
pub struct ApiRequest {
    pub system_prompt: Vec<String>,
    pub messages: Vec<ConversationMessage>,
    pub tools: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantEvent {
    TextDelta(String),
    ToolUse { id: String, name: String, input: String },
    Usage(TokenUsage),
    Stop,
}

pub trait ApiClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>>;
}

/// Runtime events emitted during execution for UI display.
pub enum RuntimeEvent<'a> {
    /// LLM is being called (show spinner)
    Thinking,
    /// Text output from the model
    Text(&'a str),
    /// Tool is about to be called
    ToolCall { name: &'a str, input: &'a str },
    /// Tool finished
    ToolResult { name: &'a str, success: bool },
    /// Turn complete
    Done,
}

/// Callback for runtime events — the CLI implements this to render the UI.
pub trait RuntimeListener {
    fn on_event(&mut self, event: RuntimeEvent<'_>);
}

/// No-op listener for when no UI is needed.
pub struct SilentListener;
impl RuntimeListener for SilentListener {
    fn on_event(&mut self, _event: RuntimeEvent<'_>) {}
}

#[derive(Debug, Clone)]
pub struct TurnSummary {
    pub text: String,
    pub tool_calls: usize,
    pub iterations: usize,
    pub usage: TokenUsage,
}

pub struct ConversationRuntime<C: ApiClient> {
    session: Session,
    api_client: C,
    permission_policy: PermissionPolicy,
    system_prompt: Vec<String>,
    tools: Vec<serde_json::Value>,
    cwd: PathBuf,
    max_iterations: usize,
    cumulative_usage: TokenUsage,
    compaction_config: CompactionConfig,
}

impl<C: ApiClient> ConversationRuntime<C> {
    pub fn new(
        api_client: C,
        permission_policy: PermissionPolicy,
        system_prompt: Vec<String>,
        tools: Vec<serde_json::Value>,
        cwd: PathBuf,
    ) -> Self {
        Self {
            session: Session::new(),
            api_client,
            permission_policy,
            system_prompt,
            tools,
            cwd,
            max_iterations: 16,
            cumulative_usage: TokenUsage::default(),
            compaction_config: CompactionConfig::default(),
        }
    }

    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }

    /// Run a turn without UI events (backwards compatible).
    pub fn run_turn(
        &mut self,
        user_input: impl Into<String>,
        _prompter: Option<&mut dyn PermissionPrompter>,
    ) -> Result<TurnSummary> {
        self.run_turn_with_listener(user_input, &mut SilentListener)
    }

    /// Run a turn with a listener for UI events.
    pub fn run_turn_with_listener(
        &mut self,
        user_input: impl Into<String>,
        listener: &mut dyn RuntimeListener,
    ) -> Result<TurnSummary> {
        self.session
            .messages
            .push(ConversationMessage::user(user_input));

        // Compact if needed
        if should_compact(&self.session, &self.compaction_config) {
            compact_session(&mut self.session, &self.compaction_config);
        }

        let mut text_output = String::new();
        let mut tool_calls = 0;
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(DxosError::TurnLimitExceeded { iterations });
            }

            // Notify UI: thinking
            listener.on_event(RuntimeEvent::Thinking);

            let request = ApiRequest {
                system_prompt: self.system_prompt.clone(),
                messages: self.session.messages.clone(),
                tools: self.tools.clone(),
            };

            let events = self.api_client.stream(request)?;

            // Build assistant message from events
            let mut blocks = Vec::new();
            let mut turn_usage = None;
            let mut pending_tools = Vec::new();

            for event in events {
                match event {
                    AssistantEvent::TextDelta(text) => {
                        listener.on_event(RuntimeEvent::Text(&text));
                        text_output.push_str(&text);
                        blocks.push(ContentBlock::Text { text });
                    }
                    AssistantEvent::ToolUse { id, name, input } => {
                        tool_calls += 1;
                        listener.on_event(RuntimeEvent::ToolCall {
                            name: &name,
                            input: &input,
                        });
                        pending_tools.push((id.clone(), name.clone(), input.clone()));
                        blocks.push(ContentBlock::ToolUse { id, name, input });
                    }
                    AssistantEvent::Usage(usage) => {
                        self.cumulative_usage.accumulate(&usage);
                        turn_usage = Some(usage);
                    }
                    AssistantEvent::Stop => {}
                }
            }

            let mut assistant_msg = ConversationMessage::assistant(blocks);
            assistant_msg.usage = turn_usage;
            self.session.messages.push(assistant_msg);

            // If no tool calls, we're done
            if pending_tools.is_empty() {
                break;
            }

            // Execute tools
            for (tool_id, tool_name, input) in pending_tools {
                let outcome = self.permission_policy.authorize(&tool_name, &input, None);

                let result_msg = match outcome {
                    PermissionOutcome::Allow => {
                        match dxos_tools::execute_tool(&tool_name, &input, &self.cwd) {
                            Ok(output) => {
                                listener.on_event(RuntimeEvent::ToolResult {
                                    name: &tool_name,
                                    success: true,
                                });
                                ConversationMessage::tool_result(tool_id, tool_name, output, false)
                            }
                            Err(e) => {
                                listener.on_event(RuntimeEvent::ToolResult {
                                    name: &tool_name,
                                    success: false,
                                });
                                ConversationMessage::tool_result(
                                    tool_id,
                                    tool_name,
                                    e.to_string(),
                                    true,
                                )
                            }
                        }
                    }
                    PermissionOutcome::Deny { reason } => {
                        listener.on_event(RuntimeEvent::ToolResult {
                            name: &tool_name,
                            success: false,
                        });
                        ConversationMessage::tool_result(tool_id, tool_name, reason, true)
                    }
                };
                self.session.messages.push(result_msg);
            }
        }

        listener.on_event(RuntimeEvent::Done);

        Ok(TurnSummary {
            text: text_output,
            tool_calls,
            iterations,
            usage: self.cumulative_usage,
        })
    }
}

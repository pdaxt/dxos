use dxos_core::{Result, TokenUsage};
use dxos_harness::{ApiClient, ApiRequest, AssistantEvent};
use serde_json::{json, Value};

/// Extract a tool call from text output.
/// Many local models output tool calls as JSON in their response text instead
/// of using the structured tool_calls field. This handles that pattern.
fn extract_tool_call_from_text(text: &str) -> Option<AssistantEvent> {
    // Strip markdown code fences if present
    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Try parsing as JSON with "name" and "arguments" fields
    if let Ok(parsed) = serde_json::from_str::<Value>(cleaned) {
        if let (Some(name), Some(args)) = (
            parsed.get("name").and_then(|v| v.as_str()),
            parsed.get("arguments"),
        ) {
            // Validate it's a known tool name
            let known_tools = [
                "read_file", "write_file", "edit_file", "bash",
                "glob", "grep", "git",
                "Read", "Write", "Edit", "Glob", "Grep",
            ];
            if known_tools.iter().any(|t| *t == name) {
                return Some(AssistantEvent::ToolUse {
                    id: format!("call_{}", rand_id()),
                    name: name.to_string(),
                    input: args.to_string(),
                });
            }
        }

        // Also handle {"name": "tool", "parameters": {...}} format
        if let (Some(name), Some(params)) = (
            parsed.get("name").and_then(|v| v.as_str()),
            parsed.get("parameters"),
        ) {
            let known_tools = [
                "read_file", "write_file", "edit_file", "bash",
                "glob", "grep", "git",
            ];
            if known_tools.iter().any(|t| *t == name) {
                return Some(AssistantEvent::ToolUse {
                    id: format!("call_{}", rand_id()),
                    name: name.to_string(),
                    input: params.to_string(),
                });
            }
        }
    }

    // Try to find JSON embedded in surrounding text
    if let Some(start) = cleaned.find('{') {
        if let Some(end) = cleaned.rfind('}') {
            if start < end {
                let json_str = &cleaned[start..=end];
                return extract_tool_call_from_text(json_str);
            }
        }
    }

    None
}

fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}

pub struct OllamaClient {
    model: String,
    base_url: String,
    http: reqwest::blocking::Client,
}

impl OllamaClient {
    pub fn new(model: String, base_url: Option<String>) -> Self {
        Self {
            model,
            base_url: base_url.unwrap_or_else(|| "http://127.0.0.1:11434".to_string()),
            http: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
        }
    }

    fn build_request_body(&self, request: &ApiRequest) -> Value {
        // Ollama uses OpenAI-compatible /v1/chat/completions format
        let mut messages: Vec<Value> = Vec::new();

        // System prompt
        let system_text = request.system_prompt.join("\n\n");
        if !system_text.is_empty() {
            messages.push(json!({
                "role": "system",
                "content": system_text
            }));
        }

        // Conversation messages
        for msg in &request.messages {
            let role = match msg.role {
                dxos_core::MessageRole::User => "user",
                dxos_core::MessageRole::Assistant => "assistant",
                dxos_core::MessageRole::Tool => "tool",
                dxos_core::MessageRole::System => "system",
            };

            // For tool results, use the OpenAI format
            for block in &msg.blocks {
                match block {
                    dxos_core::ContentBlock::Text { text } => {
                        messages.push(json!({
                            "role": role,
                            "content": text
                        }));
                    }
                    dxos_core::ContentBlock::ToolUse { id, name, input } => {
                        let input_val: Value =
                            serde_json::from_str(input).unwrap_or(json!(input));
                        messages.push(json!({
                            "role": "assistant",
                            "content": null,
                            "tool_calls": [{
                                "id": id,
                                "type": "function",
                                "function": {
                                    "name": name,
                                    "arguments": input_val.to_string()
                                }
                            }]
                        }));
                    }
                    dxos_core::ContentBlock::ToolResult {
                        tool_use_id,
                        output,
                        ..
                    } => {
                        messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_use_id,
                            "content": output
                        }));
                    }
                }
            }
        }

        // Convert tools from Anthropic format to OpenAI function format
        let tools: Vec<Value> = request
            .tools
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool["name"],
                        "description": tool["description"],
                        "parameters": tool["input_schema"]
                    }
                })
            })
            .collect();

        let mut body = json!({
            "model": self.model,
            "messages": messages,
        });

        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }

        body
    }
}

impl ApiClient for OllamaClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>> {
        let body = self.build_request_body(&request);

        let response = self
            .http
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e: reqwest::Error| {
                dxos_core::DxosError::Api(format!(
                    "Failed to connect to Ollama at {}. Is it running? (ollama serve): {e}",
                    self.base_url
                ))
            })?;

        let status = response.status();
        let text = response
            .text()
            .map_err(|e: reqwest::Error| dxos_core::DxosError::Api(e.to_string()))?;

        if !status.is_success() {
            return Err(dxos_core::DxosError::Api(format!(
                "Ollama HTTP {status}: {text}"
            )));
        }

        let resp: Value = serde_json::from_str(&text)
            .map_err(|e: serde_json::Error| dxos_core::DxosError::Api(e.to_string()))?;

        let mut events = Vec::new();

        // Parse usage
        if let Some(usage) = resp.get("usage") {
            events.push(AssistantEvent::Usage(TokenUsage {
                input_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                output_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
            }));
        }

        // Parse choices
        let mut tool_call_count = 0u32;
        if let Some(choices) = resp["choices"].as_array() {
            for choice in choices {
                let message = &choice["message"];

                // Structured tool calls (OpenAI format)
                if let Some(tool_calls) = message["tool_calls"].as_array() {
                    for tc in tool_calls {
                        tool_call_count += 1;
                        events.push(AssistantEvent::ToolUse {
                            id: tc["id"]
                                .as_str()
                                .unwrap_or_else(|| "call_1")
                                .to_string(),
                            name: tc["function"]["name"]
                                .as_str()
                                .unwrap_or("")
                                .to_string(),
                            input: tc["function"]["arguments"]
                                .as_str()
                                .unwrap_or("{}")
                                .to_string(),
                        });
                    }
                }

                // Text content — also check for text-based tool calls
                // Many local models output tool calls as JSON in their text
                if let Some(content) = message["content"].as_str() {
                    if !content.is_empty() && tool_call_count == 0 {
                        // Try to extract tool calls from text
                        if let Some(extracted) = extract_tool_call_from_text(content) {
                            tool_call_count += 1;
                            events.push(extracted);
                        } else {
                            events.push(AssistantEvent::TextDelta(content.to_string()));
                        }
                    } else if !content.is_empty() {
                        events.push(AssistantEvent::TextDelta(content.to_string()));
                    }
                }
            }
        }

        events.push(AssistantEvent::Stop);
        Ok(events)
    }
}

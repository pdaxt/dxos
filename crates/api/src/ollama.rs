use dxos_core::{Result, TokenUsage};
use dxos_harness::{ApiClient, ApiRequest, AssistantEvent};
use serde_json::{json, Value};

pub struct OllamaClient {
    model: String,
    base_url: String,
    http: reqwest::blocking::Client,
}

impl OllamaClient {
    pub fn new(model: String, base_url: Option<String>) -> Self {
        Self {
            model,
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            http: reqwest::blocking::Client::new(),
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
        if let Some(choices) = resp["choices"].as_array() {
            for choice in choices {
                let message = &choice["message"];

                // Text content
                if let Some(content) = message["content"].as_str() {
                    if !content.is_empty() {
                        events.push(AssistantEvent::TextDelta(content.to_string()));
                    }
                }

                // Tool calls
                if let Some(tool_calls) = message["tool_calls"].as_array() {
                    for tc in tool_calls {
                        events.push(AssistantEvent::ToolUse {
                            id: tc["id"]
                                .as_str()
                                .unwrap_or("call_1")
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
            }
        }

        events.push(AssistantEvent::Stop);
        Ok(events)
    }
}

use dxos_core::{Result, TokenUsage};
use dxos_harness::{ApiClient, ApiRequest, AssistantEvent};
use serde_json::{json, Value};

pub struct AnthropicClient {
    api_key: String,
    model: String,
    base_url: String,
    http: reqwest::blocking::Client,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            http: reqwest::blocking::Client::new(),
        }
    }

    fn build_request_body(&self, request: &ApiRequest) -> Value {
        let messages: Vec<Value> = request
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    dxos_core::MessageRole::User => "user",
                    dxos_core::MessageRole::Assistant => "assistant",
                    dxos_core::MessageRole::Tool => "user",
                    dxos_core::MessageRole::System => "user",
                };

                let content: Vec<Value> = msg
                    .blocks
                    .iter()
                    .map(|block| match block {
                        dxos_core::ContentBlock::Text { text } => {
                            json!({ "type": "text", "text": text })
                        }
                        dxos_core::ContentBlock::ToolUse { id, name, input } => {
                            let input_val: Value =
                                serde_json::from_str(input).unwrap_or(json!(input));
                            json!({
                                "type": "tool_use",
                                "id": id,
                                "name": name,
                                "input": input_val
                            })
                        }
                        dxos_core::ContentBlock::ToolResult {
                            tool_use_id,
                            output,
                            is_error,
                            ..
                        } => {
                            json!({
                                "type": "tool_result",
                                "tool_use_id": tool_use_id,
                                "content": output,
                                "is_error": is_error
                            })
                        }
                    })
                    .collect();

                json!({ "role": role, "content": content })
            })
            .collect();

        let system_text = request.system_prompt.join("\n\n");

        let mut body = json!({
            "model": self.model,
            "max_tokens": 8192,
            "messages": messages,
        });

        if !system_text.is_empty() {
            body["system"] = json!(system_text);
        }

        if !request.tools.is_empty() {
            body["tools"] = json!(request.tools);
        }

        body
    }
}

impl ApiClient for AnthropicClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>> {
        let body = self.build_request_body(&request);

        let response = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e: reqwest::Error| dxos_core::DxosError::Api(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .map_err(|e: reqwest::Error| dxos_core::DxosError::Api(e.to_string()))?;

        if !status.is_success() {
            return Err(dxos_core::DxosError::Api(format!(
                "HTTP {status}: {text}"
            )));
        }

        let resp: Value =
            serde_json::from_str(&text).map_err(|e: serde_json::Error| dxos_core::DxosError::Api(e.to_string()))?;

        let mut events = Vec::new();

        // Parse usage
        if let Some(usage) = resp.get("usage") {
            events.push(AssistantEvent::Usage(TokenUsage {
                input_tokens: usage["input_tokens"].as_u64().unwrap_or(0) as u32,
                output_tokens: usage["output_tokens"].as_u64().unwrap_or(0) as u32,
                cache_creation_input_tokens: usage["cache_creation_input_tokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
                cache_read_input_tokens: usage["cache_read_input_tokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
            }));
        }

        // Parse content blocks
        if let Some(content) = resp["content"].as_array() {
            for block in content {
                match block["type"].as_str() {
                    Some("text") => {
                        if let Some(text) = block["text"].as_str() {
                            events.push(AssistantEvent::TextDelta(text.to_string()));
                        }
                    }
                    Some("tool_use") => {
                        events.push(AssistantEvent::ToolUse {
                            id: block["id"].as_str().unwrap_or("").to_string(),
                            name: block["name"].as_str().unwrap_or("").to_string(),
                            input: block["input"].to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }

        events.push(AssistantEvent::Stop);
        Ok(events)
    }
}

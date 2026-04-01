use dxos_core::{ModelProvider, ProviderConfig, Result};
use dxos_harness::{ApiClient, ApiRequest, AssistantEvent};

use crate::anthropic::AnthropicClient;

/// Provider-neutral client that dispatches to the correct backend.
pub enum ProviderClient {
    Anthropic(AnthropicClient),
    // OpenAI(OpenAIClient),  — next
    // Local(LocalClient),    — next
}

impl ProviderClient {
    pub fn from_config(config: &ProviderConfig) -> Result<Self> {
        match config.provider {
            ModelProvider::Anthropic => {
                let api_key = config
                    .api_key
                    .clone()
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| {
                        dxos_core::DxosError::Config(
                            "ANTHROPIC_API_KEY not set. Run: export ANTHROPIC_API_KEY=sk-...".into(),
                        )
                    })?;
                Ok(Self::Anthropic(AnthropicClient::new(
                    api_key,
                    config.model.clone(),
                    config.base_url.clone(),
                )))
            }
            _ => Err(dxos_core::DxosError::Config(format!(
                "provider {:?} not yet supported — coming soon",
                config.provider
            ))),
        }
    }
}

impl ApiClient for ProviderClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>> {
        match self {
            Self::Anthropic(client) => client.stream(request),
        }
    }
}

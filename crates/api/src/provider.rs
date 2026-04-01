use std::process::Command;

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
                let api_key = resolve_api_key("ANTHROPIC_API_KEY", &config.api_key)?;
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

/// Resolve an API key from (in order):
/// 1. Explicit config value
/// 2. Environment variable
/// 3. Claude Max subscription (macOS Keychain: "Claude Code-credentials")
/// 4. PQVault (`pqvault get <key>`)
fn resolve_api_key(env_name: &str, config_value: &Option<String>) -> Result<String> {
    // 1. Explicit config
    if let Some(key) = config_value {
        if !key.is_empty() {
            return Ok(key.clone());
        }
    }

    // 2. Environment variable
    if let Ok(key) = std::env::var(env_name) {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    // 3. PQVault CLI
    if let Ok(output) = Command::new("pqvault").args(["get", env_name]).output() {
        if output.status.success() {
            let key = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !key.is_empty() && key.starts_with("sk-") {
                tracing::info!("Loaded {env_name} from PQVault");
                return Ok(key);
            }
        }
    }

    // 4. macOS Keychain (generic password)
    if let Ok(output) = Command::new("security")
        .args(["find-generic-password", "-s", env_name, "-w"])
        .output()
    {
        if output.status.success() {
            let key = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !key.is_empty() && key.starts_with("sk-") {
                tracing::info!("Loaded {env_name} from macOS Keychain");
                return Ok(key);
            }
        }
    }

    Err(dxos_core::DxosError::Config(format!(
        "{env_name} not found. Set it via:\n  \
         1. export {env_name}=sk-...\n  \
         2. pqvault add {env_name} <value>\n  \
         3. security add-generic-password -s {env_name} -a dxos -w <value>"
    )))
}


impl ApiClient for ProviderClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>> {
        match self {
            Self::Anthropic(client) => client.stream(request),
        }
    }
}

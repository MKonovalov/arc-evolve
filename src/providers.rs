//! Provider constants and utilities — known providers, API key env vars, default models.

/// Known provider names for the --provider flag.
pub const KNOWN_PROVIDERS: &[&str] = &[
    "anthropic",
    "openai",
    "google",
    "openrouter",
    "ollama",
    "xai",
    "groq",
    "deepseek",
    "mistral",
    "cerebras",
    "zai",
    "minimax",
    "bedrock",
    "github",
    "nousresearch",
    "opencode-zen",
    "opencode-go",
    "custom",
];

/// Get the environment variable name that holds the API key for a provider.
pub fn provider_api_key_env(provider: &str) -> Option<&'static str> {
    match provider {
        "openai" => Some("OPENAI_API_KEY"),
        "google" => Some("GOOGLE_API_KEY"),
        "groq" => Some("GROQ_API_KEY"),
        "xai" => Some("XAI_API_KEY"),
        "deepseek" => Some("DEEPSEEK_API_KEY"),
        "openrouter" => Some("OPENROUTER_API_KEY"),
        "mistral" => Some("MISTRAL_API_KEY"),
        "cerebras" => Some("CEREBRAS_API_KEY"),
        "zai" => Some("ZAI_API_KEY"),
        "minimax" => Some("MINIMAX_API_KEY"),
        "bedrock" => Some("AWS_ACCESS_KEY_ID"),
        "anthropic" => Some("ANTHROPIC_API_KEY"),
        "github" => Some("GITHUB_TOKEN"),
        "nousresearch" => Some("NOUS_API_KEY"),
        "opencode-zen" => Some("OPENCODE_API_KEY"),
        "opencode-go" => Some("OPENCODE_API_KEY"),
        _ => None,
    }
}

/// Get the `.arc.toml` config key that holds the API key for a provider.
///
/// This lets users store multiple provider keys in a single `.arc.toml`
/// without clobbering each other (e.g. `nous_api_key`, `opencode_api_key`).
/// `parse_model_config` checks this key before falling back to the shared
/// `api_key` field. `None` is returned for keyless providers (ollama, custom)
/// and any provider without a dedicated config key (it then relies on the
/// shared `api_key` or the provider env var).
pub fn provider_config_api_key(provider: &str) -> Option<&'static str> {
    match provider {
        "anthropic" => Some("anthropic_api_key"),
        "openai" => Some("openai_api_key"),
        "google" => Some("google_api_key"),
        "groq" => Some("groq_api_key"),
        "xai" => Some("xai_api_key"),
        "deepseek" => Some("deepseek_api_key"),
        "openrouter" => Some("openrouter_api_key"),
        "mistral" => Some("mistral_api_key"),
        "cerebras" => Some("cerebras_api_key"),
        "zai" => Some("zai_api_key"),
        "minimax" => Some("minimax_api_key"),
        "github" => Some("github_token"),
        "nousresearch" => Some("nous_api_key"),
        "opencode-zen" => Some("opencode_api_key"),
        "opencode-go" => Some("opencode_api_key"),
        _ => None,
    }
}

/// Get well-known model names for a provider (for diagnostic suggestions).
/// Returns a slice of commonly-used model identifiers.
pub fn known_models_for_provider(provider: &str) -> &'static [&'static str] {
    match provider {
        "anthropic" => &[
            "claude-fable-5",
            "claude-opus-4-8",
            "claude-sonnet-5",
            "claude-opus-4-7",
            "claude-opus-4-6",
            "claude-sonnet-4-7",
            "claude-sonnet-4-6",
            "claude-sonnet-4-5",
            "claude-sonnet-4-20250514",
            "claude-haiku-4-5",
            "claude-haiku-4-5-20251001",
        ],
        "openai" => &[
            "gpt-5",
            "gpt-5-mini",
            "gpt-5.5",
            "gpt-5.5-mini",
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4.1",
            "gpt-4.1-mini",
            "gpt-4.1-nano",
            "codex-mini",
            "o3",
            "o3-mini",
            "o4-mini",
            "o4-mini-high",
        ],
        "google" => &[
            "gemini-3.0-pro",
            "gemini-3.0-flash",
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-2.5-flash-lite",
            "gemini-2.0-flash",
        ],
        "groq" => &[
            "llama-4-maverick-17b",
            "llama-4-scout-17b",
            "llama-3.3-70b-versatile",
            "llama-3.1-8b-instant",
            "mixtral-8x7b-32768",
        ],
        "xai" => &["grok-4", "grok-4-mini", "grok-3", "grok-3-mini", "grok-2"],
        "deepseek" => &["deepseek-v4-pro", "deepseek-v4-flash"],
        "mistral" => &[
            "mistral-large-latest",
            "mistral-small-latest",
            "codestral-latest",
        ],
        "cerebras" => &["llama-3.3-70b"],
        "zai" => &["glm-4-plus", "glm-4-air", "glm-4-flash"],
        "minimax" => &[
            "MiniMax-M2.7",
            "MiniMax-M2.7-highspeed",
            "MiniMax-M2.5",
            "MiniMax-M2.5-highspeed",
            "MiniMax-M1",
            "MiniMax-M1-40k",
        ],
        "bedrock" => &[
            "anthropic.claude-opus-4-7",
            "anthropic.claude-sonnet-4-7",
            "anthropic.claude-sonnet-4-6",
            "anthropic.claude-sonnet-4-5-20250929-v1:0",
            "anthropic.claude-sonnet-4-20250514-v1:0",
            "anthropic.claude-haiku-4-5-20251001-v1:0",
            "amazon.nova-pro-v1:0",
            "amazon.nova-lite-v1:0",
        ],
        "ollama" => &["llama3.2", "llama3.1", "codellama", "mistral"],
        "github" => &[
            "openai/gpt-4o",
            "openai/gpt-4o-mini",
            "openai/o3-mini",
            "openai/o4-mini",
            "meta/llama-4-scout-17b-16e-instruct",
            "mistral/mistral-large-latest",
            "deepseek/deepseek-v4-pro",
            "anthropic/claude-sonnet-4-6",
        ],
        "nousresearch" => &[
            "deepseek/deepseek-v4-flash",
            "deepseek/deepseek-v4-pro",
            "qwen/qwen3-coder",
            "stepfun/step-3.7-flash",
            "meta/llama-4-scout",
            "moonshot/kimi-k2",
        ],
        "opencode-zen" => &[
            "deepseek-v4-pro",
            "deepseek-v4-flash",
            "glm-5.2",
            "glm-5.1",
            "kimi-k2.7-code",
            "kimi-k3",
            "minimax-m3",
            "claude-sonnet-4-6",
            "gpt-5",
            "gemini-3-flash",
        ],
        "opencode-go" => &[
            "deepseek-v4-flash",
            "deepseek-v4-pro",
            "hy3",
            "kimi-k2.7-code",
            "kimi-k2.6",
            "glm-5.2",
            "glm-5.1",
            "mimo-v2.5",
            "qwen3.7-plus",
        ],
        _ => &[],
    }
}

/// Get the default model for a given provider.
pub fn default_model_for_provider(provider: &str) -> String {
    match provider {
        "openai" => "gpt-5".into(),
        "google" => "gemini-2.5-flash".into(),
        "openrouter" => "anthropic/claude-sonnet-4-6".into(),
        "ollama" => "llama3.2".into(),
        "xai" => "grok-4".into(),
        "groq" => "llama-3.3-70b-versatile".into(),
        "deepseek" => "deepseek-v4-pro".into(),
        "mistral" => "mistral-large-latest".into(),
        "cerebras" => "llama-3.3-70b".into(),
        "zai" => "glm-4-plus".into(),
        "minimax" => "MiniMax-M2.7".into(),
        "bedrock" => "anthropic.claude-sonnet-4-6".into(),
        "github" => "openai/gpt-4o".into(),
        "nousresearch" => "deepseek/deepseek-v4-flash".into(),
        "opencode-zen" => "deepseek-v4-flash".into(),
        "opencode-go" => "deepseek-v4-flash".into(),
        _ => "claude-opus-4-6".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_providers_has_at_least_10() {
        assert!(
            KNOWN_PROVIDERS.len() >= 10,
            "expected at least 10 providers, got {}",
            KNOWN_PROVIDERS.len()
        );
    }

    #[test]
    fn test_every_provider_has_default_model() {
        for provider in KNOWN_PROVIDERS {
            let model = default_model_for_provider(provider);
            assert!(
                !model.is_empty(),
                "provider '{}' should have a non-empty default model",
                provider
            );
        }
    }

    #[test]
    fn test_every_non_custom_provider_has_known_models() {
        for provider in KNOWN_PROVIDERS {
            if *provider == "custom" || *provider == "openrouter" {
                // custom/openrouter don't have a fixed model list
                continue;
            }
            let models = known_models_for_provider(provider);
            assert!(
                !models.is_empty(),
                "provider '{}' should have at least one known model",
                provider
            );
        }
    }

    #[test]
    fn test_minimax_provider_api_key_env() {
        assert_eq!(provider_api_key_env("minimax"), Some("MINIMAX_API_KEY"));
    }

    #[test]
    fn test_minimax_default_model() {
        assert_eq!(default_model_for_provider("minimax"), "MiniMax-M2.7");
    }

    #[test]
    fn test_minimax_known_models() {
        let models = known_models_for_provider("minimax");
        assert!(!models.is_empty(), "minimax should have known models");
        assert!(models.contains(&"MiniMax-M1"));
        assert!(models.contains(&"MiniMax-M1-40k"));
    }

    #[test]
    fn test_bedrock_in_known_providers() {
        assert!(
            KNOWN_PROVIDERS.contains(&"bedrock"),
            "bedrock should be in KNOWN_PROVIDERS"
        );
    }

    #[test]
    fn test_bedrock_provider_api_key_env() {
        assert_eq!(provider_api_key_env("bedrock"), Some("AWS_ACCESS_KEY_ID"));
    }

    #[test]
    fn test_bedrock_default_model() {
        assert_eq!(
            default_model_for_provider("bedrock"),
            "anthropic.claude-sonnet-4-6"
        );
    }

    #[test]
    fn test_bedrock_known_models() {
        let models = known_models_for_provider("bedrock");
        assert!(!models.is_empty(), "bedrock should have known models");
        assert!(models.contains(&"anthropic.claude-sonnet-4-20250514-v1:0"));
        assert!(models.contains(&"amazon.nova-pro-v1:0"));
    }

    #[test]
    fn test_minimax_in_known_providers() {
        assert!(
            KNOWN_PROVIDERS.contains(&"minimax"),
            "minimax should be in KNOWN_PROVIDERS"
        );
    }

    #[test]
    fn test_openai_known_models_includes_gpt5() {
        let models = known_models_for_provider("openai");
        assert!(models.contains(&"gpt-5"), "openai should include gpt-5");
        assert!(
            models.contains(&"gpt-5-mini"),
            "openai should include gpt-5-mini"
        );
        assert!(models.contains(&"gpt-5.5"), "openai should include gpt-5.5");
        assert!(
            models.contains(&"gpt-5.5-mini"),
            "openai should include gpt-5.5-mini"
        );
    }

    #[test]
    fn test_xai_known_models_includes_grok4() {
        let models = known_models_for_provider("xai");
        assert!(models.contains(&"grok-4"), "xai should include grok-4");
    }

    #[test]
    fn test_google_known_models_includes_flash_lite() {
        let models = known_models_for_provider("google");
        assert!(
            models.contains(&"gemini-2.5-flash-lite"),
            "google should include gemini-2.5-flash-lite"
        );
    }

    #[test]
    fn test_anthropic_known_models_includes_fleet_models() {
        // arcagent 0.9 fleet presets (issue #568)
        let models = known_models_for_provider("anthropic");
        for fleet in ["claude-fable-5", "claude-opus-4-8", "claude-sonnet-5"] {
            assert!(
                models.contains(&fleet),
                "anthropic should include fleet model {fleet}"
            );
        }
    }

    #[test]
    fn test_anthropic_known_models_includes_new_variants() {
        let models = known_models_for_provider("anthropic");
        assert!(
            models.contains(&"claude-opus-4-7"),
            "anthropic should include claude-opus-4-7"
        );
        assert!(
            models.contains(&"claude-sonnet-4-6"),
            "anthropic should include claude-sonnet-4-6"
        );
        assert!(
            models.contains(&"claude-sonnet-4-5"),
            "anthropic should include claude-sonnet-4-5"
        );
        assert!(
            models.contains(&"claude-haiku-4-5"),
            "anthropic should include claude-haiku-4-5"
        );
        // Older models still present
        assert!(
            models.contains(&"claude-opus-4-6"),
            "anthropic should still include claude-opus-4-6"
        );
        assert!(
            models.contains(&"claude-sonnet-4-20250514"),
            "anthropic should still include claude-sonnet-4-20250514"
        );
    }

    #[test]
    fn test_bedrock_known_models_includes_new_variants() {
        let models = known_models_for_provider("bedrock");
        assert!(
            models.contains(&"anthropic.claude-opus-4-7"),
            "bedrock should include anthropic.claude-opus-4-7"
        );
        assert!(
            models.contains(&"anthropic.claude-sonnet-4-6"),
            "bedrock should include anthropic.claude-sonnet-4-6"
        );
    }

    #[test]
    fn test_openrouter_default_uses_sonnet_4_6() {
        assert_eq!(
            default_model_for_provider("openrouter"),
            "anthropic/claude-sonnet-4-6"
        );
    }

    #[test]
    fn test_anthropic_known_models_contains_sonnet_4_20250514() {
        let models = known_models_for_provider("anthropic");
        assert!(
            !models.is_empty(),
            "anthropic should have a non-empty known models list"
        );
        assert!(
            models.contains(&"claude-sonnet-4-20250514"),
            "anthropic known models should contain claude-sonnet-4-20250514"
        );
    }

    #[test]
    fn test_unknown_provider_returns_empty_known_models() {
        let models = known_models_for_provider("nonexistent-provider");
        assert!(
            models.is_empty(),
            "unknown provider should return empty known models list"
        );
    }

    #[test]
    fn test_github_in_known_providers() {
        assert!(
            KNOWN_PROVIDERS.contains(&"github"),
            "github should be in KNOWN_PROVIDERS"
        );
    }

    #[test]
    fn test_github_provider_api_key_env() {
        assert_eq!(provider_api_key_env("github"), Some("GITHUB_TOKEN"));
    }

    #[test]
    fn test_github_default_model() {
        assert_eq!(default_model_for_provider("github"), "openai/gpt-4o");
    }

    #[test]
    fn test_github_known_models() {
        let models = known_models_for_provider("github");
        assert!(!models.is_empty(), "github should have known models");
        assert!(models.contains(&"openai/gpt-4o"));
        assert!(models.contains(&"anthropic/claude-sonnet-4-6"));
    }

    #[test]
    fn test_nousresearch_in_known_providers() {
        assert!(
            KNOWN_PROVIDERS.contains(&"nousresearch"),
            "nousresearch should be in KNOWN_PROVIDERS"
        );
    }

    #[test]
    fn test_nousresearch_provider_api_key_env() {
        assert_eq!(provider_api_key_env("nousresearch"), Some("NOUS_API_KEY"));
    }

    #[test]
    fn test_nousresearch_default_model() {
        assert_eq!(
            default_model_for_provider("nousresearch"),
            "deepseek/deepseek-v4-flash"
        );
    }

    #[test]
    fn test_nousresearch_known_models() {
        let models = known_models_for_provider("nousresearch");
        assert!(!models.is_empty(), "nousresearch should have known models");
        assert!(models.contains(&"deepseek/deepseek-v4-flash"));
        assert!(models.contains(&"qwen/qwen3-coder"));
    }

    #[test]
    fn test_opencode_zen_in_known_providers() {
        assert!(
            KNOWN_PROVIDERS.contains(&"opencode-zen"),
            "opencode-zen should be in KNOWN_PROVIDERS"
        );
    }

    #[test]
    fn test_opencode_go_in_known_providers() {
        assert!(
            KNOWN_PROVIDERS.contains(&"opencode-go"),
            "opencode-go should be in KNOWN_PROVIDERS"
        );
    }

    #[test]
    fn test_opencode_zen_provider_api_key_env() {
        assert_eq!(
            provider_api_key_env("opencode-zen"),
            Some("OPENCODE_API_KEY")
        );
    }

    #[test]
    fn test_opencode_go_provider_api_key_env() {
        assert_eq!(
            provider_api_key_env("opencode-go"),
            Some("OPENCODE_API_KEY")
        );
    }

    #[test]
    fn test_opencode_zen_default_model() {
        assert_eq!(
            default_model_for_provider("opencode-zen"),
            "deepseek-v4-flash"
        );
    }

    #[test]
    fn test_opencode_go_default_model() {
        assert_eq!(
            default_model_for_provider("opencode-go"),
            "deepseek-v4-flash"
        );
    }

    #[test]
    fn test_opencode_zen_known_models() {
        let models = known_models_for_provider("opencode-zen");
        assert!(!models.is_empty(), "opencode-zen should have known models");
        assert!(models.contains(&"deepseek-v4-pro"));
        assert!(models.contains(&"claude-sonnet-4-6"));
    }

    #[test]
    fn test_opencode_go_known_models() {
        let models = known_models_for_provider("opencode-go");
        assert!(!models.is_empty(), "opencode-go should have known models");
        assert!(models.contains(&"hy3"));
        assert!(models.contains(&"deepseek-v4-flash"));
    }

    #[test]
    fn test_provider_config_api_key_nous() {
        assert_eq!(
            provider_config_api_key("nousresearch"),
            Some("nous_api_key")
        );
    }

    #[test]
    fn test_provider_config_api_key_opencode() {
        assert_eq!(
            provider_config_api_key("opencode-zen"),
            Some("opencode_api_key")
        );
        assert_eq!(
            provider_config_api_key("opencode-go"),
            Some("opencode_api_key")
        );
    }

    #[test]
    fn test_provider_config_api_key_anthropic() {
        assert_eq!(
            provider_config_api_key("anthropic"),
            Some("anthropic_api_key")
        );
    }

    #[test]
    fn test_provider_config_api_key_ollama_none() {
        // ollama/custom are keyless -> no dedicated config key
        assert_eq!(provider_config_api_key("ollama"), None);
        assert_eq!(provider_config_api_key("custom"), None);
    }

    #[test]
    fn test_provider_config_api_key_unknown_none() {
        assert_eq!(provider_config_api_key("nonexistent-provider"), None);
    }
}

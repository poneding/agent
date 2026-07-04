use std::sync::Arc;

use agent_core::EffectiveRuntimeConfig;
use agent_model::{
    AnthropicProvider, FakeModelProvider, FakeModelScript, GeminiProvider, LocalProvider,
    ModelCapabilities, ModelProviderKind, ModelProviderRegistry, OpenAiCompatibleProvider,
    RegisteredModelProvider, ReqwestAnthropicTransport, ReqwestGeminiTransport,
    ReqwestLocalTransport, ReqwestOpenAiCompatibleTransport, SecretResolver,
};

use crate::command::CliError;

pub fn build_model_provider_registry(
    effective: &EffectiveRuntimeConfig,
    prompt: &str,
    secret_resolver: Arc<dyn SecretResolver>,
) -> Result<ModelProviderRegistry, CliError> {
    let mut providers = Vec::new();

    for provider in effective
        .config
        .model
        .catalog
        .providers
        .iter()
        .filter(|provider| provider.enabled)
    {
        let registered = match provider.kind {
            ModelProviderKind::Fake => RegisteredModelProvider::new(
                provider.clone(),
                ModelCapabilities::fake_default(),
                Arc::new(FakeModelProvider::new(FakeModelScript::text(
                    &fake_response(prompt),
                ))),
            )
            .map_err(|error| prompt_runtime_error(error.to_string()))?,
            ModelProviderKind::OpenAiCompatible => {
                let Some(secret_ref) = provider.api_key.as_ref() else {
                    return Err(prompt_runtime_error(format!(
                        "model provider {} secret reference is missing",
                        provider.id
                    )));
                };
                secret_resolver.resolve(secret_ref).map_err(|_| {
                    prompt_runtime_error(format!(
                        "model provider {} secret could not be resolved before live invocation",
                        provider.id
                    ))
                })?;
                let transport = Arc::new(ReqwestOpenAiCompatibleTransport::new(Arc::clone(
                    &secret_resolver,
                )));
                let provider_impl = OpenAiCompatibleProvider::new(provider.clone(), transport)
                    .map_err(|error| {
                        prompt_runtime_error(redact_secret_like_text(&error.to_string()))
                    })?;
                RegisteredModelProvider::new(
                    provider.clone(),
                    ModelCapabilities::openai_compatible_default(),
                    Arc::new(provider_impl),
                )
                .map_err(|error| prompt_runtime_error(error.to_string()))?
            }
            ModelProviderKind::Anthropic => {
                let Some(secret_ref) = provider.api_key.as_ref() else {
                    return Err(prompt_runtime_error(format!(
                        "model provider {} secret reference is missing",
                        provider.id
                    )));
                };
                secret_resolver.resolve(secret_ref).map_err(|_| {
                    prompt_runtime_error(format!(
                        "model provider {} secret could not be resolved before live invocation",
                        provider.id
                    ))
                })?;
                let transport =
                    Arc::new(ReqwestAnthropicTransport::new(Arc::clone(&secret_resolver)));
                let provider_impl =
                    AnthropicProvider::new(provider.clone(), transport).map_err(|error| {
                        prompt_runtime_error(redact_secret_like_text(&error.to_string()))
                    })?;
                RegisteredModelProvider::new(
                    provider.clone(),
                    ModelCapabilities::anthropic_default(),
                    Arc::new(provider_impl),
                )
                .map_err(|error| prompt_runtime_error(error.to_string()))?
            }
            ModelProviderKind::Gemini => {
                let Some(secret_ref) = provider.api_key.as_ref() else {
                    return Err(prompt_runtime_error(format!(
                        "model provider {} secret reference is missing",
                        provider.id
                    )));
                };
                secret_resolver.resolve(secret_ref).map_err(|_| {
                    prompt_runtime_error(format!(
                        "model provider {} secret could not be resolved before live invocation",
                        provider.id
                    ))
                })?;
                let transport = Arc::new(ReqwestGeminiTransport::new(Arc::clone(&secret_resolver)));
                let provider_impl =
                    GeminiProvider::new(provider.clone(), transport).map_err(|error| {
                        prompt_runtime_error(redact_secret_like_text(&error.to_string()))
                    })?;
                RegisteredModelProvider::new(
                    provider.clone(),
                    ModelCapabilities::gemini_default(),
                    Arc::new(provider_impl),
                )
                .map_err(|error| prompt_runtime_error(error.to_string()))?
            }
            ModelProviderKind::Local => {
                let provider_impl =
                    LocalProvider::new(provider.clone(), Arc::new(ReqwestLocalTransport::new()))
                        .map_err(|error| {
                            prompt_runtime_error(redact_secret_like_text(&error.to_string()))
                        })?;
                RegisteredModelProvider::new(
                    provider.clone(),
                    ModelCapabilities::local_default(),
                    Arc::new(provider_impl),
                )
                .map_err(|error| prompt_runtime_error(error.to_string()))?
            }
        };
        providers.push(registered);
    }

    ModelProviderRegistry::new(providers).map_err(|error| prompt_runtime_error(error.to_string()))
}

fn prompt_runtime_error(message: impl Into<String>) -> CliError {
    CliError::PromptRuntimeFailed {
        message: message.into(),
    }
}

fn fake_response(prompt: &str) -> String {
    format!("fake response: {}", redact_secret_like_text(prompt))
}

pub(crate) fn redact_secret_like_text(input: &str) -> String {
    input
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if part.starts_with("sk-")
                || lower.contains("api_key")
                || lower.contains("token")
                || lower.contains("password")
            {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

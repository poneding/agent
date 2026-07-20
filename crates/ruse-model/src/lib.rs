#![forbid(unsafe_code)]
//! Model provider contracts for agent.

pub mod anthropic;
pub mod config;
pub mod fake;
pub mod gemini;
pub mod invocation_policy;
pub mod local;
pub mod openai_compatible;
pub mod provider;
pub mod registry;
pub mod request;
pub mod secrets;
pub mod stream;

pub use anthropic::{
    AnthropicError, AnthropicHttpRequest, AnthropicHttpResponse, AnthropicProvider,
    AnthropicTransport, FixtureAnthropicTransport, ReqwestAnthropicTransport,
    build_anthropic_request_body, parse_anthropic_sse,
};
pub use config::{
    ModelConfigError, ModelEndpointConfig, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, SecretRef,
};
pub use fake::{FakeModelProvider, FakeModelScript, FakeToolCall};
pub use gemini::{
    FixtureGeminiTransport, GeminiError, GeminiHttpRequest, GeminiHttpResponse, GeminiProvider,
    GeminiTransport, ReqwestGeminiTransport, build_gemini_request_body,
    gemini_stream_generate_content_url, parse_gemini_sse,
};
pub use invocation_policy::{
    ModelFailureKind, ModelFallbackChain, ModelInvocationPolicy, ModelInvocationPolicyError,
    ModelRetryPolicy,
};
pub use local::{
    FixtureLocalTransport, LocalError, LocalHttpRequest, LocalHttpResponse, LocalProvider,
    LocalTransport, ReqwestLocalTransport, build_local_request_body, local_chat_url,
    parse_local_jsonl,
};
pub use openai_compatible::{
    FixtureOpenAiCompatibleTransport, OpenAiCompatibleChunkStream, OpenAiCompatibleError,
    OpenAiCompatibleHttpRequest, OpenAiCompatibleHttpResponse, OpenAiCompatibleHttpStreamResponse,
    OpenAiCompatibleProvider, OpenAiCompatibleTransport, ReqwestOpenAiCompatibleTransport,
    build_openai_compatible_chat_request_body, build_openai_compatible_request_body,
    parse_openai_compatible_chat_sse, parse_openai_compatible_sse,
};
pub use provider::{ModelError, ModelProvider};
pub use registry::{
    ModelCapabilities, ModelCapability, ModelCapabilityProfileError, ModelCostProfile,
    ModelProviderRegistry, ModelRegistryError, ModelRequestRequirements, RegisteredModelProvider,
};
pub use request::{ModelMessage, ModelRequest, ModelToolDefinition};
pub use secrets::{
    CompositeSecretResolver, EnvSecretResolver, EnvSecretStore, InMemorySecretResolver,
    InMemorySecretStore, OsKeychainSecretStore, SecretResolutionError, SecretResolver,
    SecretStoreBackend, SecretStoreDiagnostic, SecretStoreError, SecretStoreHealth,
    SecretStoreKind, SecretValue,
};
pub use stream::{
    ModelEventStream, ModelStreamEvent, ModelStreamResult, collect_model_event_stream,
    model_error_to_stream, model_events_to_stream,
};

/// Returns a smoke-test string proving the model crate links shared types.
#[must_use]
pub fn model_layer_name() -> String {
    format!("ruse-model uses {}", ruse_types::crate_name())
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_model_layer_identity() {
        assert_eq!(crate::model_layer_name(), "ruse-model uses ruse-types");
    }
}

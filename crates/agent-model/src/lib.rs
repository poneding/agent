#![forbid(unsafe_code)]
//! Model provider contracts for agent.

pub mod config;
pub mod fake;
pub mod invocation_policy;
pub mod openai_compatible;
pub mod provider;
pub mod registry;
pub mod request;
pub mod secrets;
pub mod stream;

pub use config::{
    ModelConfigError, ModelEndpointConfig, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, SecretRef,
};
pub use fake::{FakeModelProvider, FakeModelScript, FakeToolCall};
pub use invocation_policy::{
    ModelFailureKind, ModelFallbackChain, ModelInvocationPolicy, ModelInvocationPolicyError,
    ModelRetryPolicy,
};
pub use openai_compatible::{
    FixtureOpenAiCompatibleTransport, OpenAiCompatibleError, OpenAiCompatibleHttpRequest,
    OpenAiCompatibleHttpResponse, OpenAiCompatibleProvider, OpenAiCompatibleTransport,
    ReqwestOpenAiCompatibleTransport, build_openai_compatible_request_body,
    parse_openai_compatible_sse,
};
pub use provider::{ModelError, ModelProvider};
pub use registry::{
    ModelCapabilities, ModelCapability, ModelProviderRegistry, ModelRegistryError,
    ModelRequestRequirements, RegisteredModelProvider,
};
pub use request::ModelRequest;
pub use secrets::{
    EnvSecretResolver, EnvSecretStore, InMemorySecretResolver, InMemorySecretStore,
    OsKeychainSecretStore, SecretResolutionError, SecretResolver, SecretStoreBackend,
    SecretStoreDiagnostic, SecretStoreError, SecretStoreHealth, SecretStoreKind, SecretValue,
};
pub use stream::{
    ModelEventStream, ModelStreamEvent, ModelStreamResult, collect_model_event_stream,
    model_error_to_stream, model_events_to_stream,
};

/// Returns a smoke-test string proving the model crate links shared types.
#[must_use]
pub fn model_layer_name() -> String {
    format!("agent-model uses {}", agent_types::crate_name())
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_model_layer_identity() {
        assert_eq!(crate::model_layer_name(), "agent-model uses agent-types");
    }
}

use std::collections::BTreeMap;

use ruse_model::{ModelCapabilities, ModelCapability, ModelProviderKind, ModelRequestRequirements};
use serde::{Deserialize, Serialize};

use crate::config::EffectiveRuntimeConfig;

#[must_use]
pub const fn coding_agent_model_requirements() -> ModelRequestRequirements {
    ModelRequestRequirements {
        streaming: true,
        tool_calls: true,
        structured_outputs: false,
        image_input: false,
        min_context_tokens: Some(1024),
        min_output_tokens: Some(1024),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderSupportReport {
    pub selected_provider: String,
    pub selected_model: String,
    pub requirements: ModelRequestRequirements,
    pub providers: Vec<ProviderSupportSummary>,
}

impl ProviderSupportReport {
    #[must_use]
    pub fn from_effective_config(
        effective: &EffectiveRuntimeConfig,
        requirements: &ModelRequestRequirements,
    ) -> Self {
        let fallback_indexes = effective
            .config
            .model
            .invocation_policy
            .fallback
            .provider_ids
            .iter()
            .enumerate()
            .map(|(index, provider_id)| (provider_id.as_str(), index as u32))
            .collect::<BTreeMap<_, _>>();

        let providers = effective
            .config
            .model
            .catalog
            .providers
            .iter()
            .map(|provider| {
                let fallback_index = fallback_indexes.get(provider.id.as_str()).copied();
                let role = if provider.id == effective.selected_provider {
                    ProviderSupportRole::Selected
                } else if fallback_index.is_some() {
                    ProviderSupportRole::Fallback
                } else {
                    ProviderSupportRole::CatalogOnly
                };
                let capabilities = ModelCapabilities::default_for_provider_kind(provider.kind);
                let missing = capabilities.missing_for(requirements);
                let compatible = provider.enabled && missing.is_empty();
                ProviderSupportSummary {
                    provider_id: provider.id.clone(),
                    kind: provider.kind,
                    role,
                    fallback_index,
                    enabled: provider.enabled,
                    compatible,
                    missing,
                }
            })
            .collect();

        Self {
            selected_provider: effective.selected_provider.clone(),
            selected_model: effective.selected_model.clone(),
            requirements: requirements.clone(),
            providers,
        }
    }

    #[must_use]
    pub fn selected(&self) -> Option<&ProviderSupportSummary> {
        self.providers
            .iter()
            .find(|provider| provider.role == ProviderSupportRole::Selected)
    }

    #[must_use]
    pub fn fallbacks(&self) -> Vec<&ProviderSupportSummary> {
        let mut fallbacks = self
            .providers
            .iter()
            .filter(|provider| provider.fallback_index.is_some())
            .collect::<Vec<_>>();
        fallbacks.sort_by_key(|provider| provider.fallback_index);
        fallbacks
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderSupportSummary {
    pub provider_id: String,
    pub kind: ModelProviderKind,
    pub role: ProviderSupportRole,
    pub fallback_index: Option<u32>,
    pub enabled: bool,
    pub compatible: bool,
    pub missing: Vec<ModelCapability>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSupportRole {
    Selected,
    Fallback,
    CatalogOnly,
}

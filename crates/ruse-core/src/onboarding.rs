use ruse_model::{SecretStoreHealth, SecretStoreKind};
use serde::{Deserialize, Serialize};

use crate::{
    EffectiveRuntimeConfig, PlatformPaths, ProviderSupportReport, ProviderSupportRole,
    SettingsProfile, StartupMigrationDiagnosticSeverity, StartupMigrationOutcome,
};

pub const ONBOARDING_READINESS_SCHEMA_VERSION: u32 = 1;
const MAX_DIAGNOSTICS: usize = 16;
const MAX_NEXT_ACTIONS: usize = 16;
const MAX_TEXT_CHARS: usize = 240;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingReadinessInput {
    pub effective: EffectiveRuntimeConfig,
    pub provider_support: ProviderSupportReport,
    pub settings: SettingsProfile,
    pub platform_paths: PlatformPaths,
    pub secret_store_health: SecretStoreHealth,
    pub secret_env_fallback_available: bool,
    pub migration_outcome: StartupMigrationOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingReadinessSnapshot {
    pub schema_version: u32,
    pub status: OnboardingReadinessStatus,
    pub settings: OnboardingSettingsSummary,
    pub paths: OnboardingPathSummary,
    pub secrets: OnboardingSecretStoreSummary,
    pub migrations: OnboardingMigrationSummary,
    pub providers: OnboardingProviderSummary,
    pub diagnostics: Vec<OnboardingDiagnostic>,
    pub next_actions: Vec<OnboardingNextAction>,
}

impl OnboardingReadinessSnapshot {
    #[must_use]
    pub fn from_input(input: OnboardingReadinessInput) -> Self {
        let settings = OnboardingSettingsSummary::from_settings(&input.settings);
        let paths = OnboardingPathSummary::from_paths(&input.platform_paths);
        let secrets = OnboardingSecretStoreSummary::from_health(
            &input.secret_store_health,
            input.secret_env_fallback_available,
        );
        let migrations = OnboardingMigrationSummary::from_outcome(&input.migration_outcome);
        let providers =
            OnboardingProviderSummary::from_report(&input.effective, &input.provider_support);
        let mut diagnostics = Vec::new();
        let mut next_actions = Vec::new();

        if !secrets.available {
            diagnostics.push(OnboardingDiagnostic::warning(
                "onboarding.secret_store_unavailable",
                "Secret store is unavailable; environment fallback may be used when configured",
                Some("secret_store"),
            ));
            next_actions.push(OnboardingNextAction::new(
                "configure_secret_store",
                OnboardingNextActionTarget::Secrets,
                "Configure keychain access or environment fallback",
            ));
        } else if !secrets.writable {
            diagnostics.push(OnboardingDiagnostic::info(
                "onboarding.secret_store_read_only",
                "Secret store is read-only; use environment references or configure a writable store",
                Some("secret_store"),
            ));
        }

        for diagnostic in input.secret_store_health.diagnostics {
            diagnostics.push(OnboardingDiagnostic::warning(
                diagnostic.code,
                diagnostic.message,
                Some("secret_store"),
            ));
        }

        if !migrations.runtime_allowed {
            diagnostics.push(OnboardingDiagnostic::blocked(
                "onboarding.migration_blocked",
                "Startup migration blocked runtime startup",
                Some("migration"),
            ));
            next_actions.push(OnboardingNextAction::new(
                "review_migration_failure",
                OnboardingNextActionTarget::Migrations,
                "Review startup migration diagnostics before starting the runtime",
            ));
        } else if migrations.diagnostic_count > 0 {
            diagnostics.push(OnboardingDiagnostic::warning(
                "onboarding.migration_diagnostics",
                "Startup migration completed with diagnostics",
                Some("migration"),
            ));
        }

        for diagnostic in &input.migration_outcome.diagnostics {
            let severity = match diagnostic.severity {
                StartupMigrationDiagnosticSeverity::Info => OnboardingDiagnosticSeverity::Info,
                StartupMigrationDiagnosticSeverity::Warning => {
                    OnboardingDiagnosticSeverity::Warning
                }
                StartupMigrationDiagnosticSeverity::Blocked => {
                    OnboardingDiagnosticSeverity::Blocked
                }
            };
            diagnostics.push(OnboardingDiagnostic::new(
                severity,
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                Some("migration"),
            ));
        }

        if !providers.selected_provider_enabled {
            diagnostics.push(OnboardingDiagnostic::blocked(
                "onboarding.provider_disabled",
                "Selected provider is disabled",
                Some("provider"),
            ));
            next_actions.push(OnboardingNextAction::new(
                "enable_selected_provider",
                OnboardingNextActionTarget::Providers,
                "Enable or switch the selected provider",
            ));
        } else if !providers.selected_provider_compatible {
            diagnostics.push(OnboardingDiagnostic::warning(
                "onboarding.provider_capability_gap",
                "Selected provider does not satisfy the default coding-agent requirements",
                Some("provider"),
            ));
            next_actions.push(OnboardingNextAction::new(
                "choose_compatible_model",
                OnboardingNextActionTarget::Providers,
                "Choose a provider/model with streaming and tool-call support",
            ));
        }

        if providers.enabled_provider_count == 0 {
            diagnostics.push(OnboardingDiagnostic::blocked(
                "onboarding.no_enabled_providers",
                "No enabled providers are configured",
                Some("provider"),
            ));
            next_actions.push(OnboardingNextAction::new(
                "configure_provider",
                OnboardingNextActionTarget::Providers,
                "Configure at least one provider and model",
            ));
        }

        diagnostics.truncate(MAX_DIAGNOSTICS);
        next_actions.truncate(MAX_NEXT_ACTIONS);

        let status = readiness_status(&diagnostics, &secrets, &migrations, &providers);

        Self {
            schema_version: ONBOARDING_READINESS_SCHEMA_VERSION,
            status,
            settings,
            paths,
            secrets,
            migrations,
            providers,
            diagnostics,
            next_actions,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingReadinessStatus {
    Ready,
    Degraded,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingSettingsSummary {
    pub profile_id: String,
    pub release_channel: String,
    pub selected_provider: Option<String>,
    pub selected_model: Option<String>,
    pub policy_profile: String,
}

impl OnboardingSettingsSummary {
    fn from_settings(settings: &SettingsProfile) -> Self {
        Self {
            profile_id: bounded_redacted_text(&settings.profile_id),
            release_channel: settings.release_channel.as_str().to_string(),
            selected_provider: settings
                .selected_provider
                .as_ref()
                .map(|value| bounded_redacted_text(value)),
            selected_model: settings
                .selected_model
                .as_ref()
                .map(|value| bounded_redacted_text(value)),
            policy_profile: bounded_redacted_text(&settings.policy_profile),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingPathSummary {
    pub config_dir: String,
    pub data_dir: String,
    pub cache_dir: String,
    pub log_dir: String,
    pub plugin_dir: String,
    pub skill_dir: String,
    pub memory_dir: String,
    pub temp_dir: String,
}

impl OnboardingPathSummary {
    fn from_paths(paths: &PlatformPaths) -> Self {
        Self {
            config_dir: bounded_redacted_text(&paths.config_dir.display().to_string()),
            data_dir: bounded_redacted_text(&paths.data_dir.display().to_string()),
            cache_dir: bounded_redacted_text(&paths.cache_dir.display().to_string()),
            log_dir: bounded_redacted_text(&paths.log_dir.display().to_string()),
            plugin_dir: bounded_redacted_text(&paths.plugin_dir.display().to_string()),
            skill_dir: bounded_redacted_text(&paths.skill_dir.display().to_string()),
            memory_dir: bounded_redacted_text(&paths.memory_dir.display().to_string()),
            temp_dir: bounded_redacted_text(&paths.temp_dir.display().to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingSecretStoreSummary {
    pub kind: SecretStoreKind,
    pub available: bool,
    pub writable: bool,
    pub env_fallback_available: bool,
    pub diagnostic_count: u32,
}

impl OnboardingSecretStoreSummary {
    fn from_health(health: &SecretStoreHealth, env_fallback_available: bool) -> Self {
        Self {
            kind: health.kind,
            available: health.available,
            writable: health.writable,
            env_fallback_available,
            diagnostic_count: health.diagnostics.len().try_into().unwrap_or(u32::MAX),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingMigrationSummary {
    pub runtime_allowed: bool,
    pub domain_count: u32,
    pub pending_step_count: u32,
    pub applied_step_count: u32,
    pub backup_required: bool,
    pub diagnostic_count: u32,
}

impl OnboardingMigrationSummary {
    fn from_outcome(outcome: &StartupMigrationOutcome) -> Self {
        Self {
            runtime_allowed: outcome.runtime_allowed,
            domain_count: outcome
                .plan
                .from_versions
                .len()
                .try_into()
                .unwrap_or(u32::MAX),
            pending_step_count: outcome.plan.steps.len().try_into().unwrap_or(u32::MAX),
            applied_step_count: outcome.applied_steps.len().try_into().unwrap_or(u32::MAX),
            backup_required: outcome.plan.backup_required,
            diagnostic_count: outcome.diagnostics.len().try_into().unwrap_or(u32::MAX),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingProviderSummary {
    pub selected_provider: String,
    pub selected_model: String,
    pub provider_count: u32,
    pub enabled_provider_count: u32,
    pub fallback_provider_count: u32,
    pub selected_provider_enabled: bool,
    pub selected_provider_compatible: bool,
    pub missing_capabilities: Vec<String>,
}

impl OnboardingProviderSummary {
    fn from_report(effective: &EffectiveRuntimeConfig, report: &ProviderSupportReport) -> Self {
        let selected = report.selected();
        Self {
            selected_provider: bounded_redacted_text(&report.selected_provider),
            selected_model: bounded_redacted_text(&report.selected_model),
            provider_count: report.providers.len().try_into().unwrap_or(u32::MAX),
            enabled_provider_count: effective
                .config
                .model
                .catalog
                .providers
                .iter()
                .filter(|provider| provider.enabled)
                .count()
                .try_into()
                .unwrap_or(u32::MAX),
            fallback_provider_count: report
                .providers
                .iter()
                .filter(|provider| provider.role == ProviderSupportRole::Fallback)
                .count()
                .try_into()
                .unwrap_or(u32::MAX),
            selected_provider_enabled: selected.is_some_and(|provider| provider.enabled),
            selected_provider_compatible: selected.is_some_and(|provider| provider.compatible),
            missing_capabilities: selected
                .map(|provider| {
                    provider
                        .missing
                        .iter()
                        .map(|capability| capability.as_str().to_string())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingDiagnostic {
    pub severity: OnboardingDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub surface: Option<String>,
}

impl OnboardingDiagnostic {
    #[must_use]
    pub fn info(
        code: impl Into<String>,
        message: impl Into<String>,
        surface: Option<&str>,
    ) -> Self {
        Self::new(OnboardingDiagnosticSeverity::Info, code, message, surface)
    }

    #[must_use]
    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        surface: Option<&str>,
    ) -> Self {
        Self::new(
            OnboardingDiagnosticSeverity::Warning,
            code,
            message,
            surface,
        )
    }

    #[must_use]
    pub fn blocked(
        code: impl Into<String>,
        message: impl Into<String>,
        surface: Option<&str>,
    ) -> Self {
        Self::new(
            OnboardingDiagnosticSeverity::Blocked,
            code,
            message,
            surface,
        )
    }

    #[must_use]
    pub fn new(
        severity: OnboardingDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        surface: Option<&str>,
    ) -> Self {
        Self {
            severity,
            code: bounded_contract_text(&code.into()),
            message: bounded_redacted_text(&message.into()),
            surface: surface.map(bounded_contract_text),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingDiagnosticSeverity {
    Info,
    Warning,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingNextAction {
    pub id: String,
    pub target: OnboardingNextActionTarget,
    pub label: String,
}

impl OnboardingNextAction {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        target: OnboardingNextActionTarget,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: bounded_contract_text(&id.into()),
            target,
            label: bounded_redacted_text(&label.into()),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingNextActionTarget {
    Settings,
    Secrets,
    Migrations,
    Providers,
}

fn readiness_status(
    diagnostics: &[OnboardingDiagnostic],
    secrets: &OnboardingSecretStoreSummary,
    migrations: &OnboardingMigrationSummary,
    providers: &OnboardingProviderSummary,
) -> OnboardingReadinessStatus {
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == OnboardingDiagnosticSeverity::Blocked)
        || !migrations.runtime_allowed
        || !providers.selected_provider_enabled
        || providers.enabled_provider_count == 0
    {
        return OnboardingReadinessStatus::Blocked;
    }

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == OnboardingDiagnosticSeverity::Warning)
        || !providers.selected_provider_compatible
        || !secrets.available
    {
        return OnboardingReadinessStatus::Degraded;
    }

    OnboardingReadinessStatus::Ready
}

fn bounded_redacted_text(value: &str) -> String {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() <= MAX_TEXT_CHARS {
        return redacted;
    }
    let mut truncated = redacted.chars().take(MAX_TEXT_CHARS).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn bounded_contract_text(value: &str) -> String {
    let value = value.trim();
    if value.chars().count() <= MAX_TEXT_CHARS {
        return value.to_string();
    }
    let mut truncated = value.chars().take(MAX_TEXT_CHARS).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if part.starts_with("sk-")
                || lower.contains("api_key")
                || lower.contains("apikey")
                || lower.contains("token")
                || lower.contains("password")
                || lower.contains("secret")
                || lower.contains("authorization")
                || lower.contains("bearer")
            {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use ruse_model::{ModelProviderKind, SecretStoreDiagnostic};

    use super::*;
    use crate::{
        AgentRuntimeConfig, ConfigLayer, ConfigLayerKind, RuntimeModelConfig, StartupMigrationPlan,
        build_effective_runtime_config, coding_agent_model_requirements,
    };
    use ruse_model::{
        ModelEndpointConfig, ModelFallbackChain, ModelInvocationPolicy, ModelProviderCatalog,
        ModelProviderConfig, ModelRetryPolicy, SecretRef, SecretStoreHealth,
    };

    #[test]
    fn onboarding_snapshot_reports_ready_local_baseline() {
        let input = fixture_input(
            ModelProviderKind::Local,
            true,
            SecretStoreHealth::healthy(SecretStoreKind::Memory, true),
        );
        let snapshot = OnboardingReadinessSnapshot::from_input(input);

        assert_eq!(snapshot.status, OnboardingReadinessStatus::Ready);
        assert_eq!(snapshot.secrets.kind, SecretStoreKind::Memory);
        assert!(snapshot.migrations.runtime_allowed);
        assert_eq!(snapshot.providers.selected_provider, "selected");
        assert!(snapshot.providers.selected_provider_compatible);
        assert!(snapshot.diagnostics.is_empty());
    }

    #[test]
    fn onboarding_snapshot_degrades_for_keychain_fallback_and_provider_gap() {
        let input = fixture_input(
            ModelProviderKind::Fake,
            true,
            SecretStoreHealth::unavailable(
                SecretStoreKind::OsKeychain,
                vec![SecretStoreDiagnostic::recoverable(
                    "secret_store.keychain_unavailable",
                    "OS keychain unavailable; OPENAI_API_KEY and sk-test-secret are not echoed",
                )],
            ),
        );
        let snapshot = OnboardingReadinessSnapshot::from_input(input);
        let json = serde_json::to_string(&snapshot).expect("serialize snapshot");

        assert_eq!(snapshot.status, OnboardingReadinessStatus::Degraded);
        assert!(
            snapshot
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "onboarding.provider_capability_gap" })
        );
        assert!(!json.contains("OPENAI_API_KEY"));
        assert!(!json.contains("sk-test-secret"));
        assert!(json.contains("[redacted]"));
    }

    #[test]
    fn onboarding_snapshot_blocks_when_migration_blocks_runtime() {
        let mut input = fixture_input(
            ModelProviderKind::Local,
            true,
            SecretStoreHealth::healthy(SecretStoreKind::Memory, true),
        );
        input.migration_outcome.runtime_allowed = false;
        input
            .migration_outcome
            .diagnostics
            .push(crate::StartupMigrationDiagnostic::blocked(
                "config",
                "config.v1_to_v2",
                "blocked by token sk-test-secret",
            ));
        let snapshot = OnboardingReadinessSnapshot::from_input(input);
        let json = serde_json::to_string(&snapshot).expect("serialize snapshot");

        assert_eq!(snapshot.status, OnboardingReadinessStatus::Blocked);
        assert!(
            snapshot
                .next_actions
                .iter()
                .any(|action| { action.id == "review_migration_failure" })
        );
        assert!(!json.contains("sk-test-secret"));
    }

    fn fixture_input(
        kind: ModelProviderKind,
        enabled: bool,
        secret_store_health: SecretStoreHealth,
    ) -> OnboardingReadinessInput {
        let endpoint_url = match kind {
            ModelProviderKind::Local => Some("http://localhost:11434"),
            ModelProviderKind::OpenAiCompatible => Some("https://api.example.test/v1"),
            _ => None,
        };
        let endpoint =
            ModelEndpointConfig::new(endpoint_url, 30_000).expect("valid endpoint config");
        let secret_ref = if matches!(
            kind,
            ModelProviderKind::OpenAiCompatible
                | ModelProviderKind::Anthropic
                | ModelProviderKind::Gemini
        ) {
            Some(SecretRef::new("env", "TEST_API_KEY").expect("valid secret ref"))
        } else {
            None
        };
        let selected_provider = ModelProviderConfig::new(
            "selected",
            kind,
            "test-model",
            endpoint,
            secret_ref,
            enabled,
        )
        .expect("valid provider");
        let fallback_provider = ModelProviderConfig::new(
            "fake",
            ModelProviderKind::Fake,
            "fake-scripted",
            ModelEndpointConfig::new(None, 1_000).expect("valid fake endpoint"),
            None,
            true,
        )
        .expect("valid fallback provider");
        let catalog = ModelProviderCatalog::new(vec![selected_provider, fallback_provider])
            .expect("valid catalog");
        let retry = ModelRetryPolicy::new(1, Vec::new()).expect("valid retry policy");
        let fallback = ModelFallbackChain::new(Vec::<String>::new()).expect("valid fallback");
        let invocation_policy =
            ModelInvocationPolicy::new(30_000, retry, fallback).expect("valid invocation policy");
        let config = AgentRuntimeConfig {
            schema_version: crate::AGENT_CONFIG_SCHEMA_VERSION,
            model: RuntimeModelConfig {
                default_provider: "selected".to_string(),
                catalog,
                invocation_policy,
            },
            terminal: Default::default(),
        };
        let effective = build_effective_runtime_config(
            vec![ConfigLayer {
                kind: ConfigLayerKind::BuiltIn,
                source: "fixture".to_string(),
                config,
            }],
            crate::ConfigOverride::default(),
        )
        .expect("effective config");
        let provider_support = ProviderSupportReport::from_effective_config(
            &effective,
            &coding_agent_model_requirements(),
        );
        let settings = SettingsProfile::default();
        let platform_paths = PlatformPaths::for_platform(
            crate::PlatformKind::MacOs,
            std::path::PathBuf::from("/Users/tester"),
            crate::PlatformPathOverrides::default(),
        )
        .expect("platform paths");
        let migration_outcome = StartupMigrationOutcome {
            plan: StartupMigrationPlan {
                from_versions: Vec::new(),
                steps: Vec::new(),
                backup_required: false,
            },
            applied_steps: Vec::new(),
            audit_records: Vec::new(),
            diagnostics: Vec::new(),
            runtime_allowed: true,
        };

        OnboardingReadinessInput {
            effective,
            provider_support,
            settings,
            platform_paths,
            secret_store_health,
            secret_env_fallback_available: true,
            migration_outcome,
        }
    }
}

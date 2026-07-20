use std::env;
use std::fmt;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ConfigDiagnostic, ConfigDiagnosticSeverity};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformKind {
    MacOs,
    Linux,
    Windows,
    Other,
}

impl PlatformKind {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Other
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformPathKey {
    ConfigDir,
    DataDir,
    CacheDir,
    LogDir,
    PluginDir,
    SkillDir,
    MemoryDir,
    TempDir,
}

impl fmt::Display for PlatformPathKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ConfigDir => "config_dir",
            Self::DataDir => "data_dir",
            Self::CacheDir => "cache_dir",
            Self::LogDir => "log_dir",
            Self::PluginDir => "plugin_dir",
            Self::SkillDir => "skill_dir",
            Self::MemoryDir => "memory_dir",
            Self::TempDir => "temp_dir",
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformPathOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portable_root: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_dir: Option<PathBuf>,
}

impl PlatformPathOverrides {
    pub fn set(&mut self, key: PlatformPathKey, path: PathBuf) {
        match key {
            PlatformPathKey::ConfigDir => self.config_dir = Some(path),
            PlatformPathKey::DataDir => self.data_dir = Some(path),
            PlatformPathKey::CacheDir => self.cache_dir = Some(path),
            PlatformPathKey::LogDir => self.log_dir = Some(path),
            PlatformPathKey::PluginDir => self.plugin_dir = Some(path),
            PlatformPathKey::SkillDir => self.skill_dir = Some(path),
            PlatformPathKey::MemoryDir => self.memory_dir = Some(path),
            PlatformPathKey::TempDir => self.temp_dir = Some(path),
        }
    }

    fn get(&self, key: PlatformPathKey) -> Option<&PathBuf> {
        match key {
            PlatformPathKey::ConfigDir => self.config_dir.as_ref(),
            PlatformPathKey::DataDir => self.data_dir.as_ref(),
            PlatformPathKey::CacheDir => self.cache_dir.as_ref(),
            PlatformPathKey::LogDir => self.log_dir.as_ref(),
            PlatformPathKey::PluginDir => self.plugin_dir.as_ref(),
            PlatformPathKey::SkillDir => self.skill_dir.as_ref(),
            PlatformPathKey::MemoryDir => self.memory_dir.as_ref(),
            PlatformPathKey::TempDir => self.temp_dir.as_ref(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
    pub plugin_dir: PathBuf,
    pub skill_dir: PathBuf,
    pub memory_dir: PathBuf,
    pub temp_dir: PathBuf,
}

impl PlatformPaths {
    pub fn current(overrides: PlatformPathOverrides) -> Result<Self, SettingsError> {
        Self::for_platform(PlatformKind::current(), current_home_dir()?, overrides)
    }

    pub fn for_platform(
        platform: PlatformKind,
        home_dir: PathBuf,
        overrides: PlatformPathOverrides,
    ) -> Result<Self, SettingsError> {
        validate_path(platform, "home_dir", &home_dir)?;
        if let Some(root) = &overrides.portable_root {
            validate_path(platform, "portable_root", root)?;
        }

        let mut paths = if let Some(root) = &overrides.portable_root {
            Self {
                config_dir: root.join("config"),
                data_dir: root.join("data"),
                cache_dir: root.join("cache"),
                log_dir: root.join("logs"),
                plugin_dir: root.join("plugins"),
                skill_dir: root.join("skills"),
                memory_dir: root.join("memory"),
                temp_dir: root.join("tmp"),
            }
        } else {
            default_paths(platform, &home_dir)
        };

        for key in [
            PlatformPathKey::ConfigDir,
            PlatformPathKey::DataDir,
            PlatformPathKey::CacheDir,
            PlatformPathKey::LogDir,
            PlatformPathKey::PluginDir,
            PlatformPathKey::SkillDir,
            PlatformPathKey::MemoryDir,
            PlatformPathKey::TempDir,
        ] {
            if let Some(path) = overrides.get(key) {
                validate_path(platform, &key.to_string(), path)?;
                paths.set(key, path.clone());
            }
        }

        Ok(paths)
    }

    fn set(&mut self, key: PlatformPathKey, path: PathBuf) {
        match key {
            PlatformPathKey::ConfigDir => self.config_dir = path,
            PlatformPathKey::DataDir => self.data_dir = path,
            PlatformPathKey::CacheDir => self.cache_dir = path,
            PlatformPathKey::LogDir => self.log_dir = path,
            PlatformPathKey::PluginDir => self.plugin_dir = path,
            PlatformPathKey::SkillDir => self.skill_dir = path,
            PlatformPathKey::MemoryDir => self.memory_dir = path,
            PlatformPathKey::TempDir => self.temp_dir = path,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
    Dev,
    Beta,
    Stable,
}

impl ReleaseChannel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Beta => "beta",
            Self::Stable => "stable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsProfile {
    pub schema_version: u32,
    pub profile_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_model: Option<String>,
    pub policy_profile: String,
    pub release_channel: ReleaseChannel,
    #[serde(default)]
    pub path_overrides: PlatformPathOverrides,
}

impl Default for SettingsProfile {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            profile_id: "default".to_string(),
            selected_provider: None,
            selected_model: None,
            policy_profile: "default".to_string(),
            release_channel: ReleaseChannel::Dev,
            path_overrides: PlatformPathOverrides::default(),
        }
    }
}

impl SettingsProfile {
    pub fn apply(&mut self, mutation: SettingsMutation) -> Result<(), SettingsError> {
        let mut next = self.clone();
        match mutation {
            SettingsMutation::SetModel { provider, model } => {
                next.selected_provider = Some(normalize_required("selected_provider", provider)?);
                next.selected_model = Some(normalize_required("selected_model", model)?);
            }
            SettingsMutation::SetReleaseChannel(channel) => {
                next.release_channel = channel;
            }
            SettingsMutation::SetPathOverride { key, path } => {
                validate_path(PlatformKind::current(), &key.to_string(), &path)?;
                next.path_overrides.set(key, path);
            }
        }
        next.validate()?;
        *self = next;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), SettingsError> {
        if self.schema_version != SETTINGS_SCHEMA_VERSION {
            return Err(SettingsError::Validation(format!(
                "unsupported settings schema version {}; supported version is {}",
                self.schema_version, SETTINGS_SCHEMA_VERSION
            )));
        }
        normalize_required("profile_id", self.profile_id.clone())?;
        normalize_required("policy_profile", self.policy_profile.clone())?;
        if let Some(provider) = &self.selected_provider {
            normalize_required("selected_provider", provider.clone())?;
        }
        if let Some(model) = &self.selected_model {
            normalize_required("selected_model", model.clone())?;
        }
        if self.selected_provider.is_some() != self.selected_model.is_some() {
            return Err(SettingsError::Validation(
                "selected_provider and selected_model must be set together".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SettingsMutation {
    SetModel { provider: String, model: String },
    SetReleaseChannel(ReleaseChannel),
    SetPathOverride { key: PlatformPathKey, path: PathBuf },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsLoadResult {
    pub profile: SettingsProfile,
    pub diagnostics: Vec<ConfigDiagnostic>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SettingsError {
    #[error("invalid settings: {0}")]
    Validation(String),
    #[error("settings file could not be read: {path}: {message}")]
    FileRead { path: String, message: String },
    #[error("settings file could not be written: {path}: {message}")]
    FileWrite { path: String, message: String },
    #[error("settings file could not be serialized: {0}")]
    Serialize(String),
}

pub fn load_settings_file(path: &Path) -> Result<SettingsLoadResult, SettingsError> {
    let input = match std::fs::read_to_string(path) {
        Ok(input) => input,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(SettingsLoadResult {
                profile: SettingsProfile::default(),
                diagnostics: Vec::new(),
            });
        }
        Err(error) => {
            return Err(SettingsError::FileRead {
                path: path.display().to_string(),
                message: error.to_string(),
            });
        }
    };

    match toml::from_str::<SettingsProfile>(&input) {
        Ok(profile) => match profile.validate() {
            Ok(()) => Ok(SettingsLoadResult {
                profile,
                diagnostics: Vec::new(),
            }),
            Err(error) => Ok(corrupt_settings_result(path, error.to_string())),
        },
        Err(error) => Ok(corrupt_settings_result(path, error.to_string())),
    }
}

pub fn write_settings_file_atomic(
    path: &Path,
    profile: &SettingsProfile,
) -> Result<(), SettingsError> {
    profile.validate()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| SettingsError::FileWrite {
            path: parent.display().to_string(),
            message: error.to_string(),
        })?;
    }
    let serialized = toml::to_string_pretty(profile)
        .map_err(|error| SettingsError::Serialize(error.to_string()))?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serialized).map_err(|error| SettingsError::FileWrite {
        path: tmp.display().to_string(),
        message: error.to_string(),
    })?;
    std::fs::rename(&tmp, path).map_err(|error| SettingsError::FileWrite {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    Ok(())
}

fn default_paths(platform: PlatformKind, home: &Path) -> PlatformPaths {
    match platform {
        PlatformKind::MacOs => {
            let app = home.join("Library/Application Support/agent");
            PlatformPaths {
                config_dir: app.join("config"),
                data_dir: app.join("data"),
                cache_dir: home.join("Library/Caches/agent"),
                log_dir: home.join("Library/Logs/agent"),
                plugin_dir: app.join("plugins"),
                skill_dir: app.join("skills"),
                memory_dir: app.join("memory"),
                temp_dir: home.join("Library/Caches/agent/tmp"),
            }
        }
        PlatformKind::Windows => {
            let roaming = home.join("AppData/Roaming/agent");
            let local = home.join("AppData/Local/agent");
            PlatformPaths {
                config_dir: roaming.join("config"),
                data_dir: roaming.join("data"),
                cache_dir: local.join("cache"),
                log_dir: local.join("logs"),
                plugin_dir: roaming.join("plugins"),
                skill_dir: roaming.join("skills"),
                memory_dir: roaming.join("memory"),
                temp_dir: local.join("tmp"),
            }
        }
        PlatformKind::Linux | PlatformKind::Other => PlatformPaths {
            config_dir: home.join(".config/agent"),
            data_dir: home.join(".local/share/agent"),
            cache_dir: home.join(".cache/agent"),
            log_dir: home.join(".local/state/agent/logs"),
            plugin_dir: home.join(".local/share/agent/plugins"),
            skill_dir: home.join(".local/share/agent/skills"),
            memory_dir: home.join(".local/share/agent/memory"),
            temp_dir: home.join(".cache/agent/tmp"),
        },
    }
}

fn current_home_dir() -> Result<PathBuf, SettingsError> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| {
            SettingsError::Validation("home directory could not be resolved".to_string())
        })
}

fn validate_path(platform: PlatformKind, label: &str, path: &Path) -> Result<(), SettingsError> {
    if !is_absolute_for_platform(platform, path) {
        return Err(SettingsError::Validation(format!(
            "{label} must be absolute: {}",
            path.display()
        )));
    }
    if path
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(SettingsError::Validation(format!(
            "{label} must not contain parent traversal: {}",
            path.display()
        )));
    }
    Ok(())
}

fn is_absolute_for_platform(platform: PlatformKind, path: &Path) -> bool {
    if path.is_absolute() {
        return true;
    }
    platform == PlatformKind::Windows
        && path
            .to_string_lossy()
            .as_bytes()
            .get(1)
            .is_some_and(|byte| *byte == b':')
}

fn normalize_required(label: &str, value: String) -> Result<String, SettingsError> {
    let value = value.trim();
    if value.is_empty() {
        Err(SettingsError::Validation(format!(
            "{label} cannot be empty"
        )))
    } else {
        Ok(value.to_string())
    }
}

fn corrupt_settings_result(path: &Path, message: String) -> SettingsLoadResult {
    SettingsLoadResult {
        profile: SettingsProfile::default(),
        diagnostics: vec![ConfigDiagnostic {
            severity: ConfigDiagnosticSeverity::Warning,
            code: "settings.file.corrupt".to_string(),
            message: "settings file could not be loaded; using safe defaults".to_string(),
            field_path: None,
            source: Some(format!("{}: {message}", path.display())),
        }],
    }
}

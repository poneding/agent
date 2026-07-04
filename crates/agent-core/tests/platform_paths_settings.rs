use std::fs;
use std::path::PathBuf;

use agent_core::{
    ConfigDiagnosticSeverity, PlatformKind, PlatformPathKey, PlatformPathOverrides, PlatformPaths,
    ReleaseChannel, SettingsMutation, SettingsProfile, load_settings_file,
    write_settings_file_atomic,
};

fn root() -> PathBuf {
    PathBuf::from("/Users/tester")
}

#[test]
fn platform_paths_settings_resolve_platform_defaults() {
    let mac = PlatformPaths::for_platform(
        PlatformKind::MacOs,
        root(),
        PlatformPathOverrides::default(),
    )
    .expect("mac paths");
    assert_eq!(
        mac.config_dir,
        PathBuf::from("/Users/tester/Library/Application Support/agent/config")
    );
    assert_eq!(
        mac.cache_dir,
        PathBuf::from("/Users/tester/Library/Caches/agent")
    );
    assert_eq!(
        mac.log_dir,
        PathBuf::from("/Users/tester/Library/Logs/agent")
    );

    let linux = PlatformPaths::for_platform(
        PlatformKind::Linux,
        PathBuf::from("/home/tester"),
        PlatformPathOverrides::default(),
    )
    .expect("linux paths");
    assert_eq!(
        linux.config_dir,
        PathBuf::from("/home/tester/.config/agent")
    );
    assert_eq!(
        linux.data_dir,
        PathBuf::from("/home/tester/.local/share/agent")
    );

    let windows = PlatformPaths::for_platform(
        PlatformKind::Windows,
        PathBuf::from("C:/Users/tester"),
        PlatformPathOverrides::default(),
    )
    .expect("windows paths");
    assert_eq!(
        windows.config_dir,
        PathBuf::from("C:/Users/tester/AppData/Roaming/agent/config")
    );
    assert_eq!(
        windows.cache_dir,
        PathBuf::from("C:/Users/tester/AppData/Local/agent/cache")
    );
}

#[test]
fn platform_paths_settings_apply_portable_and_specific_overrides() {
    let mut overrides = PlatformPathOverrides {
        portable_root: Some(PathBuf::from("/tmp/agent-portable")),
        ..PlatformPathOverrides::default()
    };
    overrides.set(
        PlatformPathKey::CacheDir,
        PathBuf::from("/tmp/custom-cache"),
    );

    let paths = PlatformPaths::for_platform(
        PlatformKind::Linux,
        PathBuf::from("/home/tester"),
        overrides,
    )
    .expect("portable paths");

    assert_eq!(
        paths.config_dir,
        PathBuf::from("/tmp/agent-portable/config")
    );
    assert_eq!(paths.data_dir, PathBuf::from("/tmp/agent-portable/data"));
    assert_eq!(paths.cache_dir, PathBuf::from("/tmp/custom-cache"));
    assert_eq!(paths.temp_dir, PathBuf::from("/tmp/agent-portable/tmp"));
}

#[test]
fn platform_paths_settings_reject_relative_overrides() {
    let mut overrides = PlatformPathOverrides::default();
    overrides.set(PlatformPathKey::ConfigDir, PathBuf::from("relative/config"));

    let err = PlatformPaths::for_platform(
        PlatformKind::Linux,
        PathBuf::from("/home/tester"),
        overrides,
    )
    .expect_err("relative override rejected");

    assert!(err.to_string().contains("must be absolute"));
}

#[test]
fn platform_paths_settings_mutates_and_validates_settings() {
    let mut settings = SettingsProfile::default();

    settings
        .apply(SettingsMutation::SetModel {
            provider: "openai".to_string(),
            model: "gpt-4.1-mini".to_string(),
        })
        .expect("set model");
    settings
        .apply(SettingsMutation::SetReleaseChannel(ReleaseChannel::Beta))
        .expect("set channel");
    settings
        .apply(SettingsMutation::SetPathOverride {
            key: PlatformPathKey::PluginDir,
            path: PathBuf::from("/tmp/agent-plugins"),
        })
        .expect("set path");

    assert_eq!(settings.selected_provider.as_deref(), Some("openai"));
    assert_eq!(settings.selected_model.as_deref(), Some("gpt-4.1-mini"));
    assert_eq!(settings.release_channel, ReleaseChannel::Beta);
    assert_eq!(
        settings.path_overrides.plugin_dir,
        Some(PathBuf::from("/tmp/agent-plugins"))
    );
    assert!(settings.validate().is_ok());
}

#[test]
fn platform_paths_settings_loads_corrupted_file_as_default_with_diagnostic() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("settings.toml");
    fs::write(&path, "schema_version =").expect("write corrupt settings");

    let loaded = load_settings_file(&path).expect("load with diagnostics");

    assert_eq!(loaded.profile, SettingsProfile::default());
    assert_eq!(
        loaded.diagnostics[0].severity,
        ConfigDiagnosticSeverity::Warning
    );
    assert_eq!(loaded.diagnostics[0].code, "settings.file.corrupt");
}

#[test]
fn platform_paths_settings_writes_and_loads_settings_atomically() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("nested/settings.toml");
    let mut settings = SettingsProfile::default();
    settings
        .apply(SettingsMutation::SetModel {
            provider: "fake".to_string(),
            model: "fake-scripted".to_string(),
        })
        .expect("set model");

    write_settings_file_atomic(&path, &settings).expect("write settings");
    let loaded = load_settings_file(&path).expect("load settings");

    assert_eq!(loaded.profile.selected_provider.as_deref(), Some("fake"));
    assert_eq!(
        loaded.profile.selected_model.as_deref(),
        Some("fake-scripted")
    );
    assert!(loaded.diagnostics.is_empty());
    assert!(!path.with_extension("tmp").exists());
}

#![forbid(unsafe_code)]
//! Shared parsing and command planning for the workspace xtask entrypoint.

use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Task {
    Help,
    Check,
    Fmt,
    FmtCheck,
    Clippy,
    Test,
    CliPrompt { prompt: String },
    CliJson { prompt: String },
    PackageCli(PackageCliOptions),
    PackageDesktop(PackageDesktopOptions),
    DesktopTest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl CargoCommand {
    fn new(args: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            program: "cargo".to_string(),
            args: args.into_iter().map(str::to_string).collect(),
        }
    }

    fn with_prompt(args: &[&str], prompt: &str) -> Self {
        let mut command = Self {
            program: "cargo".to_string(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
        };
        command.args.push(prompt.to_string());
        command
    }

    fn from_args(args: Vec<String>) -> Self {
        Self {
            program: "cargo".to_string(),
            args,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

    fn parse(value: &str) -> Result<Self, XtaskError> {
        match value {
            "dev" => Ok(Self::Dev),
            "beta" => Ok(Self::Beta),
            "stable" => Ok(Self::Stable),
            other => Err(XtaskError::InvalidReleaseChannel(other.to_string())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageCliOptions {
    pub channel: ReleaseChannel,
    pub target_triple: Option<String>,
    pub output_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageDesktopOptions {
    pub channel: ReleaseChannel,
    pub target_triple: Option<String>,
    pub output_dir: PathBuf,
    pub bundle: bool,
}

impl Default for PackageDesktopOptions {
    fn default() -> Self {
        Self {
            channel: ReleaseChannel::Dev,
            target_triple: None,
            output_dir: PathBuf::from("target/agent-release"),
            bundle: false,
        }
    }
}

impl Default for PackageCliOptions {
    fn default() -> Self {
        Self {
            channel: ReleaseChannel::Dev,
            target_triple: None,
            output_dir: PathBuf::from("target/agent-release"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

impl ReleaseCommand {
    fn new(program: impl Into<String>, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            env: Vec::new(),
        }
    }

    fn with_env(
        mut self,
        env: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.env = env
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        self
    }

    #[must_use]
    pub fn shell_line(&self) -> String {
        if self.args.is_empty() {
            return self.program.clone();
        }
        format!("{} {}", self.program, self.args.join(" "))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageCliPlan {
    pub channel: ReleaseChannel,
    pub target_triple: String,
    pub build_command: CargoCommand,
    pub artifact_source: PathBuf,
    pub artifact_output: PathBuf,
    pub checksum_path: PathBuf,
    pub manifest_path: PathBuf,
    pub smoke_root_path: PathBuf,
    pub smoke_config_path: PathBuf,
    pub smoke_commands: Vec<ReleaseCommand>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageDesktopPlan {
    pub channel: ReleaseChannel,
    pub target_triple: String,
    pub bundle: bool,
    pub bundle_command: ReleaseCommand,
    pub tauri_config_path: PathBuf,
    pub smoke_root_path: PathBuf,
    pub plan_artifact_path: PathBuf,
    pub update_manifest_path: PathBuf,
    pub smoke_commands: Vec<ReleaseCommand>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CliReleaseArtifact {
    pub name: String,
    pub target_triple: String,
    pub version: String,
    pub path: PathBuf,
    pub checksum_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CliReleaseManifest {
    pub schema_version: u32,
    pub channel: ReleaseChannel,
    pub artifact: CliReleaseArtifact,
    pub smoke_checks: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopUpdateManifest {
    pub channel: ReleaseChannel,
    pub version: String,
    pub artifacts: Vec<DesktopUpdateArtifact>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopUpdateArtifact {
    pub platform: String,
    pub url: String,
    pub checksum_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl CliReleaseManifest {
    #[must_use]
    pub const fn new(
        channel: ReleaseChannel,
        artifact: CliReleaseArtifact,
        smoke_checks: Vec<String>,
    ) -> Self {
        Self {
            schema_version: 1,
            channel,
            artifact,
            smoke_checks,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XtaskError {
    UnknownCommand(String),
    UnknownOption(String),
    MissingPrompt(String),
    MissingValue(String),
    InvalidReleaseChannel(String),
}

impl fmt::Display for XtaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand(command) => write!(formatter, "unknown xtask command: {command}"),
            Self::UnknownOption(option) => write!(formatter, "unknown xtask option: {option}"),
            Self::MissingPrompt(command) => write!(formatter, "{command} requires a prompt"),
            Self::MissingValue(option) => write!(formatter, "{option} requires a value"),
            Self::InvalidReleaseChannel(channel) => {
                write!(formatter, "invalid release channel: {channel}")
            }
        }
    }
}

impl std::error::Error for XtaskError {}

pub fn parse_args<I, S>(args: I) -> Result<Task, XtaskError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let Some(command) = args.next() else {
        return Ok(Task::Help);
    };

    match command.as_str() {
        "help" | "-h" | "--help" => Ok(Task::Help),
        "check" => Ok(Task::Check),
        "fmt" => Ok(Task::Fmt),
        "fmt-check" => Ok(Task::FmtCheck),
        "clippy" => Ok(Task::Clippy),
        "test" => Ok(Task::Test),
        "desktop-test" => Ok(Task::DesktopTest),
        "cli-prompt" => Ok(Task::CliPrompt {
            prompt: parse_prompt(&command, args.collect())?,
        }),
        "cli-json" => Ok(Task::CliJson {
            prompt: parse_prompt(&command, args.collect())?,
        }),
        "package-cli" => Ok(Task::PackageCli(parse_package_cli_options(args)?)),
        "package-desktop" => Ok(Task::PackageDesktop(parse_package_desktop_options(args)?)),
        _ => Err(XtaskError::UnknownCommand(command)),
    }
}

#[must_use]
pub fn command_plan_for(task: &Task) -> Vec<CargoCommand> {
    match task {
        Task::Help => Vec::new(),
        Task::Check => vec![fmt_check_command(), clippy_command(), test_command()],
        Task::Fmt => vec![CargoCommand::new(["fmt", "--all"])],
        Task::FmtCheck => vec![fmt_check_command()],
        Task::Clippy => vec![clippy_command()],
        Task::Test => vec![test_command()],
        Task::CliPrompt { prompt } => vec![CargoCommand::with_prompt(
            &["run", "-p", "agent-cli", "--", "--prompt"],
            prompt,
        )],
        Task::CliJson { prompt } => vec![CargoCommand::with_prompt(
            &["run", "-p", "agent-cli", "--", "--json", "--prompt"],
            prompt,
        )],
        Task::PackageCli(options) => {
            let plan = package_cli_plan(options, "host", Path::new("."), env!("CARGO_PKG_VERSION"));
            vec![plan.build_command]
        }
        Task::PackageDesktop(_options) => {
            vec![CargoCommand::new([
                "test",
                "-p",
                "agent-desktop",
                "update_manifest",
            ])]
        }
        Task::DesktopTest => vec![CargoCommand::new(["test", "-p", "agent-desktop"])],
    }
}

#[must_use]
pub fn help_text() -> &'static str {
    r#"cargo xtask <command>

Commands:
  check                 Run fmt-check, clippy, and test
  fmt                   Run cargo fmt --all
  fmt-check             Run cargo fmt --all -- --check
  clippy                Run cargo clippy --workspace --all-targets -- -D warnings
  test                  Run cargo test --workspace
  cli-prompt -- <text>  Run agent-cli with --prompt <text>
  cli-json -- <text>    Run agent-cli with --json --prompt <text>
  package-cli           Build and smoke-test a local CLI release artifact
  package-desktop       Plan or build and smoke-test a local Desktop release
  desktop-test          Run cargo test -p agent-desktop
  help                  Print this help
"#
}

fn parse_prompt(command: &str, words: Vec<String>) -> Result<String, XtaskError> {
    let words = match words.split_first() {
        Some((separator, rest)) if separator == "--" => rest,
        _ => words.as_slice(),
    };

    if words.is_empty() {
        return Err(XtaskError::MissingPrompt(command.to_string()));
    }

    Ok(words.join(" "))
}

fn parse_package_cli_options<I>(args: I) -> Result<PackageCliOptions, XtaskError>
where
    I: Iterator<Item = String>,
{
    let mut options = PackageCliOptions::default();
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--channel" => {
                options.channel = ReleaseChannel::parse(&take_value(&mut args, &arg)?)?;
            }
            "--target" => {
                options.target_triple = Some(take_value(&mut args, &arg)?);
            }
            "--out-dir" => {
                options.output_dir = PathBuf::from(take_value(&mut args, &arg)?);
            }
            other => return Err(XtaskError::UnknownOption(other.to_string())),
        }
    }

    Ok(options)
}

fn parse_package_desktop_options<I>(args: I) -> Result<PackageDesktopOptions, XtaskError>
where
    I: Iterator<Item = String>,
{
    let mut options = PackageDesktopOptions::default();
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--channel" => {
                options.channel = ReleaseChannel::parse(&take_value(&mut args, &arg)?)?;
            }
            "--target" => {
                options.target_triple = Some(take_value(&mut args, &arg)?);
            }
            "--out-dir" => {
                options.output_dir = PathBuf::from(take_value(&mut args, &arg)?);
            }
            "--bundle" => options.bundle = true,
            other => return Err(XtaskError::UnknownOption(other.to_string())),
        }
    }

    Ok(options)
}

fn take_value<I>(args: &mut I, option: &str) -> Result<String, XtaskError>
where
    I: Iterator<Item = String>,
{
    args.next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| XtaskError::MissingValue(option.to_string()))
}

#[must_use]
pub fn package_cli_plan(
    options: &PackageCliOptions,
    host_target_triple: &str,
    workspace_root: &Path,
    version: &str,
) -> PackageCliPlan {
    let target_triple = options
        .target_triple
        .clone()
        .unwrap_or_else(|| host_target_triple.to_string());
    let binary_name = binary_name_for_target(&target_triple);

    let mut build_args = ["build", "--release", "-p", "agent-cli", "--bin", "agent"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if options.target_triple.is_some() {
        build_args.push("--target".to_string());
        build_args.push(target_triple.clone());
    }

    let artifact_source = if options.target_triple.is_some() {
        workspace_root
            .join("target")
            .join(&target_triple)
            .join("release")
            .join(&binary_name)
    } else {
        workspace_root
            .join("target")
            .join("release")
            .join(&binary_name)
    };

    let artifact_stem = format!(
        "agent-{version}-{target_triple}-{}",
        options.channel.as_str()
    );
    let artifact_name = if binary_name.ends_with(".exe") {
        format!("{artifact_stem}.exe")
    } else {
        artifact_stem
    };
    let artifact_output = options.output_dir.join(&artifact_name);
    let checksum_path = options.output_dir.join(format!("{artifact_name}.sha256"));
    let manifest_path = options.output_dir.join(format!("{artifact_name}.json"));
    let smoke_root_path = options.output_dir.join("smoke-root");
    let smoke_config_path = smoke_root_path
        .join("config")
        .join("agent-release-smoke.toml");
    let smoke_config = smoke_config_path.display().to_string();
    let smoke_root_abs = if smoke_root_path.is_absolute() {
        smoke_root_path.clone()
    } else {
        workspace_root.join(&smoke_root_path)
    };
    let smoke_root = smoke_root_abs.display().to_string();
    let smoke_env = vec![
        ("HOME".to_string(), smoke_root.clone()),
        ("USERPROFILE".to_string(), smoke_root),
    ];
    let program = artifact_output.display().to_string();

    PackageCliPlan {
        channel: options.channel,
        target_triple,
        build_command: CargoCommand::from_args(build_args),
        artifact_source,
        artifact_output,
        checksum_path,
        manifest_path,
        smoke_root_path,
        smoke_config_path,
        smoke_commands: vec![
            ReleaseCommand::new(program.clone(), ["--version"]).with_env(smoke_env.clone()),
            ReleaseCommand::new(
                program.clone(),
                ["--config-check", "--config", &smoke_config],
            )
            .with_env(smoke_env.clone()),
            ReleaseCommand::new(program.clone(), ["startup-check"]).with_env(smoke_env.clone()),
            ReleaseCommand::new(program, ["--json", "--prompt", "release smoke"])
                .with_env(smoke_env),
        ],
    }
}

#[must_use]
pub fn package_desktop_plan(
    options: &PackageDesktopOptions,
    host_target_triple: &str,
    workspace_root: &Path,
    version: &str,
) -> PackageDesktopPlan {
    let target_triple = options
        .target_triple
        .clone()
        .unwrap_or_else(|| host_target_triple.to_string());
    let artifact_name = format!(
        "agent-desktop-{version}-{target_triple}-{}.plan.txt",
        options.channel.as_str()
    );
    let update_manifest_name = format!(
        "agent-desktop-{version}-{target_triple}-{}.update.json",
        options.channel.as_str()
    );
    let mut bundle_args = vec![
        "--prefix".to_string(),
        "apps/desktop".to_string(),
        "run".to_string(),
        "tauri".to_string(),
        "--".to_string(),
        "build".to_string(),
    ];
    if options.target_triple.is_some() {
        bundle_args.push("--target".to_string());
        bundle_args.push(target_triple.clone());
    }
    let smoke_root_path = options.output_dir.join("desktop-smoke-root");
    let smoke_root_abs = if smoke_root_path.is_absolute() {
        smoke_root_path.clone()
    } else {
        workspace_root.join(&smoke_root_path)
    };
    let smoke_root = smoke_root_abs.display().to_string();
    let smoke_env = vec![
        ("HOME".to_string(), smoke_root.clone()),
        ("USERPROFILE".to_string(), smoke_root),
    ];

    PackageDesktopPlan {
        channel: options.channel,
        target_triple,
        bundle: options.bundle,
        bundle_command: ReleaseCommand::new("npm", bundle_args),
        tauri_config_path: workspace_root.join("apps/desktop/src-tauri/tauri.conf.json"),
        smoke_root_path,
        plan_artifact_path: options.output_dir.join(artifact_name),
        update_manifest_path: options.output_dir.join(update_manifest_name),
        smoke_commands: vec![
            ReleaseCommand::new("cargo", ["test", "-p", "agent-desktop", "update_manifest"])
                .with_env(smoke_env.clone()),
            ReleaseCommand::new("cargo", ["test", "-p", "agent-desktop", "settings"])
                .with_env(smoke_env),
        ],
    }
}

#[derive(Debug)]
pub enum PackageCliError {
    Io(io::Error),
    CommandFailed {
        program: String,
        args: Vec<String>,
        code: Option<i32>,
    },
    HostTargetUnavailable,
    Json(serde_json::Error),
}

#[derive(Debug)]
pub enum PackageDesktopError {
    Io(io::Error),
    CommandFailed {
        program: String,
        args: Vec<String>,
        code: Option<i32>,
    },
    HostTargetUnavailable,
    InvalidTauriConfig(String),
    Json(serde_json::Error),
}

impl fmt::Display for PackageDesktopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::CommandFailed {
                program,
                args,
                code,
            } => write!(
                formatter,
                "command failed with code {}: {} {}",
                code.map_or_else(|| "signal".to_string(), |code| code.to_string()),
                program,
                args.join(" ")
            ),
            Self::HostTargetUnavailable => write!(formatter, "could not detect host target triple"),
            Self::InvalidTauriConfig(message) => {
                write!(formatter, "invalid Tauri config: {message}")
            }
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for PackageDesktopError {}

impl From<io::Error> for PackageDesktopError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for PackageDesktopError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl fmt::Display for PackageCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::CommandFailed {
                program,
                args,
                code,
            } => write!(
                formatter,
                "command failed with code {}: {} {}",
                code.map_or_else(|| "signal".to_string(), |code| code.to_string()),
                program,
                args.join(" ")
            ),
            Self::HostTargetUnavailable => write!(formatter, "could not detect host target triple"),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for PackageCliError {}

impl From<io::Error> for PackageCliError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for PackageCliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub fn run_package_cli(options: &PackageCliOptions) -> Result<CliReleaseManifest, PackageCliError> {
    let workspace_root = std::env::current_dir()?;
    let host_target = detect_host_target_triple()?;
    let plan = package_cli_plan(
        options,
        &host_target,
        &workspace_root,
        env!("CARGO_PKG_VERSION"),
    );

    run_cargo_command(&plan.build_command)?;
    fs::create_dir_all(&options.output_dir)?;
    fs::copy(&plan.artifact_source, &plan.artifact_output)?;

    let checksum = sha256_file(&plan.artifact_output)?;
    fs::write(
        &plan.checksum_path,
        format!(
            "{}  {}\n",
            checksum,
            plan.artifact_output
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("agent")
        ),
    )?;
    fs::create_dir_all(
        plan.smoke_config_path
            .parent()
            .unwrap_or_else(|| Path::new(".")),
    )?;
    fs::write(&plan.smoke_config_path, smoke_config_toml())?;

    for command in &plan.smoke_commands {
        run_release_command(command)?;
    }

    let artifact = CliReleaseArtifact {
        name: plan
            .artifact_output
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("agent")
            .to_string(),
        target_triple: plan.target_triple.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        path: plan.artifact_output.clone(),
        checksum_sha256: checksum,
    };
    let manifest = CliReleaseManifest::new(
        plan.channel,
        artifact,
        plan.smoke_commands
            .iter()
            .map(ReleaseCommand::shell_line)
            .collect(),
    );
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&plan.manifest_path, format!("{manifest_json}\n"))?;

    Ok(manifest)
}

pub fn run_package_desktop(
    options: &PackageDesktopOptions,
) -> Result<DesktopUpdateManifest, PackageDesktopError> {
    let workspace_root = std::env::current_dir()?;
    let host_target = detect_host_target_triple().map_err(|error| match error {
        PackageCliError::HostTargetUnavailable => PackageDesktopError::HostTargetUnavailable,
        PackageCliError::Io(error) => PackageDesktopError::Io(error),
        PackageCliError::CommandFailed {
            program,
            args,
            code,
        } => PackageDesktopError::CommandFailed {
            program,
            args,
            code,
        },
        PackageCliError::Json(error) => PackageDesktopError::Json(error),
    })?;
    let plan = package_desktop_plan(
        options,
        &host_target,
        &workspace_root,
        env!("CARGO_PKG_VERSION"),
    );

    validate_tauri_release_config(&plan.tauri_config_path)?;
    fs::create_dir_all(&options.output_dir)?;
    write_desktop_smoke_root(&plan)?;

    if plan.bundle {
        run_desktop_release_command(&plan.bundle_command)?;
    }

    fs::write(
        &plan.plan_artifact_path,
        format!(
            "agent desktop release plan\nversion={}\ntarget={}\nchannel={}\nbundle={}\nsmoke_root={}\nsmoke_checks={}\n",
            env!("CARGO_PKG_VERSION"),
            plan.target_triple,
            plan.channel.as_str(),
            plan.bundle,
            plan.smoke_root_path.display(),
            plan.smoke_commands
                .iter()
                .map(ReleaseCommand::shell_line)
                .collect::<Vec<_>>()
                .join("; "),
        ),
    )?;
    let checksum = sha256_file(&plan.plan_artifact_path)?;
    let artifact_url = file_url(&workspace_root, &plan.plan_artifact_path);
    let manifest = DesktopUpdateManifest {
        channel: plan.channel,
        version: env!("CARGO_PKG_VERSION").to_string(),
        artifacts: vec![DesktopUpdateArtifact {
            platform: plan.target_triple,
            url: artifact_url,
            checksum_sha256: checksum,
            signature: None,
        }],
    };
    validate_desktop_update_manifest(&manifest)?;
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&plan.update_manifest_path, format!("{manifest_json}\n"))?;

    Ok(manifest)
}

pub fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

pub fn detect_host_target_triple() -> Result<String, PackageCliError> {
    let output = Command::new("rustc").arg("-vV").output()?;
    if !output.status.success() {
        return Err(PackageCliError::HostTargetUnavailable);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_string)
        .ok_or(PackageCliError::HostTargetUnavailable)
}

fn validate_tauri_release_config(path: &Path) -> Result<(), PackageDesktopError> {
    let input = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&input)?;
    if value["productName"] != "agent" {
        return Err(PackageDesktopError::InvalidTauriConfig(
            "productName must be agent".to_string(),
        ));
    }
    if value["identifier"] != "run.agent.desktop" {
        return Err(PackageDesktopError::InvalidTauriConfig(
            "identifier must be run.agent.desktop".to_string(),
        ));
    }
    if value["bundle"]["active"] != true {
        return Err(PackageDesktopError::InvalidTauriConfig(
            "bundle.active must be true".to_string(),
        ));
    }
    if value["bundle"]["icon"]
        .as_array()
        .is_none_or(|icons| icons.is_empty())
    {
        return Err(PackageDesktopError::InvalidTauriConfig(
            "bundle.icon must include at least one icon".to_string(),
        ));
    }
    Ok(())
}

fn write_desktop_smoke_root(plan: &PackageDesktopPlan) -> Result<(), PackageDesktopError> {
    for dir in ["config", "data", "cache", "logs"] {
        fs::create_dir_all(plan.smoke_root_path.join(dir))?;
    }

    let snapshot = serde_json::json!({
        "app_version": env!("CARGO_PKG_VERSION"),
        "release_channel": plan.channel.as_str(),
        "target_triple": plan.target_triple,
        "config_dir": plan.smoke_root_path.join("config"),
        "data_dir": plan.smoke_root_path.join("data"),
        "cache_dir": plan.smoke_root_path.join("cache"),
        "log_dir": plan.smoke_root_path.join("logs"),
        "settings_profile_id": "default",
    });
    fs::write(
        plan.smoke_root_path.join("desktop-release-smoke.json"),
        format!("{}\n", serde_json::to_string_pretty(&snapshot)?),
    )?;
    Ok(())
}

fn validate_desktop_update_manifest(
    manifest: &DesktopUpdateManifest,
) -> Result<(), PackageDesktopError> {
    if manifest.version.trim().is_empty() {
        return Err(PackageDesktopError::InvalidTauriConfig(
            "update manifest version cannot be empty".to_string(),
        ));
    }
    if manifest.artifacts.is_empty() {
        return Err(PackageDesktopError::InvalidTauriConfig(
            "update manifest must include at least one artifact".to_string(),
        ));
    }
    for artifact in &manifest.artifacts {
        if artifact.platform.trim().is_empty() {
            return Err(PackageDesktopError::InvalidTauriConfig(
                "update artifact platform cannot be empty".to_string(),
            ));
        }
        if !artifact.url.starts_with("file://")
            && !artifact.url.starts_with("https://")
            && !artifact.url.starts_with("http://")
        {
            return Err(PackageDesktopError::InvalidTauriConfig(
                "update artifact url must use file, http, or https".to_string(),
            ));
        }
        if !is_sha256_hex(&artifact.checksum_sha256) {
            return Err(PackageDesktopError::InvalidTauriConfig(
                "update artifact checksum must be sha256 hex".to_string(),
            ));
        }
    }
    Ok(())
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn run_desktop_release_command(command: &ReleaseCommand) -> Result<(), PackageDesktopError> {
    let output = Command::new(&command.program)
        .args(&command.args)
        .envs(command.env.iter().map(|(key, value)| (key, value)))
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(PackageDesktopError::CommandFailed {
            program: command.program.clone(),
            args: command.args.clone(),
            code: output.status.code(),
        })
    }
}

fn file_url(workspace_root: &Path, path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    };
    format!("file://{}", absolute.display())
}

fn run_cargo_command(command: &CargoCommand) -> Result<(), PackageCliError> {
    let status = Command::new(&command.program)
        .args(&command.args)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(PackageCliError::CommandFailed {
            program: command.program.clone(),
            args: command.args.clone(),
            code: status.code(),
        })
    }
}

fn run_release_command(command: &ReleaseCommand) -> Result<(), PackageCliError> {
    let output = Command::new(&command.program)
        .args(&command.args)
        .envs(command.env.iter().map(|(key, value)| (key, value)))
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(PackageCliError::CommandFailed {
            program: command.program.clone(),
            args: command.args.clone(),
            code: output.status.code(),
        })
    }
}

fn binary_name_for_target(target_triple: &str) -> String {
    if target_triple.contains("windows") {
        "agent.exe".to_string()
    } else {
        "agent".to_string()
    }
}

fn smoke_config_toml() -> &'static str {
    r#"
schema_version = 1

[model]
default_provider = "fake"

[model.invocation]
timeout_ms = 30000
max_attempts = 1
retryable_failures = []
fallback_provider_ids = []

[[model.providers]]
id = "fake"
kind = "fake"
default_model = "fake-scripted"
enabled = true
timeout_ms = 1000
"#
}

fn fmt_check_command() -> CargoCommand {
    CargoCommand::new(["fmt", "--all", "--", "--check"])
}

fn clippy_command() -> CargoCommand {
    CargoCommand::new([
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])
}

fn test_command() -> CargoCommand {
    CargoCommand::new(["test", "--workspace"])
}

//! Process-level workspace confinement for shell tools.
//!
//! ## What this enforces
//! - Working directory is forced under [`WorkspaceSandbox::root`]
//! - Environment is cleared except an allowlist
//! - On macOS, optional `sandbox-exec` (Seatbelt) denies writes outside the
//!   workspace and denies network when requested
//!
//! ## What this does **not** claim
//! - Full OS isolation on Linux/Windows (no bwrap/landlock yet)
//! - Blocking every possible side channel (path-allowed system tools still run)
//!
//! Seatbelt profiles are intentionally small and fail **closed for writes**:
//! if `sandbox-exec` cannot start, callers should surface a diagnostic rather
//! than silently running unrestricted.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::path::WorkspaceSandbox;

/// How aggressively shell execution should be confined.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProcessSandboxMode {
    /// Only path/env/timeout controls from the tool layer (always on).
    PathOnly,
    /// macOS: wrap with `sandbox-exec` seatbelt; other OSes fall back to PathOnly.
    PreferSeatbelt,
}

/// Resolved process sandbox plan for a single shell invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessSandboxPlan {
    pub mode: ProcessSandboxMode,
    pub profile_label: &'static str,
    pub seatbelt_profile: Option<String>,
    pub workspace_root: PathBuf,
}

impl ProcessSandboxPlan {
    /// Build a plan for running a command under `cwd` inside `sandbox`.
    pub fn for_workspace(
        sandbox: &WorkspaceSandbox,
        mode: ProcessSandboxMode,
        allow_network: bool,
    ) -> Self {
        let workspace_root = sandbox.root().to_path_buf();
        match mode {
            ProcessSandboxMode::PathOnly => Self {
                mode,
                // Public audit label — never mention OS machinery (seatbelt/bwrap).
                profile_label: "workspace",
                seatbelt_profile: None,
                workspace_root,
            },
            ProcessSandboxMode::PreferSeatbelt => {
                #[cfg(target_os = "macos")]
                {
                    let profile = seatbelt_profile(&workspace_root, allow_network);
                    Self {
                        mode,
                        profile_label: if allow_network {
                            "workspace_process_net"
                        } else {
                            "workspace_process"
                        },
                        seatbelt_profile: Some(profile),
                        workspace_root,
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = allow_network;
                    Self {
                        mode: ProcessSandboxMode::PathOnly,
                        profile_label: "workspace",
                        seatbelt_profile: None,
                        workspace_root,
                    }
                }
            }
        }
    }

    /// Rewrite `command` so it runs under the planned sandbox when possible.
    ///
    /// On macOS with a seatbelt profile, this becomes:
    /// `sandbox-exec -p '<profile>' -- <program> <args...>`
    pub fn wrap_command(&self, program: &Path, args: &[String]) -> Command {
        #[cfg(target_os = "macos")]
        if let Some(profile) = &self.seatbelt_profile {
            let mut command = Command::new("sandbox-exec");
            command.arg("-p").arg(profile).arg("--").arg(program);
            command.args(args);
            return command;
        }

        let mut command = Command::new(program);
        command.args(args);
        command
    }
}

/// Rewrite process diagnostics so tool results look like normal command failures.
///
/// macOS `sandbox-exec` prints implementation details when the wrapped program
/// cannot start, e.g.:
/// `sandbox-exec: execvp() of 'python' failed: No such file or directory`
///
/// Agents and the UI should only see the command outcome, not seatbelt mechanics.
pub fn sanitize_command_stderr(stderr: &str) -> String {
    if !contains_process_sandbox_marker(stderr) {
        return stderr.to_string();
    }

    let trailing_newline = stderr.ends_with('\n');
    let mut out = stderr
        .lines()
        .map(rewrite_process_sandbox_line)
        .collect::<Vec<_>>()
        .join("\n");
    // `str::lines` drops the final empty segment after a trailing newline.
    if trailing_newline && !out.is_empty() {
        out.push('\n');
    }
    out
}

/// Rewrite spawn / ToolError messages that accidentally name the process wrapper.
pub fn sanitize_execution_error(message: &str) -> String {
    if !contains_process_sandbox_marker(message) {
        return message.to_string();
    }
    let cleaned = sanitize_command_stderr(message);
    if !cleaned.trim().is_empty() && !contains_process_sandbox_marker(&cleaned) {
        return cleaned;
    }
    // Last resort: never hand the wrapper binary name to agents/UI.
    if message.contains("No such file") || message.to_ascii_lowercase().contains("not found") {
        "failed to start command: program not found".to_string()
    } else {
        "failed to start command".to_string()
    }
}

fn contains_process_sandbox_marker(text: &str) -> bool {
    text.contains("sandbox-exec")
        || text.contains("seatbelt")
        || text.contains("bwrap:")
        || text.contains("bubblewrap")
}

fn rewrite_process_sandbox_line(line: &str) -> String {
    let trimmed = line.trim_start();

    if let Some(rest) = trimmed.strip_prefix("sandbox-exec:").map(str::trim_start) {
        return rewrite_sandbox_exec_rest(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("bwrap:").map(str::trim_start) {
        return if rest.is_empty() {
            String::new()
        } else {
            rest.to_string()
        };
    }
    if trimmed.to_ascii_lowercase().contains("seatbelt")
        || trimmed.to_ascii_lowercase().contains("bubblewrap")
        || trimmed.contains("sandbox-exec")
    {
        // Drop opaque policy/wrapper lines rather than paraphrasing unknown text.
        return String::new();
    }
    line.to_string()
}

fn rewrite_sandbox_exec_rest(rest: &str) -> String {
    // sandbox-exec: execvp() of 'python' failed: No such file or directory
    if let Some(of_part) = rest.strip_prefix("execvp()") {
        if let Some(prog_and_rest) = of_part.trim_start().strip_prefix("of '") {
            if let Some((prog, after_prog)) = prog_and_rest.split_once('\'') {
                let reason = after_prog
                    .trim_start()
                    .strip_prefix("failed:")
                    .map(str::trim_start)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("command failed");
                return format!("{prog}: {reason}");
            }
        }
        return "command not found".to_string();
    }

    // Other sandbox-exec messages: drop the wrapper name, keep the diagnostic.
    if rest.is_empty() {
        String::new()
    } else {
        rest.to_string()
    }
}

/// Minimal Seatbelt profile: allow read of system, read/write only under workspace.
#[cfg(target_os = "macos")]
fn seatbelt_profile(workspace_root: &Path, allow_network: bool) -> String {
    // Seatbelt uses literal paths; escape backslashes and quotes.
    let root = workspace_root.display().to_string();
    let escaped = root.replace('\\', "\\\\").replace('"', "\\\"");
    let network = if allow_network {
        "(allow network*)"
    } else {
        "(deny network*)"
    };
    format!(
        r#"(version 1)
(deny default)
(allow process-exec)
(allow process-fork)
(allow signal)
(allow sysctl-read)
(allow mach-lookup)
(allow file-read*)
(allow file-write* (subpath "{escaped}"))
(allow file-write* (literal "/dev/null"))
(allow file-write* (literal "/dev/dtracehelper"))
(allow file-ioctl)
{network}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorkspaceSandbox;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn path_only_plan_has_no_seatbelt_profile() {
        let temp = tempdir().expect("temp");
        let root = temp.path().join("ws");
        fs::create_dir(&root).expect("ws");
        let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
        let plan = ProcessSandboxPlan::for_workspace(&sandbox, ProcessSandboxMode::PathOnly, false);
        assert_eq!(plan.profile_label, "workspace");
        assert!(plan.seatbelt_profile.is_none());
    }

    #[test]
    fn sanitize_rewrites_sandbox_exec_execvp_failures() {
        let raw = "sandbox-exec: execvp() of 'python' failed: No such file or directory\n";
        assert_eq!(
            sanitize_command_stderr(raw),
            "python: No such file or directory\n"
        );
        assert_eq!(
            sanitize_command_stderr("hello from program\n"),
            "hello from program\n"
        );
        assert_eq!(
            sanitize_command_stderr(
                "sandbox-exec: execvp() of 'foo' failed: No such file or directory\nreal line\n"
            ),
            "foo: No such file or directory\nreal line\n"
        );
        assert_eq!(
            sanitize_execution_error("failed to spawn sandbox-exec: No such file or directory"),
            "failed to start command: program not found"
        );
    }

    #[test]
    fn prefer_seatbelt_on_macos_emits_profile() {
        let temp = tempdir().expect("temp");
        let root = temp.path().join("ws");
        fs::create_dir(&root).expect("ws");
        let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
        let plan =
            ProcessSandboxPlan::for_workspace(&sandbox, ProcessSandboxMode::PreferSeatbelt, false);
        #[cfg(target_os = "macos")]
        {
            assert!(plan.seatbelt_profile.is_some());
            assert_eq!(plan.profile_label, "workspace_process");
            assert!(!plan.profile_label.contains("seatbelt"));
            let profile = plan.seatbelt_profile.as_deref().unwrap();
            assert!(profile.contains("subpath"));
            assert!(profile.contains("deny network"));
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(plan.seatbelt_profile.is_none());
            assert_eq!(plan.profile_label, "workspace");
        }
    }
}

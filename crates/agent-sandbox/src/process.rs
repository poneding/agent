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
                profile_label: "workspace_path",
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
                            "workspace_seatbelt_net"
                        } else {
                            "workspace_seatbelt"
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
                        profile_label: "workspace_path",
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
        let plan = ProcessSandboxPlan::for_workspace(
            &sandbox,
            ProcessSandboxMode::PathOnly,
            false,
        );
        assert_eq!(plan.profile_label, "workspace_path");
        assert!(plan.seatbelt_profile.is_none());
    }

    #[test]
    fn prefer_seatbelt_on_macos_emits_profile() {
        let temp = tempdir().expect("temp");
        let root = temp.path().join("ws");
        fs::create_dir(&root).expect("ws");
        let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
        let plan = ProcessSandboxPlan::for_workspace(
            &sandbox,
            ProcessSandboxMode::PreferSeatbelt,
            false,
        );
        #[cfg(target_os = "macos")]
        {
            assert!(plan.seatbelt_profile.is_some());
            assert!(plan.profile_label.contains("seatbelt"));
            let profile = plan.seatbelt_profile.as_deref().unwrap();
            assert!(profile.contains("subpath"));
            assert!(profile.contains("deny network"));
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(plan.seatbelt_profile.is_none());
            assert_eq!(plan.profile_label, "workspace_path");
        }
    }
}

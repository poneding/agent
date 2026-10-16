#![forbid(unsafe_code)]

use std::io::IsTerminal;

fn main() {
    match ruse_cli::parse_cli_invocation(std::env::args().skip(1)) {
        Ok(invocation) => {
            let stdio = ruse_cli::StdioProfile::new(
                std::io::stdin().is_terminal(),
                std::io::stdout().is_terminal(),
            );
            let should_run_plain_repl = invocation.output_mode == ruse_cli::OutputMode::Text
                && invocation.command == ruse_cli::CliCommand::PlainRepl;
            if should_run_plain_repl {
                let status_code = ruse_cli::run_plain_repl_stdio(
                    invocation.config_path,
                    invocation.config_override,
                );
                if status_code != 0 {
                    std::process::exit(status_code);
                }
                return;
            }
            let should_run_terminal_app = invocation.output_mode == ruse_cli::OutputMode::Text
                && (invocation.command == ruse_cli::CliCommand::TuiShell
                    || invocation.command == ruse_cli::CliCommand::TtyShell
                    || (invocation.command == ruse_cli::CliCommand::RuntimeBanner
                        && stdio.is_interactive()));
            if should_run_terminal_app {
                let status_code = ruse_cli::run_terminal_app_stdio(
                    invocation.config_path,
                    invocation.config_override,
                    invocation.permission_mode,
                );
                if status_code != 0 {
                    std::process::exit(status_code);
                }
                return;
            }
            if let ruse_cli::CliCommand::McpServer(ruse_cli::McpServerCommand::Stdio {
                max_frames,
                profile_path,
                profile_name,
                workspace_root,
            }) = &invocation.command
            {
                if invocation.output_mode != ruse_cli::OutputMode::Text {
                    eprintln!("mcp server stdio does not support --json");
                    std::process::exit(2);
                }
                let mut config = ruse_cli::McpServerStdioConfig::new(*max_frames);
                if let Some(profile_path) = profile_path {
                    config = config.with_profile(profile_path.clone(), profile_name.clone());
                }
                if let Some(workspace_root) = workspace_root {
                    config = config.with_workspace(workspace_root.clone());
                }
                let status_code = ruse_cli::run_mcp_server_stdio(config);
                if status_code != 0 {
                    std::process::exit(status_code);
                }
                return;
            }

            let output = ruse_cli::render_cli_invocation_with_stdio(invocation, stdio);
            print!("{}", output.stdout);
            if output.status_code != 0 {
                std::process::exit(output.status_code);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

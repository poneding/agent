#![forbid(unsafe_code)]

use std::io::IsTerminal;

fn main() {
    match agent_cli::parse_cli_invocation(std::env::args().skip(1)) {
        Ok(invocation) => {
            if invocation.command == agent_cli::CliCommand::PlainRepl
                && invocation.output_mode == agent_cli::OutputMode::Text
            {
                let status_code = agent_cli::run_plain_repl_stdio(
                    invocation.config_path,
                    invocation.config_override,
                );
                if status_code != 0 {
                    std::process::exit(status_code);
                }
                return;
            }
            if let agent_cli::CliCommand::McpServer(agent_cli::McpServerCommand::Stdio {
                max_frames,
                profile_path,
                profile_name,
                workspace_root,
            }) = &invocation.command
            {
                if invocation.output_mode != agent_cli::OutputMode::Text {
                    eprintln!("mcp server stdio does not support --json");
                    std::process::exit(2);
                }
                let mut config = agent_cli::McpServerStdioConfig::new(*max_frames);
                if let Some(profile_path) = profile_path {
                    config = config.with_profile(profile_path.clone(), profile_name.clone());
                }
                if let Some(workspace_root) = workspace_root {
                    config = config.with_workspace(workspace_root.clone());
                }
                let status_code = agent_cli::run_mcp_server_stdio(config);
                if status_code != 0 {
                    std::process::exit(status_code);
                }
                return;
            }

            let output = agent_cli::render_cli_invocation_with_stdio(
                invocation,
                agent_cli::StdioProfile::new(
                    std::io::stdin().is_terminal(),
                    std::io::stdout().is_terminal(),
                ),
            );
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

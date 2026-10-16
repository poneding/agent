#![forbid(unsafe_code)]
//! CLI command parsing and output contracts for agent.

pub mod approval;
pub mod command;
pub mod output;
pub mod providers;
pub mod repl;

pub use approval::{
    CliApprovalChoice, CliApprovalError, CliApprovalParseError, approval_command_from_input,
    parse_approval_input, render_approval_prompt, render_approval_required_batch,
    render_json_approval_frame,
};
pub use command::{
    ApprovalCliCommand, ApprovalQueueFixture, CliCommand, CliError, CliInvocation,
    CommandRetryCliCommand, CommandRetryRuntime, ConflictCliCommand, ConflictFixture,
    EvalCliCommand, EvalSuiteCliCommand, ExtensionCommand, ExtensionCommandRuntime,
    InteractiveMode, McpServerCommand, McpServerCommandRuntime, McpServerStdioConfig,
    McpServerStdioError, MemoryCommand, MemoryCommandRuntime, ModelSelection, OutputMode,
    PermissionMode, PlanTodoCliCommand, PlanTodoCommandRuntime, PluginCommand,
    PluginCommandRuntime, PluginInstallDiagnostic, PluginInstallReport, RecoveryCliCommand,
    RecoveryCommandRuntime, ReplayCliCommand, RepoSessionCommand, SkillRootCommand,
    SkillRootCommandRuntime, SlashCommand, StdioProfile, TaskCliCommand, TaskCommandRuntime,
    TeamCliCommand, TeamWorkflowCommandRuntime, TeamWorkflowFixture, TerminalSessionFixture,
    handle_slash_command, parse_cli_command, parse_cli_invocation, render_cli_command,
    render_cli_invocation, render_cli_invocation_with_stdio, render_command_retry_command,
    render_conflict_command, render_eval_command, render_extension_command,
    render_mcp_server_command, render_memory_command, render_observability_json_bundle,
    render_observability_summary, render_plan_todo_command, render_plugin_command,
    render_recovery_command, render_replay_command, render_skill_root_command, render_task_command,
    render_task_review_summary, render_team_command, run_mcp_server_stdio, run_mcp_server_stdio_io,
    run_plain_repl_stdio, run_terminal_app_stdio,
};
pub use output::CliOutput;
pub use repl::{
    PlainReplEvent, PlainReplRuntimeError, PlainReplSymbolMode, parse_plain_repl_line,
    render_plain_repl_approval_prompt, render_plain_repl_footer,
    render_plain_repl_footer_with_width, render_plain_repl_footer_with_width_and_symbols,
    render_plain_repl_frame, render_plain_repl_frame_with_symbols,
    render_plain_repl_interactive_frame, render_plain_repl_interactive_frame_with_symbols,
    render_plain_repl_interactive_screen, render_plain_repl_prompt,
    render_plain_repl_prompt_with_width, render_plain_repl_script, render_plain_repl_separator,
    run_plain_repl_io, run_plain_repl_io_with_status, run_plain_repl_io_with_status_mode,
};

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
    CliCommand, CliError, CliInvocation, ExtensionCommand, ExtensionCommandRuntime,
    InteractiveMode, MemoryCommand, MemoryCommandRuntime, ModelSelection, OutputMode,
    PluginCommand, PluginCommandRuntime, PluginInstallDiagnostic, PluginInstallReport,
    SkillRootCommand, SkillRootCommandRuntime, SlashCommand, StdioProfile, TaskCliCommand,
    TaskCommandRuntime, handle_slash_command, parse_cli_command, parse_cli_invocation,
    render_cli_command, render_cli_invocation, render_cli_invocation_with_stdio,
    render_extension_command, render_memory_command, render_plugin_command,
    render_skill_root_command, render_task_command, render_task_review_summary,
    run_plain_repl_stdio,
};
pub use output::CliOutput;
pub use repl::{
    PlainReplEvent, PlainReplRuntimeError, parse_plain_repl_line,
    render_plain_repl_approval_prompt, render_plain_repl_prompt, render_plain_repl_script,
    run_plain_repl_io,
};

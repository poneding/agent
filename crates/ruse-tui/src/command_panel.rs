use ruse_ui_bridge::UiCommandTestSnapshot;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPanelState {
    pub command: String,
    pub cwd: String,
    pub classify_as_test: bool,
    pub status: String,
    pub classification: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_byte_count: u64,
    pub stderr_byte_count: u64,
    pub timed_out: bool,
    pub cancellable: bool,
    pub cancellation_reason: String,
    pub observation: String,
    pub diagnostic_count: u32,
    pub active_output: CommandPanelOutput,
    pub truncated: bool,
    pub redacted: bool,
}

impl CommandPanelState {
    #[must_use]
    pub fn from_snapshot(snapshot: &UiCommandTestSnapshot, max_output_chars: usize) -> Self {
        let max_output_chars = max_output_chars.max(1);
        let mut stdout = snapshot.stdout.clone();
        let mut stderr = snapshot.stderr.clone();
        let stdout_over_limit = stdout.chars().count() > max_output_chars;
        let stderr_over_limit = stderr.chars().count() > max_output_chars;
        if stdout_over_limit {
            stdout = stdout.chars().take(max_output_chars).collect();
        }
        if stderr_over_limit {
            stderr = stderr.chars().take(max_output_chars).collect();
        }

        Self {
            command: snapshot.command.clone(),
            cwd: snapshot.cwd.clone(),
            classify_as_test: snapshot.classify_as_test,
            status: snapshot.status.clone(),
            classification: snapshot.classification.clone(),
            exit_code: snapshot.exit_code,
            stdout,
            stderr,
            stdout_byte_count: snapshot.stdout_byte_count,
            stderr_byte_count: snapshot.stderr_byte_count,
            timed_out: snapshot.timed_out,
            cancellable: snapshot.cancellable,
            cancellation_reason: snapshot.cancellation_reason.clone(),
            observation: snapshot.observation.clone(),
            diagnostic_count: snapshot.diagnostic_count,
            active_output: CommandPanelOutput::Stdout,
            truncated: snapshot.truncated || stdout_over_limit || stderr_over_limit,
            redacted: snapshot.redacted,
        }
    }

    pub fn reduce(&mut self, action: CommandPanelAction) {
        match action {
            CommandPanelAction::ToggleFold => {}
            CommandPanelAction::ShowStdout => self.active_output = CommandPanelOutput::Stdout,
            CommandPanelAction::ShowStderr => self.active_output = CommandPanelOutput::Stderr,
            CommandPanelAction::ToggleOutput => {
                self.active_output = match self.active_output {
                    CommandPanelOutput::Stdout => CommandPanelOutput::Stderr,
                    CommandPanelOutput::Stderr => CommandPanelOutput::Stdout,
                };
            }
        }
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let mode = if self.classify_as_test {
            "test"
        } else {
            "command"
        };
        let cancellable = if self.cancellable { "yes" } else { "no" };
        let mut text = format!(
            "{}\nmode: {mode}\ncwd: {}\nstatus: {}\nclass: {}",
            self.command, self.cwd, self.status, self.classification
        );
        if let Some(exit_code) = self.exit_code {
            text.push_str(&format!("\nexit: {exit_code}"));
        }
        if self.timed_out {
            text.push_str("\ntimed out");
        }
        text.push_str(&format!(
            "\ncancellable: {cancellable}\ncancel: {}",
            self.cancellation_reason
        ));
        text.push_str(&format!(
            "\nstdout bytes: {}\nstderr bytes: {}",
            self.stdout_byte_count, self.stderr_byte_count
        ));
        if !self.observation.is_empty() {
            text.push_str(&format!("\nobservation: {}", self.observation));
        }
        match self.active_output {
            CommandPanelOutput::Stdout => {
                if !self.stdout.is_empty() {
                    text.push_str(&format!("\nstdout:\n{}", self.stdout));
                }
            }
            CommandPanelOutput::Stderr => {
                if !self.stderr.is_empty() {
                    text.push_str(&format!("\nstderr:\n{}", self.stderr));
                }
            }
        }
        if self.diagnostic_count > 0 {
            text.push_str(&format!("\ndiagnostics={}", self.diagnostic_count));
        }
        if self.truncated {
            text.push_str("\ntruncated");
        }
        if self.redacted {
            text.push_str("\nredacted");
        }
        text
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CommandPanelOutput {
    Stdout,
    Stderr,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CommandPanelAction {
    ToggleFold,
    ShowStdout,
    ShowStderr,
    ToggleOutput,
}

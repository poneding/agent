use agent_ui_bridge::UiCommandTestSnapshot;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPanelState {
    pub command: String,
    pub status: String,
    pub classification: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub cancellable: bool,
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
            status: snapshot.status.clone(),
            classification: snapshot.classification.clone(),
            exit_code: snapshot.exit_code,
            stdout,
            stderr,
            timed_out: snapshot.timed_out,
            cancellable: snapshot.cancellable,
            truncated: snapshot.truncated || stdout_over_limit || stderr_over_limit,
            redacted: snapshot.redacted,
        }
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let mut text = format!(
            "{}\nstatus: {}\nclass: {}",
            self.command, self.status, self.classification
        );
        if let Some(exit_code) = self.exit_code {
            text.push_str(&format!("\nexit: {exit_code}"));
        }
        if self.timed_out {
            text.push_str("\ntimed out");
        }
        if self.cancellable {
            text.push_str("\ncancellable");
        }
        if !self.stdout.is_empty() {
            text.push_str(&format!("\nstdout:\n{}", self.stdout));
        }
        if !self.stderr.is_empty() {
            text.push_str(&format!("\nstderr:\n{}", self.stderr));
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

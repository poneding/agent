#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliOutput {
    pub stdout: String,
    pub exit: bool,
}

impl CliOutput {
    #[must_use]
    pub fn exit(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            exit: true,
        }
    }

    #[must_use]
    pub fn keep_running(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            exit: false,
        }
    }
}

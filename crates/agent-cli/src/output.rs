#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliOutput {
    pub stdout: String,
    pub exit: bool,
    pub status_code: i32,
}

impl CliOutput {
    #[must_use]
    pub fn exit(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            exit: true,
            status_code: 0,
        }
    }

    #[must_use]
    pub fn failure(stdout: impl Into<String>, status_code: i32) -> Self {
        Self {
            stdout: stdout.into(),
            exit: true,
            status_code,
        }
    }

    #[must_use]
    pub fn keep_running(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            exit: false,
            status_code: 0,
        }
    }
}

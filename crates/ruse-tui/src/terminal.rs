use std::io;

use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TerminalModeError {
    #[error("terminal mode operation failed: {0}")]
    Io(String),
}

impl From<io::Error> for TerminalModeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

pub trait TerminalModeBackend {
    fn enable_raw_mode(&mut self) -> Result<(), TerminalModeError>;
    fn enter_alternate_screen(&mut self) -> Result<(), TerminalModeError>;
    fn enable_mouse_capture(&mut self) -> Result<(), TerminalModeError>;
    fn disable_mouse_capture(&mut self) -> Result<(), TerminalModeError>;
    /// Ask the host to wrap paste payloads in bracketed-paste CSI so multi-line
    /// pastes arrive as a single `Event::Paste` instead of a stream of key
    /// events (where `\n` would otherwise look like Enter → submit, or get
    /// flattened into one line).
    fn enable_bracketed_paste(&mut self) -> Result<(), TerminalModeError>;
    fn disable_bracketed_paste(&mut self) -> Result<(), TerminalModeError>;
    /// Request Kitty progressive keyboard enhancements so modified keys such as
    /// Cmd/Super+C can be delivered as distinct events (when the host supports it).
    fn enable_keyboard_enhancement(&mut self) -> Result<(), TerminalModeError>;
    fn disable_keyboard_enhancement(&mut self) -> Result<(), TerminalModeError>;
    fn leave_alternate_screen(&mut self) -> Result<(), TerminalModeError>;
    fn disable_raw_mode(&mut self) -> Result<(), TerminalModeError>;
}

pub struct TerminalModeGuard<B>
where
    B: TerminalModeBackend,
{
    backend: Option<B>,
    restored: bool,
}

impl<B> TerminalModeGuard<B>
where
    B: TerminalModeBackend,
{
    pub fn enter(mut backend: B) -> Result<Self, TerminalModeError> {
        backend.enable_raw_mode()?;
        backend.enter_alternate_screen()?;
        backend.enable_mouse_capture()?;
        // Bracketed paste must be on for multi-line pastes to stay multi-line:
        // without it, hosts inject raw key events and newlines are lost or act
        // as Enter (submit). Best-effort — unsupported hosts ignore the CSI.
        backend.enable_bracketed_paste()?;
        // Best-effort: unsupported hosts ignore the CSI and we still restore on exit.
        backend.enable_keyboard_enhancement()?;
        Ok(Self {
            backend: Some(backend),
            restored: false,
        })
    }

    pub fn restore(&mut self) -> Result<(), TerminalModeError> {
        if self.restored {
            return Ok(());
        }
        if let Some(backend) = self.backend.as_mut() {
            backend.disable_keyboard_enhancement()?;
            backend.disable_bracketed_paste()?;
            backend.disable_mouse_capture()?;
            backend.leave_alternate_screen()?;
            backend.disable_raw_mode()?;
        }
        self.restored = true;
        Ok(())
    }

    pub fn into_inner_restoring(mut self) -> Result<B, TerminalModeError> {
        self.restore()?;
        self.restored = true;
        Ok(self
            .backend
            .take()
            .expect("terminal mode guard always owns a backend"))
    }

    #[must_use]
    pub fn backend(&self) -> &B {
        self.backend
            .as_ref()
            .expect("terminal mode guard always owns a backend")
    }

    pub fn backend_mut(&mut self) -> &mut B {
        self.backend
            .as_mut()
            .expect("terminal mode guard always owns a backend")
    }
}

impl<B> Drop for TerminalModeGuard<B>
where
    B: TerminalModeBackend,
{
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[derive(Default)]
pub struct CrosstermTerminalBackend;

impl TerminalModeBackend for CrosstermTerminalBackend {
    fn enable_raw_mode(&mut self) -> Result<(), TerminalModeError> {
        enable_raw_mode().map_err(TerminalModeError::from)
    }

    fn enter_alternate_screen(&mut self) -> Result<(), TerminalModeError> {
        execute!(io::stdout(), EnterAlternateScreen).map_err(TerminalModeError::from)
    }

    fn enable_mouse_capture(&mut self) -> Result<(), TerminalModeError> {
        execute!(io::stdout(), EnableMouseCapture).map_err(TerminalModeError::from)
    }

    fn disable_mouse_capture(&mut self) -> Result<(), TerminalModeError> {
        execute!(io::stdout(), DisableMouseCapture).map_err(TerminalModeError::from)
    }

    fn enable_bracketed_paste(&mut self) -> Result<(), TerminalModeError> {
        execute!(io::stdout(), EnableBracketedPaste).map_err(TerminalModeError::from)
    }

    fn disable_bracketed_paste(&mut self) -> Result<(), TerminalModeError> {
        execute!(io::stdout(), DisableBracketedPaste).map_err(TerminalModeError::from)
    }

    fn enable_keyboard_enhancement(&mut self) -> Result<(), TerminalModeError> {
        // DISAMBIGUATE alone is enough for Super/Cmd-modified letters on Kitty-
        // protocol terminals (Ghostty, Kitty, WezTerm, recent iTerm2). REPORT_ALL
        // is intentionally omitted — it rewrites every plain key as CSI-u and is
        // more likely to break hosts that only partially implement the protocol.
        execute!(
            io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )
        .map_err(TerminalModeError::from)
    }

    fn disable_keyboard_enhancement(&mut self) -> Result<(), TerminalModeError> {
        execute!(io::stdout(), PopKeyboardEnhancementFlags).map_err(TerminalModeError::from)
    }

    fn leave_alternate_screen(&mut self) -> Result<(), TerminalModeError> {
        execute!(io::stdout(), LeaveAlternateScreen).map_err(TerminalModeError::from)
    }

    fn disable_raw_mode(&mut self) -> Result<(), TerminalModeError> {
        disable_raw_mode().map_err(TerminalModeError::from)
    }
}

#[derive(Default)]
pub struct FixtureTerminalBackend {
    calls: Vec<&'static str>,
}

impl FixtureTerminalBackend {
    #[must_use]
    pub fn calls(&self) -> &[&'static str] {
        &self.calls
    }

    pub fn clear_calls(&mut self) {
        self.calls.clear();
    }

    fn push(&mut self, call: &'static str) {
        self.calls.push(call);
    }
}

impl TerminalModeBackend for FixtureTerminalBackend {
    fn enable_raw_mode(&mut self) -> Result<(), TerminalModeError> {
        self.push("enable_raw_mode");
        Ok(())
    }

    fn enter_alternate_screen(&mut self) -> Result<(), TerminalModeError> {
        self.push("enter_alternate_screen");
        Ok(())
    }

    fn enable_mouse_capture(&mut self) -> Result<(), TerminalModeError> {
        self.push("enable_mouse_capture");
        Ok(())
    }

    fn disable_mouse_capture(&mut self) -> Result<(), TerminalModeError> {
        self.push("disable_mouse_capture");
        Ok(())
    }

    fn enable_bracketed_paste(&mut self) -> Result<(), TerminalModeError> {
        self.push("enable_bracketed_paste");
        Ok(())
    }

    fn disable_bracketed_paste(&mut self) -> Result<(), TerminalModeError> {
        self.push("disable_bracketed_paste");
        Ok(())
    }

    fn enable_keyboard_enhancement(&mut self) -> Result<(), TerminalModeError> {
        self.push("enable_keyboard_enhancement");
        Ok(())
    }

    fn disable_keyboard_enhancement(&mut self) -> Result<(), TerminalModeError> {
        self.push("disable_keyboard_enhancement");
        Ok(())
    }

    fn leave_alternate_screen(&mut self) -> Result<(), TerminalModeError> {
        self.push("leave_alternate_screen");
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> Result<(), TerminalModeError> {
        self.push("disable_raw_mode");
        Ok(())
    }
}

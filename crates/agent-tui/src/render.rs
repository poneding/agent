use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};

use crate::{
    ApprovalPanelState, ComposerState, DiffPanelState, TaskPanelState, TracePanelState,
    TranscriptState, TuiApp, TuiScreen, TuiSidePanel,
};

pub fn render_app(frame: &mut Frame<'_>, app: &TuiApp) {
    match app.screen {
        TuiScreen::Chat => render_chat(frame, app),
        TuiScreen::Help => render_help(frame, app),
    }
}

pub fn render_app_to_string(app: &TuiApp, width: u16, height: u16) -> std::io::Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render_app(frame, app))?;
    Ok(buffer_to_string(terminal.backend().buffer()))
}

fn render_chat(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    if area.width >= 60 {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20), Constraint::Length(24)])
            .split(vertical[0]);
        render_transcript(frame, &app.transcript, horizontal[0]);
        render_panel(frame, app, horizontal[1]);
    } else {
        render_transcript(frame, &app.transcript, vertical[0]);
    }

    render_composer(frame, &app.composer, vertical[1]);
    render_status(frame, app, vertical[2]);
}

fn render_help(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);
    let help = Paragraph::new("agent Help\nEsc returns to chat\nq quits")
        .block(Block::default().title("Help").borders(Borders::ALL));
    frame.render_widget(help, vertical[0]);
    render_status(frame, app, vertical[1]);
}

fn render_transcript(frame: &mut Frame<'_>, transcript: &TranscriptState, area: Rect) {
    let text = if transcript.items.is_empty() {
        "No messages yet".to_string()
    } else {
        transcript
            .items
            .iter()
            .map(|item| item.render_line())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let transcript =
        Paragraph::new(text).block(Block::default().title("Transcript").borders(Borders::ALL));
    frame.render_widget(transcript, area);
}

fn render_panel(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let (title, text) = match app.side_panel {
        TuiSidePanel::Trace => ("Trace", render_trace_panel(&app.trace_panel)),
        TuiSidePanel::Tasks => ("Tasks", render_task_panel(&app.task_panel)),
        TuiSidePanel::Auto => {
            if let Some(diff) = &app.diff_panel {
                ("Diff", render_diff_panel(diff))
            } else if !app.approval_panel.pending.is_empty() {
                ("Approval", render_approval_panel(&app.approval_panel))
            } else {
                ("Panel", "No active panel".to_string())
            }
        }
    };
    let panel = Paragraph::new(text).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(panel, area);
}

fn render_approval_panel(panel: &ApprovalPanelState) -> String {
    let Some(request) = panel.focused_request() else {
        return "No focused approval".to_string();
    };

    let mut text = format!(
        "{} {}\n{}\n{}\n{}",
        request.action,
        request.resource_kind,
        request.resource,
        request.reason,
        request.preview.summary
    );
    if let Some(content_hash) = &request.preview.content_hash {
        text.push_str(&format!("\n{content_hash}"));
    }
    if request.preview.truncated {
        text.push_str("\ntruncated");
    }
    if request.preview.redacted {
        text.push_str("\nredacted");
    }
    text
}

fn render_diff_panel(diff: &DiffPanelState) -> String {
    diff.render_text()
}

fn render_trace_panel(panel: &TracePanelState) -> String {
    panel.render_text()
}

fn render_task_panel(panel: &TaskPanelState) -> String {
    panel.render_text()
}

fn render_composer(frame: &mut Frame<'_>, composer: &ComposerState, area: Rect) {
    let text = if composer.text.is_empty() {
        "Type a message"
    } else {
        composer.text.as_str()
    };
    let composer =
        Paragraph::new(text).block(Block::default().title("Composer").borders(Borders::ALL));
    frame.render_widget(composer, area);
}

fn render_status(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let status = format!(
        "agent | {}/{} | {} | {} | {} | {}",
        app.status.provider,
        app.status.model,
        app.status.workspace,
        app.status.mode,
        app.status.connection,
        app.status.permission_state
    );
    frame.render_widget(Paragraph::new(status), area);
}

fn buffer_to_string(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut output = String::new();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            output.push_str(buffer[(x, y)].symbol());
        }
        output.push('\n');
    }
    output
}

//! Thought Stream Widget
//!
//! Displays AI processing steps and progress in a dynamic, engaging way.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::tui::{
    app::{App, ThoughtItem, ThoughtStatus},
    theme::{colors, styles},
};

/// Render the AI thought stream panel
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Processing ", styles::title()))
        .borders(Borders::ALL)
        .border_style(if app.ai_state.is_processing {
            styles::border_focused()
        } else {
            styles::border()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Task header
            Constraint::Length(2), // Progress bar
            Constraint::Min(4),    // Thought list
        ])
        .margin(1)
        .split(inner);

    render_task_header(frame, app, chunks[0]);
    render_progress(frame, app, chunks[1]);
    render_thoughts(frame, app, chunks[2]);
}

fn render_task_header(frame: &mut Frame, app: &App, area: Rect) {
    let task_name = app
        .ai_state
        .current_task
        .as_deref()
        .unwrap_or("Ready");

    let elapsed = app
        .ai_state
        .start_time
        .map(|t| format!(" ({:.1}s)", t.elapsed().as_secs_f64()))
        .unwrap_or_default();

    let status_icon = if app.ai_state.is_processing {
        Span::styled(
            format!("{} ", app.spinner()),
            Style::default().fg(colors::ACCENT_CYAN),
        )
    } else if app.ai_state.progress == Some(1.0) {
        Span::styled("✓ ", styles::success())
    } else {
        Span::styled("○ ", styles::subtitle())
    };

    let line = Line::from(vec![
        status_icon,
        Span::styled(task_name, styles::value()),
        Span::styled(elapsed, styles::subtitle()),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn render_progress(frame: &mut Frame, app: &App, area: Rect) {
    let progress = app.ai_state.progress.unwrap_or(0.0);
    let percentage = (progress * 100.0) as u16;

    let label = format!("{}%", percentage);

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(if progress >= 1.0 {
                    colors::SUCCESS
                } else {
                    colors::PRIMARY
                })
                .bg(colors::SURFACE),
        )
        .label(Span::styled(label, styles::value()))
        .ratio(progress);

    frame.render_widget(gauge, area);
}

fn render_thoughts(frame: &mut Frame, app: &App, area: Rect) {
    if app.ai_state.thoughts.is_empty() {
        let placeholder = Paragraph::new(Line::from(vec![
            Span::styled("  Waiting for task...", styles::subtitle()),
        ]));
        frame.render_widget(placeholder, area);
        return;
    }

    // Show last N thoughts that fit in the area
    let max_thoughts = area.height as usize;
    let thoughts: Vec<Line> = app
        .ai_state
        .thoughts
        .iter()
        .rev()
        .take(max_thoughts)
        .rev()
        .map(|thought| format_thought(thought, app))
        .collect();

    let paragraph = Paragraph::new(thoughts);
    frame.render_widget(paragraph, area);
}

fn format_thought(thought: &ThoughtItem, app: &App) -> Line<'static> {
    let (status_char, status_style) = match thought.status {
        ThoughtStatus::Pending => ('○', styles::subtitle()),
        ThoughtStatus::InProgress => (
            app.spinner(),
            Style::default().fg(colors::ACCENT_CYAN),
        ),
        ThoughtStatus::Complete => ('✓', styles::success()),
        ThoughtStatus::Error => ('✗', styles::error()),
    };

    let elapsed = thought.timestamp.elapsed();
    let time_str = if elapsed.as_secs() >= 60 {
        format!("{}m", elapsed.as_secs() / 60)
    } else if elapsed.as_millis() >= 1000 {
        format!("{:.1}s", elapsed.as_secs_f64())
    } else {
        format!("{}ms", elapsed.as_millis())
    };

    Line::from(vec![
        Span::raw("  "),
        Span::styled(status_char.to_string(), status_style),
        Span::raw(" "),
        Span::styled(
            thought.icon.to_string(),
            Style::default().fg(colors::TEXT_DIM),
        ),
        Span::raw(" "),
        Span::styled(thought.text.clone(), styles::value()),
        Span::raw(" "),
        Span::styled(time_str, styles::subtitle()),
    ])
}

/// Render a compact inline progress indicator (for non-TUI mode)
pub fn render_inline(app: &App) -> String {
    let spinner = if app.ai_state.is_processing {
        app.spinner().to_string()
    } else {
        "✓".to_string()
    };

    let task = app
        .ai_state
        .current_task
        .as_deref()
        .unwrap_or("Done");

    let progress = app.ai_state.progress.unwrap_or(0.0);
    let bar_width = 20;
    let filled = (progress * bar_width as f64) as usize;
    let bar = format!(
        "[{}{}]",
        "█".repeat(filled),
        "░".repeat(bar_width - filled)
    );

    format!("{} {} {} {:.0}%", spinner, task, bar, progress * 100.0)
}

/// Icons for different AI operations
pub mod icons {
    pub const ANALYZE: char = '🔍';
    pub const PARSE: char = '📄';
    pub const CALCULATE: char = '📊';
    pub const VALIDATE: char = '✅';
    pub const SEARCH: char = '🔎';
    pub const THINK: char = '💭';
    pub const WRITE: char = '✏';
    pub const LOAD: char = '📥';
    pub const SAVE: char = '💾';
    pub const NETWORK: char = '🌐';
    pub const ERROR: char = '❌';
    pub const SUCCESS: char = '✨';
    pub const WARNING: char = '⚠';
    pub const INFO: char = 'ℹ';
    pub const CLOCK: char = '⏱';
    pub const GEAR: char = '⚙';
    pub const BRAIN: char = '🧠';
}

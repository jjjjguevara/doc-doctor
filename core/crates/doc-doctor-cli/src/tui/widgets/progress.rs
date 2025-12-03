//! Progress Widget
//!
//! Progress bars and batch processing visualization.

use std::time::Duration;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{
    app::{App, BatchState},
    theme::{colors, styles},
};

/// Render batch processing progress panel
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Batch Processing ", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(batch) = &app.batch_state {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Progress bar
                Constraint::Length(3), // Stats
                Constraint::Length(2), // Current file
                Constraint::Min(4),    // Errors (if any)
            ])
            .margin(1)
            .split(inner);

        render_progress_bar(frame, batch, app, chunks[0]);
        render_stats(frame, batch, chunks[1]);
        render_current_file(frame, batch, app, chunks[2]);
        render_errors(frame, batch, chunks[3]);
    } else {
        let placeholder = Paragraph::new("No batch operation in progress")
            .style(styles::subtitle());
        frame.render_widget(placeholder, inner);
    }
}

fn render_progress_bar(frame: &mut Frame, batch: &BatchState, _app: &App, area: Rect) {
    let progress = batch.progress();
    let percentage = (progress * 100.0) as u16;

    let label = format!(
        "{}/{} ({}%)",
        batch.processed, batch.total, percentage
    );

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(if batch.failed > 0 {
                    colors::WARNING
                } else if progress >= 1.0 {
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

fn render_stats(frame: &mut Frame, batch: &BatchState, area: Rect) {
    let elapsed = format_duration(batch.elapsed());
    let eta = batch
        .eta()
        .map(format_duration)
        .unwrap_or_else(|| "calculating...".to_string());

    let rate = if batch.elapsed().as_secs() > 0 {
        batch.processed as f64 / batch.elapsed().as_secs_f64()
    } else {
        0.0
    };

    let stats = Line::from(vec![
        Span::styled("✓ ", styles::success()),
        Span::styled(batch.succeeded.to_string(), styles::success()),
        Span::raw("  "),
        Span::styled("✗ ", styles::error()),
        Span::styled(batch.failed.to_string(), styles::error()),
        Span::raw("  │  "),
        Span::styled("⏱ ", styles::subtitle()),
        Span::styled(elapsed, styles::value()),
        Span::raw("  "),
        Span::styled("ETA: ", styles::subtitle()),
        Span::styled(eta, styles::value()),
        Span::raw("  "),
        Span::styled(format!("({:.1}/s)", rate), styles::subtitle()),
    ]);

    let paragraph = Paragraph::new(stats);
    frame.render_widget(paragraph, area);
}

fn render_current_file(frame: &mut Frame, batch: &BatchState, app: &App, area: Rect) {
    let current = batch
        .current_file
        .as_deref()
        .unwrap_or("Waiting...");

    let line = Line::from(vec![
        Span::styled(app.spinner().to_string(), Style::default().fg(colors::ACCENT_CYAN)),
        Span::raw(" "),
        Span::styled("Processing: ", styles::label()),
        Span::styled(truncate_path(current, 60), styles::value()),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn render_errors(frame: &mut Frame, batch: &BatchState, area: Rect) {
    if batch.errors.is_empty() {
        return;
    }

    let block = Block::default()
        .title(Span::styled(
            format!(" Errors ({}) ", batch.errors.len()),
            styles::error(),
        ))
        .borders(Borders::TOP)
        .border_style(styles::border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = batch
        .errors
        .iter()
        .rev()
        .take(inner.height as usize)
        .map(|(file, error)| {
            ListItem::new(Line::from(vec![
                Span::styled("✗ ", styles::error()),
                Span::styled(truncate_path(file, 30), styles::value()),
                Span::styled(": ", styles::subtitle()),
                Span::styled(error.clone(), styles::error()),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Format duration as human-readable string
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs > 0 {
        format!("{}s", secs)
    } else {
        format!("{}ms", d.as_millis())
    }
}

/// Truncate path for display (UTF-8 aware)
fn truncate_path(path: &str, max_chars: usize) -> String {
    let char_count = path.chars().count();
    if char_count <= max_chars {
        path.to_string()
    } else {
        let skip = char_count - max_chars + 3;
        let truncated: String = path.chars().skip(skip).collect();
        format!("...{}", truncated)
    }
}

/// Console progress bar for non-TUI mode
pub struct ConsoleProgress {
    bar: indicatif::ProgressBar,
}

impl ConsoleProgress {
    pub fn new(total: u64, message: &str) -> Self {
        let bar = indicatif::ProgressBar::new(total);
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.cyan} {msg}\n{wide_bar:.cyan/dim} {pos}/{len} ({percent}%) ETA: {eta}")
                .unwrap()
                .progress_chars("█▓▒░"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));
        Self { bar }
    }

    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    pub fn abandon_with_message(&self, message: &str) {
        self.bar.abandon_with_message(message.to_string());
    }
}

/// Multi-progress bar for parallel operations
pub struct MultiProgress {
    multi: indicatif::MultiProgress,
}

impl MultiProgress {
    pub fn new() -> Self {
        Self {
            multi: indicatif::MultiProgress::new(),
        }
    }

    pub fn add(&self, total: u64, message: &str) -> ConsoleProgress {
        let bar = indicatif::ProgressBar::new(total);
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("  {spinner:.cyan} {msg} [{wide_bar:.cyan/dim}] {pos}/{len}")
                .unwrap()
                .progress_chars("█▓▒░"),
        );
        bar.set_message(message.to_string());
        let bar = self.multi.add(bar);
        ConsoleProgress { bar }
    }

    pub fn add_spinner(&self, message: &str) -> indicatif::ProgressBar {
        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("  {spinner:.cyan} {msg}")
                .unwrap(),
        );
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(Duration::from_millis(80));
        self.multi.add(spinner)
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

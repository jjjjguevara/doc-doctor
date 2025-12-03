//! Document Viewer Widget
//!
//! Displays detailed document analysis with L1 properties and L2 dimensions.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::{
    app::{App, DocumentSummary},
    theme::{health_color, health_style, styles, audience_color},
};

/// Render the document viewer
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Content
            Constraint::Length(3),  // Footer
        ])
        .split(area);

    render_header(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let doc = get_selected_document(app);

    let title = doc
        .map(|d| d.title.clone().unwrap_or_else(|| d.path.file_name().unwrap().to_string_lossy().to_string()))
        .unwrap_or_else(|| "No document selected".to_string());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(&title, styles::title()),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(styles::border()),
    );

    frame.render_widget(header, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(styles::border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(doc) = get_selected_document(app) else {
        let empty = Paragraph::new("No document selected. Press 'd' to return to dashboard.")
            .style(styles::subtitle());
        frame.render_widget(empty, inner);
        return;
    };

    // Split into two columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    render_properties(frame, doc, columns[0]);
    render_dimensions(frame, doc, app, columns[1]);
}

fn render_properties(frame: &mut Frame, doc: &DocumentSummary, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" L1 Properties ", styles::label()))
        .borders(Borders::RIGHT)
        .border_style(styles::border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Path: ", styles::label()),
            Span::styled(
                truncate_path(&doc.path.display().to_string(), 35),
                styles::subtitle(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Refinement: ", styles::label()),
            Span::styled(
                format!("{:.0}%", doc.refinement * 100.0),
                health_style(doc.refinement),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Audience: ", styles::label()),
            Span::styled(
                &doc.audience,
                Style::default().fg(audience_color(&doc.audience)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Form: ", styles::label()),
            Span::styled(&doc.form, styles::value()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Stubs: ", styles::label()),
            if doc.stub_count > 0 {
                Span::styled(
                    doc.stub_count.to_string(),
                    styles::warning(),
                )
            } else {
                Span::styled(
                    doc.stub_count.to_string(),
                    styles::success(),
                )
            },
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_dimensions(frame: &mut Frame, doc: &DocumentSummary, _app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" L2 Dimensions ", styles::label()))
        .borders(Borders::NONE);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Health bar visualization
    let health_bar = create_health_bar(doc.health);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Health Score", styles::label()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(
                format!("{:.0}%", doc.health * 100.0),
                Style::default()
                    .fg(health_color(doc.health))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(health_bar, health_style(doc.health)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Calculation Breakdown:", styles::label()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Refinement: ", styles::subtitle()),
            Span::styled(
                format!("{:.2} × 0.70 = {:.2}", doc.refinement, doc.refinement * 0.7),
                styles::value(),
            ),
        ]),
        Line::from(vec![
            Span::styled("    Stub Factor: ", styles::subtitle()),
            Span::styled(
                format!("{:.2} × 0.30 = {:.2}", 1.0 - stub_penalty(doc), (1.0 - stub_penalty(doc)) * 0.3),
                styles::value(),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_footer(frame: &mut Frame, _app: &App, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("d", styles::highlight()),
        Span::styled(" Dashboard  ", styles::subtitle()),
        Span::styled("s", styles::highlight()),
        Span::styled(" Stubs  ", styles::subtitle()),
        Span::styled("c", styles::highlight()),
        Span::styled(" Content  ", styles::subtitle()),
        Span::styled("↑/↓", styles::highlight()),
        Span::styled(" Navigate  ", styles::subtitle()),
        Span::styled("q", styles::highlight()),
        Span::styled(" Back", styles::subtitle()),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(styles::border()),
    );

    frame.render_widget(footer, area);
}

fn get_selected_document(app: &App) -> Option<&DocumentSummary> {
    app.documents.get(app.selected_document)
}

fn create_health_bar(health: f64) -> String {
    let width = 20;
    let filled = (health * width as f64).round() as usize;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

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

fn stub_penalty(doc: &DocumentSummary) -> f64 {
    // Approximate penalty based on stub count
    // In reality this would use the actual stub forms
    (doc.stub_count as f64 * 0.05).min(1.0)
}

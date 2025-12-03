//! Stub List Widget
//!
//! Displays all stubs for the selected document.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use doc_doctor_domain::StubForm;

use crate::tui::{
    app::{App, DocumentSummary},
    theme::styles,
};

/// Render the stub list view
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
    render_footer(frame, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let doc = get_selected_document(app);

    let title = doc
        .map(|d| {
            let name = d.title.clone().unwrap_or_else(|| {
                d.path.file_name().unwrap().to_string_lossy().to_string()
            });
            format!("Stubs: {} ({} total)", name, d.stubs.len())
        })
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

    if doc.stubs.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("No stubs found in this document.", styles::success()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("This document has no pending gaps or issues.", styles::subtitle()),
            ]),
        ]);
        frame.render_widget(empty, inner);
        return;
    }

    // Create list items for each stub
    let items: Vec<ListItem> = doc.stubs.iter().map(|stub| {
        let form_color = stub_form_color(&stub.stub_form);
        let form_icon = stub_form_icon(&stub.stub_form);

        let lines = vec![
            Line::from(vec![
                Span::styled(format!(" {} ", form_icon), Style::default().fg(form_color)),
                Span::styled(
                    format!("[{}] ", stub.stub_type.as_str()),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    stub.stub_form.display_name(),
                    Style::default().fg(form_color),
                ),
                Span::styled(" | ", styles::border()),
                Span::styled(
                    stub.priority.display_name(),
                    priority_style(&stub.priority),
                ),
            ]),
            Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(&stub.description, styles::value()),
            ]),
            Line::from(""),
        ];

        ListItem::new(lines)
    }).collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("d", styles::highlight()),
        Span::styled(" Dashboard  ", styles::subtitle()),
        Span::styled("Enter", styles::highlight()),
        Span::styled(" Details  ", styles::subtitle()),
        Span::styled("c", styles::highlight()),
        Span::styled(" Content  ", styles::subtitle()),
        Span::styled("q", styles::highlight()),
        Span::styled(" Quit", styles::subtitle()),
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

fn stub_form_color(form: &StubForm) -> Color {
    match form {
        StubForm::Transient => Color::Green,
        StubForm::Persistent => Color::Yellow,
        StubForm::Blocking => Color::Red,
        StubForm::Structural => Color::Magenta,
    }
}

fn stub_form_icon(form: &StubForm) -> &'static str {
    match form {
        StubForm::Transient => "○",
        StubForm::Persistent => "◐",
        StubForm::Blocking => "●",
        StubForm::Structural => "◆",
    }
}

fn priority_style(priority: &doc_doctor_domain::Priority) -> Style {
    match priority {
        doc_doctor_domain::Priority::Low => Style::default().fg(Color::DarkGray),
        doc_doctor_domain::Priority::Medium => Style::default().fg(Color::White),
        doc_doctor_domain::Priority::High => Style::default().fg(Color::Yellow),
        doc_doctor_domain::Priority::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    }
}

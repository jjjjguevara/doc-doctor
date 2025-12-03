//! Content Viewer Widget
//!
//! Displays the raw markdown content of the selected document.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::tui::{
    app::{App, DocumentSummary},
    theme::{colors, health_style, styles},
};

/// Render the content viewer
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(2),  // Metadata
            Constraint::Min(10),    // Content
            Constraint::Length(3),  // Footer
        ])
        .split(area);

    render_header(frame, app, chunks[0]);
    render_metadata(frame, app, chunks[1]);
    render_content(frame, app, chunks[2]);
    render_footer(frame, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let doc = get_selected_document(app);

    let title = doc
        .map(|d| {
            let name = d.title.clone().unwrap_or_else(|| {
                d.path.file_name().unwrap().to_string_lossy().to_string()
            });
            let lines = d.content.lines().count();
            format!("Content: {} ({} lines)", name, lines)
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

fn render_metadata(frame: &mut Frame, app: &App, area: Rect) {
    let Some(doc) = get_selected_document(app) else {
        return;
    };

    let health_pct = format!("{:.0}%", doc.health * 100.0);
    let refine_pct = format!("{:.0}%", doc.refinement * 100.0);

    // Format file size
    let size_str = if doc.file_size < 1024 {
        format!("{}B", doc.file_size)
    } else if doc.file_size < 1024 * 1024 {
        format!("{:.1}K", doc.file_size as f64 / 1024.0)
    } else {
        format!("{:.1}M", doc.file_size as f64 / (1024.0 * 1024.0))
    };

    let metadata = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("Health: ", styles::subtitle()),
        Span::styled(health_pct, health_style(doc.health)),
        Span::styled("  │  ", Style::default().fg(colors::BORDER)),
        Span::styled("Refinement: ", styles::subtitle()),
        Span::styled(refine_pct, health_style(doc.refinement)),
        Span::styled("  │  ", Style::default().fg(colors::BORDER)),
        Span::styled("Audience: ", styles::subtitle()),
        Span::styled(&doc.audience, Style::default().fg(crate::tui::theme::audience_color(&doc.audience))),
        Span::styled("  │  ", Style::default().fg(colors::BORDER)),
        Span::styled("Stubs: ", styles::subtitle()),
        Span::styled(
            doc.stub_count.to_string(),
            if doc.stub_count > 0 { styles::warning() } else { styles::value() },
        ),
        Span::styled("  │  ", Style::default().fg(colors::BORDER)),
        Span::styled("Lines: ", styles::subtitle()),
        Span::styled(doc.line_count.to_string(), styles::value()),
        Span::styled("  │  ", Style::default().fg(colors::BORDER)),
        Span::styled("Size: ", styles::subtitle()),
        Span::styled(size_str, styles::value()),
    ]));

    frame.render_widget(metadata, area);
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

    // Split content into lines with syntax highlighting
    let lines: Vec<Line> = doc.content.lines().enumerate().map(|(i, line)| {
        let line_num = format!("{:4} ", i + 1);
        let styled_line = highlight_markdown_line(line);

        let mut spans = vec![
            Span::styled(line_num, styles::line_number()),
            Span::styled("│ ", styles::border()),
        ];
        spans.extend(styled_line);

        Line::from(spans)
    }).collect();

    let total_lines = lines.len();
    let visible_height = inner.height as usize;

    // Clamp scroll offset
    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll_offset = app.scroll_offset.min(max_scroll);

    let paragraph = Paragraph::new(lines)
        .scroll((scroll_offset as u16, 0));

    frame.render_widget(paragraph, inner);

    // Render scrollbar if content is longer than visible area
    if total_lines > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            inner,
            &mut scrollbar_state,
        );
    }
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("d", styles::highlight()),
        Span::styled(" Dashboard  ", styles::subtitle()),
        Span::styled("Enter", styles::highlight()),
        Span::styled(" Details  ", styles::subtitle()),
        Span::styled("s", styles::highlight()),
        Span::styled(" Stubs  ", styles::subtitle()),
        Span::styled("↑/↓", styles::highlight()),
        Span::styled(" Scroll  ", styles::subtitle()),
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
    app.get_selected_document()
}

/// Simple markdown syntax highlighting
fn highlight_markdown_line(line: &str) -> Vec<Span<'static>> {
    let line = line.to_string();

    // Headers
    if line.starts_with("# ") {
        return vec![Span::styled(line, styles::title())];
    }
    if line.starts_with("## ") || line.starts_with("### ") {
        return vec![Span::styled(line, styles::highlight())];
    }
    if line.starts_with("#### ") || line.starts_with("##### ") || line.starts_with("###### ") {
        return vec![Span::styled(line, styles::label())];
    }

    // YAML frontmatter
    if line == "---" {
        return vec![Span::styled(line, styles::border())];
    }

    // YAML key-value pairs (simple detection)
    if line.contains(": ") && !line.starts_with(" ") && !line.starts_with("-") {
        if let Some((key, value)) = line.split_once(": ") {
            return vec![
                Span::styled(key.to_string(), styles::label()),
                Span::styled(": ".to_string(), styles::border()),
                Span::styled(value.to_string(), styles::value()),
            ];
        }
    }

    // List items
    if line.trim_start().starts_with("- ") || line.trim_start().starts_with("* ") {
        return vec![Span::styled(line, styles::value())];
    }

    // Numbered lists
    if line.trim_start().chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
        && line.contains(". ")
    {
        return vec![Span::styled(line, styles::value())];
    }

    // Code blocks
    if line.starts_with("```") {
        return vec![Span::styled(line, styles::highlight())];
    }

    // Blockquotes
    if line.starts_with("> ") {
        return vec![Span::styled(line, styles::subtitle())];
    }

    // Default
    vec![Span::styled(line, styles::value())]
}

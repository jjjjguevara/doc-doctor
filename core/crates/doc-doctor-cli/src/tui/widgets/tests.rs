//! Test Runner Widget
//!
//! Displays the test runner interface with:
//! - List of test files from configured test directory
//! - Predefined test commands
//! - File preview (in-memory, not saved)
//! - Test results

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use crate::tui::{
    app::{App, TestCategory, TestFocus},
    styles,
};

/// Render the test runner view
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Main layout: header, content, footer
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
    let test_dir = app.test_state.test_dir.as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Not configured".to_string());

    let file_count = app.test_state.test_files.len();
    let cmd_count = app.test_state.commands.len();

    // Check if current file is modified
    let modified_indicator = if let Some(path) = app.test_state.selected_file() {
        if app.test_state.is_modified(path) {
            " [MODIFIED]"
        } else {
            ""
        }
    } else {
        ""
    };

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" Test Runner ", styles::title()),
            Span::styled(modified_indicator, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("  ", Style::default()),
            Span::styled(format!("Dir: {}", truncate_path(&test_dir, 40)), styles::subtitle()),
            Span::styled(format!("  |  {} files  |  {} commands", file_count, cmd_count), styles::subtitle()),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(styles::border())
    );

    frame.render_widget(header, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    // Split into two rows: top (files + commands + preview), bottom (results)
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Top row
            Constraint::Percentage(40),  // Results
        ])
        .split(area);

    // Top row: files, commands, preview
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),  // Files
            Constraint::Percentage(25),  // Commands
            Constraint::Percentage(55),  // Preview
        ])
        .split(rows[0]);

    render_files(frame, app, columns[0]);
    render_commands(frame, app, columns[1]);
    render_preview(frame, app, columns[2]);
    render_results(frame, app, rows[1]);
}

fn render_files(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.test_state.focus == TestFocus::Files;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        styles::border()
    };

    let items: Vec<ListItem> = app.test_state.test_files
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");

            // Show indicator if file is modified
            let modified = if app.test_state.is_modified(path) { "* " } else { "  " };

            let style = if i == app.test_state.selected_file && is_focused {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else if i == app.test_state.selected_file {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            let mod_style = if app.test_state.is_modified(path) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(modified, mod_style),
                Span::styled(name, style),
            ]))
        })
        .collect();

    let title = if is_focused { " Files [*] " } else { " Files " };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, styles::title()))
                .borders(Borders::ALL)
                .border_style(border_style)
        );

    frame.render_widget(list, area);

    // Show "No files" message if empty
    if app.test_state.test_files.is_empty() {
        let msg = if app.test_state.test_dir.is_none() {
            "No test_dir configured"
        } else {
            "No .md files found"
        };
        let inner = area.inner(ratatui::layout::Margin { horizontal: 2, vertical: 2 });
        let paragraph = Paragraph::new(Span::styled(msg, styles::subtitle()));
        frame.render_widget(paragraph, inner);
    }
}

fn render_commands(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.test_state.focus == TestFocus::Commands;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        styles::border()
    };

    let items: Vec<ListItem> = app.test_state.commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let icon = cmd.category.icon();
            let is_selected = i == app.test_state.selected_command;

            let style = if is_selected && is_focused {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            let category_style = match cmd.category {
                TestCategory::Parse => Style::default().fg(Color::Blue),
                TestCategory::Analyze => Style::default().fg(Color::Magenta),
                TestCategory::Stubs => Style::default().fg(Color::Yellow),
                TestCategory::Health => Style::default().fg(Color::Green),
                TestCategory::Validate => Style::default().fg(Color::Cyan),
                TestCategory::Modify => Style::default().fg(Color::LightRed),
                TestCategory::Anchor => Style::default().fg(Color::LightGreen),
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", icon), category_style),
                Span::styled(&cmd.name, style),
            ]))
        })
        .collect();

    let title = if is_focused { " Commands [*] " } else { " Commands " };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, styles::title()))
                .borders(Borders::ALL)
                .border_style(border_style)
        );

    frame.render_widget(list, area);

    // Show command description at bottom of area
    if let Some(cmd) = app.test_state.selected_command() {
        let desc_area = Rect::new(area.x + 1, area.y + area.height - 2, area.width - 2, 1);
        let desc = Paragraph::new(Span::styled(
            truncate_output(&cmd.description, (area.width - 3) as usize),
            styles::subtitle()
        ));
        frame.render_widget(desc, desc_area);
    }
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.test_state.focus == TestFocus::Preview;
    let is_modified = app.test_state.selected_file()
        .map(|p| app.test_state.is_modified(p))
        .unwrap_or(false);

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else if is_modified {
        Style::default().fg(Color::Yellow)
    } else {
        styles::border()
    };

    let title = if is_focused && is_modified {
        " Preview [*] [modified] "
    } else if is_focused {
        " Preview [*] "
    } else if is_modified {
        " Preview [modified] "
    } else {
        " Preview "
    };

    let block = Block::default()
        .title(Span::styled(title, styles::title()))
        .borders(Borders::ALL)
        .border_style(border_style);

    // Get file content
    let content = if let Some(path) = app.test_state.selected_file() {
        app.test_state.get_content(path)
            .cloned()
            .unwrap_or_else(|| {
                // Load from disk if not in memory
                std::fs::read_to_string(path).unwrap_or_else(|_| "Error reading file".to_string())
            })
    } else {
        "Select a file to preview".to_string()
    };

    // Create lines with line numbers and selected line highlighting
    let inner_height = area.height.saturating_sub(2) as usize;
    let selected_line = app.test_state.selected_line;

    let lines: Vec<Line> = content
        .lines()
        .skip(app.test_state.preview_scroll)
        .take(inner_height)
        .enumerate()
        .map(|(i, line)| {
            let line_num = app.test_state.preview_scroll + i + 1;
            let is_selected = is_focused && line_num == selected_line;

            // Check for anchors in this line (^anchor-id pattern)
            let has_anchor = line.contains('^') && {
                // Quick check - look for ^ followed by alphanumeric
                let mut found = false;
                for (idx, c) in line.char_indices() {
                    if c == '^' && idx + 1 < line.len() {
                        let next = line.chars().nth(idx + 1);
                        if next.map(|c| c.is_alphanumeric() || c == '-').unwrap_or(false) {
                            // Skip markdown footnote references [^
                            if idx > 0 && line.as_bytes().get(idx - 1) == Some(&b'[') {
                                continue;
                            }
                            found = true;
                            break;
                        }
                    }
                }
                found
            };

            let line_num_style = if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else if has_anchor {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let cursor = if is_selected { ">" } else { " " };

            Line::from(vec![
                Span::styled(cursor, Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:4} ", line_num),
                    line_num_style
                ),
                Span::styled(line, line_style),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // Scrollbar
    let total_lines = content.lines().count();
    if total_lines > inner_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(app.test_state.preview_scroll);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }),
            &mut scrollbar_state,
        );
    }
}

fn render_results(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Results ", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border());

    if app.test_state.results.is_empty() {
        let msg = "Press Enter to run command  |  PageUp/PageDown: scroll preview  |  R: reset file";
        let paragraph = Paragraph::new(Span::styled(msg, styles::subtitle()))
            .block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    // Build result list
    let items: Vec<ListItem> = app.test_state.results
        .iter()
        .map(|result| {
            let (icon, color) = if result.success {
                ("✓", Color::Green)
            } else {
                ("✗", Color::Red)
            };

            let duration = format!("{:.2}s", result.duration.as_secs_f64());

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", icon), Style::default().fg(color)),
                    Span::styled(&result.command_id, Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!("  ({})", duration), styles::subtitle()),
                ]),
                Line::from(Span::styled(
                    format!("   {}", truncate_output(&result.output, 80)),
                    styles::subtitle()
                )),
            ])
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);

    // Scrollbar
    if app.test_state.results.len() > (area.height as usize - 2) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scrollbar_state = ScrollbarState::new(app.test_state.results.len())
            .position(app.test_state.results_scroll);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }),
            &mut scrollbar_state,
        );
    }
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let running_info = if let Some(ref cmd_id) = app.test_state.running {
        format!(" Running: {} {} ", app.spinner(), cmd_id)
    } else {
        String::new()
    };

    let selected_file = app.test_state.selected_file()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("none");

    let selected_cmd = app.test_state.selected_command()
        .map(|c| c.name.as_str())
        .unwrap_or("none");

    // Show line info when in preview mode
    let line_info = if app.test_state.focus == TestFocus::Preview {
        format!(" L:{}/{}", app.test_state.selected_line, app.test_state.content_line_count)
    } else {
        String::new()
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Tab", styles::highlight()),
        Span::styled(":cycle ", styles::subtitle()),
        Span::styled("↑↓", styles::highlight()),
        Span::styled(":nav ", styles::subtitle()),
        Span::styled("Enter", styles::highlight()),
        Span::styled(":run ", styles::subtitle()),
        Span::styled("R", styles::highlight()),
        Span::styled(":reset ", styles::subtitle()),
        Span::styled("q", styles::highlight()),
        Span::styled(":back", styles::subtitle()),
        Span::styled(&line_info, Style::default().fg(Color::Cyan)),
        Span::styled(running_info, Style::default().fg(Color::Yellow)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(styles::border())
    );

    frame.render_widget(footer, area);

    // Show selected items info
    let status = format!("{}  →  {}", selected_file, selected_cmd);
    let status_len = status.chars().count();
    if (status_len as u16 + 4) < area.width {
        let status_area = Rect::new(
            area.x + area.width - status_len as u16 - 3,
            area.y + 1,
            status_len as u16 + 2,
            1
        );
        let status_p = Paragraph::new(Span::styled(status, styles::subtitle()));
        frame.render_widget(status_p, status_area);
    }
}

fn truncate_output(output: &str, max_len: usize) -> String {
    let first_line = output.lines().next().unwrap_or("");
    if first_line.len() > max_len {
        format!("{}...", &first_line[..max_len])
    } else {
        first_line.to_string()
    }
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() > max_len {
        format!("...{}", &path[path.len() - max_len + 3..])
    } else {
        path.to_string()
    }
}

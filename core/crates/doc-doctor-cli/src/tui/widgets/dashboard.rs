//! Dashboard Widget
//!
//! Main dashboard showing vault health overview.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::tui::{
    app::{App, AppMode, Column, DashboardView, SortField},
    theme::{colors, health_color, health_style, styles},
};

/// Render the main dashboard
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    match app.dashboard_view {
        DashboardView::Results => render_results_view(frame, app, area),
        DashboardView::Vault => render_vault_view(frame, app, area),
        DashboardView::HealthOverview => render_health_overview(frame, app, area),
        DashboardView::StubsDetail => render_stubs_detail(frame, app, area),
        DashboardView::AudienceDetail => render_audience_detail(frame, app, area),
        DashboardView::FormDetail => render_form_detail(frame, app, area),
    }

    // Render popup menus on top
    match app.mode {
        AppMode::SortMenu => render_sort_menu(frame, app, area),
        AppMode::ColumnConfig => render_column_config(frame, app, area),
        _ => {}
    }
}

/// Render the Results view (main document list with compact summary)
fn render_results_view(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with view tabs
            Constraint::Length(4),  // Compact summary
            Constraint::Min(10),    // Document list
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    render_header_with_tabs(frame, app, chunks[0]);
    render_compact_summary(frame, app, chunks[1]);
    render_document_list(frame, app, chunks[2]);
    render_status_bar(frame, app, chunks[3]);
}

/// Render the Health Overview view
fn render_health_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with view tabs
            Constraint::Length(7),  // Stats cards
            Constraint::Min(10),    // Health distribution + details
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    render_header_with_tabs(frame, app, chunks[0]);
    render_stats_overview(frame, app, chunks[1]);
    render_health_details(frame, app, chunks[2]);
    render_status_bar(frame, app, chunks[3]);
}

/// Render the Stubs Detail view
fn render_stubs_detail(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with view tabs
            Constraint::Min(10),    // Stubs breakdown
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    render_header_with_tabs(frame, app, chunks[0]);
    render_stubs_breakdown(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);
}

/// Render the Audience Detail view
fn render_audience_detail(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with view tabs
            Constraint::Min(10),    // Audience breakdown
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    render_header_with_tabs(frame, app, chunks[0]);
    render_audience_breakdown(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);
}

/// Render the Form Detail view
fn render_form_detail(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with view tabs
            Constraint::Min(10),    // Form breakdown
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    render_header_with_tabs(frame, app, chunks[0]);
    render_form_breakdown(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);
}

/// Render the Vault view (file/folder statistics)
fn render_vault_view(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with view tabs
            Constraint::Length(7),  // File stats cards
            Constraint::Min(10),    // Folder breakdown
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    render_header_with_tabs(frame, app, chunks[0]);
    render_vault_stats_cards(frame, app, chunks[1]);
    render_folder_breakdown(frame, app, chunks[2]);
    render_status_bar(frame, app, chunks[3]);
}

/// Render vault statistics cards
fn render_vault_stats_cards(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    if let Some(stats) = &app.vault_stats {
        // Total Files
        render_stat_card(
            frame,
            chunks[0],
            "Total Files",
            &stats.total_files.to_string(),
            "markdown files",
            colors::PRIMARY,
        );

        // Files with Frontmatter
        let fm_pct = if stats.total_files > 0 {
            (stats.files_with_frontmatter as f64 / stats.total_files as f64) * 100.0
        } else {
            0.0
        };
        render_stat_card(
            frame,
            chunks[1],
            "With Frontmatter",
            &stats.files_with_frontmatter.to_string(),
            &format!("{:.0}% of files", fm_pct),
            colors::SUCCESS,
        );

        // Files without Frontmatter
        render_stat_card(
            frame,
            chunks[2],
            "Without Frontmatter",
            &stats.files_without_frontmatter.to_string(),
            "not tracked",
            if stats.files_without_frontmatter > 0 { colors::WARNING } else { colors::SUCCESS },
        );

        // Total Folders
        render_stat_card(
            frame,
            chunks[3],
            "Folders",
            &stats.total_folders.to_string(),
            "subdirectories",
            colors::ACCENT_CYAN,
        );
    }
}

/// Render folder breakdown
fn render_folder_breakdown(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    if let Some(stats) = &app.vault_stats {
        // Left: Files by top-level folder
        let mut items: Vec<ListItem> = vec![
            ListItem::new(Line::from(Span::styled("  Files by Top-Level Folder", styles::subtitle()))),
            ListItem::new(Line::from(Span::raw(""))),
        ];

        let max_count = stats.files_by_top_folder.iter()
            .map(|(_, total, _)| *total)
            .max()
            .unwrap_or(1);

        for (folder, total, with_fm) in stats.files_by_top_folder.iter().take(12) {
            let bar_width = (*total as f64 / max_count as f64 * 15.0) as usize;
            let bar = "█".repeat(bar_width.max(1));
            let fm_pct = if *total > 0 {
                (*with_fm as f64 / *total as f64) * 100.0
            } else {
                0.0
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("  {:>20} ", truncate(folder, 20)), styles::label()),
                Span::styled(bar, Style::default().fg(colors::ACCENT_CYAN)),
                Span::styled(format!(" {} ", total), Style::default().fg(colors::TEXT)),
                Span::styled(format!("({:.0}% fm)", fm_pct), styles::subtitle()),
            ])));
        }

        let list = List::new(items)
            .block(Block::default().borders(Borders::RIGHT).border_style(styles::border()));
        frame.render_widget(list, chunks[0]);

        // Right: Vault info
        let mut right_items: Vec<ListItem> = vec![
            ListItem::new(Line::from(Span::styled("  Vault Information", styles::subtitle()))),
            ListItem::new(Line::from(Span::raw(""))),
        ];

        if let Some(root) = &stats.vault_root {
            right_items.push(ListItem::new(Line::from(vec![
                Span::styled("  Root: ", styles::label()),
                Span::styled(
                    truncate(&root.display().to_string(), 40),
                    Style::default().fg(colors::TEXT),
                ),
            ])));
        }

        right_items.push(ListItem::new(Line::from(Span::raw(""))));
        right_items.push(ListItem::new(Line::from(vec![
            Span::styled("  Coverage: ", styles::label()),
        ])));

        // Coverage bar
        let coverage = if stats.total_files > 0 {
            stats.files_with_frontmatter as f64 / stats.total_files as f64
        } else {
            0.0
        };
        let bar_filled = (coverage * 30.0) as usize;
        let bar_empty = 30 - bar_filled;
        right_items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("█".repeat(bar_filled), Style::default().fg(colors::SUCCESS)),
            Span::styled("░".repeat(bar_empty), Style::default().fg(colors::TEXT_DIM)),
            Span::styled(format!(" {:.0}%", coverage * 100.0), health_style(coverage)),
        ])));

        right_items.push(ListItem::new(Line::from(Span::raw(""))));
        right_items.push(ListItem::new(Line::from(vec![
            Span::styled("  Legend: ", styles::label()),
        ])));
        right_items.push(ListItem::new(Line::from(vec![
            Span::styled("    fm = frontmatter", styles::subtitle()),
        ])));

        let right_list = List::new(right_items)
            .block(Block::default());
        frame.render_widget(right_list, chunks[1]);
    }
}

/// Render header with view tabs
fn render_header_with_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let spinner = if app.ai_state.is_processing {
        format!(" {} ", app.spinner())
    } else {
        String::new()
    };

    // Build tab spans
    let mut tabs = vec![
        Span::styled("  Doc Doctor", styles::title()),
        Span::styled(&spinner, Style::default().fg(colors::ACCENT_CYAN)),
        Span::styled(" │ ", styles::label()),
    ];

    for view in DashboardView::all() {
        let is_active = *view == app.dashboard_view;
        let label = format!(" {} ", view.label());
        if is_active {
            tabs.push(Span::styled(label, Style::default()
                .fg(colors::BACKGROUND)
                .bg(colors::ACCENT_CYAN)
                .add_modifier(Modifier::BOLD)));
        } else {
            tabs.push(Span::styled(label, styles::label()));
        }
        tabs.push(Span::raw(" "));
    }

    tabs.push(Span::styled(" [Tab] to switch", styles::label()));

    let title = Line::from(tabs);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(styles::border());

    let header = Paragraph::new(title).block(block);
    frame.render_widget(header, area);
}

/// Render compact vault summary (for Results view)
fn render_compact_summary(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(stats) = &app.vault_stats {
        let health_color = health_color(stats.average_health);

        // Calculate documents with/without frontmatter from filter info
        let total_files = app.documents.len() + (app.documents.len() as f64 * 0.3) as usize; // Estimate
        let with_frontmatter = stats.total_documents;

        let line1 = Line::from(vec![
            Span::styled("  Documents: ", styles::label()),
            Span::styled(format!("{}", stats.total_documents), Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("  │  ", styles::label()),
            Span::styled("Health: ", styles::label()),
            Span::styled(format!("{:.0}%", stats.average_health * 100.0), Style::default().fg(health_color).add_modifier(Modifier::BOLD)),
            Span::styled("  │  ", styles::label()),
            Span::styled("Stubs: ", styles::label()),
            Span::styled(format!("{}", stats.total_stubs), Style::default().fg(if stats.blocking_stubs > 0 { colors::WARNING } else { colors::SUCCESS })),
            Span::styled(format!(" ({} blocking)", stats.blocking_stubs), styles::label()),
            Span::styled("  │  ", styles::label()),
            Span::styled("Refinement: ", styles::label()),
            Span::styled(format!("{:.0}%", stats.average_refinement * 100.0), Style::default().fg(health_color)),
        ]);

        let filter_info = if !app.filter_text.is_empty() {
            format!("  │  Filter: \"{}\" ({} results)", app.filter_text, app.visible_document_count())
        } else {
            String::new()
        };

        let line2 = Line::from(vec![
            Span::styled("  Showing: ", styles::label()),
            Span::styled(format!("{} documents", app.visible_document_count()), Style::default().fg(colors::TEXT_DIM)),
            Span::styled(&filter_info, styles::label()),
        ]);

        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(styles::border());

        let para = Paragraph::new(vec![line1, line2]).block(block);
        frame.render_widget(para, area);
    }
}

/// Render health distribution details
fn render_health_details(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(area);

    // Left: Health distribution histogram
    let health_ranges = calculate_health_distribution(app);
    let mut items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled("  Health Distribution", styles::subtitle()))),
        ListItem::new(Line::from(Span::raw(""))),
    ];

    for (label, count, color) in health_ranges {
        let bar_width = (count as f64 / app.documents.len().max(1) as f64 * 20.0) as usize;
        let bar = "█".repeat(bar_width);
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {:>8} ", label), styles::label()),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(format!(" {}", count), Style::default().fg(colors::TEXT_DIM)),
        ])));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).border_style(styles::border()));
    frame.render_widget(list, chunks[0]);

    // Right: Documents needing attention
    let mut attention_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled("  Documents Needing Attention", styles::subtitle()))),
        ListItem::new(Line::from(Span::raw(""))),
    ];

    let needs_attention: Vec<_> = app.documents.iter()
        .filter(|d| d.health < 0.5)
        .take(8)
        .collect();

    for doc in needs_attention {
        let title = doc.title.as_deref()
            .unwrap_or_else(|| doc.path.file_name().unwrap().to_str().unwrap());
        let title_truncated: String = title.chars().take(35).collect();
        attention_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {:>3}% ", (doc.health * 100.0) as u8), health_style(doc.health)),
            Span::styled(title_truncated, Style::default().fg(colors::TEXT)),
        ])));
    }

    let attention_list = List::new(attention_items)
        .block(Block::default());
    frame.render_widget(attention_list, chunks[1]);
}

/// Render stubs breakdown
fn render_stubs_breakdown(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(65),
        ])
        .split(area);

    // Left: Stub type breakdown
    let stub_counts = calculate_stub_type_distribution(app);
    let mut items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled("  Stubs by Type", styles::subtitle()))),
        ListItem::new(Line::from(Span::raw(""))),
    ];

    for (stub_type, count) in stub_counts.iter().take(12) {
        let bar_width = (*count as f64 / stub_counts.iter().map(|(_, c)| *c).max().unwrap_or(1) as f64 * 15.0) as usize;
        let bar = "█".repeat(bar_width.max(1));
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {:>15} ", stub_type), styles::label()),
            Span::styled(bar, Style::default().fg(colors::ACCENT_CYAN)),
            Span::styled(format!(" {}", count), Style::default().fg(colors::TEXT_DIM)),
        ])));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).border_style(styles::border()));
    frame.render_widget(list, chunks[0]);

    // Right: Stub form breakdown + blocking stubs
    let mut form_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled("  Stubs by Form", styles::subtitle()))),
        ListItem::new(Line::from(Span::raw(""))),
    ];

    let form_counts = calculate_stub_form_distribution(app);
    for (form, count, color) in form_counts {
        form_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {:>12} ", form), styles::label()),
            Span::styled(format!("{}", count), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ])));
    }

    form_items.push(ListItem::new(Line::from(Span::raw(""))));
    form_items.push(ListItem::new(Line::from(Span::styled("  Blocking Stubs", styles::subtitle()))));
    form_items.push(ListItem::new(Line::from(Span::raw(""))));

    let blocking: Vec<_> = app.documents.iter()
        .flat_map(|d| d.stubs.iter().filter(|s| s.is_blocking()).map(move |s| (d, s)))
        .take(6)
        .collect();

    for (doc, stub) in blocking {
        let title = doc.title.as_deref()
            .unwrap_or_else(|| doc.path.file_name().unwrap().to_str().unwrap());
        let title_truncated: String = title.chars().take(25).collect();
        let desc_truncated: String = stub.description.chars().take(30).collect();
        form_items.push(ListItem::new(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(title_truncated, Style::default().fg(colors::TEXT)),
            Span::styled(": ", styles::label()),
            Span::styled(desc_truncated, Style::default().fg(colors::WARNING)),
        ])));
    }

    let form_list = List::new(form_items)
        .block(Block::default());
    frame.render_widget(form_list, chunks[1]);
}

/// Render audience breakdown
fn render_audience_breakdown(frame: &mut Frame, app: &App, area: Rect) {
    let audience_counts = calculate_audience_distribution(app);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(area);

    // Left: Audience distribution
    let mut items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled("  Documents by Audience", styles::subtitle()))),
        ListItem::new(Line::from(Span::raw(""))),
    ];

    let total = app.documents.len().max(1);
    for (audience, count, color) in &audience_counts {
        let pct = (*count as f64 / total as f64) * 100.0;
        let bar_width = (pct / 5.0) as usize;
        let bar = "█".repeat(bar_width.max(1));
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {:>10} ", audience), styles::label()),
            Span::styled(bar, Style::default().fg(*color)),
            Span::styled(format!(" {} ({:.0}%)", count, pct), Style::default().fg(colors::TEXT_DIM)),
        ])));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).border_style(styles::border()));
    frame.render_widget(list, chunks[0]);

    // Right: Low health docs by audience
    let mut right_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled("  Low Health by Audience", styles::subtitle()))),
        ListItem::new(Line::from(Span::raw(""))),
    ];

    for (audience, _, _) in &audience_counts {
        let low_health: Vec<_> = app.documents.iter()
            .filter(|d| d.audience == *audience && d.health < 0.5)
            .take(2)
            .collect();

        if !low_health.is_empty() {
            right_items.push(ListItem::new(Line::from(Span::styled(
                format!("  {}", audience),
                Style::default().fg(colors::TEXT).add_modifier(Modifier::BOLD)
            ))));
            for doc in low_health {
                let title = doc.title.as_deref()
                    .unwrap_or_else(|| doc.path.file_name().unwrap().to_str().unwrap());
                let title_truncated: String = title.chars().take(40).collect();
                right_items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("    {:>3}% ", (doc.health * 100.0) as u8), health_style(doc.health)),
                    Span::styled(title_truncated, styles::label()),
                ])));
            }
        }
    }

    let right_list = List::new(right_items)
        .block(Block::default());
    frame.render_widget(right_list, chunks[1]);
}

/// Render form breakdown
fn render_form_breakdown(frame: &mut Frame, app: &App, area: Rect) {
    let form_counts = calculate_form_distribution(app);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(area);

    // Left: Form distribution
    let mut items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled("  Documents by Form", styles::subtitle()))),
        ListItem::new(Line::from(Span::raw(""))),
    ];

    let total = app.documents.len().max(1);
    for (form, count, color) in &form_counts {
        let pct = (*count as f64 / total as f64) * 100.0;
        let bar_width = (pct / 5.0) as usize;
        let bar = "█".repeat(bar_width.max(1));
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {:>12} ", form), styles::label()),
            Span::styled(bar, Style::default().fg(*color)),
            Span::styled(format!(" {} ({:.0}%)", count, pct), Style::default().fg(colors::TEXT_DIM)),
        ])));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).border_style(styles::border()));
    frame.render_widget(list, chunks[0]);

    // Right: Form lifecycle info
    let mut right_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled("  Form Lifecycle", styles::subtitle()))),
        ListItem::new(Line::from(Span::raw(""))),
        ListItem::new(Line::from(vec![
            Span::styled("  Transient   ", Style::default().fg(colors::TEXT_DIM)),
            Span::styled("- Scratch, 7-day staleness", styles::label()),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  Developing  ", Style::default().fg(colors::WARNING)),
            Span::styled("- Work in progress, 30-day", styles::label()),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  Stable      ", Style::default().fg(colors::SUCCESS)),
            Span::styled("- Mature, 90-day staleness", styles::label()),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  Evergreen   ", Style::default().fg(colors::ACCENT_CYAN)),
            Span::styled("- Long-lived, 365-day", styles::label()),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  Canonical   ", Style::default().fg(colors::PRIMARY)),
            Span::styled("- Authoritative, never stale", styles::label()),
        ])),
    ];

    let right_list = List::new(right_items)
        .block(Block::default());
    frame.render_widget(right_list, chunks[1]);
}

// Helper functions for statistics

fn calculate_health_distribution(app: &App) -> Vec<(&'static str, usize, ratatui::style::Color)> {
    let mut critical = 0;
    let mut low = 0;
    let mut medium = 0;
    let mut good = 0;
    let mut excellent = 0;

    for doc in &app.documents {
        if doc.health < 0.3 {
            critical += 1;
        } else if doc.health < 0.5 {
            low += 1;
        } else if doc.health < 0.7 {
            medium += 1;
        } else if doc.health < 0.9 {
            good += 1;
        } else {
            excellent += 1;
        }
    }

    vec![
        ("0-30%", critical, colors::ERROR),
        ("30-50%", low, colors::WARNING),
        ("50-70%", medium, colors::WARNING),
        ("70-90%", good, colors::SUCCESS),
        ("90-100%", excellent, colors::ACCENT_CYAN),
    ]
}

fn calculate_stub_type_distribution(app: &App) -> Vec<(String, usize)> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();

    for doc in &app.documents {
        for stub in &doc.stubs {
            *counts.entry(stub.stub_type.as_str().to_string()).or_insert(0) += 1;
        }
    }

    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted
}

fn calculate_stub_form_distribution(app: &App) -> Vec<(&'static str, usize, ratatui::style::Color)> {
    let mut transient = 0;
    let mut persistent = 0;
    let mut blocking = 0;
    let mut structural = 0;

    for doc in &app.documents {
        for stub in &doc.stubs {
            match stub.stub_form {
                doc_doctor_domain::StubForm::Transient => transient += 1,
                doc_doctor_domain::StubForm::Persistent => persistent += 1,
                doc_doctor_domain::StubForm::Blocking => blocking += 1,
                doc_doctor_domain::StubForm::Structural => structural += 1,
            }
        }
    }

    vec![
        ("Transient", transient, colors::TEXT_DIM),
        ("Persistent", persistent, colors::WARNING),
        ("Blocking", blocking, colors::ERROR),
        ("Structural", structural, colors::STUB_STRUCTURAL),
    ]
}

fn calculate_audience_distribution(app: &App) -> Vec<(&'static str, usize, ratatui::style::Color)> {
    let mut personal = 0;
    let mut internal = 0;
    let mut trusted = 0;
    let mut public = 0;

    for doc in &app.documents {
        match doc.audience.as_str() {
            "personal" => personal += 1,
            "internal" => internal += 1,
            "trusted" => trusted += 1,
            "public" => public += 1,
            _ => {}
        }
    }

    vec![
        ("Personal", personal, colors::TEXT_DIM),
        ("Internal", internal, colors::WARNING),
        ("Trusted", trusted, colors::SUCCESS),
        ("Public", public, colors::ACCENT_CYAN),
    ]
}

fn calculate_form_distribution(app: &App) -> Vec<(&'static str, usize, ratatui::style::Color)> {
    let mut transient = 0;
    let mut developing = 0;
    let mut stable = 0;
    let mut evergreen = 0;
    let mut canonical = 0;

    for doc in &app.documents {
        match doc.form.as_str() {
            "transient" => transient += 1,
            "developing" => developing += 1,
            "stable" => stable += 1,
            "evergreen" => evergreen += 1,
            "canonical" => canonical += 1,
            _ => {}
        }
    }

    vec![
        ("Transient", transient, colors::TEXT_DIM),
        ("Developing", developing, colors::WARNING),
        ("Stable", stable, colors::SUCCESS),
        ("Evergreen", evergreen, colors::ACCENT_CYAN),
        ("Canonical", canonical, colors::PRIMARY),
    ]
}

fn render_stats_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    if let Some(stats) = &app.vault_stats {
        // Documents count
        render_stat_card(
            frame,
            chunks[0],
            "Documents",
            &stats.total_documents.to_string(),
            "",
            colors::PRIMARY,
        );

        // Average health
        render_stat_card(
            frame,
            chunks[1],
            "Avg Health",
            &format!("{:.0}%", stats.average_health * 100.0),
            "",
            health_color(stats.average_health),
        );

        // Total stubs
        let stub_color = if stats.blocking_stubs > 0 {
            colors::ERROR
        } else if stats.total_stubs > 10 {
            colors::WARNING
        } else {
            colors::SUCCESS
        };
        render_stat_card(
            frame,
            chunks[2],
            "Stubs",
            &stats.total_stubs.to_string(),
            &format!("{} blocking", stats.blocking_stubs),
            stub_color,
        );

        // Average refinement
        render_stat_card(
            frame,
            chunks[3],
            "Avg Refinement",
            &format!("{:.0}%", stats.average_refinement * 100.0),
            "",
            health_color(stats.average_refinement),
        );
    } else {
        let loading = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(app.spinner().to_string(), Style::default().fg(colors::PRIMARY)),
                Span::raw(" Loading vault statistics..."),
            ]),
        ])
        .style(styles::subtitle());
        frame.render_widget(loading, area);
    }
}

fn render_stat_card(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    value: &str,
    subtitle: &str,
    color: ratatui::style::Color,
) {
    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            styles::label(),
        ))
        .borders(Borders::ALL)
        .border_style(styles::border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = if subtitle.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", value),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                format!("  {}", value),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("  {}", subtitle),
                styles::subtitle(),
            )),
        ]
    };

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, inner);
}

fn render_document_list(frame: &mut Frame, app: &App, area: Rect) {
    // Calculate list dimensions
    let inner_height = area.height.saturating_sub(3) as usize; // -3 for borders + header row

    let total_docs = app.visible_document_count();

    // Build title with sort and filter info
    let sort_dir = if app.sort_ascending { "↑" } else { "↓" };

    let title = if !app.filter_text.is_empty() {
        format!(" Documents ({}/{}) [filter: \"{}\"] [sort: {} {}] ",
            total_docs, app.documents.len(), app.filter_text, app.sort_field.label(), sort_dir)
    } else {
        format!(" Documents ({}) [sort: {} {}] ", total_docs, app.sort_field.label(), sort_dir)
    };

    let block = Block::default()
        .title(Span::styled(title, styles::label()))
        .borders(Borders::ALL)
        .border_style(styles::border());

    if app.documents.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from("  No documents loaded. Use 'ddoc dashboard <path>' to scan a vault."),
        ])
        .style(styles::subtitle())
        .block(block);
        frame.render_widget(empty, area);
        return;
    }

    if total_docs == 0 {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(format!("  No documents match filter: \"{}\"", app.filter_text)),
            Line::from(""),
            Line::from("  Press 'x' to clear filter"),
        ])
        .style(styles::subtitle())
        .block(block);
        frame.render_widget(empty, area);
        return;
    }

    // Calculate visible window
    let list_offset = app.list_offset;
    let visible_end = (list_offset + inner_height).min(total_docs);

    // Build column constraints based on visible columns
    let constraints: Vec<Constraint> = std::iter::once(Constraint::Length(2)) // Selection indicator
        .chain(app.visible_columns.iter().map(|col| Constraint::Length(col.width())))
        .collect();

    // Build header row
    let header_cells: Vec<Span> = std::iter::once(Span::raw(""))
        .chain(app.visible_columns.iter().map(|col| {
            let label = col.label();
            let is_sort_col = matches!(
                (col, app.sort_field),
                (Column::Health, SortField::Health) |
                (Column::Name, SortField::Name) |
                (Column::Stubs, SortField::Stubs) |
                (Column::Refinement, SortField::Refinement) |
                (Column::Audience, SortField::Audience) |
                (Column::Lines, SortField::Lines) |
                (Column::Size, SortField::Size) |
                (Column::Modified, SortField::Modified)
            );
            if is_sort_col {
                let arrow = if app.sort_ascending { "↑" } else { "↓" };
                Span::styled(format!("{}{}", label, arrow), styles::highlight())
            } else {
                Span::styled(label.to_string(), styles::subtitle())
            }
        }))
        .collect();
    let header = Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD));

    // Build data rows
    let rows: Vec<Row> = (list_offset..visible_end)
        .filter_map(|visible_idx| {
            let doc = app.get_visible_document(visible_idx)?;
            let is_selected = visible_idx == app.selected_document;

            let mut cells: Vec<Span> = vec![
                Span::styled(
                    if is_selected { "▶" } else { " " },
                    Style::default().fg(colors::PRIMARY),
                ),
            ];

            for col in &app.visible_columns {
                let cell = match col {
                    Column::Health => {
                        Span::styled(
                            format!("{:.0}%", doc.health * 100.0),
                            health_style(doc.health),
                        )
                    }
                    Column::Name => {
                        let title = doc.title.as_deref()
                            .unwrap_or_else(|| doc.path.file_name().unwrap().to_str().unwrap());
                        Span::styled(
                            truncate(title, col.width() as usize - 1),
                            if is_selected { styles::highlight() } else { styles::value() },
                        )
                    }
                    Column::Audience => {
                        Span::styled(
                            truncate(&doc.audience, col.width() as usize - 1),
                            Style::default().fg(crate::tui::theme::audience_color(&doc.audience)),
                        )
                    }
                    Column::Stubs => {
                        Span::styled(
                            doc.stub_count.to_string(),
                            if doc.stub_count > 0 { styles::warning() } else { styles::subtitle() },
                        )
                    }
                    Column::Refinement => {
                        Span::styled(
                            format!("{:.0}%", doc.refinement * 100.0),
                            health_style(doc.refinement),
                        )
                    }
                    Column::Lines => {
                        Span::styled(doc.line_count.to_string(), styles::subtitle())
                    }
                    Column::Size => {
                        Span::styled(format_size(doc.file_size), styles::subtitle())
                    }
                    Column::Modified => {
                        Span::styled(format_time(doc.modified), styles::subtitle())
                    }
                    Column::Created => {
                        Span::styled(format_time(doc.created), styles::subtitle())
                    }
                    Column::Author => {
                        Span::styled(
                            truncate(doc.author.as_deref().unwrap_or("-"), col.width() as usize - 1),
                            styles::subtitle(),
                        )
                    }
                    Column::LastCommit => {
                        Span::styled(
                            truncate(doc.last_commit.as_deref().unwrap_or("-"), col.width() as usize - 1),
                            styles::subtitle(),
                        )
                    }
                    Column::Folder => {
                        Span::styled(
                            truncate(doc.folder_name(), col.width() as usize - 1),
                            styles::subtitle(),
                        )
                    }
                    Column::Path => {
                        Span::styled(
                            truncate(doc.display_path(), col.width() as usize - 1),
                            styles::subtitle(),
                        )
                    }
                    Column::Form => {
                        Span::styled(
                            truncate(&doc.form, col.width() as usize - 1),
                            Style::default().fg(crate::tui::theme::form_color(&doc.form)),
                        )
                    }
                    Column::Origin => {
                        Span::styled(
                            truncate(&doc.origin, col.width() as usize - 1),
                            Style::default().fg(crate::tui::theme::origin_color(&doc.origin)),
                        )
                    }
                };
                cells.push(cell);
            }

            Some(Row::new(cells))
        })
        .collect();

    // Add scroll indicator if needed
    let scroll_info = if total_docs > inner_height {
        let pos = list_offset + 1;
        let end = visible_end;
        format!(" [{}-{}/{}] ", pos, end, total_docs)
    } else {
        String::new()
    };

    let block = block.title_bottom(Line::from(Span::styled(scroll_info, styles::subtitle())));

    let table = Table::new(rows, constraints)
        .header(header)
        .block(block);
    frame.render_widget(table, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_line = if app.mode == AppMode::Search {
        // Search mode - show search input
        Line::from(vec![
            Span::styled("  Search: ", styles::highlight()),
            Span::styled(&app.filter_text, styles::value()),
            Span::styled("█", Style::default().fg(colors::PRIMARY)),
            Span::styled("  (Enter to apply, Esc to cancel)", styles::subtitle()),
        ])
    } else if app.mode == AppMode::SortMenu {
        Line::from(vec![
            Span::styled("  Sort: ", styles::highlight()),
            Span::styled("←→/o", styles::value()),
            Span::styled(" field  ", styles::subtitle()),
            Span::styled("↑↓", styles::value()),
            Span::styled(" direction  ", styles::subtitle()),
            Span::styled("Enter/Esc", styles::value()),
            Span::styled(" close", styles::subtitle()),
        ])
    } else if app.mode == AppMode::ColumnConfig {
        Line::from(vec![
            Span::styled("  Columns: ", styles::highlight()),
            Span::styled("↑↓", styles::value()),
            Span::styled(" select  ", styles::subtitle()),
            Span::styled("Space", styles::value()),
            Span::styled(" toggle  ", styles::subtitle()),
            Span::styled("[]", styles::value()),
            Span::styled(" reorder  ", styles::subtitle()),
            Span::styled("Esc", styles::value()),
            Span::styled(" close", styles::subtitle()),
        ])
    } else if let Some((msg, _)) = &app.status_message {
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(msg.clone(), styles::subtitle()),
        ])
    } else {
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("↑↓", styles::highlight()),
            Span::styled(" Nav  ", styles::subtitle()),
            Span::styled("/", styles::highlight()),
            Span::styled(" Search  ", styles::subtitle()),
            Span::styled("o", styles::highlight()),
            Span::styled(" Sort  ", styles::subtitle()),
            Span::styled("v", styles::highlight()),
            Span::styled(" Columns  ", styles::subtitle()),
            Span::styled("Enter", styles::highlight()),
            Span::styled(" View  ", styles::subtitle()),
            Span::styled("?", styles::highlight()),
            Span::styled(" Help", styles::subtitle()),
        ])
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(styles::border());

    let paragraph = Paragraph::new(status_line).block(block);
    frame.render_widget(paragraph, area);
}

/// Truncate string to max length (UTF-8 aware)
fn truncate(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// Format file size for display
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Format timestamp for display
fn format_time(time: Option<std::time::SystemTime>) -> String {
    match time {
        Some(t) => {
            let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let secs = duration.as_secs();
            // Convert to local datetime (simplified - just show relative)
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let diff = now.saturating_sub(secs);
            if diff < 60 {
                "just now".to_string()
            } else if diff < 3600 {
                format!("{}m ago", diff / 60)
            } else if diff < 86400 {
                format!("{}h ago", diff / 3600)
            } else if diff < 86400 * 30 {
                format!("{}d ago", diff / 86400)
            } else {
                format!("{}mo ago", diff / (86400 * 30))
            }
        }
        None => "-".to_string(),
    }
}

/// Render the sort menu popup
fn render_sort_menu(frame: &mut Frame, app: &App, area: Rect) {
    // Center popup
    let popup_width = 50u16;
    let popup_height = 5u16;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear area behind popup
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(" Sort Options ", styles::title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::PRIMARY));

    // Build sort field selector line
    let fields = SortField::all();
    let mut field_spans: Vec<Span> = vec![Span::raw("  ")];
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            field_spans.push(Span::styled(" │ ", styles::border()));
        }
        if *field == app.sort_field {
            field_spans.push(Span::styled(
                format!("▶{}◀", field.label()),
                Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD),
            ));
        } else {
            field_spans.push(Span::styled(field.label().to_string(), styles::subtitle()));
        }
    }

    // Build direction line
    let asc_style = if app.sort_ascending {
        Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)
    } else {
        styles::subtitle()
    };
    let desc_style = if !app.sort_ascending {
        Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)
    } else {
        styles::subtitle()
    };

    let content = vec![
        Line::from(field_spans),
        Line::from(vec![
            Span::raw("  Direction: "),
            Span::styled(if app.sort_ascending { "▶" } else { " " }, asc_style),
            Span::styled("Ascending", asc_style),
            Span::raw("  "),
            Span::styled(if !app.sort_ascending { "▶" } else { " " }, desc_style),
            Span::styled("Descending", desc_style),
        ]),
    ];

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, popup_area);
}

/// Render the column configuration popup
fn render_column_config(frame: &mut Frame, app: &App, area: Rect) {
    let all_cols = Column::all();
    // Popup size
    let popup_width = 35u16;
    let popup_height = (all_cols.len() + 3) as u16; // +3 for borders and title
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear area behind popup
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(" Configure Columns ", styles::title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::PRIMARY));

    let items: Vec<ListItem> = all_cols
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let is_selected = i == app.column_menu_index;
            let is_visible = app.is_column_visible(*col);
            let checkbox = if is_visible { "[✓]" } else { "[ ]" };
            let indicator = if is_selected { "▶" } else { " " };

            // Show position if visible
            let position = if is_visible {
                let pos = app.visible_columns.iter().position(|c| c == col).unwrap_or(0) + 1;
                format!(" ({})", pos)
            } else {
                String::new()
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", indicator), Style::default().fg(colors::PRIMARY)),
                Span::styled(
                    checkbox,
                    if is_visible { styles::highlight() } else { styles::subtitle() },
                ),
                Span::raw(" "),
                Span::styled(
                    col.label().to_string(),
                    if is_selected { styles::highlight() } else { styles::value() },
                ),
                Span::styled(position, styles::subtitle()),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, popup_area);
}

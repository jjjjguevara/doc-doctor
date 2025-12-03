//! Doc Doctor TUI
//!
//! Terminal user interface for interactive document analysis.
//!
//! # Features
//!
//! - **Dashboard**: Real-time vault health overview
//! - **Document Viewer**: Interactive document analysis
//! - **Batch Progress**: Visual progress for batch operations
//! - **Thought Stream**: AI processing visualization
//! - **Styled Tables**: Beautiful output for static commands
//!
//! # Usage
//!
//! For interactive TUI mode:
//! ```bash
//! ddoc dashboard ./vault
//! ```
//!
//! For styled output (automatic):
//! ```bash
//! ddoc health --refinement 0.8
//! ddoc parse document.md
//! ```

pub mod app;
pub mod terminal;
pub mod theme;
pub mod widgets;

// Re-exports
pub use app::{App, AppMode, BatchState, DashboardView, Event, ThoughtStatus};
pub use terminal::{init, install_panic_hook, poll_event, restore, Tui};
pub use theme::{colors, console_styles, health_color, health_style, styles};
pub use widgets::{icons, ConsoleProgress, MultiProgress};

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers, MouseEventKind};
use ratatui::Frame;

/// Run the interactive TUI application
pub fn run_tui(mut app: App) -> Result<()> {
    install_panic_hook();
    let mut terminal = init()?;

    let tick_rate = Duration::from_millis(100);

    while !app.should_quit {
        terminal.draw(|frame| render(frame, &app))?;

        if let Some(event) = poll_event(tick_rate)? {
            handle_event(&mut app, event)?;
        }
    }

    restore()?;
    Ok(())
}

/// Main render function
fn render(frame: &mut Frame, app: &App) {
    match app.mode {
        AppMode::Dashboard | AppMode::Search | AppMode::SortMenu | AppMode::ColumnConfig => {
            widgets::dashboard::render(frame, app, frame.area());
        }
        AppMode::DocumentViewer => widgets::document_viewer::render(frame, app, frame.area()),
        AppMode::StubList => widgets::stub_list::render(frame, app, frame.area()),
        AppMode::DocumentContent => widgets::content_viewer::render(frame, app, frame.area()),
        AppMode::Tests => widgets::tests::render(frame, app, frame.area()),
        AppMode::BatchProgress => {
            if app.ai_state.is_processing {
                // Split screen: progress on top, thoughts on bottom
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Percentage(50),
                        ratatui::layout::Constraint::Percentage(50),
                    ])
                    .split(frame.area());

                widgets::progress::render(frame, app, chunks[0]);
                widgets::thought_stream::render(frame, app, chunks[1]);
            } else {
                widgets::progress::render(frame, app, frame.area());
            }
        }
        AppMode::Help => render_help(frame, app),
    }
}

/// Handle input events
fn handle_event(app: &mut App, event: Event) -> Result<()> {
    // Handle special modes separately
    if app.mode == AppMode::Search {
        return handle_search_input(app, event);
    }
    if app.mode == AppMode::SortMenu {
        return handle_sort_menu_input(app, event);
    }
    if app.mode == AppMode::ColumnConfig {
        return handle_column_config_input(app, event);
    }
    if app.mode == AppMode::Tests {
        return handle_tests_input(app, event);
    }

    match event {
        Event::Key(key) => {
            match (key.code, key.modifiers) {
                // Force quit
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    app.should_quit = true;
                }
                // Quit or back
                (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => {
                    match app.mode {
                        AppMode::Help | AppMode::DocumentViewer | AppMode::StubList | AppMode::DocumentContent | AppMode::Tests => {
                            app.mode = AppMode::Dashboard;
                        }
                        _ => {
                            app.should_quit = true;
                        }
                    }
                }
                // Help
                (KeyCode::Char('?'), _) => {
                    app.mode = AppMode::Help;
                }
                // Navigation
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    match app.mode {
                        AppMode::DocumentContent => app.scroll_up(),
                        AppMode::StubList => app.scroll_up(),
                        _ => app.previous_document(),
                    }
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    match app.mode {
                        AppMode::DocumentContent => app.scroll_down(),
                        AppMode::StubList => app.scroll_down(),
                        _ => app.next_document(),
                    }
                }
                // Page scroll - moves cursor along with scroll in Dashboard
                (KeyCode::PageUp, modifiers) if modifiers.contains(KeyModifiers::SHIFT) => {
                    // Shift+PgUp: Go to top
                    match app.mode {
                        AppMode::DocumentContent | AppMode::StubList => {
                            app.scroll_offset = 0;
                        }
                        AppMode::Dashboard => {
                            app.selected_document = 0;
                            app.list_offset = 0;
                        }
                        _ => {}
                    }
                }
                (KeyCode::PageDown, modifiers) if modifiers.contains(KeyModifiers::SHIFT) => {
                    // Shift+PgDn: Go to bottom
                    match app.mode {
                        AppMode::DocumentContent | AppMode::StubList => {
                            app.scroll_offset = usize::MAX; // Will be clamped during render
                        }
                        AppMode::Dashboard => {
                            let count = app.visible_document_count();
                            if count > 0 {
                                app.selected_document = count - 1;
                                app.ensure_selection_visible();
                            }
                        }
                        _ => {}
                    }
                }
                // Home: Go to top
                (KeyCode::Home, _) => {
                    match app.mode {
                        AppMode::DocumentContent | AppMode::StubList => {
                            app.scroll_offset = 0;
                        }
                        AppMode::Dashboard => {
                            app.selected_document = 0;
                            app.list_offset = 0;
                        }
                        _ => {}
                    }
                }
                // End: Go to bottom
                (KeyCode::End, _) => {
                    match app.mode {
                        AppMode::DocumentContent | AppMode::StubList => {
                            app.scroll_offset = usize::MAX;
                        }
                        AppMode::Dashboard => {
                            let count = app.visible_document_count();
                            if count > 0 {
                                app.selected_document = count - 1;
                                app.ensure_selection_visible();
                            }
                        }
                        _ => {}
                    }
                }
                (KeyCode::PageUp, _) => {
                    match app.mode {
                        AppMode::DocumentContent | AppMode::StubList => {
                            for _ in 0..10 { app.scroll_up(); }
                        }
                        AppMode::Dashboard => {
                            // Move cursor up by page size
                            app.selected_document = app.selected_document.saturating_sub(10);
                            app.ensure_selection_visible();
                        }
                        _ => {}
                    }
                }
                (KeyCode::PageDown, _) | (KeyCode::Char(' '), _) => {
                    match app.mode {
                        AppMode::DocumentContent | AppMode::StubList => {
                            for _ in 0..10 { app.scroll_down(); }
                        }
                        AppMode::Dashboard => {
                            // Move cursor down by page size
                            let count = app.visible_document_count();
                            if count > 0 {
                                app.selected_document = (app.selected_document + 10).min(count - 1);
                                app.ensure_selection_visible();
                            }
                        }
                        _ => {}
                    }
                }
                // View document details
                (KeyCode::Enter, _) => {
                    if app.visible_document_count() > 0 {
                        app.scroll_offset = 0;
                        app.mode = AppMode::DocumentViewer;
                    }
                }
                // Dashboard
                (KeyCode::Char('d'), _) => {
                    app.mode = AppMode::Dashboard;
                }
                // Stubs
                (KeyCode::Char('s'), _) => {
                    if app.visible_document_count() > 0 {
                        app.mode = AppMode::StubList;
                    }
                }
                // Content - toggle if already in content view
                (KeyCode::Char('c'), _) => {
                    if app.mode == AppMode::DocumentContent {
                        app.mode = AppMode::Dashboard;
                    } else if app.visible_document_count() > 0 {
                        app.scroll_offset = 0;
                        app.mode = AppMode::DocumentContent;
                    }
                }
                // Search
                (KeyCode::Char('/'), _) => {
                    app.mode = AppMode::Search;
                }
                // Open sort menu
                (KeyCode::Char('o'), _) => {
                    if app.mode == AppMode::Dashboard {
                        app.mode = AppMode::SortMenu;
                    }
                }
                // Open column config
                (KeyCode::Char('v'), _) => {
                    if app.mode == AppMode::Dashboard {
                        app.column_menu_index = 0;
                        app.mode = AppMode::ColumnConfig;
                    }
                }
                // Reverse sort (quick toggle)
                (KeyCode::Char('r'), _) => {
                    if app.mode == AppMode::Dashboard {
                        app.toggle_sort_direction();
                        let dir = if app.sort_ascending { "ascending" } else { "descending" };
                        app.set_status(format!("Sort: {}", dir));
                    }
                }
                // Tab: cycle dashboard views or navigate docs
                (KeyCode::Tab, _) => {
                    match app.mode {
                        AppMode::Dashboard => {
                            app.dashboard_view = app.dashboard_view.next();
                            app.set_status(format!("View: {}", app.dashboard_view.label()));
                        }
                        AppMode::DocumentViewer | AppMode::StubList | AppMode::DocumentContent => {
                            app.next_document();
                            app.scroll_offset = 0;
                        }
                        _ => {}
                    }
                }
                (KeyCode::BackTab, _) => {
                    match app.mode {
                        AppMode::Dashboard => {
                            app.dashboard_view = app.dashboard_view.prev();
                            app.set_status(format!("View: {}", app.dashboard_view.label()));
                        }
                        AppMode::DocumentViewer | AppMode::StubList | AppMode::DocumentContent => {
                            app.previous_document();
                            app.scroll_offset = 0;
                        }
                        _ => {}
                    }
                }
                // Navigate to next/prev doc from viewing panes
                (KeyCode::Char('n'), _) => {
                    match app.mode {
                        AppMode::DocumentViewer | AppMode::StubList | AppMode::DocumentContent => {
                            app.next_document();
                            app.scroll_offset = 0;
                        }
                        _ => {}
                    }
                }
                (KeyCode::Char('p'), _) => {
                    match app.mode {
                        AppMode::DocumentViewer | AppMode::StubList | AppMode::DocumentContent => {
                            app.previous_document();
                            app.scroll_offset = 0;
                        }
                        _ => {}
                    }
                }
                // Clear filter
                (KeyCode::Char('x'), _) => {
                    if app.mode == AppMode::Dashboard && !app.filter_text.is_empty() {
                        app.filter_text.clear();
                        app.apply_filter();
                        app.set_status("Filter cleared");
                    }
                }
                // Tests view
                (KeyCode::Char('t'), _) => {
                    if app.mode == AppMode::Dashboard {
                        // Load test files from config
                        if let Some(test_dir) = crate::config::get_test_dir() {
                            app.test_state.load_test_files(&test_dir);
                            app.mode = AppMode::Tests;
                            app.set_status(format!("Loaded {} test files", app.test_state.test_files.len()));
                        } else {
                            app.set_status("No test_dir configured");
                        }
                    }
                }
                _ => {}
            }
        }
        Event::Mouse(mouse) => {
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    match app.mode {
                        AppMode::DocumentContent | AppMode::StubList => {
                            for _ in 0..3 { app.scroll_up(); }
                        }
                        AppMode::Dashboard => {
                            app.previous_document();
                        }
                        _ => {}
                    }
                }
                MouseEventKind::ScrollDown => {
                    match app.mode {
                        AppMode::DocumentContent | AppMode::StubList => {
                            for _ in 0..3 { app.scroll_down(); }
                        }
                        AppMode::Dashboard => {
                            app.next_document();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Event::Tick => {
            app.tick();
        }
        Event::Resize(_, _) => {
            // Terminal will re-render automatically
        }
    }

    Ok(())
}

/// Handle search mode input
fn handle_search_input(app: &mut App, event: Event) -> Result<()> {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Esc => {
                app.mode = AppMode::Dashboard;
            }
            KeyCode::Enter => {
                app.apply_filter();
                app.mode = AppMode::Dashboard;
                let count = app.visible_document_count();
                app.set_status(format!("Found {} documents", count));
            }
            KeyCode::Backspace => {
                app.filter_text.pop();
            }
            KeyCode::Char(c) => {
                app.filter_text.push(c);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle sort menu input
fn handle_sort_menu_input(app: &mut App, event: Event) -> Result<()> {
    if let Event::Key(key) = event {
        let fields = app::SortField::all();
        let current_idx = fields.iter().position(|&f| f == app.sort_field).unwrap_or(0);

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::Dashboard;
            }
            KeyCode::Enter => {
                app.mode = AppMode::Dashboard;
                app.set_status(format!("Sorted by {} {}",
                    app.sort_field.label(),
                    if app.sort_ascending { "↑" } else { "↓" }
                ));
            }
            // Cycle through sort fields with o or left/right
            KeyCode::Char('o') | KeyCode::Right | KeyCode::Char('l') => {
                app.sort_field = fields[(current_idx + 1) % fields.len()];
                app.sort_documents();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.sort_field = fields[(current_idx + fields.len() - 1) % fields.len()];
                app.sort_documents();
            }
            // Toggle direction with up/down
            KeyCode::Up | KeyCode::Char('k') => {
                if !app.sort_ascending {
                    app.sort_ascending = true;
                    app.sort_documents();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.sort_ascending {
                    app.sort_ascending = false;
                    app.sort_documents();
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle column config input
fn handle_column_config_input(app: &mut App, event: Event) -> Result<()> {
    if let Event::Key(key) = event {
        let all_cols = app::Column::all();
        let mut config_changed = false;

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => {
                app.mode = AppMode::Dashboard;
            }
            KeyCode::Enter => {
                app.mode = AppMode::Dashboard;
            }
            // Navigate through columns
            KeyCode::Up | KeyCode::Char('k') => {
                if app.column_menu_index > 0 {
                    app.column_menu_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.column_menu_index < all_cols.len() - 1 {
                    app.column_menu_index += 1;
                }
            }
            // Toggle column visibility with space
            KeyCode::Char(' ') => {
                let col = all_cols[app.column_menu_index];
                app.toggle_column(col);
                config_changed = true;
            }
            // Reorder columns with [ and ]
            KeyCode::Char('[') => {
                let col = all_cols[app.column_menu_index];
                app.move_column_up(col);
                config_changed = true;
            }
            KeyCode::Char(']') => {
                let col = all_cols[app.column_menu_index];
                app.move_column_down(col);
                config_changed = true;
            }
            _ => {}
        }

        // Save config if columns changed
        if config_changed {
            if let Err(e) = crate::config::save_dashboard_columns(&app.visible_columns) {
                app.set_status(format!("Failed to save config: {}", e));
            }
        }
    }
    Ok(())
}

/// Handle tests view input
fn handle_tests_input(app: &mut App, event: Event) -> Result<()> {
    use crate::tui::app::TestFocus;

    if let Event::Key(key) = event {
        match key.code {
            // Back to dashboard
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::Dashboard;
            }
            // Navigate up
            KeyCode::Up | KeyCode::Char('k') => {
                match app.test_state.focus {
                    TestFocus::Commands => {
                        app.test_state.prev_command();
                    }
                    TestFocus::Files => {
                        app.test_state.prev_file();
                        // Load file content when navigating
                        if let Some(path) = app.test_state.selected_file().cloned() {
                            app.test_state.load_file_content(&path);
                            app.test_state.preview_scroll = 0;
                            app.test_state.selected_line = 1;
                        }
                    }
                    TestFocus::Preview => {
                        app.test_state.prev_line();
                    }
                }
            }
            // Navigate down
            KeyCode::Down | KeyCode::Char('j') => {
                match app.test_state.focus {
                    TestFocus::Commands => {
                        app.test_state.next_command();
                    }
                    TestFocus::Files => {
                        app.test_state.next_file();
                        // Load file content when navigating
                        if let Some(path) = app.test_state.selected_file().cloned() {
                            app.test_state.load_file_content(&path);
                            app.test_state.preview_scroll = 0;
                            app.test_state.selected_line = 1;
                        }
                    }
                    TestFocus::Preview => {
                        app.test_state.next_line();
                        // Ensure line is visible (assume ~20 lines visible)
                        app.test_state.ensure_line_visible(20);
                    }
                }
            }
            // Cycle focus: Files -> Commands -> Preview -> Files
            KeyCode::Tab => {
                app.test_state.cycle_focus();
            }
            // Left/Right to move between Files and Commands (skip Preview)
            KeyCode::Left | KeyCode::Char('h') => {
                if app.test_state.focus == TestFocus::Commands {
                    app.test_state.focus = TestFocus::Files;
                } else if app.test_state.focus == TestFocus::Preview {
                    app.test_state.focus = TestFocus::Commands;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if app.test_state.focus == TestFocus::Files {
                    app.test_state.focus = TestFocus::Commands;
                } else if app.test_state.focus == TestFocus::Commands {
                    app.test_state.focus = TestFocus::Preview;
                }
            }
            // Run selected command on selected file
            KeyCode::Enter => {
                run_test_command(app);
            }
            // Reset file to original content
            KeyCode::Char('R') => {
                if let Some(path) = app.test_state.selected_file().cloned() {
                    app.test_state.reset_file(&path);
                    app.set_status("File reset to original content");
                }
            }
            // Reload test files (lowercase r)
            KeyCode::Char('r') => {
                if let Some(test_dir) = app.test_state.test_dir.clone() {
                    app.test_state.load_test_files(&test_dir);
                    app.set_status(format!("Reloaded {} test files", app.test_state.test_files.len()));
                }
            }
            // Page scroll for preview (works in any focus mode)
            KeyCode::PageUp => {
                for _ in 0..5 {
                    app.test_state.scroll_preview_up();
                }
                // Move selected line if it goes out of view
                if app.test_state.selected_line > app.test_state.preview_scroll + 20 {
                    app.test_state.selected_line = app.test_state.preview_scroll + 1;
                }
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    app.test_state.scroll_preview_down();
                }
                // Move selected line if it goes out of view
                if app.test_state.selected_line <= app.test_state.preview_scroll {
                    app.test_state.selected_line = app.test_state.preview_scroll + 1;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Run a test command (either shell command or in-memory modify command)
fn run_test_command(app: &mut App) {
    use std::process::Command;
    use std::time::Instant;

    let (file, cmd) = match (
        app.test_state.selected_file().cloned(),
        app.test_state.selected_command().cloned(),
    ) {
        (Some(f), Some(c)) => (f, c),
        _ => {
            app.set_status("Select a file and command first");
            return;
        }
    };

    // Load file content if not loaded
    app.test_state.load_file_content(&file);

    let start = Instant::now();

    // Check if this is a modify command (in-memory operation)
    if cmd.command.starts_with("__modify:") {
        let result = execute_modify_command(app, &file, &cmd.command);
        let duration = start.elapsed();

        let test_result = app::TestResult {
            command_id: cmd.id.clone(),
            success: result.is_ok(),
            output: result.unwrap_or_else(|e| e),
            duration,
            timestamp: Instant::now(),
        };
        app.test_state.add_result(test_result);

        app.set_status(format!("{} completed", cmd.name));
    } else if cmd.command.starts_with("__anchor:") {
        let result = execute_anchor_command(app, &file, &cmd.command);
        let duration = start.elapsed();

        let test_result = app::TestResult {
            command_id: cmd.id.clone(),
            success: result.is_ok(),
            output: result.unwrap_or_else(|e| e),
            duration,
            timestamp: Instant::now(),
        };
        app.test_state.add_result(test_result);

        app.set_status(format!("{} completed", cmd.name));
    } else {
        // Regular shell command - use in-memory content
        let file_path = file.display().to_string();

        // For shell commands, we need to write temp content if modified
        let content = app.test_state.get_content(&file).cloned();
        let use_temp = app.test_state.is_modified(&file);

        let actual_path = if use_temp {
            // Write to temp file for shell commands
            if let Some(ref content) = content {
                let temp_path = std::env::temp_dir().join("ddoc_test_temp.md");
                if std::fs::write(&temp_path, content).is_ok() {
                    temp_path.display().to_string()
                } else {
                    file_path.clone()
                }
            } else {
                file_path.clone()
            }
        } else {
            file_path.clone()
        };

        let command_str = cmd.command.replace("{file}", &format!("\"{}\"", actual_path));

        app.test_state.running = Some(cmd.id.clone());
        app.set_status(format!("Running: {}", cmd.name));

        let output = Command::new("sh")
            .arg("-c")
            .arg(&command_str)
            .output();

        let duration = start.elapsed();
        app.test_state.running = None;

        match output {
            Ok(out) => {
                let success = out.status.success();
                let output_str = if success {
                    String::from_utf8_lossy(&out.stdout).to_string()
                } else {
                    String::from_utf8_lossy(&out.stderr).to_string()
                };

                let result = app::TestResult {
                    command_id: cmd.id.clone(),
                    success,
                    output: output_str,
                    duration,
                    timestamp: Instant::now(),
                };

                app.test_state.add_result(result);

                if success {
                    app.set_status(format!("{} completed in {:.2}s", cmd.name, duration.as_secs_f64()));
                } else {
                    app.set_status(format!("{} failed", cmd.name));
                }
            }
            Err(e) => {
                let result = app::TestResult {
                    command_id: cmd.id.clone(),
                    success: false,
                    output: format!("Failed to execute: {}", e),
                    duration,
                    timestamp: Instant::now(),
                };
                app.test_state.add_result(result);
                app.set_status(format!("Error: {}", e));
            }
        }
    }
}

/// Execute an in-memory modify command
fn execute_modify_command(app: &mut App, file: &std::path::PathBuf, command: &str) -> Result<String, String> {
    use doc_doctor_application::{ApplicationSwitchboard, NewStub, Switchboard};
    use doc_doctor_domain::EmbeddedSchemaProvider;
    use doc_doctor_parser_yaml::YamlParser;
    use std::sync::Arc;

    let content = app.test_state.get_content(file)
        .ok_or("File content not loaded")?
        .clone();

    // Parse the command: __modify:action:arg
    let parts: Vec<&str> = command.split(':').collect();
    if parts.len() < 2 {
        return Err("Invalid modify command".to_string());
    }

    let action = parts[1];

    match action {
        "add_stub" => {
            let stub_type = parts.get(2).unwrap_or(&"expand");

            // Create switchboard
            let parser = Arc::new(YamlParser::new());
            let writer = Arc::clone(&parser);
            let schema_provider = Arc::new(EmbeddedSchemaProvider);
            let switchboard = ApplicationSwitchboard::new(parser, writer, schema_provider);

            let new_stub = NewStub {
                stub_type: stub_type.to_string(),
                description: format!("Test {} stub added via test runner", stub_type),
                priority: None,
                stub_form: None,
                anchor: None,
            };

            match switchboard.add_stub(&content, new_stub) {
                Ok(result) => {
                    app.test_state.set_content(file, result.updated_content);
                    Ok(format!("Added {} stub at index {}", stub_type, result.stub_index))
                }
                Err(e) => Err(format!("Failed to add stub: {}", e))
            }
        }
        "remove_stub" | "resolve_stub" => {
            let index: usize = parts.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let parser = Arc::new(YamlParser::new());
            let writer = Arc::clone(&parser);
            let schema_provider = Arc::new(EmbeddedSchemaProvider);
            let switchboard = ApplicationSwitchboard::new(parser, writer, schema_provider);

            match switchboard.resolve_stub(&content, index) {
                Ok(result) => {
                    app.test_state.set_content(file, result.updated_content);
                    Ok(format!("Resolved stub at index {} (was {:?})", index, result.resolved_stub.stub_type))
                }
                Err(e) => Err(format!("Failed to resolve stub: {}", e))
            }
        }
        "reset" => {
            app.test_state.reset_file(file);
            Ok("File reset to original content".to_string())
        }
        _ => Err(format!("Unknown modify action: {}", action))
    }
}

/// Execute an anchor command (add/remove anchor in content, link/unlink to stub)
fn execute_anchor_command(app: &mut App, file: &std::path::PathBuf, command: &str) -> Result<String, String> {
    use doc_doctor_application::{ApplicationSwitchboard, Switchboard};
    use doc_doctor_domain::EmbeddedSchemaProvider;
    use doc_doctor_parser_yaml::YamlParser;
    use std::sync::Arc;

    let parts: Vec<&str> = command.split(':').collect();
    let action = parts.get(1).unwrap_or(&"");

    let selected_line = app.test_state.selected_line;
    if selected_line == 0 {
        return Err("No line selected. Focus on preview and select a line first.".to_string());
    }

    let content = app.test_state.get_content(file)
        .ok_or("File content not loaded")?
        .clone();

    match *action {
        "add" => {
            // Generate a unique anchor ID based on timestamp
            let anchor_id = format!("anchor-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() % 10000)
                .unwrap_or(0));

            if let Some(new_content) = app.test_state.insert_anchor_at_line(file, selected_line, &anchor_id) {
                app.test_state.set_content(file, new_content);
                // Update line count
                if let Some(c) = app.test_state.get_content(file) {
                    app.test_state.content_line_count = c.lines().count();
                }
                Ok(format!("Added ^{} at line {}", anchor_id, selected_line))
            } else {
                Err(format!("Failed to add anchor at line {}", selected_line))
            }
        }
        "remove" => {
            // Find the anchor at this line
            if let Some(anchor_id) = app.test_state.get_anchor_at_line(file, selected_line) {
                if let Some(new_content) = app.test_state.remove_anchor_from_line(file, selected_line, &anchor_id) {
                    app.test_state.set_content(file, new_content);
                    Ok(format!("Removed ^{} from line {}", anchor_id, selected_line))
                } else {
                    Err("Failed to remove anchor".to_string())
                }
            } else {
                Err(format!("No anchor found at line {}", selected_line))
            }
        }
        "link" => {
            let stub_index: usize = parts.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // Get anchor at selected line
            let anchor_id = app.test_state.get_anchor_at_line(file, selected_line)
                .ok_or(format!("No anchor found at line {}. Add an anchor first.", selected_line))?;

            // Create switchboard to link anchor to stub
            let parser = Arc::new(YamlParser::new());
            let writer = Arc::clone(&parser);
            let schema_provider = Arc::new(EmbeddedSchemaProvider);
            let switchboard = ApplicationSwitchboard::new(parser, writer, schema_provider);

            match switchboard.link_stub_anchor(&content, stub_index, &anchor_id) {
                Ok(result) => {
                    app.test_state.set_content(file, result.updated_content);
                    Ok(format!("Linked ^{} to stub {}", anchor_id, stub_index))
                }
                Err(e) => Err(format!("Failed to link anchor: {}", e))
            }
        }
        "unlink" => {
            let stub_index: usize = parts.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // Get anchor at selected line
            let anchor_id = app.test_state.get_anchor_at_line(file, selected_line)
                .ok_or(format!("No anchor found at line {}", selected_line))?;

            // Create switchboard to unlink anchor from stub
            let parser = Arc::new(YamlParser::new());
            let writer = Arc::clone(&parser);
            let schema_provider = Arc::new(EmbeddedSchemaProvider);
            let switchboard = ApplicationSwitchboard::new(parser, writer, schema_provider);

            match switchboard.unlink_stub_anchor(&content, stub_index, &anchor_id) {
                Ok(result) => {
                    app.test_state.set_content(file, result.updated_content);
                    Ok(format!("Unlinked ^{} from stub {}", anchor_id, stub_index))
                }
                Err(e) => Err(format!("Failed to unlink anchor: {}", e))
            }
        }
        _ => Err(format!("Unknown anchor action: {}", action))
    }
}

/// Render help screen
fn render_help(frame: &mut Frame, _app: &App) {
    use ratatui::{
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };

    let block = Block::default()
        .title(Span::styled(" Help ", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border());

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Navigation", styles::highlight()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ↑/k       ", styles::value()),
            Span::styled("Move up / Scroll up", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    ↓/j       ", styles::value()),
            Span::styled("Move down / Scroll down", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    PgUp      ", styles::value()),
            Span::styled("Page up (moves cursor)", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    PgDn/Space", styles::value()),
            Span::styled(" Page down (moves cursor)", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    Home      ", styles::value()),
            Span::styled("Go to top", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    End       ", styles::value()),
            Span::styled("Go to bottom", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    n/Tab     ", styles::value()),
            Span::styled("Next document (in viewer)", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    p/S-Tab   ", styles::value()),
            Span::styled("Previous document (in viewer)", styles::subtitle()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Views", styles::highlight()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Enter     ", styles::value()),
            Span::styled("View document details", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    s         ", styles::value()),
            Span::styled("View stubs list", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    c         ", styles::value()),
            Span::styled("View/close document content", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    d         ", styles::value()),
            Span::styled("Return to dashboard", styles::subtitle()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Search & Sort (Dashboard)", styles::highlight()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    /         ", styles::value()),
            Span::styled("Search/filter documents", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    x         ", styles::value()),
            Span::styled("Clear filter", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    o         ", styles::value()),
            Span::styled("Open sort menu (←→ field, ↑↓ dir)", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    v         ", styles::value()),
            Span::styled("Configure columns (Space toggle, [] reorder)", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    r         ", styles::value()),
            Span::styled("Reverse sort direction (quick)", styles::subtitle()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  General", styles::highlight()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    q/Esc     ", styles::value()),
            Span::styled("Quit / Back to dashboard", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    ?         ", styles::value()),
            Span::styled("Show this help", styles::subtitle()),
        ]),
        Line::from(vec![
            Span::styled("    Mouse     ", styles::value()),
            Span::styled("Scroll with mouse wheel", styles::subtitle()),
        ]),
    ];

    let paragraph = Paragraph::new(help_text).block(block);
    frame.render_widget(paragraph, frame.area());
}

//! Dashboard Command
//!
//! Interactive TUI dashboard for vault health overview.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::config::{get_dashboard_columns, get_test_dir, resolve_path, should_ignore_path};
use crate::output::OutputFormat;
use crate::tui::{
    app::{App, AppMode, DocumentSummary, VaultStats},
    run_tui,
    widgets::ConsoleProgress,
};

#[derive(Args)]
pub struct DashboardCommand {
    #[command(subcommand)]
    pub subcommand: Option<DashboardSubcommand>,

    /// Path to vault or document directory (can be a path alias from config)
    #[arg(default_value = ".")]
    pub path: String,

    /// Run in non-interactive mode (just show stats and exit)
    #[arg(long)]
    pub no_interactive: bool,
}

#[derive(Subcommand)]
pub enum DashboardSubcommand {
    /// Run the test runner UI (requires test_dir in config)
    Tests,

    /// Show vault health overview (default)
    Vault {
        /// Path to vault (can be a path alias)
        #[arg(default_value = ".")]
        path: String,

        /// Run in non-interactive mode
        #[arg(long)]
        no_interactive: bool,
    },
}

impl DashboardCommand {
    pub fn run(&self, _format: OutputFormat, verbose: bool) -> Result<()> {
        // Handle subcommands
        match &self.subcommand {
            Some(DashboardSubcommand::Tests) => {
                return self.run_tests(verbose);
            }
            Some(DashboardSubcommand::Vault { path, no_interactive }) => {
                return self.run_vault(path, *no_interactive, verbose);
            }
            None => {
                // Default: run vault dashboard with top-level args
            }
        }

        // Resolve path alias if configured
        let path = resolve_path(&self.path);

        if verbose {
            if path.to_string_lossy() != self.path {
                eprintln!("Resolved '{}' to: {}", self.path, path.display());
            }
            eprintln!("Scanning vault: {}", path.display());
        }

        // Scan vault for markdown files
        let pattern = path.join("**/*.md");
        let pattern_str = pattern.to_string_lossy();

        let all_files: Vec<PathBuf> = glob::glob(&pattern_str)?
            .filter_map(|r| r.ok())
            .collect();

        // Filter out ignored paths
        let files: Vec<PathBuf> = all_files
            .into_iter()
            .filter(|f| !should_ignore_path(f))
            .collect();

        if files.is_empty() {
            println!("No markdown files found in {}", path.display());
            return Ok(());
        }

        if verbose {
            eprintln!("Found {} markdown files (after ignore filters)", files.len());
        }

        // Create progress bar for scanning
        let progress = ConsoleProgress::new(files.len() as u64, "Analyzing documents...");

        // Analyze each document
        let parser = doc_doctor_parser_yaml::YamlParser::new();
        let mut documents = Vec::new();
        let mut total_health = 0.0;
        let mut total_refinement = 0.0;
        let mut total_stubs = 0;
        let mut blocking_stubs = 0;

        for file in &files {
            progress.set_message(&format!("Analyzing {}", file.file_name().unwrap().to_string_lossy()));

            if let Ok(content) = std::fs::read_to_string(file) {
                use doc_doctor_domain::DocumentParser;

                if let Ok(props) = parser.parse(&content) {
                    let health = doc_doctor_domain::calculate_health(
                        props.refinement.value(),
                        &props.stubs,
                    );

                    let stub_count = props.stubs.len();
                    let blocking = props.stubs.iter()
                        .filter(|s| matches!(s.stub_form, doc_doctor_domain::StubForm::Blocking))
                        .count();

                    total_health += health;
                    total_refinement += props.refinement.value();
                    total_stubs += stub_count;
                    blocking_stubs += blocking;

                    // Get file metadata
                    let metadata = std::fs::metadata(file).ok();
                    let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                    let modified = metadata.as_ref().and_then(|m| m.modified().ok());
                    let created = metadata.as_ref().and_then(|m| m.created().ok());
                    let line_count = content.lines().count();

                    // Get last git commit info
                    let last_commit = get_last_commit(file);

                    // Calculate relative path from vault root
                    let relative_path = file.strip_prefix(&path)
                        .ok()
                        .map(|p| p.to_string_lossy().to_string());

                    // Extract raw origin value from YAML (before enum normalization)
                    let raw_origin = extract_raw_field(&content, "origin");

                    documents.push(DocumentSummary {
                        path: file.clone(),
                        relative_path,
                        title: props.title.clone(),
                        health,
                        refinement: props.refinement.value(),
                        stub_count,
                        audience: props.audience.to_string(),
                        form: props.form.to_string(),
                        origin: raw_origin,
                        stubs: props.stubs.clone(),
                        content: content.clone(),
                        line_count,
                        file_size,
                        modified,
                        created,
                        author: None, // Not yet supported in L1Properties
                        last_commit,
                    });
                }
            }

            progress.inc(1);
        }

        let doc_count = documents.len();
        let skipped = files.len() - doc_count;
        progress.finish_with_message(&format!(
            "Analyzed {} documents ({} skipped - no frontmatter)",
            doc_count, skipped
        ));

        // Calculate folder statistics
        let mut folders: HashSet<PathBuf> = HashSet::new();
        let mut top_folder_total: HashMap<String, usize> = HashMap::new();
        let mut top_folder_with_fm: HashMap<String, usize> = HashMap::new();

        for file in &files {
            // Collect all parent folders
            let mut current = file.parent();
            while let Some(parent) = current {
                if parent.starts_with(&path) && parent != path {
                    folders.insert(parent.to_path_buf());
                }
                current = parent.parent();
            }

            // Get top-level folder relative to vault
            if let Ok(relative) = file.strip_prefix(&path) {
                if let Some(first_component) = relative.components().next() {
                    let top_folder = first_component.as_os_str().to_string_lossy().to_string();
                    *top_folder_total.entry(top_folder.clone()).or_insert(0) += 1;
                }
            }
        }

        // Count files with frontmatter by top folder
        for doc in &documents {
            if let Ok(relative) = doc.path.strip_prefix(&path) {
                if let Some(first_component) = relative.components().next() {
                    let top_folder = first_component.as_os_str().to_string_lossy().to_string();
                    *top_folder_with_fm.entry(top_folder).or_insert(0) += 1;
                }
            }
        }

        // Build files_by_top_folder Vec sorted by total count
        let mut files_by_top_folder: Vec<(String, usize, usize)> = top_folder_total
            .into_iter()
            .map(|(folder, total)| {
                let with_fm = *top_folder_with_fm.get(&folder).unwrap_or(&0);
                (folder, total, with_fm)
            })
            .collect();
        files_by_top_folder.sort_by(|a, b| b.1.cmp(&a.1));

        let stats = VaultStats {
            total_documents: doc_count,
            average_health: if doc_count > 0 { total_health / doc_count as f64 } else { 0.0 },
            average_refinement: if doc_count > 0 { total_refinement / doc_count as f64 } else { 0.0 },
            total_stubs,
            blocking_stubs,
            documents_by_audience: Vec::new(), // TODO: Calculate
            documents_by_form: Vec::new(),     // TODO: Calculate
            health_distribution: Vec::new(),   // TODO: Calculate
            total_files: files.len(),
            files_with_frontmatter: doc_count,
            files_without_frontmatter: skipped,
            total_folders: folders.len(),
            files_by_top_folder,
            vault_root: Some(path.clone()),
        };

        // Sort documents by health (lowest first - needs attention)
        documents.sort_by(|a, b| a.health.partial_cmp(&b.health).unwrap());

        if self.no_interactive {
            // Print summary and exit
            print_summary(&stats, &documents, files.len(), skipped);
            return Ok(());
        }

        // Run interactive TUI
        let mut app = App::new();
        app.vault_stats = Some(stats);
        app.documents = documents;
        app.visible_columns = get_dashboard_columns();

        run_tui(app)?;

        Ok(())
    }

    /// Run the test runner UI
    fn run_tests(&self, verbose: bool) -> Result<()> {
        // Get test directory from config
        let test_dir = match get_test_dir() {
            Some(dir) => dir,
            None => {
                eprintln!("Error: No test_dir configured.");
                eprintln!("Add 'test_dir: /path/to/test/files' to your config.");
                return Ok(());
            }
        };

        if verbose {
            eprintln!("Test directory: {}", test_dir.display());
        }

        // Check if directory exists
        if !test_dir.exists() {
            eprintln!("Error: Test directory does not exist: {}", test_dir.display());
            return Ok(());
        }

        // Create app and start in Tests mode
        let mut app = App::new();
        app.mode = AppMode::Tests;
        app.test_state.test_dir = Some(test_dir.clone());
        app.test_state.load_test_files(&test_dir);

        if verbose {
            eprintln!("Loaded {} test files", app.test_state.test_files.len());
        }

        run_tui(app)?;

        Ok(())
    }

    /// Run the vault dashboard (used by subcommand)
    fn run_vault(&self, path: &str, no_interactive: bool, verbose: bool) -> Result<()> {
        let resolved_path = resolve_path(path);

        if verbose {
            if resolved_path.to_string_lossy() != path {
                eprintln!("Resolved '{}' to: {}", path, resolved_path.display());
            }
            eprintln!("Scanning vault: {}", resolved_path.display());
        }

        // Create a temporary command with the resolved settings
        let temp_cmd = DashboardCommand {
            subcommand: None,
            path: resolved_path.to_string_lossy().to_string(),
            no_interactive,
        };

        // Use the main run method (it will skip subcommand matching since subcommand is None)
        temp_cmd.run(crate::output::OutputFormat::Human, verbose)
    }
}

/// Get the last git commit info for a file
fn get_last_commit(file: &Path) -> Option<String> {
    // Get the directory containing the file to run git from there
    let dir = file.parent()?;
    let filename = file.file_name()?;

    let output = Command::new("git")
        .current_dir(dir)
        .args(["log", "-1", "--format=%h %s", "--"])
        .arg(filename)
        .output()
        .ok()?;

    if output.status.success() {
        let commit = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        if commit.is_empty() {
            None
        } else {
            Some(commit)
        }
    } else {
        None
    }
}

fn print_summary(stats: &VaultStats, documents: &[DocumentSummary], total_files: usize, skipped: usize) {
    use crate::tui::theme::console_styles;
    use console::style;

    println!();
    println!("{}", console_styles::title("Vault Health Summary"));
    println!("{}", "─".repeat(50));
    println!();

    println!(
        "  {} {} {} {}",
        console_styles::dim("Scanned:"),
        style(total_files).white(),
        console_styles::dim("files,"),
        if skipped > 0 {
            style(format!("{} skipped (no frontmatter)", skipped)).dim()
        } else {
            style("all parsed".to_string()).green()
        }
    );

    println!(
        "  {} {}",
        console_styles::dim("Documents:"),
        style(stats.total_documents).white().bold()
    );

    let health_styled = if stats.average_health >= 0.8 {
        style(format!("{:.0}%", stats.average_health * 100.0)).green()
    } else if stats.average_health >= 0.5 {
        style(format!("{:.0}%", stats.average_health * 100.0)).yellow()
    } else {
        style(format!("{:.0}%", stats.average_health * 100.0)).red()
    };

    println!(
        "  {} {}",
        console_styles::dim("Avg Health:"),
        health_styled
    );

    println!(
        "  {} {} ({} blocking)",
        console_styles::dim("Total Stubs:"),
        if stats.total_stubs > 0 {
            style(stats.total_stubs.to_string()).yellow()
        } else {
            style(stats.total_stubs.to_string()).green()
        },
        if stats.blocking_stubs > 0 {
            style(stats.blocking_stubs.to_string()).red()
        } else {
            style(stats.blocking_stubs.to_string()).dim()
        }
    );

    // Show documents needing attention (lowest health)
    let needs_attention: Vec<_> = documents.iter()
        .filter(|d| d.health < 0.7)
        .take(5)
        .collect();

    if !needs_attention.is_empty() {
        println!();
        println!("{}", console_styles::warning("Documents needing attention:"));
        for doc in needs_attention {
            let title = doc.title.as_deref()
                .unwrap_or_else(|| doc.path.file_name().unwrap().to_str().unwrap());

            let health = if doc.health < 0.5 {
                style(format!("{:.0}%", doc.health * 100.0)).red()
            } else {
                style(format!("{:.0}%", doc.health * 100.0)).yellow()
            };

            println!("  {} {} - {}", health, title, console_styles::dim(&doc.path.display().to_string()));
        }
    }

    println!();
    println!("{}", console_styles::dim("Run without --no-interactive for full TUI dashboard"));
    println!();
}

/// Extract a raw field value from YAML frontmatter
/// Returns the value as-is, or "-" if not found
fn extract_raw_field(content: &str, field: &str) -> String {
    // Find frontmatter boundaries
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return "-".to_string();
    }

    // Find the closing ---
    let end_idx = lines.iter()
        .skip(1)
        .position(|line| line.trim() == "---")
        .map(|i| i + 1);

    let Some(end) = end_idx else {
        return "-".to_string();
    };

    // Search for the field in frontmatter lines
    let pattern = format!("{}:", field);
    for line in &lines[1..end] {
        let trimmed = line.trim();
        if trimmed.starts_with(&pattern) {
            let value = trimmed[pattern.len()..].trim();
            // Remove quotes if present
            let value = value.trim_matches('"').trim_matches('\'');
            if value.is_empty() {
                return "-".to_string();
            }
            return value.to_string();
        }
    }

    "-".to_string()
}

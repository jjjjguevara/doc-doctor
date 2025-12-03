//! TUI Application State
//!
//! Manages the state and event handling for the interactive TUI.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use doc_doctor_domain::{L1Properties, StateDimensions, Stub};

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    Dashboard,
    DocumentViewer,
    StubList,
    DocumentContent,
    BatchProgress,
    Help,
    Search,
    SortMenu,
    ColumnConfig,
    /// Test runner view for running mock tests
    Tests,
}

/// Dashboard view types (cycled with Tab)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardView {
    /// Results list with compact vault summary at top
    #[default]
    Results,
    /// Vault file/folder statistics
    Vault,
    /// Vault Health overview with all key metrics
    HealthOverview,
    /// Detailed stubs statistics and breakdown
    StubsDetail,
    /// Audience distribution view
    AudienceDetail,
    /// Form/lifecycle distribution view
    FormDetail,
}

impl DashboardView {
    pub fn all() -> &'static [DashboardView] {
        &[
            DashboardView::Results,
            DashboardView::Vault,
            DashboardView::HealthOverview,
            DashboardView::StubsDetail,
            DashboardView::AudienceDetail,
            DashboardView::FormDetail,
        ]
    }

    pub fn next(&self) -> DashboardView {
        match self {
            DashboardView::Results => DashboardView::Vault,
            DashboardView::Vault => DashboardView::HealthOverview,
            DashboardView::HealthOverview => DashboardView::StubsDetail,
            DashboardView::StubsDetail => DashboardView::AudienceDetail,
            DashboardView::AudienceDetail => DashboardView::FormDetail,
            DashboardView::FormDetail => DashboardView::Results,
        }
    }

    pub fn prev(&self) -> DashboardView {
        match self {
            DashboardView::Results => DashboardView::FormDetail,
            DashboardView::Vault => DashboardView::Results,
            DashboardView::HealthOverview => DashboardView::Vault,
            DashboardView::StubsDetail => DashboardView::HealthOverview,
            DashboardView::AudienceDetail => DashboardView::StubsDetail,
            DashboardView::FormDetail => DashboardView::AudienceDetail,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DashboardView::Results => "Results",
            DashboardView::Vault => "Vault",
            DashboardView::HealthOverview => "Health",
            DashboardView::StubsDetail => "Stubs",
            DashboardView::AudienceDetail => "Audience",
            DashboardView::FormDetail => "Form",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            DashboardView::Results => 0,
            DashboardView::Vault => 1,
            DashboardView::HealthOverview => 2,
            DashboardView::StubsDetail => 3,
            DashboardView::AudienceDetail => 4,
            DashboardView::FormDetail => 5,
        }
    }

    /// Get description for this view
    pub fn description(&self) -> &'static str {
        match self {
            DashboardView::Results => "Document list with health scores",
            DashboardView::Vault => "Vault file and folder statistics",
            DashboardView::HealthOverview => "Health distribution and metrics",
            DashboardView::StubsDetail => "Stubs breakdown by type and form",
            DashboardView::AudienceDetail => "Documents by audience level",
            DashboardView::FormDetail => "Documents by lifecycle form",
        }
    }
}

/// Sort field for document list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum SortField {
    #[default]
    Health,
    Name,
    Stubs,
    Refinement,
    Audience,
    Lines,
    Size,
    Modified,
}

impl SortField {
    pub fn all() -> &'static [SortField] {
        &[
            SortField::Health,
            SortField::Name,
            SortField::Stubs,
            SortField::Refinement,
            SortField::Audience,
            SortField::Lines,
            SortField::Size,
            SortField::Modified,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortField::Health => "Health",
            SortField::Name => "Name",
            SortField::Stubs => "Stubs",
            SortField::Refinement => "Refinement",
            SortField::Audience => "Audience",
            SortField::Lines => "Lines",
            SortField::Size => "Size",
            SortField::Modified => "Modified",
        }
    }
}

/// Column types for the document list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Column {
    Health,
    Name,
    Folder,
    Path,
    Audience,
    Stubs,
    Refinement,
    Lines,
    Size,
    Modified,
    Created,
    Author,
    LastCommit,
    Form,
    Origin,
}

impl Column {
    pub fn all() -> &'static [Column] {
        &[
            Column::Health,
            Column::Name,
            Column::Folder,
            Column::Path,
            Column::Audience,
            Column::Stubs,
            Column::Refinement,
            Column::Lines,
            Column::Size,
            Column::Modified,
            Column::Created,
            Column::Author,
            Column::LastCommit,
            Column::Form,
            Column::Origin,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Column::Health => "Health",
            Column::Name => "Name",
            Column::Folder => "Folder",
            Column::Path => "Path",
            Column::Audience => "Audience",
            Column::Stubs => "Stubs",
            Column::Refinement => "Refine",
            Column::Lines => "Lines",
            Column::Size => "Size",
            Column::Modified => "Modified",
            Column::Created => "Created",
            Column::Author => "Author",
            Column::LastCommit => "Last Commit",
            Column::Form => "Form",
            Column::Origin => "Origin",
        }
    }

    pub fn width(&self) -> u16 {
        match self {
            Column::Health => 7,
            Column::Name => 30,
            Column::Folder => 20,
            Column::Path => 40,
            Column::Audience => 10,
            Column::Stubs => 6,
            Column::Refinement => 7,
            Column::Lines => 6,
            Column::Size => 8,
            Column::Modified => 12,
            Column::Created => 12,
            Column::Author => 15,
            Column::LastCommit => 25,
            Column::Form => 12,
            Column::Origin => 12,
        }
    }

    /// Get description for this column
    pub fn description(&self) -> &'static str {
        match self {
            Column::Health => "Document health score (0-100%)",
            Column::Name => "Document title or filename",
            Column::Folder => "Containing folder name",
            Column::Path => "Relative path from vault root",
            Column::Audience => "Target audience level",
            Column::Stubs => "Number of stubs/gaps",
            Column::Refinement => "Refinement score (0-100%)",
            Column::Lines => "Line count",
            Column::Size => "File size",
            Column::Modified => "Last modified date",
            Column::Created => "Creation date",
            Column::Author => "Document author",
            Column::LastCommit => "Last git commit",
            Column::Form => "Document lifecycle form",
            Column::Origin => "Content origin (human/ai)",
        }
    }

    pub fn default_columns() -> Vec<Column> {
        vec![
            Column::Health,
            Column::Name,
            Column::Audience,
            Column::Stubs,
            Column::Lines,
        ]
    }
}

/// Navigation focus within the current mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Main,
    Sidebar,
    StatusBar,
    Popup,
}

/// Document summary for dashboard display
#[derive(Debug, Clone)]
pub struct DocumentSummary {
    pub path: PathBuf,
    /// Relative path from vault root
    pub relative_path: Option<String>,
    pub title: Option<String>,
    pub health: f64,
    pub refinement: f64,
    pub stub_count: usize,
    pub audience: String,
    pub form: String,
    pub origin: String,
    /// Full list of stubs for this document
    pub stubs: Vec<Stub>,
    /// Raw document content
    pub content: String,
    /// File metadata
    pub line_count: usize,
    pub file_size: u64,
    pub modified: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
    pub author: Option<String>,
    /// Last git commit info (short hash + message)
    pub last_commit: Option<String>,
}

impl DocumentSummary {
    /// Get the containing folder name
    pub fn folder_name(&self) -> &str {
        self.path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
    }

    /// Get the relative path or fall back to filename
    pub fn display_path(&self) -> &str {
        self.relative_path.as_deref()
            .unwrap_or_else(|| self.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(""))
    }
}

/// Vault statistics for dashboard
#[derive(Debug, Clone, Default)]
pub struct VaultStats {
    pub total_documents: usize,
    pub average_health: f64,
    pub average_refinement: f64,
    pub total_stubs: usize,
    pub blocking_stubs: usize,
    pub documents_by_audience: Vec<(String, usize)>,
    pub documents_by_form: Vec<(String, usize)>,
    pub health_distribution: Vec<(String, usize)>, // "Low", "Medium", "High"

    // File/folder statistics for Vault view
    pub total_files: usize,
    pub files_with_frontmatter: usize,
    pub files_without_frontmatter: usize,
    pub total_folders: usize,
    pub files_by_top_folder: Vec<(String, usize, usize)>, // (folder, total, with_frontmatter)
    pub vault_root: Option<PathBuf>,
}

/// Batch processing state
#[derive(Debug, Clone)]
pub struct BatchState {
    pub total: usize,
    pub processed: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub current_file: Option<String>,
    pub start_time: Instant,
    pub errors: Vec<(String, String)>, // (file, error)
}

impl BatchState {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            processed: 0,
            succeeded: 0,
            failed: 0,
            current_file: None,
            start_time: Instant::now(),
            errors: Vec::new(),
        }
    }

    pub fn progress(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.processed as f64 / self.total as f64
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn eta(&self) -> Option<Duration> {
        if self.processed == 0 {
            return None;
        }
        let elapsed = self.elapsed();
        let rate = self.processed as f64 / elapsed.as_secs_f64();
        let remaining = self.total - self.processed;
        Some(Duration::from_secs_f64(remaining as f64 / rate))
    }
}

/// AI processing state for thought streams
#[derive(Debug, Clone)]
pub struct AiState {
    pub is_processing: bool,
    pub current_task: Option<String>,
    pub thoughts: Vec<ThoughtItem>,
    pub progress: Option<f64>,
    pub start_time: Option<Instant>,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            is_processing: false,
            current_task: None,
            thoughts: Vec::new(),
            progress: None,
            start_time: None,
        }
    }
}

/// A single thought/step in the AI processing stream
#[derive(Debug, Clone)]
pub struct ThoughtItem {
    pub icon: char,
    pub text: String,
    pub status: ThoughtStatus,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThoughtStatus {
    Pending,
    InProgress,
    Complete,
    Error,
}

impl ThoughtItem {
    pub fn new(icon: char, text: impl Into<String>) -> Self {
        Self {
            icon,
            text: text.into(),
            status: ThoughtStatus::Pending,
            timestamp: Instant::now(),
        }
    }

    pub fn in_progress(icon: char, text: impl Into<String>) -> Self {
        Self {
            icon,
            text: text.into(),
            status: ThoughtStatus::InProgress,
            timestamp: Instant::now(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//                              TEST RUNNER STATE
// ═══════════════════════════════════════════════════════════════════════════

/// A predefined test command that can be run
#[derive(Debug, Clone)]
pub struct TestCommand {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub category: TestCategory,
}

/// Test command categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    Parse,
    Analyze,
    Stubs,
    Health,
    Validate,
    Modify,  // For functional tests that modify content
    Anchor,  // For anchor-related operations
}

/// Focus area in test runner
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestFocus {
    #[default]
    Files,
    Commands,
    Preview,
}

impl TestCategory {
    pub fn label(&self) -> &'static str {
        match self {
            TestCategory::Parse => "Parse",
            TestCategory::Analyze => "Analyze",
            TestCategory::Stubs => "Stubs",
            TestCategory::Health => "Health",
            TestCategory::Validate => "Validate",
            TestCategory::Modify => "Modify",
            TestCategory::Anchor => "Anchor",
        }
    }

    pub fn icon(&self) -> char {
        match self {
            TestCategory::Parse => '📄',
            TestCategory::Analyze => '🔍',
            TestCategory::Stubs => '📌',
            TestCategory::Health => '❤',
            TestCategory::Validate => '✓',
            TestCategory::Modify => '✏',
            TestCategory::Anchor => '⚓',
        }
    }
}

/// Result of running a test command
#[derive(Debug, Clone)]
pub struct TestResult {
    pub command_id: String,
    pub success: bool,
    pub output: String,
    pub duration: Duration,
    pub timestamp: Instant,
}

/// Test runner state
#[derive(Debug, Clone, Default)]
pub struct TestState {
    /// Test directory path
    pub test_dir: Option<PathBuf>,
    /// Files in the test directory
    pub test_files: Vec<PathBuf>,
    /// Predefined test commands
    pub commands: Vec<TestCommand>,
    /// Selected command index
    pub selected_command: usize,
    /// Selected file index (for file list)
    pub selected_file: usize,
    /// Current focus area
    pub focus: TestFocus,
    /// Test results
    pub results: Vec<TestResult>,
    /// Currently running test
    pub running: Option<String>,
    /// Scroll offset for results
    pub results_scroll: usize,
    /// In-memory file content (not saved to disk)
    /// Key: file path, Value: modified content
    pub file_content: std::collections::HashMap<PathBuf, String>,
    /// Original file content (for reset)
    pub original_content: std::collections::HashMap<PathBuf, String>,
    /// Scroll offset for file preview
    pub preview_scroll: usize,
    /// Selected line in preview (1-indexed, for anchor operations)
    pub selected_line: usize,
    /// Total lines in current content (for boundary checking)
    pub content_line_count: usize,
}

impl TestState {
    pub fn new() -> Self {
        Self {
            commands: Self::default_commands(),
            ..Default::default()
        }
    }

    /// Load test files from directory
    pub fn load_test_files(&mut self, dir: &std::path::Path) {
        self.test_dir = Some(dir.to_path_buf());
        self.test_files.clear();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "md") {
                    self.test_files.push(path);
                }
            }
        }

        // Sort by filename
        self.test_files.sort_by(|a, b| {
            a.file_name().cmp(&b.file_name())
        });
    }

    /// Get the currently selected file
    pub fn selected_file(&self) -> Option<&PathBuf> {
        self.test_files.get(self.selected_file)
    }

    /// Get the currently selected command
    pub fn selected_command(&self) -> Option<&TestCommand> {
        self.commands.get(self.selected_command)
    }

    /// Default test commands
    fn default_commands() -> Vec<TestCommand> {
        vec![
            TestCommand {
                id: "parse".to_string(),
                name: "Parse Document".to_string(),
                description: "Parse frontmatter and extract L1 properties".to_string(),
                command: "ddoc parse {file}".to_string(),
                category: TestCategory::Parse,
            },
            TestCommand {
                id: "analyze".to_string(),
                name: "Analyze Document".to_string(),
                description: "Full analysis with L2 dimensions".to_string(),
                command: "ddoc dimensions {file}".to_string(),
                category: TestCategory::Analyze,
            },
            TestCommand {
                id: "health".to_string(),
                name: "Check Health".to_string(),
                description: "Calculate document health score".to_string(),
                command: "ddoc health {file}".to_string(),
                category: TestCategory::Health,
            },
            TestCommand {
                id: "validate".to_string(),
                name: "Validate Schema".to_string(),
                description: "Validate frontmatter against schema".to_string(),
                command: "ddoc validate {file}".to_string(),
                category: TestCategory::Validate,
            },
            TestCommand {
                id: "stubs-list".to_string(),
                name: "List Stubs".to_string(),
                description: "List all stubs in document".to_string(),
                command: "ddoc stubs list {file}".to_string(),
                category: TestCategory::Stubs,
            },
            TestCommand {
                id: "stubs-blocking".to_string(),
                name: "Blocking Stubs".to_string(),
                description: "List blocking stubs only".to_string(),
                command: "ddoc stubs list {file} --blocking-only".to_string(),
                category: TestCategory::Stubs,
            },
            TestCommand {
                id: "anchors".to_string(),
                name: "Find Anchors".to_string(),
                description: "Find inline anchors in content".to_string(),
                command: "ddoc stubs anchors {file}".to_string(),
                category: TestCategory::Stubs,
            },
            TestCommand {
                id: "usefulness".to_string(),
                name: "Usefulness Margin".to_string(),
                description: "Calculate usefulness for all audiences".to_string(),
                command: "ddoc usefulness {file}".to_string(),
                category: TestCategory::Analyze,
            },
            // Functional/Modify commands (operate on in-memory content)
            TestCommand {
                id: "add-stub-expand".to_string(),
                name: "Add Expand Stub".to_string(),
                description: "Add a new 'expand' stub to the document".to_string(),
                command: "__modify:add_stub:expand".to_string(),
                category: TestCategory::Modify,
            },
            TestCommand {
                id: "add-stub-link".to_string(),
                name: "Add Link Stub".to_string(),
                description: "Add a new 'link' stub to the document".to_string(),
                command: "__modify:add_stub:link".to_string(),
                category: TestCategory::Modify,
            },
            TestCommand {
                id: "add-stub-cite".to_string(),
                name: "Add Cite Stub".to_string(),
                description: "Add a new 'cite' stub to the document".to_string(),
                command: "__modify:add_stub:cite".to_string(),
                category: TestCategory::Modify,
            },
            TestCommand {
                id: "remove-stub".to_string(),
                name: "Remove First Stub".to_string(),
                description: "Remove the first stub from the document".to_string(),
                command: "__modify:remove_stub:0".to_string(),
                category: TestCategory::Modify,
            },
            TestCommand {
                id: "resolve-stub".to_string(),
                name: "Resolve First Stub".to_string(),
                description: "Mark the first stub as resolved".to_string(),
                command: "__modify:resolve_stub:0".to_string(),
                category: TestCategory::Modify,
            },
            TestCommand {
                id: "reset-file".to_string(),
                name: "Reset File".to_string(),
                description: "Reset file to original content (undo all changes)".to_string(),
                command: "__modify:reset".to_string(),
                category: TestCategory::Modify,
            },
            // Anchor operations (require preview focus to select line)
            TestCommand {
                id: "add-anchor".to_string(),
                name: "Add Anchor at Line".to_string(),
                description: "Insert ^anchor-id at the selected line in preview".to_string(),
                command: "__anchor:add".to_string(),
                category: TestCategory::Anchor,
            },
            TestCommand {
                id: "remove-anchor".to_string(),
                name: "Remove Anchor".to_string(),
                description: "Remove anchor from the selected line".to_string(),
                command: "__anchor:remove".to_string(),
                category: TestCategory::Anchor,
            },
            TestCommand {
                id: "link-anchor".to_string(),
                name: "Link Anchor to Stub 0".to_string(),
                description: "Add anchor at selected line to first stub's inline_anchors".to_string(),
                command: "__anchor:link:0".to_string(),
                category: TestCategory::Anchor,
            },
            TestCommand {
                id: "unlink-anchor".to_string(),
                name: "Unlink Anchor from Stub 0".to_string(),
                description: "Remove anchor at selected line from first stub's inline_anchors".to_string(),
                command: "__anchor:unlink:0".to_string(),
                category: TestCategory::Anchor,
            },
        ]
    }

    /// Add a test result
    pub fn add_result(&mut self, result: TestResult) {
        self.results.insert(0, result); // Most recent first
        if self.results.len() > 50 {
            self.results.pop();
        }
    }

    /// Navigate commands
    pub fn next_command(&mut self) {
        if !self.commands.is_empty() {
            self.selected_command = (self.selected_command + 1) % self.commands.len();
        }
    }

    pub fn prev_command(&mut self) {
        if !self.commands.is_empty() {
            self.selected_command = self.selected_command
                .checked_sub(1)
                .unwrap_or(self.commands.len() - 1);
        }
    }

    /// Navigate files
    pub fn next_file(&mut self) {
        if !self.test_files.is_empty() {
            self.selected_file = (self.selected_file + 1) % self.test_files.len();
        }
    }

    pub fn prev_file(&mut self) {
        if !self.test_files.is_empty() {
            self.selected_file = self.selected_file
                .checked_sub(1)
                .unwrap_or(self.test_files.len() - 1);
        }
    }

    /// Cycle focus: Files -> Commands -> Preview -> Files
    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            TestFocus::Files => TestFocus::Commands,
            TestFocus::Commands => TestFocus::Preview,
            TestFocus::Preview => TestFocus::Files,
        };
    }

    /// Load file content into memory (for preview and modification)
    pub fn load_file_content(&mut self, path: &PathBuf) {
        if !self.file_content.contains_key(path) {
            if let Ok(content) = std::fs::read_to_string(path) {
                self.content_line_count = content.lines().count();
                self.original_content.insert(path.clone(), content.clone());
                self.file_content.insert(path.clone(), content);
                // Reset selected line when loading new file
                self.selected_line = 1;
            }
        } else if let Some(content) = self.file_content.get(path) {
            self.content_line_count = content.lines().count();
        }
    }

    /// Get the current content for a file (in-memory version)
    pub fn get_content(&self, path: &PathBuf) -> Option<&String> {
        self.file_content.get(path)
    }

    /// Update file content in memory (not saved to disk)
    pub fn set_content(&mut self, path: &PathBuf, content: String) {
        self.file_content.insert(path.clone(), content);
    }

    /// Reset file to original content
    pub fn reset_file(&mut self, path: &PathBuf) {
        if let Some(original) = self.original_content.get(path) {
            self.file_content.insert(path.clone(), original.clone());
        }
    }

    /// Reset all files to original content
    pub fn reset_all(&mut self) {
        for (path, original) in &self.original_content {
            self.file_content.insert(path.clone(), original.clone());
        }
    }

    /// Check if a file has been modified
    pub fn is_modified(&self, path: &PathBuf) -> bool {
        match (self.file_content.get(path), self.original_content.get(path)) {
            (Some(current), Some(original)) => current != original,
            _ => false,
        }
    }

    /// Scroll preview up
    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    /// Scroll preview down
    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    /// Move selected line up in preview
    pub fn prev_line(&mut self) {
        if self.selected_line > 1 {
            self.selected_line -= 1;
            // Ensure selected line is visible
            if self.selected_line <= self.preview_scroll {
                self.preview_scroll = self.selected_line.saturating_sub(1);
            }
        }
    }

    /// Move selected line down in preview
    pub fn next_line(&mut self) {
        if self.content_line_count > 0 && self.selected_line < self.content_line_count {
            self.selected_line += 1;
        }
    }

    /// Ensure selected line is visible in preview (call after next_line with view height)
    pub fn ensure_line_visible(&mut self, view_height: usize) {
        // If selected line is below visible area, scroll down
        if self.selected_line > self.preview_scroll + view_height {
            self.preview_scroll = self.selected_line - view_height;
        }
        // If selected line is above visible area, scroll up
        if self.selected_line <= self.preview_scroll {
            self.preview_scroll = self.selected_line.saturating_sub(1);
        }
    }

    /// Find all anchors in the current content
    /// Returns (line_number, anchor_id, column_position)
    pub fn find_anchors_in_content(&self, path: &PathBuf) -> Vec<(usize, String, usize)> {
        let mut anchors = Vec::new();
        if let Some(content) = self.file_content.get(path) {
            for (line_idx, line) in content.lines().enumerate() {
                // Find anchors in format ^anchor-id
                let mut pos = 0;
                while let Some(start) = line[pos..].find('^') {
                    let abs_start = pos + start;
                    let rest = &line[abs_start + 1..];
                    // Anchor ID: alphanumeric and hyphens until whitespace or end
                    let end = rest.find(|c: char| c.is_whitespace() || c == '^')
                        .unwrap_or(rest.len());
                    if end > 0 {
                        let anchor_id = rest[..end].to_string();
                        // Skip if it looks like a markdown footnote reference [^
                        if abs_start > 0 && line.as_bytes().get(abs_start - 1) == Some(&b'[') {
                            pos = abs_start + 1 + end;
                            continue;
                        }
                        anchors.push((line_idx + 1, anchor_id, abs_start));
                    }
                    pos = abs_start + 1 + end;
                }
            }
        }
        anchors
    }

    /// Insert an anchor at the end of a specific line
    /// Returns the new content if successful
    pub fn insert_anchor_at_line(&self, path: &PathBuf, line_num: usize, anchor_id: &str) -> Option<String> {
        let content = self.file_content.get(path)?;
        let lines: Vec<&str> = content.lines().collect();

        if line_num == 0 || line_num > lines.len() {
            return None;
        }

        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        let target_line = &mut new_lines[line_num - 1];

        // Append anchor at end of line (with space if line is not empty)
        if target_line.is_empty() {
            *target_line = format!("^{}", anchor_id);
        } else {
            *target_line = format!("{} ^{}", target_line, anchor_id);
        }

        Some(new_lines.join("\n"))
    }

    /// Remove an anchor from a specific line
    /// Returns the new content if successful
    pub fn remove_anchor_from_line(&self, path: &PathBuf, line_num: usize, anchor_id: &str) -> Option<String> {
        let content = self.file_content.get(path)?;
        let lines: Vec<&str> = content.lines().collect();

        if line_num == 0 || line_num > lines.len() {
            return None;
        }

        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        let target_line = &mut new_lines[line_num - 1];

        // Remove the anchor pattern (with possible leading space)
        let patterns = [
            format!(" ^{}", anchor_id),
            format!("^{} ", anchor_id),
            format!("^{}", anchor_id),
        ];

        for pattern in &patterns {
            if target_line.contains(pattern) {
                *target_line = target_line.replace(pattern, "");
                break;
            }
        }

        Some(new_lines.join("\n"))
    }

    /// Get the anchor ID at the current selected line (if any)
    pub fn get_anchor_at_line(&self, path: &PathBuf, line_num: usize) -> Option<String> {
        let anchors = self.find_anchors_in_content(path);
        anchors.iter()
            .find(|(ln, _, _)| *ln == line_num)
            .map(|(_, id, _)| id.clone())
    }
}

/// Main application state
#[derive(Debug)]
pub struct App {
    pub mode: AppMode,
    pub focus: Focus,
    pub should_quit: bool,

    // Dashboard state
    pub dashboard_view: DashboardView,
    pub vault_stats: Option<VaultStats>,
    pub documents: Vec<DocumentSummary>,
    pub filtered_indices: Vec<usize>,  // Indices into documents after filtering
    pub selected_document: usize,
    pub list_offset: usize,            // Scroll offset for document list
    pub list_height: usize,            // Visible height for document list

    // Filter/sort state
    pub filter_text: String,
    pub sort_field: SortField,
    pub sort_ascending: bool,

    // Column configuration
    pub visible_columns: Vec<Column>,
    pub column_menu_index: usize,      // For column config menu

    // Document viewer state
    pub current_document: Option<(PathBuf, L1Properties, StateDimensions)>,
    pub scroll_offset: usize,

    // Batch processing
    pub batch_state: Option<BatchState>,

    // AI state
    pub ai_state: AiState,

    // Test runner state
    pub test_state: TestState,

    // Animation state
    pub tick_count: u64,
    pub last_tick: Instant,

    // Status message
    pub status_message: Option<(String, Instant)>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: AppMode::Dashboard,
            focus: Focus::Main,
            should_quit: false,
            dashboard_view: DashboardView::default(),
            vault_stats: None,
            documents: Vec::new(),
            filtered_indices: Vec::new(),
            selected_document: 0,
            list_offset: 0,
            list_height: 10,
            filter_text: String::new(),
            sort_field: SortField::Health,
            sort_ascending: true,
            visible_columns: Column::default_columns(),
            column_menu_index: 0,
            current_document: None,
            scroll_offset: 0,
            batch_state: None,
            ai_state: AiState::default(),
            test_state: TestState::new(),
            tick_count: 0,
            last_tick: Instant::now(),
            status_message: None,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        self.last_tick = Instant::now();

        // Clear old status messages
        if let Some((_, time)) = &self.status_message {
            if time.elapsed() > Duration::from_secs(5) {
                self.status_message = None;
            }
        }
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some((message.into(), Instant::now()));
    }

    pub fn next_document(&mut self) {
        let count = self.visible_document_count();
        if count > 0 {
            self.selected_document = (self.selected_document + 1) % count;
            self.ensure_selection_visible();
        }
    }

    pub fn previous_document(&mut self) {
        let count = self.visible_document_count();
        if count > 0 {
            self.selected_document = self
                .selected_document
                .checked_sub(1)
                .unwrap_or(count - 1);
            self.ensure_selection_visible();
        }
    }

    /// Get count of visible documents (filtered)
    pub fn visible_document_count(&self) -> usize {
        if self.filter_text.is_empty() {
            self.documents.len()
        } else {
            self.filtered_indices.len()
        }
    }

    /// Get document at visible index
    pub fn get_visible_document(&self, visible_idx: usize) -> Option<&DocumentSummary> {
        if self.filter_text.is_empty() {
            self.documents.get(visible_idx)
        } else {
            self.filtered_indices.get(visible_idx)
                .and_then(|&idx| self.documents.get(idx))
        }
    }

    /// Get currently selected document
    pub fn get_selected_document(&self) -> Option<&DocumentSummary> {
        self.get_visible_document(self.selected_document)
    }

    /// Ensure selected document is visible in the list
    pub fn ensure_selection_visible(&mut self) {
        if self.selected_document < self.list_offset {
            self.list_offset = self.selected_document;
        } else if self.selected_document >= self.list_offset + self.list_height {
            self.list_offset = self.selected_document.saturating_sub(self.list_height - 1);
        }
    }

    /// Scroll the document list up
    pub fn list_scroll_up(&mut self, amount: usize) {
        self.list_offset = self.list_offset.saturating_sub(amount);
    }

    /// Scroll the document list down
    pub fn list_scroll_down(&mut self, amount: usize) {
        let max_offset = self.visible_document_count().saturating_sub(self.list_height);
        self.list_offset = (self.list_offset + amount).min(max_offset);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Apply filter to documents
    pub fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_indices.clear();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_indices = self.documents.iter()
                .enumerate()
                .filter(|(_, doc)| {
                    let title = doc.title.as_deref().unwrap_or("");
                    let path = doc.path.to_string_lossy();
                    title.to_lowercase().contains(&filter_lower)
                        || path.to_lowercase().contains(&filter_lower)
                        || doc.audience.to_lowercase().contains(&filter_lower)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.selected_document = 0;
        self.list_offset = 0;
    }

    /// Cycle to next sort field
    pub fn cycle_sort(&mut self) {
        let fields = SortField::all();
        let current_idx = fields.iter().position(|&f| f == self.sort_field).unwrap_or(0);
        self.sort_field = fields[(current_idx + 1) % fields.len()];
        self.sort_documents();
    }

    /// Cycle to previous sort field
    pub fn cycle_sort_prev(&mut self) {
        let fields = SortField::all();
        let current_idx = fields.iter().position(|&f| f == self.sort_field).unwrap_or(0);
        self.sort_field = fields[(current_idx + fields.len() - 1) % fields.len()];
        self.sort_documents();
    }

    /// Set sort field directly
    pub fn set_sort_field(&mut self, field: SortField) {
        self.sort_field = field;
        self.sort_documents();
    }

    /// Toggle sort direction
    pub fn toggle_sort_direction(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.sort_documents();
    }

    /// Sort documents by current field
    pub fn sort_documents(&mut self) {
        let ascending = self.sort_ascending;
        match self.sort_field {
            SortField::Health => {
                self.documents.sort_by(|a, b| {
                    let cmp = a.health.partial_cmp(&b.health).unwrap();
                    if ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::Name => {
                self.documents.sort_by(|a, b| {
                    let a_name = a.title.as_deref().unwrap_or("");
                    let b_name = b.title.as_deref().unwrap_or("");
                    let cmp = a_name.to_lowercase().cmp(&b_name.to_lowercase());
                    if ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::Stubs => {
                self.documents.sort_by(|a, b| {
                    let cmp = a.stub_count.cmp(&b.stub_count);
                    if ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::Refinement => {
                self.documents.sort_by(|a, b| {
                    let cmp = a.refinement.partial_cmp(&b.refinement).unwrap();
                    if ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::Audience => {
                self.documents.sort_by(|a, b| {
                    let cmp = a.audience.cmp(&b.audience);
                    if ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::Lines => {
                self.documents.sort_by(|a, b| {
                    let cmp = a.line_count.cmp(&b.line_count);
                    if ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::Size => {
                self.documents.sort_by(|a, b| {
                    let cmp = a.file_size.cmp(&b.file_size);
                    if ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::Modified => {
                self.documents.sort_by(|a, b| {
                    let cmp = a.modified.cmp(&b.modified);
                    if ascending { cmp } else { cmp.reverse() }
                });
            }
        }
        self.apply_filter(); // Re-apply filter after sort
    }

    /// Toggle column visibility
    pub fn toggle_column(&mut self, col: Column) {
        if let Some(pos) = self.visible_columns.iter().position(|&c| c == col) {
            // Don't remove if only one column left
            if self.visible_columns.len() > 1 {
                self.visible_columns.remove(pos);
            }
        } else {
            self.visible_columns.push(col);
        }
    }

    /// Move column up in order
    pub fn move_column_up(&mut self, col: Column) {
        if let Some(pos) = self.visible_columns.iter().position(|&c| c == col) {
            if pos > 0 {
                self.visible_columns.swap(pos, pos - 1);
            }
        }
    }

    /// Move column down in order
    pub fn move_column_down(&mut self, col: Column) {
        if let Some(pos) = self.visible_columns.iter().position(|&c| c == col) {
            if pos < self.visible_columns.len() - 1 {
                self.visible_columns.swap(pos, pos + 1);
            }
        }
    }

    /// Check if column is visible
    pub fn is_column_visible(&self, col: Column) -> bool {
        self.visible_columns.contains(&col)
    }

    /// Get spinner character for animations
    pub fn spinner(&self) -> char {
        const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER[(self.tick_count as usize / 2) % SPINNER.len()]
    }

    /// Get progress bar characters
    pub fn progress_bar(&self) -> char {
        const PROGRESS: &[char] = &['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
        PROGRESS[(self.tick_count as usize) % PROGRESS.len()]
    }

    // AI thought stream methods
    pub fn start_ai_task(&mut self, task: impl Into<String>) {
        self.ai_state.is_processing = true;
        self.ai_state.current_task = Some(task.into());
        self.ai_state.thoughts.clear();
        self.ai_state.progress = Some(0.0);
        self.ai_state.start_time = Some(Instant::now());
    }

    pub fn add_thought(&mut self, icon: char, text: impl Into<String>) {
        // Mark previous in-progress thought as complete
        if let Some(last) = self.ai_state.thoughts.last_mut() {
            if last.status == ThoughtStatus::InProgress {
                last.status = ThoughtStatus::Complete;
            }
        }
        self.ai_state.thoughts.push(ThoughtItem::in_progress(icon, text));
    }

    pub fn complete_ai_task(&mut self) {
        // Mark last thought as complete
        if let Some(last) = self.ai_state.thoughts.last_mut() {
            last.status = ThoughtStatus::Complete;
        }
        self.ai_state.is_processing = false;
        self.ai_state.progress = Some(1.0);
    }

    pub fn fail_ai_task(&mut self, error: impl Into<String>) {
        if let Some(last) = self.ai_state.thoughts.last_mut() {
            last.status = ThoughtStatus::Error;
        }
        self.ai_state.thoughts.push(ThoughtItem {
            icon: '✗',
            text: error.into(),
            status: ThoughtStatus::Error,
            timestamp: Instant::now(),
        });
        self.ai_state.is_processing = false;
    }
}

/// Input event
#[derive(Debug, Clone)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    Tick,
}

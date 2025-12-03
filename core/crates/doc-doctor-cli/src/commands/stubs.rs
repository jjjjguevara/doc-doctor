//! Stubs Command
//!
//! Manage stubs in documents using the Application Switchboard.
//!
//! # Subcommands
//!
//! - `list` - List stubs with optional filtering
//! - `add` - Add a new stub to document frontmatter
//! - `resolve` - Remove a resolved stub
//! - `update` - Update stub properties

use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

use doc_doctor_application::{NewStub, StubFilter, StubUpdates, Switchboard};

use crate::commands::{create_switchboard, read_file, write_file};
use crate::output::{
    format_output, AnchorInfo, AnchorsOutput, OutputFormat, StubAddOutput, StubAnchorInfo,
    StubLinkOutput, StubOutput, StubResolveOutput, StubUpdateOutput, StubsOutput,
};

#[derive(Args)]
pub struct StubsCommand {
    #[command(subcommand)]
    pub command: StubsSubcommand,
}

#[derive(Subcommand)]
pub enum StubsSubcommand {
    /// List stubs from a document
    List(ListCommand),

    /// Add a stub to document frontmatter
    Add(AddCommand),

    /// Remove a resolved stub
    Resolve(ResolveCommand),

    /// Update stub properties
    Update(UpdateCommand),

    /// Find anchors and link to stubs
    Anchors(AnchorsCommand),
}

// ═══════════════════════════════════════════════════════════════════════════
//                              LIST COMMAND
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Args)]
pub struct ListCommand {
    /// Path to markdown file
    pub path: PathBuf,

    /// Filter by stub type (e.g., "link", "expand", "fix")
    #[arg(long)]
    pub type_filter: Option<String>,

    /// Filter by stub form (transient, persistent, blocking, structural)
    #[arg(long)]
    pub form_filter: Option<String>,

    /// Only show blocking stubs
    #[arg(long)]
    pub blocking_only: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
//                              ADD COMMAND
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Args)]
pub struct AddCommand {
    /// Path to markdown file
    pub path: PathBuf,

    /// Stub type (e.g., expand, link, verify, fix, review-needed)
    #[arg(short = 't', long)]
    pub stub_type: String,

    /// Description of what needs to be done
    #[arg(short, long)]
    pub description: String,

    /// Priority level (low, medium, high, critical)
    #[arg(short, long, default_value = "medium")]
    pub priority: String,

    /// Stub form (transient, persistent, blocking, structural)
    #[arg(short = 'f', long)]
    pub stub_form: Option<String>,

    /// Inline anchor to link (without ^ prefix)
    #[arg(short, long)]
    pub anchor: Option<String>,

    /// Don't actually modify the file, just show what would change
    #[arg(long)]
    pub dry_run: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
//                            RESOLVE COMMAND
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Args)]
pub struct ResolveCommand {
    /// Path to markdown file
    pub path: PathBuf,

    /// Index of the stub to resolve (0-based)
    #[arg(short, long)]
    pub index: usize,

    /// Don't actually modify the file, just show what would change
    #[arg(long)]
    pub dry_run: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
//                            UPDATE COMMAND
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Args)]
pub struct UpdateCommand {
    /// Path to markdown file
    pub path: PathBuf,

    /// Index of the stub to update (0-based)
    #[arg(short, long)]
    pub index: usize,

    /// New description
    #[arg(short, long)]
    pub description: Option<String>,

    /// New priority (low, medium, high, critical)
    #[arg(short, long)]
    pub priority: Option<String>,

    /// New stub form (transient, persistent, blocking, structural)
    #[arg(short = 'f', long)]
    pub stub_form: Option<String>,

    /// Don't actually modify the file, just show what would change
    #[arg(long)]
    pub dry_run: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
//                           ANCHORS COMMAND
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Args)]
pub struct AnchorsCommand {
    /// Path to markdown file
    pub path: PathBuf,

    /// Link stub at index to anchor ID
    #[arg(short, long)]
    pub link: Option<String>,

    /// Stub index for linking (required with --link)
    #[arg(short, long)]
    pub index: Option<usize>,

    /// Don't actually modify the file, just show what would change
    #[arg(long)]
    pub dry_run: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
//                          COMMAND EXECUTION
// ═══════════════════════════════════════════════════════════════════════════

impl StubsCommand {
    pub fn run(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        match &self.command {
            StubsSubcommand::List(cmd) => run_list(cmd, format, verbose),
            StubsSubcommand::Add(cmd) => run_add(cmd, format, verbose),
            StubsSubcommand::Resolve(cmd) => run_resolve(cmd, format, verbose),
            StubsSubcommand::Update(cmd) => run_update(cmd, format, verbose),
            StubsSubcommand::Anchors(cmd) => run_anchors(cmd, format, verbose),
        }
    }
}

fn run_list(cmd: &ListCommand, format: OutputFormat, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Listing stubs: {}", cmd.path.display());
    }

    let content = read_file(&cmd.path)?;
    let switchboard = create_switchboard();

    let filter = Some(StubFilter {
        stub_type: cmd.type_filter.clone(),
        blocking_only: cmd.blocking_only,
        priority: None,
    });

    let stubs = switchboard
        .list_stubs(&content, filter)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Apply form filter (not in StubFilter currently)
    let filtered_stubs: Vec<_> = if let Some(form_filter) = &cmd.form_filter {
        stubs
            .into_iter()
            .filter(|s| s.stub_form.to_string().to_lowercase() == form_filter.to_lowercase())
            .collect()
    } else {
        stubs
    };

    let output = StubsOutput {
        path: cmd.path.display().to_string(),
        total: filtered_stubs.len(),
        stubs: filtered_stubs
            .iter()
            .map(|s| StubOutput {
                stub_type: s.stub_type.as_str().to_string(),
                description: s.description.clone(),
                stub_form: s.stub_form.to_string(),
                priority: s.priority.to_string(),
                is_blocking: s.is_blocking(),
            })
            .collect(),
    };

    println!("{}", format_output(&output, format)?);
    Ok(())
}

fn run_add(cmd: &AddCommand, format: OutputFormat, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!(
            "Adding stub to {}: {} - {}",
            cmd.path.display(),
            cmd.stub_type,
            cmd.description
        );
    }

    let content = read_file(&cmd.path)?;
    let switchboard = create_switchboard();

    let new_stub = NewStub {
        stub_type: cmd.stub_type.clone(),
        description: cmd.description.clone(),
        priority: Some(cmd.priority.clone()),
        stub_form: cmd.stub_form.clone(),
        anchor: cmd.anchor.clone(),
    };

    let result = switchboard
        .add_stub(&content, new_stub)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if cmd.dry_run {
        eprintln!("Dry run - would add stub at index {}", result.stub_index);
        println!("{}", result.updated_content);
    } else {
        write_file(&cmd.path, &result.updated_content)?;
        let output = StubAddOutput {
            action: "added".to_string(),
            path: cmd.path.display().to_string(),
            stub_index: result.stub_index,
            stub_type: result.stub.stub_type.as_str().to_string(),
            description: result.stub.description,
        };
        println!("{}", format_output(&output, format)?);
    }

    Ok(())
}

fn run_resolve(cmd: &ResolveCommand, format: OutputFormat, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!(
            "Resolving stub {} in {}",
            cmd.index,
            cmd.path.display()
        );
    }

    let content = read_file(&cmd.path)?;
    let switchboard = create_switchboard();

    let result = switchboard
        .resolve_stub(&content, cmd.index)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if cmd.dry_run {
        eprintln!(
            "Dry run - would resolve stub: {} - {}",
            result.resolved_stub.stub_type.as_str(),
            result.resolved_stub.description
        );
        println!("{}", result.updated_content);
    } else {
        write_file(&cmd.path, &result.updated_content)?;
        let output = StubResolveOutput {
            action: "resolved".to_string(),
            path: cmd.path.display().to_string(),
            resolved_type: result.resolved_stub.stub_type.as_str().to_string(),
            resolved_description: result.resolved_stub.description,
        };
        println!("{}", format_output(&output, format)?);
    }

    Ok(())
}

fn run_update(cmd: &UpdateCommand, format: OutputFormat, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Updating stub {} in {}", cmd.index, cmd.path.display());
    }

    let content = read_file(&cmd.path)?;
    let switchboard = create_switchboard();

    let updates = StubUpdates {
        description: cmd.description.clone(),
        priority: cmd.priority.clone(),
        stub_form: cmd.stub_form.clone(),
    };

    let result = switchboard
        .update_stub(&content, cmd.index, updates)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if cmd.dry_run {
        eprintln!(
            "Dry run - would update stub to: {} - {}",
            result.stub.stub_type.as_str(),
            result.stub.description
        );
        println!("{}", result.updated_content);
    } else {
        write_file(&cmd.path, &result.updated_content)?;
        let output = StubUpdateOutput {
            action: "updated".to_string(),
            path: cmd.path.display().to_string(),
            stub_type: result.stub.stub_type.as_str().to_string(),
            description: result.stub.description,
            priority: result.stub.priority.to_string(),
            stub_form: result.stub.stub_form.to_string(),
        };
        println!("{}", format_output(&output, format)?);
    }

    Ok(())
}

fn run_anchors(cmd: &AnchorsCommand, format: OutputFormat, verbose: bool) -> Result<()> {
    let content = read_file(&cmd.path)?;
    let switchboard = create_switchboard();

    // If linking, perform the link operation
    if let (Some(anchor_id), Some(index)) = (&cmd.link, cmd.index) {
        if verbose {
            eprintln!(
                "Linking stub {} to anchor ^{} in {}",
                index,
                anchor_id,
                cmd.path.display()
            );
        }

        let result = switchboard
            .link_stub_anchor(&content, index, anchor_id)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        if cmd.dry_run {
            eprintln!("Dry run - would link stub {} to anchor ^{}", index, anchor_id);
            println!("{}", result.updated_content);
        } else {
            write_file(&cmd.path, &result.updated_content)?;
            let output = StubLinkOutput {
                action: "linked".to_string(),
                path: cmd.path.display().to_string(),
                stub_index: index,
                anchor_id: anchor_id.to_string(),
            };
            println!("{}", format_output(&output, format)?);
        }
    } else {
        // Just show anchors found in the document
        if verbose {
            eprintln!("Finding anchors in {}", cmd.path.display());
        }

        let matches = switchboard
            .find_stub_anchors(&content)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let output = AnchorsOutput {
            path: cmd.path.display().to_string(),
            anchors: matches
                .anchors
                .iter()
                .map(|(id, line)| AnchorInfo {
                    id: id.clone(),
                    line: *line,
                })
                .collect(),
            stub_anchors: matches
                .stub_anchors
                .iter()
                .map(|(idx, anchors)| StubAnchorInfo {
                    stub_index: *idx,
                    anchors: anchors.clone(),
                })
                .collect(),
        };
        println!("{}", format_output(&output, format)?);
    }

    Ok(())
}

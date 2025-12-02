//! Doc Doctor CLI
//!
//! Command-line tool for J-Editorial document analysis.

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "doc-doctor")]
#[command(about = "J-Editorial document analysis and quality management")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(short, long, default_value = "human")]
    format: OutputFormat,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and display document L1 properties
    Parse {
        /// Path to markdown file
        path: String,
    },

    /// Validate frontmatter against J-Editorial schema
    Validate {
        /// File pattern (glob)
        pattern: String,

        /// Strict mode - reject unknown fields
        #[arg(short, long)]
        strict: bool,
    },

    /// List stubs from documents
    Stubs {
        /// Path to markdown file
        path: String,

        /// Filter by stub type
        #[arg(long)]
        type_filter: Option<String>,

        /// Filter by stub form
        #[arg(long)]
        form_filter: Option<String>,
    },

    /// Calculate L2 dimensions
    Dimensions {
        /// Path to markdown file
        path: String,
    },

    /// Calculate health score
    Health {
        /// Refinement score (0.0-1.0)
        #[arg(long)]
        refinement: f64,

        /// Stubs as JSON array
        #[arg(long)]
        stubs: Option<String>,
    },

    /// Calculate usefulness margin
    Usefulness {
        /// Refinement score (0.0-1.0)
        #[arg(long)]
        refinement: f64,

        /// Target audience
        #[arg(long)]
        audience: String,
    },

    /// Batch process multiple documents
    Batch {
        /// File pattern (glob)
        pattern: String,

        /// Include L2 dimensions
        #[arg(long)]
        dimensions: bool,

        /// Number of parallel jobs
        #[arg(short, long, default_value = "4")]
        jobs: usize,
    },

    /// Check stub-anchor sync status
    Sync {
        /// Path to markdown file
        path: String,
    },

    /// Export JSON schema definitions
    Schema {
        /// Schema type (frontmatter, stubs)
        schema_type: String,
    },
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Human,
    Json,
    Yaml,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { path } => {
            println!("Parsing: {}", path);
            let content = std::fs::read_to_string(&path)?;
            let props = doc_doctor_core::parse_document(&content)?;
            println!("{:#?}", props);
        }

        Commands::Health { refinement, stubs } => {
            let stub_vec: Vec<doc_doctor_core::Stub> = if let Some(json) = stubs {
                serde_json::from_str(&json)?
            } else {
                Vec::new()
            };
            let health = doc_doctor_core::dimensions::calculate_health(refinement, &stub_vec);
            println!("Health: {:.4}", health);
        }

        Commands::Usefulness { refinement, audience } => {
            let aud: doc_doctor_core::Audience = audience.parse()?;
            let result = doc_doctor_core::dimensions::calculate_usefulness(refinement, aud);
            println!("Margin: {:.4}", result.margin);
            println!("Is useful: {}", result.is_useful);
            println!("Gate: {:.2}", result.gate);
        }

        _ => {
            println!("Command not yet implemented - Phase 3");
        }
    }

    Ok(())
}

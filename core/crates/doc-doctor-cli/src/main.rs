//! Doc Doctor CLI
//!
//! Command-line tool for J-Editorial document analysis.
//!
//! # Usage
//!
//! ```bash
//! ddoc parse document.md
//! ddoc validate "docs/**/*.md" --strict
//! ddoc dimensions document.md
//! ddoc batch "vault/**/*.md" --dimensions
//! ddoc health --refinement 0.75
//! ddoc usefulness --refinement 0.8 --audience internal
//! ddoc config --show
//! ddoc config --init
//! ```

mod commands;
mod config;
mod output;
pub mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{
    batch::BatchCommand, config::ConfigCommand, dashboard::DashboardCommand,
    dimensions::DimensionsCommand, health::HealthCommand, parse::ParseCommand,
    schema::SchemaCommand, stubs::StubsCommand, test::TestCommand,
    usefulness::UsefulnessCommand, validate::ValidateCommand,
};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "dd")]
#[command(about = "J-Editorial document analysis and quality management")]
#[command(version)]
#[command(author)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(short, long, global = true, default_value = "human")]
    format: OutputFormat,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and display document L1 properties
    Parse(ParseCommand),

    /// Validate frontmatter against J-Editorial schema
    Validate(ValidateCommand),

    /// List stubs from documents
    Stubs(StubsCommand),

    /// Calculate L2 dimensions
    Dimensions(DimensionsCommand),

    /// Calculate health score
    Health(HealthCommand),

    /// Calculate usefulness margin
    Usefulness(UsefulnessCommand),

    /// Batch process multiple documents
    Batch(BatchCommand),

    /// Export JSON schema definitions
    Schema(SchemaCommand),

    /// Show or initialize configuration
    Config(ConfigCommand),

    /// Interactive vault health dashboard
    Dashboard(DashboardCommand),

    /// Interactive test runner for document operations
    Test(TestCommand),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Validate(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Stubs(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Dimensions(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Health(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Usefulness(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Batch(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Schema(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Config(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Dashboard(cmd) => cmd.run(cli.format, cli.verbose),
        Commands::Test(cmd) => cmd.run(cli.format, cli.verbose),
    }
}

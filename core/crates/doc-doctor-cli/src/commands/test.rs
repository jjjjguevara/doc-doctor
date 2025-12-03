//! Test Command
//!
//! Interactive test runner for document operations.

use anyhow::Result;
use clap::Args;

use crate::config::get_test_dir;
use crate::output::OutputFormat;
use crate::tui::{
    app::{App, AppMode},
    run_tui,
};

#[derive(Args)]
pub struct TestCommand {
    /// Run in verbose mode
    #[arg(short, long)]
    pub verbose: bool,
}

impl TestCommand {
    pub fn run(&self, _format: OutputFormat, verbose: bool) -> Result<()> {
        let verbose = verbose || self.verbose;

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
}

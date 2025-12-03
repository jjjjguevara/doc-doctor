//! Parse Command
//!
//! Parse and display document L1 properties.

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;


use crate::commands::{create_parser, read_file};
use crate::output::{format_output, OutputFormat, ParseOutput};

#[derive(Args)]
pub struct ParseCommand {
    /// Path to markdown file
    pub path: PathBuf,
}

impl ParseCommand {
    pub fn run(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        if verbose {
            eprintln!("Parsing: {}", self.path.display());
        }

        let content = read_file(&self.path)?;
        let parser = create_parser();

        let props = parser
            .parse(&content)
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

        let output = ParseOutput {
            path: self.path.display().to_string(),
            title: props.title.clone(),
            refinement: props.refinement.value(),
            audience: props.audience.to_string(),
            origin: props.origin.to_string(),
            form: props.form.to_string(),
            stub_count: props.stubs.len(),
            tags: props.tags.clone(),
        };

        println!("{}", format_output(&output, format)?);
        Ok(())
    }
}

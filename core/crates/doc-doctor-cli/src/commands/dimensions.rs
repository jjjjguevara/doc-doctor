//! Dimensions Command
//!
//! Calculate L2 dimensions for a document.

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use doc_doctor_domain::AnalyzeDocument;

use crate::commands::{create_analyze_use_case, read_file};
use crate::output::{format_output, DimensionsOutput, OutputFormat};

#[derive(Args)]
pub struct DimensionsCommand {
    /// Path to markdown file
    pub path: PathBuf,
}

impl DimensionsCommand {
    pub fn run(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        if verbose {
            eprintln!("Calculating dimensions: {}", self.path.display());
        }

        let content = read_file(&self.path)?;
        let use_case = create_analyze_use_case();

        let analysis = use_case
            .analyze(&content)
            .map_err(|e| anyhow::anyhow!("Analysis error: {}", e))?;

        let output = DimensionsOutput {
            path: self.path.display().to_string(),
            health: analysis.dimensions.health,
            usefulness_margin: analysis.dimensions.usefulness.margin,
            is_useful: analysis.dimensions.usefulness.is_useful,
            trust_level: analysis.dimensions.trust_level,
            freshness: analysis.dimensions.freshness,
        };

        println!("{}", format_output(&output, format)?);
        Ok(())
    }
}

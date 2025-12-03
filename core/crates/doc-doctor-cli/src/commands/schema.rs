//! Schema Command
//!
//! Export JSON schema definitions.

use anyhow::Result;
use clap::Args;


use crate::commands::create_schema_provider;
use crate::output::{format_output, OutputFormat, SchemaOutput};

#[derive(Args)]
pub struct SchemaCommand {
    /// Schema type (frontmatter, stubs)
    pub schema_type: String,
}

impl SchemaCommand {
    pub fn run(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        if verbose {
            eprintln!("Exporting schema: {}", self.schema_type);
        }

        let provider = create_schema_provider();

        let schema_json = match self.schema_type.to_lowercase().as_str() {
            "frontmatter" => provider.frontmatter_schema(),
            "stubs" => provider.stubs_schema(),
            other => {
                anyhow::bail!(
                    "Unknown schema type: '{}'. Valid types: frontmatter, stubs",
                    other
                );
            }
        };

        let schema: serde_json::Value = serde_json::from_str(schema_json)
            .map_err(|e| anyhow::anyhow!("Invalid schema JSON: {}", e))?;

        let output = SchemaOutput {
            schema_type: self.schema_type.clone(),
            schema,
        };

        println!("{}", format_output(&output, format)?);
        Ok(())
    }
}

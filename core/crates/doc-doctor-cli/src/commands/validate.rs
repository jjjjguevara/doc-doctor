//! Validate Command
//!
//! Validate frontmatter against J-Editorial schema.

use anyhow::Result;
use clap::Args;
use glob::glob;

use doc_doctor_domain::ValidateDocument;

use crate::commands::{create_validate_use_case, read_file};
use crate::output::{format_output, OutputFormat, ValidationOutput};

#[derive(Args)]
pub struct ValidateCommand {
    /// File pattern (glob)
    pub pattern: String,

    /// Strict mode - reject unknown fields
    #[arg(short, long)]
    pub strict: bool,
}

impl ValidateCommand {
    pub fn run(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        let use_case = create_validate_use_case();
        let paths: Vec<_> = glob(&self.pattern)
            .map_err(|e| anyhow::anyhow!("Invalid pattern: {}", e))?
            .filter_map(|p| p.ok())
            .collect();

        if paths.is_empty() {
            println!("No files match pattern: {}", self.pattern);
            return Ok(());
        }

        let mut all_valid = true;

        for path in paths {
            if verbose {
                eprintln!("Validating: {}", path.display());
            }

            let content = match read_file(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading {}: {}", path.display(), e);
                    all_valid = false;
                    continue;
                }
            };

            match use_case.validate(&content, self.strict) {
                Ok(result) => {
                    let output = ValidationOutput {
                        path: path.display().to_string(),
                        is_valid: result.is_valid,
                        error_count: result.errors.len(),
                        warning_count: result.warnings.len(),
                        errors: result.errors.iter().map(|e| e.message.clone()).collect(),
                        warnings: result.warnings.iter().map(|w| w.message.clone()).collect(),
                    };

                    if !result.is_valid {
                        all_valid = false;
                    }

                    println!("{}", format_output(&output, format)?);
                    println!();
                }
                Err(e) => {
                    eprintln!("Validation error for {}: {}", path.display(), e);
                    all_valid = false;
                }
            }
        }

        if !all_valid {
            std::process::exit(1);
        }

        Ok(())
    }
}

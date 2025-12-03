//! Batch Command
//!
//! Process multiple documents matching a glob pattern.

use anyhow::Result;
use clap::Args;
use glob::glob;
use rayon::prelude::*;
use std::path::PathBuf;

use doc_doctor_domain::AnalyzeDocument;

use crate::commands::{create_analyze_use_case, create_parser, read_file};
use crate::output::{format_output, BatchDocumentOutput, BatchOutput, OutputFormat};

#[derive(Args)]
pub struct BatchCommand {
    /// File pattern (glob)
    pub pattern: String,

    /// Include L2 dimensions
    #[arg(long)]
    pub dimensions: bool,

    /// Number of parallel jobs
    #[arg(short, long, default_value = "4")]
    pub jobs: usize,
}

impl BatchCommand {
    pub fn run(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        if verbose {
            eprintln!("Batch processing: {}", self.pattern);
        }

        // Collect paths
        let paths: Vec<PathBuf> = glob(&self.pattern)
            .map_err(|e| anyhow::anyhow!("Invalid pattern: {}", e))?
            .filter_map(|p| p.ok())
            .filter(|p| p.is_file())
            .collect();

        if paths.is_empty() {
            println!("No files match pattern: {}", self.pattern);
            return Ok(());
        }

        if verbose {
            eprintln!("Found {} files", paths.len());
        }

        // Configure thread pool
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.jobs)
            .build_global()
            .ok(); // Ignore if already set

        // Process documents in parallel
        let results: Vec<BatchDocumentOutput> = if self.dimensions {
            self.process_with_dimensions(&paths, verbose)
        } else {
            self.process_parse_only(&paths, verbose)
        };

        // Calculate statistics
        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = results.len() - succeeded;

        let average_health = if self.dimensions {
            let healths: Vec<f64> = results.iter().filter_map(|r| r.health).collect();
            if healths.is_empty() {
                None
            } else {
                Some(healths.iter().sum::<f64>() / healths.len() as f64)
            }
        } else {
            None
        };

        let output = BatchOutput {
            total: results.len(),
            succeeded,
            failed,
            average_health,
            results,
        };

        println!("{}", format_output(&output, format)?);
        Ok(())
    }

    fn process_with_dimensions(&self, paths: &[PathBuf], verbose: bool) -> Vec<BatchDocumentOutput> {
        let use_case = create_analyze_use_case();

        paths
            .par_iter()
            .map(|path| {
                if verbose {
                    eprintln!("  Processing: {}", path.display());
                }

                match read_file(path) {
                    Ok(content) => match use_case.analyze(&content) {
                        Ok(analysis) => BatchDocumentOutput {
                            path: path.display().to_string(),
                            success: true,
                            health: Some(analysis.dimensions.health),
                            error: None,
                        },
                        Err(e) => BatchDocumentOutput {
                            path: path.display().to_string(),
                            success: false,
                            health: None,
                            error: Some(e.to_string()),
                        },
                    },
                    Err(e) => BatchDocumentOutput {
                        path: path.display().to_string(),
                        success: false,
                        health: None,
                        error: Some(e.to_string()),
                    },
                }
            })
            .collect()
    }

    fn process_parse_only(&self, paths: &[PathBuf], verbose: bool) -> Vec<BatchDocumentOutput> {
        let parser = create_parser();

        paths
            .par_iter()
            .map(|path| {
                if verbose {
                    eprintln!("  Processing: {}", path.display());
                }

                match read_file(path) {
                    Ok(content) => match parser.parse(&content) {
                        Ok(_) => BatchDocumentOutput {
                            path: path.display().to_string(),
                            success: true,
                            health: None,
                            error: None,
                        },
                        Err(e) => BatchDocumentOutput {
                            path: path.display().to_string(),
                            success: false,
                            health: None,
                            error: Some(e.to_string()),
                        },
                    },
                    Err(e) => BatchDocumentOutput {
                        path: path.display().to_string(),
                        success: false,
                        health: None,
                        error: Some(e.to_string()),
                    },
                }
            })
            .collect()
    }
}

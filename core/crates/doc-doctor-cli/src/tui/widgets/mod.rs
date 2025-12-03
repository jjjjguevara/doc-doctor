//! TUI Widgets
//!
//! Custom widgets for the Doc Doctor TUI.

pub mod content_viewer;
pub mod dashboard;
pub mod document_viewer;
pub mod progress;
pub mod stub_list;
pub mod tables;
pub mod tests;
pub mod thought_stream;

// Re-exports for convenience
pub use progress::{ConsoleProgress, MultiProgress};
pub use tables::*;
pub use thought_stream::icons;

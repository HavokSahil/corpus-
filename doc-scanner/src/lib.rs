//! # doc-scanner
//!
//! A modular document image processing pipeline in Rust.
//!
//! Takes an input photo of a document, detects the document region,
//! corrects perspective, enhances readability, normalizes resolution,
//! and saves the result.
//!
//! ## Pipeline
//!
//! ```text
//! Input → Detect → Warp → Enhance → Resize → Encode
//! ```
//!
//! ## Usage
//!
//! ```bash
//! cargo run -- input.jpg output.png
//! cargo run -- input.jpg output.png --debug
//! ```

pub mod config;
pub mod convert;
pub mod detect;
pub mod encode;
pub mod enhance;
pub mod error;
pub mod pipeline;
pub mod resize;
pub mod transform;

// Re-export the main entry points for library consumers.
pub use config::PipelineConfig;
pub use error::{PipelineError, Result};
pub use pipeline::{run, run_in_memory};

//! Custom error types for the document scanning pipeline.
//!
//! Uses `thiserror` for ergonomic error derivation and implements
//! conversions from upstream error types (`opencv::Error`, `image::ImageError`).

use thiserror::Error;

/// All errors that can occur during pipeline execution.
#[derive(Debug, Error)]
pub enum PipelineError {
    /// Failed to open or decode the input image.
    #[error("failed to load image: {0}")]
    ImageLoad(String),

    /// No quadrilateral document contour was found in the image.
    #[error("no document contour found — ensure the document has clear edges against the background")]
    NoDocumentFound,

    /// An OpenCV operation failed.
    #[error("OpenCV error: {0}")]
    OpenCvError(String),

    /// Failed to encode or write the output image.
    #[error("encoding error: {0}")]
    EncodingError(String),

    /// Invalid configuration or CLI arguments.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Generic I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Conversions from upstream error types
// ---------------------------------------------------------------------------

impl From<opencv::Error> for PipelineError {
    fn from(e: opencv::Error) -> Self {
        PipelineError::OpenCvError(e.to_string())
    }
}

impl From<image::ImageError> for PipelineError {
    fn from(e: image::ImageError) -> Self {
        match e {
            image::ImageError::IoError(io) => PipelineError::Io(io),
            other => PipelineError::ImageLoad(other.to_string()),
        }
    }
}

/// Convenience alias used throughout the pipeline.
pub type Result<T> = std::result::Result<T, PipelineError>;

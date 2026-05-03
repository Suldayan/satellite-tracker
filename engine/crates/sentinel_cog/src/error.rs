use thiserror::Error;

#[derive(Debug, Error)]
pub enum CogError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid TIFF header: {0}")]
    InvalidHeader(String),

    #[error("TIFF tag {tag} not found ({name})")]
    MissingTag { tag: u16, name: &'static str },

    #[error("Out-of-bounds read: offset {offset}, size {size}")]
    OutOfBounds { offset: usize, size: usize },

    #[error("Unsupported TIFF tag type: tag {tag}, type {type_id}")]
    UnsupportedTagType { tag: u16, type_id: u16 },

    #[error("Tile {index} decompression failed: {source}")]
    DecompressFailed {
        index: usize,
        #[source]
        source: std::io::Error,
    },

    #[error("Buffer size mismatch: expected {expected}, got {actual}")]
    BufferMismatch { expected: usize, actual: usize },

    #[error("Tile length mismatch: expected {expected}, got {actual}")]
    TileLengthMismatch { expected: usize, actual: usize },

    #[error("Unsupported TIFF feature: {0}")]
    Unsupported(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type CogResult<T> = Result<T, CogError>;

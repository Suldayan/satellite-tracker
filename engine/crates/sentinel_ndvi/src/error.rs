use thiserror::Error;

#[derive(Debug, Error)]
pub enum NdviError {
    /// The two input bands have different pixel counts.
    #[error("Band dimension mismatch: B04 has {b04} pixels, B08 has {b08}")]
    DimensionMismatch { b04: usize, b08: usize },

    /// `past` and `present` slices passed to `calc_difference_map` differ in length.
    #[error("Difference map input length mismatch: past={past}, present={present}")]
    LengthMismatch { past: usize, present: usize },

    /// The pixel buffer was the wrong size to construct an image.
    #[error("Image buffer size mismatch: expected {expected} bytes, got {actual}")]
    BufferMismatch { expected: usize, actual: usize },

    /// An I/O error occurred while writing output files.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The `image` crate returned an error during encoding.
    #[error("Image encode error: {0}")]
    ImageEncode(#[from] image::ImageError),
}

pub type NdviResult<T> = Result<T, NdviError>;
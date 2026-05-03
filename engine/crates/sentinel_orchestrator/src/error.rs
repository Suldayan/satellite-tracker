use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("HTTP error fetching TLE: {0}")]
    TleFetch(#[from] reqwest::Error),

    #[error("TLE response had fewer than 3 lines")]
    TleMalformed,

    #[error("Pass prediction error: {0}")]
    Prediction(String),

    #[error("Invalid observer position: {0}")]
    InvalidObserver(String),
}

pub type OrchestratorResult<T> = Result<T, OrchestratorError>;
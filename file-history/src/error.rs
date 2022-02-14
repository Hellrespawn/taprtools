use thiserror::Error;

#[derive(Error, Debug)]

/// Error relating to file-history
pub enum HistoryError {
    /// Represents std::io::Error
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// Represents a generic error
    #[error("{0}")]
    Generic(String),
}

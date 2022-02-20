use thiserror::Error;

#[derive(Error, Debug)]
/// Error relating to file-history
pub enum HistoryError {
    /// Represents std::io::Error
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// Represents serde_json::Error
    // #[error("JSON error: {0}")]
    // JSON(#[from] serde_json::Error),

    /// Represents bincode::Error
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    /// Represents a generic error
    #[error("{0}")]
    Generic(String),
}

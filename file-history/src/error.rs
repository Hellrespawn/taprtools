use thiserror::Error;

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("Unable to save history, there is no associated path!")]
    NoPath,

    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("SerDe error: {0}")]
    SerDe(#[from] bincode::Error),

    #[error("TryFromInt error: {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),
}

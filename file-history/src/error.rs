use thiserror::Error;

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("TryFromInt error: {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),
}

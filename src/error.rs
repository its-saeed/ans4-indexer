use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Parsing error: {0}")]
    ParsingError(String),
    #[error("File I/O error: {0}")]
    FileIOError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

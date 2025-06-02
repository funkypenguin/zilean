#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Invalid UTF-8 slice index")]
    InvalidUtf8Index,

    #[error("Parsing logic panicked")]
    Panic,

    #[error("Other parsing error: {0}")]
    Other(String),
}

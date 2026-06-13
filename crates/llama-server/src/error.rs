#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Server not found: {0}")]
    NotFound(String),

    #[error("Failed to start server: {0}")]
    StartFailed(String),

    #[error("Server already running")]
    AlreadyRunning,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ServerError>;

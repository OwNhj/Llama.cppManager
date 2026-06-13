use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum LlamaError {
    #[error("Model not found: {0}")]
    ModelNotFound(PathBuf),

    #[error("Invalid model format: {0}")]
    InvalidModelFormat(String),

    #[error("Quantize failed: {0}")]
    QuantizeFailed(String),

    #[error("Environment detection failed: {0}")]
    EnvironmentDetectionFailed(String),

    #[error("Network unavailable")]
    NetworkUnavailable,

    #[error("HuggingFace API error: {0}")]
    HuggingFaceApiError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, LlamaError>;

use llama_core::error::LlamaError;

#[test]
fn test_model_not_found_error() {
    let err = LlamaError::ModelNotFound("/path/to/model.gguf".into());
    assert!(err.to_string().contains("Model not found"));
    assert!(err.to_string().contains("/path/to/model.gguf"));
}

#[test]
fn test_invalid_model_format_error() {
    let err = LlamaError::InvalidModelFormat("unknown magic bytes".into());
    assert!(err.to_string().contains("Invalid model format"));
    assert!(err.to_string().contains("unknown magic bytes"));
}

#[test]
fn test_huggingface_api_error() {
    let err = LlamaError::HuggingFaceApiError("rate limit exceeded".into());
    assert!(err.to_string().contains("HuggingFace API error"));
    assert!(err.to_string().contains("rate limit exceeded"));
}

#[test]
fn test_io_error_from_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let llama_err: LlamaError = io_err.into();
    assert!(llama_err.to_string().contains("IO error"));
    assert!(llama_err.to_string().contains("file missing"));
}

#[test]
fn test_quantize_error() {
    let err = LlamaError::QuantizeFailed("invalid format".into());
    assert!(err.to_string().contains("Quantize failed"));
}

#[test]
fn test_network_error() {
    let err = LlamaError::NetworkUnavailable;
    assert!(err.to_string().contains("Network unavailable"));
}

#[test]
fn test_environment_error() {
    let err = LlamaError::EnvironmentDetectionFailed("no GPU".into());
    assert!(err.to_string().contains("Environment detection failed"));
}

#[test]
fn test_config_error() {
    let err = LlamaError::ConfigError("invalid JSON".into());
    assert!(err.to_string().contains("Config error"));
}

use llama_core::model::{ModelFormat, ModelInfo};
use std::path::PathBuf;

#[test]
fn test_model_format_detection() {
    assert_eq!(ModelFormat::detect("model.gguf"), ModelFormat::Gguf);
    assert_eq!(ModelFormat::detect("model.bin"), ModelFormat::PyTorch);
    assert_eq!(
        ModelFormat::detect("model.safetensors"),
        ModelFormat::SafeTensors
    );
    assert_eq!(ModelFormat::detect("model.unknown"), ModelFormat::Unknown);
}

#[test]
fn test_model_format_is_gguf() {
    assert!(ModelFormat::Gguf.is_gguf());
    assert!(!ModelFormat::PyTorch.is_gguf());
    assert!(!ModelFormat::SafeTensors.is_gguf());
}

#[test]
fn test_model_info_creation() {
    let info = ModelInfo {
        path: PathBuf::from("model.gguf"),
        format: ModelFormat::Gguf,
        name: "test-model".into(),
        size_bytes: 1024 * 1024 * 1024,
    };
    assert_eq!(info.size_gb(), 1.0);
}

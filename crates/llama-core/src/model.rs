use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum ModelFormat {
    Gguf,
    PyTorch,
    SafeTensors,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub path: PathBuf,
    pub format: ModelFormat,
    pub name: String,
    pub size_bytes: u64,
}

impl ModelFormat {
    pub fn detect(filename: &str) -> Self {
        if filename.ends_with(".gguf") {
            ModelFormat::Gguf
        } else if filename.ends_with(".bin") {
            ModelFormat::PyTorch
        } else if filename.ends_with(".safetensors") {
            ModelFormat::SafeTensors
        } else {
            ModelFormat::Unknown
        }
    }

    pub fn is_gguf(&self) -> bool {
        matches!(self, ModelFormat::Gguf)
    }
}

impl ModelInfo {
    pub fn size_gb(&self) -> f64 {
        self.size_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    }
}

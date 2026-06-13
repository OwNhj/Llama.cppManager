use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub context_size: u32,
    pub batch_size: u32,
    pub gpu_offload_layers: u32,
    pub flash_attention: bool,
    // MTP (Multi-Token Prediction) 参数
    pub mtp_enabled: bool,
    pub mtp_n_predict: u32,
    pub mtp_n_vocab: u32,
    pub mtp_n_embd: u32,
}

#[derive(Debug, Clone)]
pub struct ParamMeta {
    pub name: &'static str,
    pub display_name: &'static str,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub default: f32,
    pub description: &'static str,
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            context_size: 2048,
            batch_size: 512,
            gpu_offload_layers: 32,
            flash_attention: true,
            mtp_enabled: false,
            mtp_n_predict: 1,
            mtp_n_vocab: 32000,
            mtp_n_embd: 4096,
        }
    }
}

impl ModelParams {
    pub fn meta() -> Vec<ParamMeta> {
        vec![
            ParamMeta {
                name: "temperature",
                display_name: "Temperature",
                min: Some(0.0),
                max: Some(2.0),
                step: Some(0.05),
                default: 0.8,
                description: "采样温度，越高越随机",
            },
            ParamMeta {
                name: "top_p",
                display_name: "Top-P",
                min: Some(0.0),
                max: Some(1.0),
                step: Some(0.05),
                default: 0.9,
                description: "核采样概率",
            },
            ParamMeta {
                name: "top_k",
                display_name: "Top-K",
                min: Some(1.0),
                max: Some(100.0),
                step: Some(1.0),
                default: 40.0,
                description: "Top-K采样",
            },
            ParamMeta {
                name: "repeat_penalty",
                display_name: "Repeat Penalty",
                min: Some(1.0),
                max: Some(2.0),
                step: Some(0.05),
                default: 1.1,
                description: "重复惩罚",
            },
            ParamMeta {
                name: "context_size",
                display_name: "Context Size",
                min: Some(128.0),
                max: Some(131072.0),
                step: Some(128.0),
                default: 2048.0,
                description: "上下文长度",
            },
            ParamMeta {
                name: "batch_size",
                display_name: "Batch Size",
                min: Some(1.0),
                max: Some(8192.0),
                step: Some(1.0),
                default: 512.0,
                description: "批处理大小",
            },
            ParamMeta {
                name: "gpu_offload_layers",
                display_name: "GPU Offload Layers",
                min: Some(0.0),
                max: Some(128.0),
                step: Some(1.0),
                default: 32.0,
                description: "GPU卸载层数",
            },
            ParamMeta {
                name: "flash_attention",
                display_name: "Flash Attention",
                min: None,
                max: None,
                step: None,
                default: 1.0,
                description: "Flash Attention加速",
            },
            ParamMeta {
                name: "mtp_n_predict",
                display_name: "MTP N Predict",
                min: Some(1.0),
                max: Some(8.0),
                step: Some(1.0),
                default: 1.0,
                description: "MTP每次预测的token数",
            },
            ParamMeta {
                name: "mtp_n_vocab",
                display_name: "MTP Vocab Size",
                min: Some(1000.0),
                max: Some(256000.0),
                step: Some(1000.0),
                default: 32000.0,
                description: "MTP词表大小",
            },
            ParamMeta {
                name: "mtp_n_embd",
                display_name: "MTP Embedding",
                min: Some(256.0),
                max: Some(16384.0),
                step: Some(256.0),
                default: 4096.0,
                description: "MTP嵌入维度",
            },
        ]
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.temperature < 0.0 || self.temperature > 2.0 {
            anyhow::bail!("Temperature must be in [0, 2]");
        }
        if self.top_p < 0.0 || self.top_p > 1.0 {
            anyhow::bail!("Top-P must be in [0, 1]");
        }
        if self.top_k < 1 {
            anyhow::bail!("Top-K must be >= 1");
        }
        if self.repeat_penalty < 1.0 || self.repeat_penalty > 2.0 {
            anyhow::bail!("Repeat Penalty must be in [1, 2]");
        }
        if self.context_size < 128 {
            anyhow::bail!("Context Size must be >= 128");
        }
        if self.batch_size < 1 {
            anyhow::bail!("Batch Size must be >= 1");
        }
        if self.gpu_offload_layers > 128 {
            anyhow::bail!("GPU Offload Layers must be in [0, 128]");
        }
        if self.mtp_enabled {
            if self.mtp_n_predict < 1 || self.mtp_n_predict > 8 {
                anyhow::bail!("MTP N Predict must be in [1, 8]");
            }
            if self.mtp_n_vocab < 1000 || self.mtp_n_vocab > 256000 {
                anyhow::bail!("MTP Vocab Size must be in [1000, 256000]");
            }
            if self.mtp_n_embd < 256 || self.mtp_n_embd > 16384 {
                anyhow::bail!("MTP Embedding must be in [256, 16384]");
            }
        }
        Ok(())
    }
}

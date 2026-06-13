use llama_core::environment::DeviceType;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CommProtocol {
    SharedMemory,
    Tcp,
    Rdma,
}

impl fmt::Display for CommProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommProtocol::SharedMemory => write!(f, "SharedMemory"),
            CommProtocol::Tcp => write!(f, "Tcp"),
            CommProtocol::Rdma => write!(f, "Rdma"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OffloadMode {
    Normal,
    AfSeparation,
    PdSeparation,
    Custom,
}

impl fmt::Display for OffloadMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OffloadMode::Normal => write!(f, "Normal"),
            OffloadMode::AfSeparation => write!(f, "AF Separation"),
            OffloadMode::PdSeparation => write!(f, "PD Separation"),
            OffloadMode::Custom => write!(f, "Custom"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayerOffload {
    pub layer_index: u32,
    pub device: DeviceType,
    pub vram_mb: u64,
}

#[derive(Debug, Clone)]
pub struct OffloadConfig {
    pub mode: OffloadMode,
    pub layers: Vec<LayerOffload>,
    pub pd_prefill_addr: Option<String>,
    pub pd_decode_addr: Option<String>,
    pub comm_protocol: Option<CommProtocol>,
}

impl Default for OffloadConfig {
    fn default() -> Self {
        Self {
            mode: OffloadMode::Normal,
            layers: Vec::new(),
            pd_prefill_addr: None,
            pd_decode_addr: None,
            comm_protocol: None,
        }
    }
}

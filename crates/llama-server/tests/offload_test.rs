use llama_core::environment::DeviceType;
use llama_server::offload::{CommProtocol, LayerOffload, OffloadConfig, OffloadMode};

#[test]
fn test_comm_protocol_display() {
    assert_eq!(CommProtocol::SharedMemory.to_string(), "SharedMemory");
    assert_eq!(CommProtocol::Tcp.to_string(), "Tcp");
    assert_eq!(CommProtocol::Rdma.to_string(), "Rdma");
}

#[test]
fn test_offload_mode_display() {
    assert_eq!(OffloadMode::Normal.to_string(), "Normal");
    assert_eq!(OffloadMode::AfSeparation.to_string(), "AF Separation");
    assert_eq!(OffloadMode::PdSeparation.to_string(), "PD Separation");
    assert_eq!(OffloadMode::Custom.to_string(), "Custom");
}

#[test]
fn test_offload_config_default() {
    let config = OffloadConfig::default();
    assert_eq!(config.mode, OffloadMode::Normal);
    assert_eq!(config.layers.len(), 0);
}

#[test]
fn test_layer_offload() {
    let layer = LayerOffload {
        layer_index: 0,
        device: DeviceType::Cuda(0),
        vram_mb: 590,
    };
    assert_eq!(layer.device, DeviceType::Cuda(0));
}

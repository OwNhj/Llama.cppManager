use llama_core::environment::{DeviceType, Environment};

#[test]
fn test_cpu_info_detection() {
    let env = Environment::detect();
    assert!(!env.cpu.model.is_empty());
    assert!(env.cpu.cores > 0);
    assert!(env.cpu.threads > 0);
}

#[test]
fn test_device_type_display() {
    assert_eq!(DeviceType::Cpu.to_string(), "CPU");
    assert_eq!(DeviceType::Cuda(0).to_string(), "CUDA:0");
    assert_eq!(DeviceType::Rocm(0).to_string(), "ROCm:0");
    assert_eq!(DeviceType::Metal.to_string(), "Metal");
    assert_eq!(DeviceType::Npu.to_string(), "NPU");
}

#[test]
fn test_offload_recommendation() {
    let env = Environment::detect();
    let rec = env.recommend_offload(32);
    assert!(rec.total_layers > 0);
    assert!(rec.gpu_layers <= rec.total_layers);
}

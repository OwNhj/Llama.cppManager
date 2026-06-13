use llama_core::network::{NetworkMonitor, NetworkStatus};

#[tokio::test]
async fn test_network_status_default() {
    let status = NetworkStatus::default();
    assert_eq!(status, NetworkStatus::Offline);
}

#[test]
fn test_mirror_config() {
    let monitor = NetworkMonitor::new();
    let mirrors = monitor.mirrors();
    assert!(mirrors.len() >= 2);
    assert!(mirrors.iter().any(|m| m.name == "HuggingFace 官方"));
    assert!(mirrors.iter().any(|m| m.name == "hf-mirror.com"));
}

use llama_server::process::{ServerConfig, ServerState};

#[test]
fn test_server_config_default() {
    let config = ServerConfig::default();
    assert_eq!(config.port, 8080);
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.ctx_size, 2048);
}

#[test]
fn test_server_state() {
    assert_eq!(ServerState::Stopped.to_string(), "Stopped");
    assert_eq!(ServerState::Starting.to_string(), "Starting");
    assert_eq!(ServerState::Running.to_string(), "Running");
}

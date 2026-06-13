use std::fmt;
use tokio::process::{Child, Command};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub model_path: String,
    pub ctx_size: u32,
    pub n_gpu_layers: u32,
}

#[derive(Debug, PartialEq)]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Error(String),
}

impl fmt::Display for ServerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerState::Stopped => write!(f, "Stopped"),
            ServerState::Starting => write!(f, "Starting"),
            ServerState::Running => write!(f, "Running"),
            ServerState::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

pub struct ServerProcess {
    config: ServerConfig,
    state: ServerState,
    child: Option<Child>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".into(),
            model_path: String::new(),
            ctx_size: 2048,
            n_gpu_layers: 32,
        }
    }
}

impl ServerProcess {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            state: ServerState::Stopped,
            child: None,
        }
    }

    pub fn state(&self) -> &ServerState {
        &self.state
    }

    pub async fn start(&mut self) -> crate::error::Result<()> {
        if self.state == ServerState::Running || self.state == ServerState::Starting {
            return Err(crate::error::ServerError::AlreadyRunning);
        }

        self.state = ServerState::Starting;

        let child = Command::new("llama-server")
            .args([
                "--model",
                &self.config.model_path,
                "--port",
                &self.config.port.to_string(),
                "--host",
                &self.config.host,
                "--ctx-size",
                &self.config.ctx_size.to_string(),
                "--n-gpu-layers",
                &self.config.n_gpu_layers.to_string(),
            ])
            .spawn()?;

        self.child = Some(child);
        self.state = ServerState::Running;
        Ok(())
    }

    pub async fn stop(&mut self) -> crate::error::Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?;
            let _ = child.wait().await;
            self.state = ServerState::Stopped;
        }
        Ok(())
    }
}

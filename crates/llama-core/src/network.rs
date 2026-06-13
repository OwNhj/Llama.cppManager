use serde::{Deserialize, Serialize};
use tokio::sync::watch;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum NetworkStatus {
    #[default]
    Offline,
    Online {
        latency_ms: u64,
    },
    RateLimited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfig {
    pub name: String,
    pub url: String,
    pub priority: u32,
}

pub struct NetworkMonitor {
    mirrors: Vec<MirrorConfig>,
    status_tx: watch::Sender<NetworkStatus>,
    status_rx: watch::Receiver<NetworkStatus>,
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkMonitor {
    pub fn new() -> Self {
        let (status_tx, status_rx) = watch::channel(NetworkStatus::Offline);
        Self {
            mirrors: vec![
                MirrorConfig {
                    name: "HuggingFace 官方".into(),
                    url: "https://huggingface.co".into(),
                    priority: 3,
                },
                MirrorConfig {
                    name: "hf-mirror.com".into(),
                    url: "https://hf-mirror.com".into(),
                    priority: 2,
                },
            ],
            status_tx,
            status_rx,
        }
    }

    pub fn mirrors(&self) -> &[MirrorConfig] {
        &self.mirrors
    }

    pub async fn check_status(&self) -> NetworkStatus {
        for mirror in &self.mirrors {
            let start = std::time::Instant::now();
            let url = format!("{}/api/models", mirror.url);
            match reqwest::get(&url).await {
                Ok(response) => {
                    if response.status().as_u16() == 429 {
                        return NetworkStatus::RateLimited;
                    }
                    if response.status().is_success() {
                        return NetworkStatus::Online {
                            latency_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                }
                Err(_) => continue,
            }
        }
        NetworkStatus::Offline
    }

    pub fn on_status_change(&self) -> watch::Receiver<NetworkStatus> {
        self.status_rx.clone()
    }

    pub async fn start_monitoring(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let status = self.check_status().await;
            let _ = self.status_tx.send(status);
        }
    }
}

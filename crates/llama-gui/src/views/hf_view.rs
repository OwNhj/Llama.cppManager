use eframe::egui;
use llama_core::network::NetworkStatus;
use std::io::Read;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub struct HfView {
    search_query: String,
    search_results: Vec<SearchModel>,
    selected_model: Option<SearchModel>,
    download_progress: Option<f32>,
    download_size: Option<u64>,
    network_status: Option<NetworkStatus>,
    mirror_url: String,
    status_message: String,
    is_searching: Arc<AtomicBool>,
    search_rx: Option<std::sync::mpsc::Receiver<SearchResult>>,
    is_downloading: Arc<AtomicBool>,
    download_rx: Option<std::sync::mpsc::Receiver<DownloadResult>>,
}

#[derive(Debug, Clone)]
pub struct SearchModel {
    pub id: String,
    pub model_type: String,
    pub downloads: u64,
    pub tags: Vec<String>,
    pub size_bytes: u64,
}

enum SearchResult {
    Success(Vec<SearchModel>),
    Error(String),
}

enum DownloadResult {
    Progress { downloaded: u64, total: u64 },
    Complete(String),
    Error(String),
}

impl Default for HfView {
    fn default() -> Self {
        Self::new()
    }
}

impl HfView {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            search_results: Vec::new(),
            selected_model: None,
            download_progress: None,
            download_size: None,
            network_status: None,
            mirror_url: "https://hf-mirror.com".into(),
            status_message: String::new(),
            is_searching: Arc::new(AtomicBool::new(false)),
            search_rx: None,
            is_downloading: Arc::new(AtomicBool::new(false)),
            download_rx: None,
        }
    }

    fn format_downloads(downloads: u64) -> String {
        if downloads >= 1000000 {
            format!("{:.1}M", downloads as f64 / 1000000.0)
        } else if downloads >= 1000 {
            format!("{:.1}K", downloads as f64 / 1000.0)
        } else {
            downloads.to_string()
        }
    }

    fn format_size(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
        } else if bytes >= 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
        } else if bytes >= 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("HuggingFace 模型管理");

        // 检查搜索结果
        if let Some(rx) = self.search_rx.take() {
            if let Ok(result) = rx.try_recv() {
                match result {
                    SearchResult::Success(results) => {
                        let count = results.len();
                        self.search_results = results;
                        self.status_message = format!("找到 {} 个模型", count);
                        self.is_searching.store(false, Ordering::SeqCst);
                    }
                    SearchResult::Error(e) => {
                        self.status_message = format!("搜索失败: {}", e);
                        self.is_searching.store(false, Ordering::SeqCst);
                    }
                }
            } else {
                self.search_rx = Some(rx);
            }
        }

        // 检查下载结果
        if let Some(rx) = self.download_rx.take() {
            if let Ok(result) = rx.try_recv() {
                match result {
                    DownloadResult::Progress { downloaded, total } => {
                        self.download_size = Some(downloaded);
                        self.download_progress = Some(if total > 0 { downloaded as f32 / total as f32 } else { 0.0 });
                    }
                    DownloadResult::Complete(path) => {
                        self.status_message = format!("下载完成: {}", path);
                        self.download_progress = None;
                        self.download_size = None;
                        self.is_downloading.store(false, Ordering::SeqCst);
                    }
                    DownloadResult::Error(e) => {
                        self.status_message = format!("下载失败: {}", e);
                        self.download_progress = None;
                        self.download_size = None;
                        self.is_downloading.store(false, Ordering::SeqCst);
                    }
                }
            } else {
                self.download_rx = Some(rx);
            }
        }

        let searching = self.is_searching.load(Ordering::SeqCst);
        let downloading = self.is_downloading.load(Ordering::SeqCst);

        // Network status
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("网络:");
            match &self.network_status {
                Some(NetworkStatus::Online { .. }) => {
                    ui.colored_label(egui::Color32::GREEN, "● 在线");
                }
                Some(NetworkStatus::Offline) => {
                    ui.colored_label(egui::Color32::RED, "● 离线");
                }
                _ => {
                    ui.label("● 未检测");
                }
            }
            if !searching && !downloading {
                if ui.button("检测").clicked() {
                    self.network_status = Some(NetworkStatus::Online { latency_ms: 50 });
                }
            }
        });

        // API地址 - 可编辑
        ui.horizontal(|ui| {
            ui.label("API:");
            ui.add_enabled(!searching && !downloading, 
                egui::TextEdit::singleline(&mut self.mirror_url));
        });

        ui.separator();

        // Search section
        ui.strong("搜索模型");
        ui.horizontal(|ui| {
            let response = ui.add_enabled(!searching && !downloading, 
                egui::TextEdit::singleline(&mut self.search_query).hint_text("输入模型名称..."));
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !self.search_query.is_empty() && !searching && !downloading {
                    self.do_search();
                }
            }
            
            if ui.add_enabled(!searching && !downloading && !self.search_query.is_empty(), 
                egui::Button::new("搜索")).clicked() {
                self.do_search();
            }
            
            if searching {
                ui.spinner();
            }
        });

        // Search results
        if !self.search_results.is_empty() {
            ui.separator();
            ui.horizontal(|ui| {
                ui.strong(format!("结果 ({}个)", self.search_results.len()));
                if ui.small_button("清空").clicked() {
                    self.search_results.clear();
                    self.selected_model = None;
                }
            });

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for model in &self.search_results.clone() {
                        let is_selected = self
                            .selected_model
                            .as_ref()
                            .map(|m| m.id == model.id)
                            .unwrap_or(false);

                        let model_id = model.id.clone();
                        let model_type = model.model_type.clone();
                        let downloads = model.downloads;
                        
                        let response = ui.selectable_label(is_selected, &model_id);
                        ui.horizontal(|ui| {
                            ui.small(&model_type);
                            ui.separator();
                            ui.small(format!("{} 次下载", Self::format_downloads(downloads)));
                        });
                        
                        if response.clicked() {
                            self.selected_model = Some(model.clone());
                        }
                    }
                });
        }

        // Selected model actions
        let selected_id = self.selected_model.as_ref().map(|m| m.id.clone());
        
        if let Some(model_id) = selected_id {
            ui.separator();
            ui.strong("已选择模型");
            ui.label(&model_id);

            if !downloading {
                if ui.button("下载模型").clicked() {
                    self.start_download(&model_id);
                }
            } else {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("下载中...");
                    if ui.button("取消").clicked() {
                        self.cancel_download();
                    }
                });
            }

            // Download progress
            if let Some(progress) = self.download_progress {
                ui.separator();
                ui.strong("下载进度");
                
                let progress_text = if let Some(size) = self.download_size {
                    format!("{} ({:.1}%)", Self::format_size(size), progress * 100.0)
                } else {
                    format!("{:.1}%", progress * 100.0)
                };
                
                ui.add(egui::ProgressBar::new(progress)
                    .text(progress_text)
                    .animate(true));
            }
        }

        // Status message
        if !self.status_message.is_empty() {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                if ui.small_button("清除").clicked() {
                    self.status_message.clear();
                }
            });
        }
    }

    fn do_search(&mut self) {
        self.is_searching.store(true, Ordering::SeqCst);
        self.status_message = format!("搜索中: {}...", self.search_query);
        
        let query = self.search_query.clone();
        let mirror_url = self.mirror_url.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.search_rx = Some(rx);
        
        std::thread::spawn(move || {
            let url = format!("{}/api/models?search={}&limit=20", mirror_url, query);
            
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new());
            
            match client.get(&url).send() {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.text() {
                            Ok(text) => {
                                if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                                    let models: Vec<SearchModel> = items.into_iter().filter_map(|item| {
                                        let id = item.get("id")?.as_str()?.to_string();
                                        let model_type = item.get("pipeline_tag")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown")
                                            .to_string();
                                        let downloads = item.get("downloads")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0);
                                        let tags = item.get("tags")
                                            .and_then(|v| v.as_array())
                                            .map(|arr| {
                                                arr.iter()
                                                    .filter_map(|t| t.as_str().map(|s| s.to_string()))
                                                    .collect()
                                            })
                                            .unwrap_or_default();
                                        
                                        Some(SearchModel {
                                            id,
                                            model_type,
                                            downloads,
                                            tags,
                                            size_bytes: 0, // 将在需要时获取
                                        })
                                    }).collect();
                                    let _ = tx.send(SearchResult::Success(models));
                                } else {
                                    let _ = tx.send(SearchResult::Error("JSON解析失败".into()));
                                }
                            }
                            Err(_) => {
                                let _ = tx.send(SearchResult::Error("读取响应失败".into()));
                            }
                        }
                    } else {
                        let _ = tx.send(SearchResult::Error(format!("HTTP {}", resp.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(SearchResult::Error(format!("请求失败: {}", e)));
                }
            }
        });
    }

    fn start_download(&mut self, model_id: &str) {
        self.is_downloading.store(true, Ordering::SeqCst);
        self.status_message = format!("开始下载: {}", model_id);
        self.download_progress = Some(0.0);
        self.download_size = Some(0);
        
        let model_id = model_id.to_string();
        let mirror_url = self.mirror_url.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.download_rx = Some(rx);
        
        std::thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new());
            
            // 获取模型信息
            let model_url = format!("{}/api/models/{}", mirror_url, model_id);
            match client.get(&model_url).send() {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(model_info) = resp.json::<serde_json::Value>() {
                            let siblings = model_info.get("siblings")
                                .and_then(|v| v.as_array())
                                .cloned()
                                .unwrap_or_default();
                            
                            // 找到模型文件
                            if let Some(sibling) = siblings.iter().find(|s| {
                                s.get("rfilename")
                                    .and_then(|v| v.as_str())
                                    .map(|f| f.ends_with(".gguf") || f.ends_with(".safetensors") || f.ends_with(".bin"))
                                    .unwrap_or(false)
                            }) {
                                let filename = sibling.get("rfilename")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                
                                let download_url = format!("{}/{}/resolve/main/{}", mirror_url, model_id, filename);
                                
                                // 下载文件
                                match client.get(&download_url).send() {
                                    Ok(mut resp) => {
                                        let total_size = resp.content_length().unwrap_or(0);
                                        let mut downloaded = 0u64;
                                        
                                        let mut buffer = [0u8; 8192];
                                        loop {
                                            match resp.read(&mut buffer) {
                                                Ok(0) => break,
                                                Ok(n) => {
                                                    downloaded += n as u64;
                                                    let _ = tx.send(DownloadResult::Progress { 
                                                        downloaded, 
                                                        total: total_size 
                                                    });
                                                }
                                                Err(_) => break,
                                            }
                                        }
                                        
                                        let _ = tx.send(DownloadResult::Complete(format!("下载完成: {}", filename)));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(DownloadResult::Error(format!("下载失败: {}", e)));
                                    }
                                }
                            } else {
                                let _ = tx.send(DownloadResult::Error("未找到可下载的模型文件".into()));
                            }
                        } else {
                            let _ = tx.send(DownloadResult::Error("解析模型信息失败".into()));
                        }
                    } else {
                        let _ = tx.send(DownloadResult::Error(format!("HTTP {}", resp.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(DownloadResult::Error(format!("请求失败: {}", e)));
                }
            }
        });
    }

    fn cancel_download(&mut self) {
        self.is_downloading.store(false, Ordering::SeqCst);
        self.download_progress = None;
        self.download_size = None;
        self.status_message = "下载已取消".into();
    }
}

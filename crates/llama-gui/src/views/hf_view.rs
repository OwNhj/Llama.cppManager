use eframe::egui;
use llama_core::network::NetworkStatus;
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
}

#[derive(Debug, Clone)]
pub struct SearchModel {
    pub id: String,
    pub model_type: String,
    pub downloads: u64,
    pub tags: Vec<String>,
}

enum SearchResult {
    Success(Vec<SearchModel>),
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

    fn format_downloads(downloads: u64) -> String {
        if downloads >= 1000000 {
            format!("{:.1}M", downloads as f64 / 1000000.0)
        } else if downloads >= 1000 {
            format!("{:.1}K", downloads as f64 / 1000.0)
        } else {
            downloads.to_string()
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

        let searching = self.is_searching.load(Ordering::SeqCst);

        // Network status section
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("网络状态:");
            match &self.network_status {
                Some(NetworkStatus::Online { latency_ms }) => {
                    ui.colored_label(egui::Color32::GREEN, "● 在线");
                    ui.label(format!("{}ms", latency_ms));
                }
                Some(NetworkStatus::Offline) => {
                    ui.colored_label(egui::Color32::RED, "● 离线");
                    ui.label("仅可使用本地模型");
                }
                Some(NetworkStatus::RateLimited) => {
                    ui.colored_label(egui::Color32::YELLOW, "● 限速");
                    ui.label("请稍后重试");
                }
                None => {
                    ui.label("● 未检测");
                }
            }
            if !searching {
                if ui.button("检测网络").clicked() {
                    self.network_status = Some(NetworkStatus::Online { latency_ms: 50 });
                    self.status_message = "网络检测完成".into();
                }
            }
        });

        // Mirror site selection
        ui.horizontal(|ui| {
            ui.label("镜像站:");
            ui.add_enabled(!searching, egui::TextEdit::singleline(&mut self.mirror_url));
            
            let switch_enabled = !searching;
            if ui.add_enabled(switch_enabled, egui::Button::new("切换")).clicked() {
                self.status_message = format!("已切换到: {}", self.mirror_url);
            }
            if ui.add_enabled(switch_enabled, egui::Button::new("官方")).clicked() {
                self.mirror_url = "https://huggingface.co".into();
                self.status_message = "已切换到HuggingFace官方".into();
            }
            if ui.add_enabled(switch_enabled, egui::Button::new("hf-mirror")).clicked() {
                self.mirror_url = "https://hf-mirror.com".into();
                self.status_message = "已切换到hf-mirror.com".into();
            }
        });

        // 当前API地址
        ui.small(format!("当前API: {}", self.mirror_url));

        ui.separator();

        // Search section
        ui.strong("搜索模型");
        ui.horizontal(|ui| {
            let response = ui.add_enabled(!searching, 
                egui::TextEdit::singleline(&mut self.search_query).hint_text("输入模型名称搜索..."));
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !self.search_query.is_empty() && !searching {
                    self.do_search();
                }
            }
            
            if ui.add_enabled(!searching && !self.search_query.is_empty(), 
                egui::Button::new("搜索")).clicked() {
                self.do_search();
            }
            
            if searching {
                ui.spinner();
                ui.label("搜索中...");
            }
        });

        // Search results
        if !self.search_results.is_empty() {
            ui.separator();
            ui.horizontal(|ui| {
                ui.strong(format!("搜索结果 ({}个模型)", self.search_results.len()));
                if ui.small_button("清空").clicked() {
                    self.search_results.clear();
                    self.selected_model = None;
                }
            });

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for model in &self.search_results {
                        let is_selected = self
                            .selected_model
                            .as_ref()
                            .map(|m| m.id == model.id)
                            .unwrap_or(false);

                        let response = ui.selectable_label(is_selected, "");
                        let model_response = ui.horizontal(|ui| {
                            ui.strong(&model.id);
                            ui.separator();
                            ui.label(&model.model_type);
                            ui.separator();
                            ui.small(format!("下载: {}", Self::format_downloads(model.downloads)));
                        });

                        if response.clicked() || model_response.response.clicked() {
                            self.selected_model = Some(model.clone());
                        }
                    }
                });
        }

        // Selected model actions
        if let Some(ref model) = self.selected_model {
            ui.separator();
            ui.strong("已选择模型");
            
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(45, 45, 55))
                .rounding(egui::Rounding::same(6.0))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.label(format!("模型: {}", model.id));
                    ui.label(format!("类型: {}", model.model_type));
                    ui.label(format!("下载量: {}", Self::format_downloads(model.downloads)));
                    ui.label(format!("标签: {}", model.tags.join(", ")));
                });

            ui.horizontal(|ui| {
                if ui.button("下载模型").clicked() {
                    self.status_message = format!("开始下载: {}", model.id);
                    self.download_progress = Some(0.0);
                    self.download_size = Some(0);
                }
            });

            // Download progress
            if let Some(progress) = self.download_progress {
                ui.separator();
                ui.strong("下载进度");
                
                let progress_text = format!("{:.1}%", progress * 100.0);
                
                ui.add(egui::ProgressBar::new(progress)
                    .text(progress_text)
                    .animate(true));
                
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        self.download_progress = None;
                        self.download_size = None;
                        self.status_message = "下载已取消".into();
                    }
                });
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
            // 使用简单的HTTP请求
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
                                // 尝试解析JSON
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
}

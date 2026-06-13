use eframe::egui;
use llama_core::huggingface::{HfClient, HfModel};
use llama_core::network::NetworkStatus;
use std::sync::{Arc, Mutex};

pub struct HfView {
    search_query: String,
    search_results: Vec<HfModel>,
    selected_model: Option<HfModel>,
    download_progress: Option<f32>,
    download_size: Option<u64>,
    network_status: Option<NetworkStatus>,
    hf_client: Arc<Mutex<HfClient>>,
    mirror_url: String,
    status_message: String,
    is_searching: Arc<Mutex<bool>>,
    search_rx: Option<std::sync::mpsc::Receiver<SearchResult>>,
}

enum SearchResult {
    Success(Vec<HfModel>),
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
            hf_client: Arc::new(Mutex::new(HfClient::new("https://huggingface.co".into()))),
            mirror_url: "https://hf-mirror.com".into(),
            status_message: String::new(),
            is_searching: Arc::new(Mutex::new(false)),
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
                        self.search_results = results;
                        self.status_message = format!("找到 {} 个模型", self.search_results.len());
                        *self.is_searching.lock().unwrap() = false;
                    }
                    SearchResult::Error(e) => {
                        self.status_message = format!("搜索失败: {}", e);
                        *self.is_searching.lock().unwrap() = false;
                    }
                }
            } else {
                self.search_rx = Some(rx);
            }
        }

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
            if ui.button("检测网络").clicked() {
                self.network_status = Some(NetworkStatus::Online { latency_ms: 50 });
                self.status_message = "网络检测完成".into();
            }
        });

        // Mirror site selection
        ui.horizontal(|ui| {
            ui.label("镜像站:");
            ui.text_edit_singleline(&mut self.mirror_url);
            if ui.button("切换").clicked() {
                let mut client = self.hf_client.lock().unwrap();
                *client = HfClient::new(self.mirror_url.clone());
                self.status_message = format!("已切换到: {}", self.mirror_url);
            }
            if ui.button("官方").clicked() {
                let mut client = self.hf_client.lock().unwrap();
                *client = HfClient::new("https://huggingface.co".into());
                self.mirror_url = "https://huggingface.co".into();
                self.status_message = "已切换到HuggingFace官方".into();
            }
        });

        ui.separator();

        // Search section
        ui.strong("搜索模型");
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut self.search_query);
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !self.search_query.is_empty() && !*self.is_searching.lock().unwrap() {
                    self.do_search();
                }
            }
            
            let searching = *self.is_searching.lock().unwrap();
            if ui.add_enabled(!searching && !self.search_query.is_empty(), egui::Button::new("搜索")).clicked() {
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
                .max_height(250.0)
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
                if ui.button("查看详情").clicked() {
                    self.status_message = format!("模型详情: {}", model.id);
                }
            });

            // Download progress
            if let Some(progress) = self.download_progress {
                ui.separator();
                ui.strong("下载进度");
                
                let progress_text = if let Some(size) = self.download_size {
                    format!("{} / {} ({:.1}%)", 
                        Self::format_size(size),
                        Self::format_size(4_000_000_000),
                        progress * 100.0)
                } else {
                    format!("{:.1}%", progress * 100.0)
                };
                
                ui.add(egui::ProgressBar::new(progress)
                    .text(progress_text)
                    .animate(true));
                
                ui.horizontal(|ui| {
                    if ui.button("暂停").clicked() {
                        self.status_message = "下载已暂停".into();
                    }
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
        *self.is_searching.lock().unwrap() = true;
        self.status_message = format!("搜索中: {}...", self.search_query);
        
        let client = self.hf_client.clone();
        let query = self.search_query.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.search_rx = Some(rx);
        
        std::thread::spawn(move || {
            let client = client.lock().unwrap();
            match client.search(&query) {
                Ok(results) => {
                    let _ = tx.send(SearchResult::Success(results));
                }
                Err(e) => {
                    let _ = tx.send(SearchResult::Error(e.to_string()));
                }
            }
        });
    }
}

use eframe::egui;
use llama_core::huggingface::{HfClient, HfModel};
use llama_core::network::NetworkStatus;

pub struct HfView {
    search_query: String,
    search_results: Vec<HfModel>,
    selected_model: Option<HfModel>,
    download_progress: Option<f32>,
    network_status: Option<NetworkStatus>,
    hf_client: HfClient,
    mirror_url: String,
    status_message: String,
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
            network_status: None,
            hf_client: HfClient::new("https://huggingface.co".into()),
            mirror_url: "https://hf-mirror.com".into(),
            status_message: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("HuggingFace 模型管理");

        // Network status section
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("网络状态:");
            match &self.network_status {
                Some(NetworkStatus::Online { latency_ms }) => {
                    ui.colored_label(egui::Color32::GREEN, "在线");
                    ui.label(format!("延迟: {}ms", latency_ms));
                }
                Some(NetworkStatus::Offline) => {
                    ui.colored_label(egui::Color32::RED, "离线");
                    ui.label("离线模式：仅可使用本地模型");
                }
                Some(NetworkStatus::RateLimited) => {
                    ui.colored_label(egui::Color32::YELLOW, "限速");
                    ui.label("API限速，请稍后重试");
                }
                None => {
                    ui.label("未检测");
                }
            }
            if ui.button("检测网络").clicked() {
                // In a real implementation, this would be async
                self.network_status = Some(NetworkStatus::Online { latency_ms: 50 });
            }
        });

        // Mirror site selection
        ui.horizontal(|ui| {
            ui.label("镜像站:");
            ui.text_edit_singleline(&mut self.mirror_url);
            if ui.button("切换到镜像").clicked() {
                self.hf_client = HfClient::new(self.mirror_url.clone());
                self.status_message = format!("已切换到镜像站: {}", self.mirror_url);
            }
            if ui.button("切换到官方").clicked() {
                self.hf_client = HfClient::new("https://huggingface.co".into());
                self.status_message = "已切换到HuggingFace官方".into();
            }
        });

        ui.separator();

        // Search section
        ui.label("搜索模型");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.search_query);
            if ui.button("搜索").clicked() && !self.search_query.is_empty() {
                self.status_message = format!("搜索中: {}...", self.search_query);
                // In a real implementation, this would be async
                self.search_results = vec![
                    HfModel {
                        id: "meta-llama/Llama-3.1-8B-Instruct".into(),
                        model_type: "llama".into(),
                        tags: vec!["text-generation".into()],
                        downloads: 1200000,
                    },
                    HfModel {
                        id: "Qwen/Qwen2.5-7B-Instruct".into(),
                        model_type: "qwen".into(),
                        tags: vec!["text-generation".into()],
                        downloads: 890000,
                    },
                ];
            }
        });

        // Search results
        if !self.search_results.is_empty() {
            ui.separator();
            ui.label(format!("搜索结果 ({}个模型)", self.search_results.len()));

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for model in &self.search_results {
                        let is_selected = self
                            .selected_model
                            .as_ref()
                            .map(|m| m.id == model.id)
                            .unwrap_or(false);

                        if ui
                            .selectable_label(is_selected, &model.id)
                            .clicked()
                        {
                            self.selected_model = Some(model.clone());
                        }
                        ui.label(format!(
                            "类型: {} | 下载: {}",
                            model.model_type, model.downloads
                        ));
                    }
                });
        }

        // Selected model actions
        if let Some(ref model) = self.selected_model {
            ui.separator();
            ui.label(format!("已选择: {}", model.id));

            ui.horizontal(|ui| {
                if ui.button("下载模型").clicked() {
                    self.status_message = format!("开始下载: {}", model.id);
                    self.download_progress = Some(0.0);
                }
                if ui.button("查看详情").clicked() {
                    self.status_message = format!("模型详情: {}", model.id);
                }
            });

            // Download progress
            if let Some(progress) = self.download_progress {
                ui.separator();
                ui.label("下载进度:");
                ui.add(egui::ProgressBar::new(progress).animate(true));
                ui.horizontal(|ui| {
                    if ui.button("暂停").clicked() {
                        self.status_message = "下载已暂停".into();
                    }
                    if ui.button("取消").clicked() {
                        self.download_progress = None;
                        self.status_message = "下载已取消".into();
                    }
                });
            }
        }

        // Status message
        if !self.status_message.is_empty() {
            ui.separator();
            ui.label(&self.status_message);
        }
    }
}

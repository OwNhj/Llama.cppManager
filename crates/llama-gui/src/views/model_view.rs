use eframe::egui;
use llama_core::model::{ModelFormat, ModelInfo};
use llama_core::params::ModelParams;

pub struct ModelView {
    pub selected_path: Option<String>,
    model_info: Option<ModelInfo>,
    models: Vec<ModelInfo>,
    params: ModelParams,
    presets: Vec<(String, ModelParams)>,
    preset_name: String,
    status_message: String,
    model_supports_mtp: bool,
}

impl Default for ModelView {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelView {
    pub fn new() -> Self {
        Self {
            selected_path: None,
            model_info: None,
            models: Vec::new(),
            params: ModelParams::default(),
            presets: Vec::new(),
            preset_name: String::new(),
            status_message: String::new(),
            model_supports_mtp: false,
        }
    }

    /// 获取当前选中模型的名称
    pub fn current_model_name(&self) -> Option<String> {
        self.model_info.as_ref().map(|m| m.name.clone())
    }

    /// 获取当前选中模型的路径
    pub fn current_model_path(&self) -> Option<String> {
        self.selected_path.clone()
    }

    /// 获取当前模型的参数
    pub fn current_params(&self) -> &ModelParams {
        &self.params
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("模型管理");

        // Model selection section
        ui.separator();
        ui.label("选择模型");

        ui.horizontal(|ui| {
            if ui.button("浏览本地模型").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("GGUF", &["gguf"])
                    .add_filter("PyTorch", &["bin"])
                    .add_filter("SafeTensors", &["safetensors"])
                    .pick_file()
                {
                    let path_str = path.display().to_string();
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    let format = ModelFormat::detect(&filename);
                    let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                    let info = ModelInfo {
                        path: path.clone(),
                        format: format.clone(),
                        name: filename.to_string(),
                        size_bytes,
                    };

                    self.selected_path = Some(path_str);
                    self.model_info = Some(info.clone());

                    // 检测模型是否支持MTP
                    self.detect_mtp_support(&path);

                    // Check if model already exists in list
                    if !self.models.iter().any(|m| m.path == path) {
                        self.models.push(info);
                    }

                    self.status_message = if format.is_gguf() {
                        let mtp_status = if self.model_supports_mtp {
                            " (支持MTP)"
                        } else {
                            ""
                        };
                        format!("已加载GGUF模型: {}{}", filename, mtp_status)
                    } else {
                        format!(
                            "已选择{}格式模型: {} (需要导出为GGUF)",
                            match format {
                                ModelFormat::PyTorch => "PyTorch",
                                ModelFormat::SafeTensors => "SafeTensors",
                                _ => "未知",
                            },
                            filename
                        )
                    };
                }
            }

            if ui.button("从HuggingFace下载").clicked() {
                self.status_message = "请在HuggingFace标签页下载模型".into();
            }
        });

        // Model list
        if !self.models.is_empty() {
            ui.separator();
            ui.label(format!("已加载模型 ({}个)", self.models.len()));

            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    let mut to_remove = None;
                    for (i, model) in self.models.iter().enumerate() {
                        let is_selected = self
                            .selected_path
                            .as_ref()
                            .map(|p| *p == model.path.display().to_string())
                            .unwrap_or(false);

                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(is_selected, &model.name)
                                .clicked()
                            {
                                self.selected_path = Some(model.path.display().to_string());
                                self.model_info = Some(model.clone());
                            }

                            ui.label(format!(
                                "{} ({:.2} GB)",
                                match model.format {
                                    ModelFormat::Gguf => "GGUF",
                                    ModelFormat::PyTorch => "PyTorch",
                                    ModelFormat::SafeTensors => "SafeTensors",
                                    ModelFormat::Unknown => "未知",
                                },
                                model.size_gb()
                            ));

                            if ui.small_button("移除").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }

                    if let Some(index) = to_remove {
                        self.models.remove(index);
                        if self.models.is_empty() {
                            self.selected_path = None;
                            self.model_info = None;
                        }
                    }
                });
        }

        // Model information display
        if let Some(ref info) = self.model_info {
            ui.separator();
            ui.label("模型信息");

            egui::Grid::new("model_info_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("文件名:");
                    ui.label(&info.name);
                    ui.end_row();

                    ui.label("路径:");
                    ui.label(info.path.display().to_string());
                    ui.end_row();

                    ui.label("格式:");
                    ui.label(match info.format {
                        ModelFormat::Gguf => "GGUF",
                        ModelFormat::PyTorch => "PyTorch",
                        ModelFormat::SafeTensors => "SafeTensors",
                        ModelFormat::Unknown => "未知",
                    });
                    ui.end_row();

                    ui.label("大小:");
                    ui.label(format!("{:.2} GB", info.size_gb()));
                    ui.end_row();

                    ui.label("状态:");
                    ui.label(if info.format.is_gguf() {
                        "可直接加载"
                    } else {
                        "需要导出为GGUF"
                    });
                    ui.end_row();
                });

            // Export prompt for non-GGUF formats
            if !info.format.is_gguf() {
                ui.separator();
                ui.colored_label(egui::Color32::YELLOW, "此模型需要导出为GGUF格式才能使用");
                if ui.button("导出为GGUF").clicked() {
                    self.status_message = "导出功能将在量化工具中实现".into();
                }
            }
        }

        // Parameter presets section
        ui.separator();
        ui.label("参数预设");

        ui.horizontal(|ui| {
            ui.label("预设名称:");
            ui.text_edit_singleline(&mut self.preset_name);
        });

        ui.horizontal(|ui| {
            if ui.button("保存当前预设").clicked() && !self.preset_name.is_empty() {
                self.presets
                    .push((self.preset_name.clone(), self.params.clone()));
                self.status_message = format!("已保存预设: {}", self.preset_name);
                self.preset_name.clear();
            }

            if ui.button("加载预设").clicked() && !self.presets.is_empty() {
                // Show preset selection dialog
                self.status_message = "请选择要加载的预设".into();
            }

            if ui.button("导出预设").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .save_file()
                {
                    if let Ok(json) = serde_json::to_string_pretty(&self.params) {
                        if let Err(e) = std::fs::write(path, json) {
                            self.status_message = format!("导出失败: {}", e);
                        } else {
                            self.status_message = "预设已导出".into();
                        }
                    }
                }
            }

            if ui.button("导入预设").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file()
                {
                    if let Ok(json) = std::fs::read_to_string(path) {
                        match serde_json::from_str::<ModelParams>(&json) {
                            Ok(params) => {
                                self.params = params;
                                self.status_message = "预设已导入".into();
                            }
                            Err(e) => {
                                self.status_message = format!("导入失败: {}", e);
                            }
                        }
                    }
                }
            }
        });

        // Show saved presets
        if !self.presets.is_empty() {
            ui.label("已保存的预设:");
            egui::ScrollArea::vertical()
                .max_height(100.0)
                .show(ui, |ui| {
                    let mut to_load = None;
                    let mut to_remove = None;
                    for (i, (name, _params)) in self.presets.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(false, name).clicked() {
                                to_load = Some(i);
                            }
                            if ui.small_button("删除").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }

                    if let Some(index) = to_load {
                        self.params = self.presets[index].1.clone();
                        self.status_message = format!("已加载预设: {}", self.presets[index].0);
                    }

                    if let Some(index) = to_remove {
                        self.presets.remove(index);
                    }
                });
        }

        // Parameter adjustment section
        ui.separator();
        ui.label("模型参数调整");

        ui.horizontal(|ui| {
            ui.label("Temperature:");
            ui.add(
                egui::Slider::new(&mut self.params.temperature, 0.0..=2.0)
                    .step_by(0.05)
                    .show_value(true),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Top-P:");
            ui.add(
                egui::Slider::new(&mut self.params.top_p, 0.0..=1.0)
                    .step_by(0.05)
                    .show_value(true),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Top-K:");
            ui.add(egui::Slider::new(&mut self.params.top_k, 1..=100).show_value(true));
        });
        ui.horizontal(|ui| {
            ui.label("Repeat Penalty:");
            ui.add(
                egui::Slider::new(&mut self.params.repeat_penalty, 1.0..=2.0)
                    .step_by(0.05)
                    .show_value(true),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Context Size:");
            ui.add(
                egui::Slider::new(&mut self.params.context_size, 128..=131072)
                    .step_by(128.0)
                    .show_value(true),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Batch Size:");
            ui.add(
                egui::Slider::new(&mut self.params.batch_size, 1..=8192).show_value(true),
            );
        });
        ui.horizontal(|ui| {
            ui.label("GPU Offload Layers:");
            ui.add(
                egui::Slider::new(&mut self.params.gpu_offload_layers, 0..=128).show_value(true),
            );
        });
        ui.checkbox(&mut self.params.flash_attention, "Flash Attention");

        // MTP设置（仅当模型支持时可用）
        ui.separator();
        ui.label("MTP (Multi-Token Prediction)");
        if self.model_supports_mtp {
            ui.checkbox(&mut self.params.mtp_enabled, "启用 MTP");
            if self.params.mtp_enabled {
                ui.horizontal(|ui| {
                    ui.label("N Predict:");
                    ui.add(
                        egui::Slider::new(&mut self.params.mtp_n_predict, 1..=8).show_value(true),
                    );
                    ui.label("(每次预测的token数)");
                });
                ui.horizontal(|ui| {
                    ui.label("Vocab Size:");
                    ui.add(
                        egui::Slider::new(&mut self.params.mtp_n_vocab, 1000..=256000)
                            .step_by(1000.0)
                            .show_value(true),
                    );
                    ui.label("(词表大小)");
                });
                ui.horizontal(|ui| {
                    ui.label("Embedding:");
                    ui.add(
                        egui::Slider::new(&mut self.params.mtp_n_embd, 256..=16384)
                            .step_by(256.0)
                            .show_value(true),
                    );
                    ui.label("(嵌入维度)");
                });
            }
        } else {
            ui.horizontal(|ui| {
                ui.add_enabled(false, egui::Checkbox::new(&mut false, "启用 MTP"));
                ui.label("(当前模型不支持MTP)");
            });
        }

        // Status message
        if !self.status_message.is_empty() {
            ui.separator();
            ui.label(&self.status_message);
        }
    }

    /// 检测模型是否支持MTP
    fn detect_mtp_support(&mut self, path: &std::path::Path) {
        // 检查文件名中是否包含mtp关键字
        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
        if filename.contains("mtp") || filename.contains("multi-token") {
            self.model_supports_mtp = true;
            return;
        }

        // 尝试读取GGUF文件头检测MTP支持
        if let Ok(mut file) = std::fs::File::open(path) {
            use std::io::Read;
            let mut header = [0u8; 64];
            if file.read_exact(&mut header).is_ok() {
                // 检查GGUF魔数
                if header[0..4] == [0x47, 0x47, 0x55, 0x46] { // "GGUF"
                    // 检查是否有mtp相关的metadata
                    let content = String::from_utf8_lossy(&header);
                    if content.contains("mtp") || content.contains("multi_token") {
                        self.model_supports_mtp = true;
                        return;
                    }
                }
            }
        }

        // 默认不支持MTP
        self.model_supports_mtp = false;
    }
}

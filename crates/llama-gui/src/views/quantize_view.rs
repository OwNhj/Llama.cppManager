use eframe::egui;
use llama_core::params::ModelParams;
use llama_core::quantize::{LayerConfig, QuantConfig, QuantType};

pub struct QuantizeView {
    params: ModelParams,
    quant_config: QuantConfig,
    selected_layers: Vec<bool>,
    model_loaded: bool,
    status_message: String,
}

impl Default for QuantizeView {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantizeView {
    pub fn new() -> Self {
        Self {
            params: ModelParams::default(),
            quant_config: QuantConfig::default(),
            selected_layers: Vec::new(),
            model_loaded: false,
            status_message: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("可视化量化工具");

        if !self.model_loaded {
            ui.label("请先在'模型管理'页面加载模型");
            return;
        }

        // Global parameters section
        ui.separator();
        ui.label("全局默认参数");

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

        // Quantization type selection
        ui.separator();
        ui.label("量化方式");
        ui.horizontal_wrapped(|ui| {
            for qt in QuantType::all() {
                if ui
                    .selectable_label(self.quant_config.global_quant == *qt, qt.to_string())
                    .clicked()
                {
                    self.quant_config.global_quant = *qt;
                }
            }
        });

        // Batch operations
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("批量操作:");
            if ui.button("全选").clicked() {
                self.selected_layers.iter_mut().for_each(|s| *s = true);
                self.status_message = "已全选所有层".into();
            }
            if ui.button("全不选").clicked() {
                self.selected_layers.iter_mut().for_each(|s| *s = false);
                self.status_message = "已取消全选".into();
            }
            if ui.button("重置为默认").clicked() {
                self.quant_config.layers.clear();
                self.status_message = "已重置为默认量化配置".into();
            }
            if ui.button("全部保持原始精度").clicked() {
                // Set all layers to F16 (original precision)
                for i in 0..self.selected_layers.len() {
                    let tensor_name = format!("layer_{}", i);
                    self.quant_config.layers.push(LayerConfig {
                        tensor: tensor_name,
                        quant_type: QuantType::F16,
                    });
                }
                self.status_message = "已设置所有层为原始精度".into();
            }
        });

        // Import/Export configuration
        ui.horizontal(|ui| {
            if ui.button("导出配置").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .save_file()
                {
                    if let Ok(json) = serde_json::to_string_pretty(&self.quant_config) {
                        if let Err(e) = std::fs::write(path, json) {
                            self.status_message = format!("导出失败: {}", e);
                        } else {
                            self.status_message = "配置已导出".into();
                        }
                    }
                }
            }
            if ui.button("导入配置").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file()
                {
                    if let Ok(json) = std::fs::read_to_string(path) {
                        match serde_json::from_str::<QuantConfig>(&json) {
                            Ok(config) => {
                                self.quant_config = config;
                                self.status_message = "配置已导入".into();
                            }
                            Err(e) => {
                                self.status_message = format!("导入失败: {}", e);
                            }
                        }
                    }
                }
            }
        });

        // Layer and tensor configuration
        ui.separator();
        ui.label("模型层与张量配置");

        // Estimated size display
        ui.horizontal(|ui| {
            ui.label("预估输出大小:");
            let estimated_size = self.estimate_output_size();
            ui.label(format!("{:.2} GB", estimated_size));
            ui.label("|");
            ui.label("质量评分:");
            ui.label(format!("{:.1}/5.0", self.calculate_quality_score()));
        });

        // Layer list with per-layer quantization
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                let placeholder_layers = [
                    (
                        "token_embd",
                        vec!["weight"],
                    ),
                    (
                        "blk.0",
                        vec![
                            "attn_q.weight",
                            "attn_k.weight",
                            "attn_v.weight",
                            "attn_output.weight",
                            "ffn_gate.weight",
                            "ffn_up.weight",
                            "ffn_down.weight",
                        ],
                    ),
                    (
                        "blk.1",
                        vec![
                            "attn_q.weight",
                            "attn_k.weight",
                            "attn_v.weight",
                            "attn_output.weight",
                            "ffn_gate.weight",
                            "ffn_up.weight",
                            "ffn_down.weight",
                        ],
                    ),
                    (
                        "blk.2",
                        vec![
                            "attn_q.weight",
                            "attn_k.weight",
                            "attn_v.weight",
                            "attn_output.weight",
                            "ffn_gate.weight",
                            "ffn_up.weight",
                            "ffn_down.weight",
                        ],
                    ),
                    (
                        "blk.3",
                        vec![
                            "attn_q.weight",
                            "attn_k.weight",
                            "attn_v.weight",
                            "attn_output.weight",
                            "ffn_gate.weight",
                            "ffn_up.weight",
                            "ffn_down.weight",
                        ],
                    ),
                    ("output_norm", vec!["weight"]),
                    ("output", vec!["weight"]),
                ];

                egui::Grid::new("layer_grid")
                    .striped(true)
                    .num_columns(4)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("选择").strong());
                        ui.label(egui::RichText::new("层").strong());
                        ui.label(egui::RichText::new("张量").strong());
                        ui.label(egui::RichText::new("量化方式").strong());
                        ui.end_row();

                        let mut row_index = 0;
                        for (layer_name, tensors) in &placeholder_layers {
                            for (i, tensor) in tensors.iter().enumerate() {
                                // Ensure selected_layers has enough entries
                                if row_index >= self.selected_layers.len() {
                                    self.selected_layers.push(false);
                                }

                                // Checkbox for selection
                                ui.checkbox(&mut self.selected_layers[row_index], "");

                                // Layer name (only show for first tensor)
                                if i == 0 {
                                    ui.label(*layer_name);
                                } else {
                                    ui.label("");
                                }

                                // Tensor name
                                ui.label(format!("{}.{}", layer_name, tensor));

                                // Quantization type selector
                                let tensor_key = format!("{}.{}", layer_name, tensor);
                                let current_quant = self
                                    .quant_config
                                    .layers
                                    .iter()
                                    .find(|l| l.tensor == tensor_key)
                                    .map(|l| l.quant_type)
                                    .unwrap_or(self.quant_config.global_quant);

                                egui::ComboBox::from_id_salt(&tensor_key)
                                    .selected_text(current_quant.to_string())
                                    .show_ui(ui, |ui| {
                                        for qt in QuantType::all() {
                                            if ui
                                                .selectable_label(current_quant == *qt, qt.to_string())
                                                .clicked()
                                            {
                                                // Update or add layer config
                                                if let Some(existing) = self
                                                    .quant_config
                                                    .layers
                                                    .iter_mut()
                                                    .find(|l| l.tensor == tensor_key)
                                                {
                                                    existing.quant_type = *qt;
                                                } else {
                                                    self.quant_config.layers.push(LayerConfig {
                                                        tensor: tensor_key.clone(),
                                                        quant_type: *qt,
                                                    });
                                                }
                                            }
                                        }
                                    });

                                ui.end_row();
                                row_index += 1;
                            }
                        }
                    });
            });

        // Status message
        if !self.status_message.is_empty() {
            ui.separator();
            ui.label(&self.status_message);
        }
    }

    fn estimate_output_size(&self) -> f64 {
        // Simplified estimation based on quantization type
        let base_size = 4.0; // GB for a 7B model
        match self.quant_config.global_quant {
            QuantType::F32 => base_size * 2.0,
            QuantType::F16 | QuantType::BF16 => base_size,
            QuantType::Q8_0 | QuantType::Q8_K => base_size * 0.5,
            QuantType::Q6_K => base_size * 0.375,
            QuantType::Q5_0 | QuantType::Q5_1 | QuantType::Q5_K_S | QuantType::Q5_K_M | QuantType::Q5_K_L => {
                base_size * 0.3125
            }
            QuantType::Q4_0 | QuantType::Q4_1 | QuantType::Q4_K_S | QuantType::Q4_K_M => {
                base_size * 0.25
            }
            QuantType::Q3_K_S | QuantType::Q3_K_M | QuantType::Q3_K_L => base_size * 0.1875,
            QuantType::Q2_K | QuantType::Q2_K_S => base_size * 0.125,
            QuantType::IQ1_S => base_size * 0.0625,
            QuantType::IQ2_XS => base_size * 0.125,
            QuantType::IQ3_XS => base_size * 0.1875,
        }
    }

    fn calculate_quality_score(&self) -> f32 {
        match self.quant_config.global_quant {
            QuantType::F32 => 5.0,
            QuantType::F16 | QuantType::BF16 => 4.8,
            QuantType::Q8_0 | QuantType::Q8_K => 4.7,
            QuantType::Q6_K => 4.5,
            QuantType::Q5_0 | QuantType::Q5_1 | QuantType::Q5_K_S | QuantType::Q5_K_M | QuantType::Q5_K_L => {
                4.4
            }
            QuantType::Q4_0 | QuantType::Q4_1 | QuantType::Q4_K_S | QuantType::Q4_K_M => 4.1,
            QuantType::Q3_K_S | QuantType::Q3_K_M | QuantType::Q3_K_L => 3.6,
            QuantType::Q2_K | QuantType::Q2_K_S => 2.8,
            QuantType::IQ1_S => 2.0,
            QuantType::IQ2_XS => 2.5,
            QuantType::IQ3_XS => 3.2,
        }
    }
}

use eframe::egui;
use llama_core::params::ModelParams;
use llama_core::quantize::{QuantConfig, QuantType};

pub struct QuantizeView {
    params: ModelParams,
    quant_config: QuantConfig,
    _selected_layers: Vec<bool>,
    model_loaded: bool,
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
            _selected_layers: Vec::new(),
            model_loaded: false,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("可视化量化工具");

        if !self.model_loaded {
            ui.label("请先在'模型管理'页面加载模型");
            return;
        }

        ui.separator();
        ui.label("全局默认参数");

        ui.add(
            egui::Slider::new(&mut self.params.temperature, 0.0..=2.0)
                .step_by(0.05)
                .text("Temperature"),
        );
        ui.add(
            egui::Slider::new(&mut self.params.top_p, 0.0..=1.0)
                .step_by(0.05)
                .text("Top-P"),
        );
        ui.add(egui::Slider::new(&mut self.params.top_k, 1..=100).text("Top-K"));
        ui.add(
            egui::Slider::new(&mut self.params.repeat_penalty, 1.0..=2.0)
                .step_by(0.05)
                .text("Repeat Penalty"),
        );
        ui.add(
            egui::Slider::new(&mut self.params.context_size, 128..=131072)
                .step_by(128.0)
                .text("Context Size"),
        );
        ui.add(egui::Slider::new(&mut self.params.batch_size, 1..=8192).text("Batch Size"));
        ui.add(
            egui::Slider::new(&mut self.params.gpu_offload_layers, 0..=128)
                .text("GPU Offload Layers"),
        );
        ui.checkbox(&mut self.params.flash_attention, "Flash Attention");

        ui.separator();
        ui.label("量化方式");
        ui.horizontal(|ui| {
            for qt in QuantType::all() {
                if ui
                    .selectable_label(self.quant_config.global_quant == *qt, qt.to_string())
                    .clicked()
                {
                    self.quant_config.global_quant = *qt;
                }
            }
        });

        ui.separator();
        ui.label("模型层与张量信息");
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                let placeholder_layers = [
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
                    ("output", vec!["weight", "bias"]),
                    ("token_embd", vec!["weight"]),
                    ("pos_embd", vec!["weight"]),
                ];

                egui::Grid::new("layer_grid").striped(true).show(ui, |ui| {
                    ui.label(egui::RichText::new("层").strong());
                    ui.label(egui::RichText::new("张量").strong());
                    ui.end_row();

                    for (layer_name, tensors) in &placeholder_layers {
                        for (i, tensor) in tensors.iter().enumerate() {
                            if i == 0 {
                                ui.label(*layer_name);
                            } else {
                                ui.label("");
                            }
                            ui.label(format!("{}.{}", layer_name, tensor));
                            ui.end_row();
                        }
                    }
                });
            });
    }
}

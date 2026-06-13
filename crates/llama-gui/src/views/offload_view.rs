use eframe::egui;
use llama_core::environment::DeviceType;
use llama_server::offload::{OffloadConfig, OffloadMode};

pub struct OffloadView {
    config: OffloadConfig,
    total_layers: u32,
    model_name: Option<String>,
    af_attention_device: DeviceType,
    af_ffn_device: DeviceType,
}

impl Default for OffloadView {
    fn default() -> Self {
        Self::new()
    }
}

impl OffloadView {
    pub fn new() -> Self {
        Self {
            config: OffloadConfig::default(),
            total_layers: 0,
            model_name: None,
            af_attention_device: DeviceType::Cuda(0),
            af_ffn_device: DeviceType::Cpu,
        }
    }

    /// 设置模型信息
    pub fn set_model_info(&mut self, name: &str, total_layers: u32) {
        self.model_name = Some(name.to_string());
        self.total_layers = total_layers;
        self.config = OffloadConfig::default();
    }

    /// 清除模型信息
    pub fn clear_model_info(&mut self) {
        self.model_name = None;
        self.total_layers = 0;
        self.config = OffloadConfig::default();
    }

    pub fn total_layers(&self) -> u32 {
        self.total_layers
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Offload 配置");

        // 显示当前模型信息
        if let Some(ref name) = self.model_name {
            ui.horizontal(|ui| {
                ui.label("当前模型:");
                ui.strong(name);
                ui.separator();
                ui.label(format!("总层数: {}", self.total_layers));
            });
        } else {
            ui.label("请先在首页选择GGUF格式模型");
        }

        ui.separator();

        // 分离模式选择
        ui.label("分离模式:");
        ui.horizontal(|ui| {
            let modes = [
                (OffloadMode::Normal, "普通模式"),
                (OffloadMode::AfSeparation, "AF分离"),
            ];
            
            for (mode, label) in modes {
                let selected = self.config.mode == mode;
                if ui.selectable_label(selected, label).clicked() {
                    self.config.mode = mode;
                }
            }
            
            // PD分离不可用
            ui.add_enabled(false, egui::Button::new("PD分离 (开发中)"));
        });

        ui.separator();

        // 根据模式显示不同配置
        match self.config.mode {
            OffloadMode::Normal => {
                ui.strong("层配置");
                ui.label("GPU:0 / GPU:1 表示不同的GPU设备（如果有多块GPU）");
                
                ui.horizontal(|ui| {
                    if ui.button("全部 GPU").clicked() {
                        self.set_all_layers(DeviceType::Cuda(0));
                    }
                    if ui.button("全部 CPU").clicked() {
                        self.set_all_layers(DeviceType::Cpu);
                    }
                    if ui.button("自动分配").clicked() {
                        self.auto_assign_layers();
                    }
                });

                ui.separator();

                if self.total_layers > 0 {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for i in 0..self.total_layers {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Layer {}: ", i));
                                    let current_device = self
                                        .config
                                        .layers
                                        .iter()
                                        .find(|l| l.layer_index == i)
                                        .map(|l| l.device.clone())
                                        .unwrap_or(DeviceType::Cpu);

                                    if ui
                                        .selectable_label(current_device == DeviceType::Cpu, "CPU")
                                        .clicked()
                                    {
                                        self.set_layer_device(i, DeviceType::Cpu);
                                    }
                                    if ui
                                        .selectable_label(current_device == DeviceType::Cuda(0), "GPU:0")
                                        .clicked()
                                    {
                                        self.set_layer_device(i, DeviceType::Cuda(0));
                                    }
                                    if ui
                                        .selectable_label(current_device == DeviceType::Cuda(1), "GPU:1")
                                        .clicked()
                                    {
                                        self.set_layer_device(i, DeviceType::Cuda(1));
                                    }
                                });
                            }
                        });
                } else {
                    ui.label("请先加载模型以获取层数信息");
                }
            }
            OffloadMode::AfSeparation => {
                ui.strong("AF分离配置");
                ui.label("将模型的注意力层(Attention)和前馈层(FFN)分开处理");
                ui.label("注意力层通常需要更多显存，FFN层可以放在CPU上");
                
                ui.separator();
                
                // 注意力层设备选择
                ui.horizontal(|ui| {
                    ui.label("注意力层:");
                    let att_cpu = self.af_attention_device == DeviceType::Cpu;
                    let att_gpu = self.af_attention_device == DeviceType::Cuda(0);
                    
                    if ui.selectable_label(att_cpu, "CPU").clicked() {
                        self.af_attention_device = DeviceType::Cpu;
                    }
                    ui.label(if att_cpu { "✓" } else { "" });
                    
                    if ui.selectable_label(att_gpu, "GPU").clicked() {
                        self.af_attention_device = DeviceType::Cuda(0);
                    }
                    ui.label(if att_gpu { "✓" } else { "" });
                });
                
                // FFN层设备选择
                ui.horizontal(|ui| {
                    ui.label("FFN层:");
                    let ffn_cpu = self.af_ffn_device == DeviceType::Cpu;
                    let ffn_gpu = self.af_ffn_device == DeviceType::Cuda(0);
                    
                    if ui.selectable_label(ffn_cpu, "CPU").clicked() {
                        self.af_ffn_device = DeviceType::Cpu;
                    }
                    ui.label(if ffn_cpu { "✓" } else { "" });
                    
                    if ui.selectable_label(ffn_gpu, "GPU").clicked() {
                        self.af_ffn_device = DeviceType::Cuda(0);
                    }
                    ui.label(if ffn_gpu { "✓" } else { "" });
                });
                
                ui.separator();
                ui.label("提示: 注意力层对推理速度影响较大，建议放在GPU上");
                
                if ui.button("应用配置").clicked() {
                    self.apply_af_config();
                }
            }
            OffloadMode::PdSeparation => {
                ui.colored_label(egui::Color32::YELLOW, "⚠ PD分离功能开发中");
                ui.label("此功能暂不可用，将在后续版本中实现。");
            }
            OffloadMode::Custom => {
                ui.strong("逐层 Offload 配置");
                ui.label("GPU:0 / GPU:1 表示不同的GPU设备（如果有多块GPU）");
                
                ui.horizontal(|ui| {
                    if ui.button("全部 GPU").clicked() {
                        self.set_all_layers(DeviceType::Cuda(0));
                    }
                    if ui.button("全部 CPU").clicked() {
                        self.set_all_layers(DeviceType::Cpu);
                    }
                    if ui.button("自动分配").clicked() {
                        self.auto_assign_layers();
                    }
                });

                ui.separator();

                if self.total_layers > 0 {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for i in 0..self.total_layers {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Layer {}: ", i));
                                    let current_device = self
                                        .config
                                        .layers
                                        .iter()
                                        .find(|l| l.layer_index == i)
                                        .map(|l| l.device.clone())
                                        .unwrap_or(DeviceType::Cpu);

                                    if ui
                                        .selectable_label(current_device == DeviceType::Cpu, "CPU")
                                        .clicked()
                                    {
                                        self.set_layer_device(i, DeviceType::Cpu);
                                    }
                                    if ui
                                        .selectable_label(current_device == DeviceType::Cuda(0), "GPU:0")
                                        .clicked()
                                    {
                                        self.set_layer_device(i, DeviceType::Cuda(0));
                                    }
                                    if ui
                                        .selectable_label(current_device == DeviceType::Cuda(1), "GPU:1")
                                        .clicked()
                                    {
                                        self.set_layer_device(i, DeviceType::Cuda(1));
                                    }
                                });
                            }
                        });
                } else {
                    ui.label("请先加载模型以获取层数信息");
                }
            }
            _ => {}
        }

        ui.separator();
        self.show_vram_stats(ui);
    }

    fn apply_af_config(&mut self) {
        self.config.layers.clear();
        
        if self.total_layers == 0 {
            return;
        }
        
        for i in 0..self.total_layers {
            // 偶数层为注意力层，奇数层为FFN层
            let device = if i % 2 == 0 {
                self.af_attention_device.clone()
            } else {
                self.af_ffn_device.clone()
            };
            
            self.config.layers.push(llama_server::offload::LayerOffload {
                layer_index: i,
                device,
                vram_mb: 0,
            });
        }
    }

    fn set_all_layers(&mut self, device: DeviceType) {
        self.config.layers.clear();
        for i in 0..self.total_layers {
            self.config.layers.push(llama_server::offload::LayerOffload {
                layer_index: i,
                device: device.clone(),
                vram_mb: 0,
            });
        }
    }

    fn set_layer_device(&mut self, layer_index: u32, device: DeviceType) {
        self.config.layers.retain(|l| l.layer_index != layer_index);
        self.config.layers.push(llama_server::offload::LayerOffload {
            layer_index,
            device,
            vram_mb: 0,
        });
        self.config.layers.sort_by_key(|l| l.layer_index);
    }

    fn auto_assign_layers(&mut self) {
        self.config.layers.clear();
        let half = self.total_layers / 2;
        for i in 0..self.total_layers {
            let device = if i < half {
                DeviceType::Cuda(0)
            } else {
                DeviceType::Cpu
            };
            self.config.layers.push(llama_server::offload::LayerOffload {
                layer_index: i,
                device,
                vram_mb: 0,
            });
        }
    }

    fn show_vram_stats(&self, ui: &mut egui::Ui) {
        ui.strong("显存占用统计");

        let gpu_layers: Vec<_> = self
            .config
            .layers
            .iter()
            .filter(|l| !matches!(l.device, DeviceType::Cpu))
            .collect();
        let cpu_layers = self.config.layers.len() - gpu_layers.len();

        ui.horizontal(|ui| {
            ui.label(format!("GPU 层数: {}", gpu_layers.len()));
            ui.separator();
            ui.label(format!("CPU 层数: {}", cpu_layers));
            ui.separator();
            ui.label(format!("总层数: {}", self.total_layers));
        });

        let total_vram: u64 = self.config.layers.iter().map(|l| l.vram_mb).sum();
        ui.horizontal(|ui| {
            ui.label(format!("预估显存: {} MB", total_vram));
        });
    }
}

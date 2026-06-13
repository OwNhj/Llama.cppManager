use eframe::egui;
use llama_core::environment::DeviceType;
use llama_server::offload::{CommProtocol, OffloadConfig, OffloadMode};

pub struct OffloadView {
    config: OffloadConfig,
    total_layers: u32,
    model_name: Option<String>,
    edit_total_layers: bool,
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
            total_layers: 32,
            model_name: None,
            edit_total_layers: false,
        }
    }

    /// 设置模型信息（从模型管理页面传递）
    pub fn set_model_info(&mut self, name: &str, total_layers: u32) {
        self.model_name = Some(name.to_string());
        self.total_layers = total_layers;
        // 重置配置
        self.config = OffloadConfig::default();
    }

    /// 清除模型信息
    pub fn clear_model_info(&mut self) {
        self.model_name = None;
        self.total_layers = 32;
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
                if self.edit_total_layers {
                    ui.label("总层数:");
                    ui.add(egui::Slider::new(&mut self.total_layers, 1..=200));
                    if ui.button("确认").clicked() {
                        self.edit_total_layers = false;
                        // 重置配置
                        self.config.layers.clear();
                    }
                } else {
                    ui.label(format!("总层数: {}", self.total_layers));
                    if ui.small_button("编辑").clicked() {
                        self.edit_total_layers = true;
                    }
                }
            });
        } else {
            ui.label("请先在首页选择模型");
        }

        ui.separator();

        ui.label("分离模式:");
        ui.horizontal(|ui| {
            for mode in [
                OffloadMode::Normal,
                OffloadMode::AfSeparation,
                OffloadMode::PdSeparation,
                OffloadMode::Custom,
            ] {
                if ui
                    .selectable_label(self.config.mode == mode, mode.to_string())
                    .clicked()
                {
                    self.config.mode = mode;
                }
            }
        });

        if self.config.mode == OffloadMode::PdSeparation {
            ui.separator();
            ui.label("PD 分离配置");

            ui.horizontal(|ui| {
                ui.label("Prefill 地址:");
                ui.text_edit_singleline(
                    self.config
                        .pd_prefill_addr
                        .get_or_insert_with(|| "127.0.0.1:8080".into()),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Decode 地址:");
                ui.text_edit_singleline(
                    self.config
                        .pd_decode_addr
                        .get_or_insert_with(|| "127.0.0.1:8081".into()),
                );
            });

            ui.horizontal(|ui| {
                ui.label("通信协议:");
                let protocols = [
                    (CommProtocol::SharedMemory, "SharedMemory"),
                    (CommProtocol::Tcp, "Tcp"),
                    (CommProtocol::Rdma, "Rdma"),
                ];
                for (proto, label) in protocols {
                    let selected = self.config.comm_protocol.as_ref() == Some(&proto);
                    if ui.selectable_label(selected, label).clicked() {
                        self.config.comm_protocol = Some(proto);
                    }
                }
            });
        }

        if self.config.mode == OffloadMode::Custom {
            ui.separator();
            ui.label("逐层 Offload 配置");

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
                                .selectable_label(current_device == DeviceType::Cuda(0), "CUDA:0")
                                .clicked()
                            {
                                self.set_layer_device(i, DeviceType::Cuda(0));
                            }
                            if ui
                                .selectable_label(current_device == DeviceType::Cuda(1), "CUDA:1")
                                .clicked()
                            {
                                self.set_layer_device(i, DeviceType::Cuda(1));
                            }
                        });
                    }
                });
        }

        ui.separator();
        self.show_vram_stats(ui);
    }

    fn set_all_layers(&mut self, device: DeviceType) {
        self.config.layers.clear();
        for i in 0..self.total_layers {
            self.config
                .layers
                .push(llama_server::offload::LayerOffload {
                    layer_index: i,
                    device: device.clone(),
                    vram_mb: 0,
                });
        }
    }

    fn set_layer_device(&mut self, layer_index: u32, device: DeviceType) {
        self.config.layers.retain(|l| l.layer_index != layer_index);
        self.config
            .layers
            .push(llama_server::offload::LayerOffload {
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
            self.config
                .layers
                .push(llama_server::offload::LayerOffload {
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

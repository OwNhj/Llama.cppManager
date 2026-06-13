use eframe::egui;
use llama_core::environment::Environment;

pub struct EnvView {
    env: Option<Environment>,
    auto_refresh: bool,
    refresh_interval: u32,
    last_refresh: std::time::Instant,
}

impl Default for EnvView {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvView {
    pub fn new() -> Self {
        Self {
            env: None,
            auto_refresh: false,
            refresh_interval: 30,
            last_refresh: std::time::Instant::now(),
        }
    }

    /// 启动时自动检测环境
    pub fn auto_detect(&mut self) {
        self.env = Some(Environment::detect());
        self.last_refresh = std::time::Instant::now();
    }

    /// 手动刷新环境检测
    pub fn refresh(&mut self) {
        self.env = Some(Environment::detect());
        self.last_refresh = std::time::Instant::now();
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("运行环境检测");

        // 自动刷新控制
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.auto_refresh, "自动刷新");
            if self.auto_refresh {
                ui.label("间隔:");
                ui.add(egui::Slider::new(&mut self.refresh_interval, 5..=60).suffix("秒"));
            }
            if ui.button("立即刷新").clicked() {
                self.env = Some(Environment::detect());
                self.last_refresh = std::time::Instant::now();
            }
        });

        // 自动刷新逻辑
        if self.auto_refresh
            && self.last_refresh.elapsed() >= std::time::Duration::from_secs(self.refresh_interval as u64)
        {
            self.env = Some(Environment::detect());
            self.last_refresh = std::time::Instant::now();
        }

        if let Some(ref env) = self.env {
            // 操作系统信息
            ui.separator();
            ui.collapsing("操作系统信息", |ui| {
                egui::Grid::new("os_grid").striped(true).show(ui, |ui| {
                    ui.label("系统:");
                    ui.label(&env.os.name);
                    ui.end_row();
                    ui.label("版本:");
                    ui.label(&env.os.version);
                    ui.end_row();
                    ui.label("架构:");
                    ui.label(&env.os.architecture);
                    ui.end_row();
                    ui.label("Rust版本:");
                    ui.label(&env.rust_toolchain);
                    ui.end_row();
                });
            });

            // CPU信息
            ui.separator();
            ui.collapsing("CPU 信息", |ui| {
                egui::Grid::new("cpu_grid").striped(true).show(ui, |ui| {
                    ui.label("型号:");
                    ui.label(&env.cpu.model);
                    ui.end_row();
                    ui.label("物理核心:");
                    ui.label(format!("{} 核", env.cpu.cores));
                    ui.end_row();
                    ui.label("逻辑线程:");
                    ui.label(format!("{} 线程", env.cpu.threads));
                    ui.end_row();
                    ui.label("支持指令集:");
                    ui.label(if env.cpu.features.is_empty() {
                        "无".to_string()
                    } else {
                        env.cpu.features.join(", ")
                    });
                    ui.end_row();
                    ui.label("总内存:");
                    ui.label(format!("{} GB", env.cpu.total_memory_mb / 1024));
                    ui.end_row();
                    ui.label("可用内存:");
                    ui.label(format!("{} GB", env.cpu.available_memory_mb / 1024));
                    ui.end_row();
                    ui.label("内存使用率:");
                    let usage = if env.cpu.total_memory_mb > 0 {
                        ((env.cpu.total_memory_mb - env.cpu.available_memory_mb) as f64
                            / env.cpu.total_memory_mb as f64
                            * 100.0) as u32
                    } else {
                        0
                    };
                    ui.label(format!("{}%", usage));
                    ui.end_row();
                });

                // 内存使用进度条
                ui.label("内存使用:");
                let usage = if env.cpu.total_memory_mb > 0 {
                    (env.cpu.total_memory_mb - env.cpu.available_memory_mb) as f32
                        / env.cpu.total_memory_mb as f32
                } else {
                    0.0
                };
                ui.add(egui::ProgressBar::new(usage).text(format!("{}%", (usage * 100.0) as u32)));
            });

            // GPU信息
            ui.separator();
            ui.collapsing("GPU 信息", |ui| {
                if env.gpus.is_empty() {
                    ui.label("未检测到GPU");
                } else {
                    for (i, gpu) in env.gpus.iter().enumerate() {
                        ui.label(format!("GPU {}:", i));
                        egui::Grid::new(format!("gpu_grid_{}", i))
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("型号:");
                                ui.label(&gpu.name);
                                ui.end_row();
                                ui.label("后端:");
                                ui.label(gpu.backend.to_string());
                                ui.end_row();
                                ui.label("驱动版本:");
                                ui.label(&gpu.driver_version);
                                ui.end_row();
                                ui.label("计算能力:");
                                ui.label(&gpu.compute_capability);
                                ui.end_row();
                                ui.label("总显存:");
                                ui.label(format!("{} GB", gpu.vram_mb / 1024));
                                ui.end_row();
                                ui.label("可用显存:");
                                ui.label(format!("{} GB", gpu.available_vram_mb / 1024));
                                ui.end_row();
                                ui.label("显存使用率:");
                                let usage = if gpu.vram_mb > 0 {
                                    ((gpu.vram_mb - gpu.available_vram_mb) as f64 / gpu.vram_mb as f64
                                        * 100.0) as u32
                                } else {
                                    0
                                };
                                ui.label(format!("{}%", usage));
                                ui.end_row();
                            });

                        // 显存使用进度条
                        ui.label("显存使用:");
                        let usage = if gpu.vram_mb > 0 {
                            (gpu.vram_mb - gpu.available_vram_mb) as f32 / gpu.vram_mb as f32
                        } else {
                            0.0
                        };
                        ui.add(
                            egui::ProgressBar::new(usage)
                                .text(format!("{}%", (usage * 100.0) as u32)),
                        );

                        if i < env.gpus.len() - 1 {
                            ui.separator();
                        }
                    }
                }
            });

            // NPU信息
            ui.separator();
            ui.collapsing("NPU 信息", |ui| {
                if let Some(ref npu) = env.npu {
                    egui::Grid::new("npu_grid").striped(true).show(ui, |ui| {
                        ui.label("型号:");
                        ui.label(&npu.name);
                        ui.end_row();
                        ui.label("厂商:");
                        ui.label(&npu.vendor);
                        ui.end_row();
                        ui.label("算力:");
                        ui.label(format!("{} TOPS", npu.tops));
                        ui.end_row();
                    });
                } else {
                    ui.label("未检测到NPU");
                }
            });

            // Offload建议
            ui.separator();
            ui.collapsing("Offload 建议", |ui| {
                let rec = env.recommend_offload(32);
                egui::Grid::new("offload_grid").striped(true).show(ui, |ui| {
                    ui.label("模型总层数:");
                    ui.label(format!("{} 层", rec.total_layers));
                    ui.end_row();
                    ui.label("建议GPU层数:");
                    ui.label(format!("{} 层", rec.gpu_layers));
                    ui.end_row();
                    ui.label("建议原因:");
                    ui.label(&rec.reason);
                    ui.end_row();
                });

                // GPU层数建议进度条
                ui.label("GPU卸载比例:");
                let ratio = rec.gpu_layers as f32 / rec.total_layers as f32;
                ui.add(
                    egui::ProgressBar::new(ratio)
                        .text(format!("{}/{} 层", rec.gpu_layers, rec.total_layers)),
                );
            });
        } else {
            ui.separator();
            ui.label("点击\"立即刷新\"按钮检测系统环境");
        }
    }
}

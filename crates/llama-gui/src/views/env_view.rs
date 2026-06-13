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
                self.refresh();
            }
        });

        // 自动刷新逻辑
        if self.auto_refresh
            && self.last_refresh.elapsed()
                >= std::time::Duration::from_secs(self.refresh_interval as u64)
        {
            self.refresh();
        }

        if let Some(ref env) = self.env {
            // 运行环境信息（CUDA/ROCm/Vulkan等）
            ui.separator();
            ui.collapsing("运行环境", |ui| {
                self.show_runtime_env(ui, env);
            });

            // 操作系统信息
            ui.separator();
            ui.collapsing("操作系统", |ui| {
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
                });
            });

            // CPU信息
            ui.separator();
            ui.collapsing("CPU", |ui| {
                egui::Grid::new("cpu_grid").striped(true).show(ui, |ui| {
                    ui.label("型号:");
                    ui.label(&env.cpu.model);
                    ui.end_row();
                    ui.label("核心/线程:");
                    ui.label(format!("{} 核 / {} 线程", env.cpu.cores, env.cpu.threads));
                    ui.end_row();
                    ui.label("指令集:");
                    ui.label(if env.cpu.features.is_empty() {
                        "无".to_string()
                    } else {
                        env.cpu.features.join(", ")
                    });
                    ui.end_row();
                    ui.label("内存:");
                    ui.label(format!(
                        "{} GB / {} GB (使用 {}%)",
                        env.cpu.available_memory_mb / 1024,
                        env.cpu.total_memory_mb / 1024,
                        if env.cpu.total_memory_mb > 0 {
                            (env.cpu.total_memory_mb - env.cpu.available_memory_mb) * 100
                                / env.cpu.total_memory_mb
                        } else {
                            0
                        }
                    ));
                    ui.end_row();
                });
            });

            // GPU信息
            ui.separator();
            ui.collapsing("GPU", |ui| {
                if env.gpus.is_empty() {
                    ui.label("未检测到GPU");
                } else {
                    for (i, gpu) in env.gpus.iter().enumerate() {
                        egui::Grid::new(format!("gpu_grid_{}", i))
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("型号:");
                                ui.label(&gpu.name);
                                ui.end_row();
                                ui.label("后端:");
                                ui.label(gpu.backend.to_string());
                                ui.end_row();
                                ui.label("显存:");
                                ui.label(format!(
                                    "{} GB / {} GB",
                                    gpu.available_vram_mb / 1024,
                                    gpu.vram_mb / 1024
                                ));
                                ui.end_row();
                                ui.label("驱动:");
                                ui.label(&gpu.driver_version);
                                ui.end_row();
                                ui.label("计算能力:");
                                ui.label(&gpu.compute_capability);
                                ui.end_row();
                            });
                        if i < env.gpus.len() - 1 {
                            ui.separator();
                        }
                    }
                }
            });

            // Offload建议
            ui.separator();
            ui.collapsing("Offload 建议", |ui| {
                let rec = env.recommend_offload(32);
                ui.label(&rec.reason);
                ui.label(format!("建议GPU层数: {}/{}", rec.gpu_layers, rec.total_layers));
            });
        } else {
            ui.separator();
            ui.label("点击\"立即刷新\"检测系统环境");
        }
    }

    fn show_runtime_env(&self, ui: &mut egui::Ui, env: &Environment) {
        let runtime = &env.runtime;

        // CUDA
        ui.horizontal(|ui| {
            ui.strong("CUDA:");
            if let Some(ref cuda) = runtime.cuda {
                ui.colored_label(egui::Color32::GREEN, "已安装");
                ui.label(format!("版本: {}", cuda.version));
                if let Some(ref cudnn) = cuda.cudnn_version {
                    ui.label(format!("cuDNN: {}", cudnn));
                }
            } else {
                ui.colored_label(egui::Color32::GRAY, "未安装");
            }
        });

        // ROCm
        ui.horizontal(|ui| {
            ui.strong("ROCm:");
            if let Some(ref rocm) = runtime.rocm {
                ui.colored_label(egui::Color32::GREEN, "已安装");
                ui.label(format!("版本: {}", rocm.version));
                if let Some(ref hipcc) = rocm.hipcc_version {
                    ui.label(format!("hipcc: {}", hipcc));
                }
            } else {
                ui.colored_label(egui::Color32::GRAY, "未安装");
            }
        });

        // Vulkan
        ui.horizontal(|ui| {
            ui.strong("Vulkan:");
            if let Some(ref vulkan) = runtime.vulkan {
                ui.colored_label(egui::Color32::GREEN, "已安装");
                ui.label(format!("版本: {}", vulkan.version));
            } else {
                ui.colored_label(egui::Color32::GRAY, "未安装");
            }
        });

        // Metal (macOS)
        ui.horizontal(|ui| {
            ui.strong("Metal:");
            if let Some(ref metal) = runtime.metal {
                ui.colored_label(egui::Color32::GREEN, "已支持");
                ui.label(&metal.version);
            } else {
                ui.colored_label(egui::Color32::GRAY, "不支持/未检测");
            }
        });

        // OneAPI
        ui.horizontal(|ui| {
            ui.strong("OneAPI:");
            if let Some(ref oneapi) = runtime.oneapi {
                ui.colored_label(egui::Color32::GREEN, "已安装");
                if let Some(ref dpcpp) = oneapi.dpcpp_version {
                    ui.label(format!("dpcpp: {}", dpcpp));
                }
            } else {
                ui.colored_label(egui::Color32::GRAY, "未安装");
            }
        });
    }
}

use eframe::egui;
use llama_core::environment::Environment;

pub struct EnvView {
    env: Option<Environment>,
}

impl Default for EnvView {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvView {
    pub fn new() -> Self {
        Self { env: None }
    }

    /// 启动时自动检测环境
    pub fn auto_detect(&mut self) {
        self.env = Some(Environment::detect());
    }

    /// 手动刷新环境检测
    pub fn refresh(&mut self) {
        self.env = Some(Environment::detect());
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("运行环境检测");

        if ui.button("刷新检测").clicked() {
            self.refresh();
        }

        if let Some(ref env) = self.env {
            // 运行环境信息（CUDA/ROCm/Vulkan等）
            ui.separator();
            ui.strong("运行环境");
            self.show_runtime_env(ui, env);

            // 操作系统信息
            ui.separator();
            ui.strong("操作系统");
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

            // CPU信息
            ui.separator();
            ui.strong("CPU");
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
                    "{} GB / {} GB",
                    env.cpu.available_memory_mb / 1024,
                    env.cpu.total_memory_mb / 1024
                ));
                ui.end_row();
            });

            // GPU信息
            ui.separator();
            ui.strong("GPU");
            if env.gpus.is_empty() {
                ui.label("未检测到GPU");
            } else {
                // 去重：按名称去重
                let mut seen_names = std::collections::HashSet::new();
                let unique_gpus: Vec<_> = env.gpus.iter().filter(|gpu| {
                    seen_names.insert(gpu.name.clone())
                }).collect();
                
                for gpu in unique_gpus {
                    ui.horizontal(|ui| {
                        ui.label("●");
                        ui.strong(&gpu.name);
                        ui.separator();
                        ui.label(gpu.backend.to_string());
                        ui.separator();
                        ui.label(format!("{} GB", gpu.vram_mb / 1024));
                    });
                }
            }

            // llama.cpp 信息
            ui.separator();
            ui.strong("llama.cpp");
            let llama = &env.llama_cpp;
            if llama.installed {
                ui.colored_label(egui::Color32::GREEN, "● 已安装");
                if let Some(ref v) = llama.version {
                    ui.label(format!("版本: {}", v));
                }
                if let Some(ref p) = llama.server_path {
                    ui.label(format!("server: {}", p));
                }
                if let Some(ref p) = llama.cli_path {
                    ui.label(format!("cli: {}", p));
                }
                if let Some(ref p) = llama.quantize_path {
                    ui.label(format!("quantize: {}", p));
                }
            } else {
                ui.colored_label(egui::Color32::YELLOW, "● 未检测到");
                ui.label("请安装 llama.cpp 或将其添加到 PATH");
            }
        } else {
            ui.separator();
            ui.label("点击\"刷新检测\"按钮检测系统环境");
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

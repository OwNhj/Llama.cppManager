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

    pub fn auto_detect(&mut self) {
        self.env = Some(Environment::detect());
    }

    pub fn refresh(&mut self) {
        self.env = Some(Environment::detect());
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("运行环境检测");

        if ui.button("刷新检测").clicked() {
            self.refresh();
        }

        if let Some(ref env) = self.env {
            // 运行环境信息（基于llama.cpp）
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
                    ui.label(format!("路径: {}", p));
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
        let llama = &env.llama_cpp;
        
        if llama.installed {
            ui.label("当前后端:");
            let mut backends = Vec::new();
            
            // 检测GPU后端
            if !env.gpus.is_empty() {
                let gpu = &env.gpus[0];
                match gpu.backend {
                    llama_core::environment::GpuBackend::Cuda => {
                        backends.push("CUDA".to_string());
                        if let Some(ref cuda) = env.runtime.cuda {
                            backends.push(format!("v{}", cuda.version));
                        }
                    }
                    llama_core::environment::GpuBackend::Rocm => {
                        backends.push("ROCm".to_string());
                        if let Some(ref rocm) = env.runtime.rocm {
                            backends.push(format!("v{}", rocm.version));
                        }
                    }
                    llama_core::environment::GpuBackend::Vulkan => {
                        backends.push("Vulkan".to_string());
                        if let Some(ref vulkan) = env.runtime.vulkan {
                            backends.push(format!("v{}", vulkan.version));
                        }
                    }
                    llama_core::environment::GpuBackend::Metal => {
                        backends.push("Metal".to_string());
                    }
                    _ => {}
                }
            }
            
            // 检测CPU指令集
            if env.cpu.features.contains(&"AVX2".to_string()) {
                backends.push("AVX2".to_string());
            }
            if env.cpu.features.contains(&"AVX-512".to_string()) {
                backends.push("AVX-512".to_string());
            }
            
            if backends.is_empty() {
                ui.label("CPU (无加速)");
            } else {
                ui.label(backends.join(" + "));
            }
        } else {
            ui.colored_label(egui::Color32::YELLOW, "● 未加载 llama.cpp");
            ui.label("请在 llama.cpp 标签页安装");
        }
    }
}

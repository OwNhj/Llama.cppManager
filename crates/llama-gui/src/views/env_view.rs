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

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("运行环境检测");

        if ui.button("刷新检测").clicked() {
            self.env = Some(Environment::detect());
        }

        if let Some(ref env) = self.env {
            ui.separator();
            ui.label("CPU 信息");
            ui.label(format!("型号: {}", env.cpu.model));
            ui.label(format!("核心数: {}", env.cpu.cores));
            ui.label(format!("线程数: {}", env.cpu.threads));
            ui.label(format!("指令集: {:?}", env.cpu.features));
            ui.label(format!(
                "内存: {} MB / {} MB",
                env.cpu.available_memory_mb, env.cpu.total_memory_mb
            ));

            if !env.gpus.is_empty() {
                ui.separator();
                ui.label("GPU 信息");
                for gpu in &env.gpus {
                    ui.label(format!(
                        "{} ({:?}): {} MB / {} MB",
                        gpu.name, gpu.backend, gpu.available_vram_mb, gpu.vram_mb
                    ));
                }
            }

            if env.has_npu {
                ui.separator();
                ui.label("NPU: 检测到神经处理单元");
            }

            ui.separator();
            ui.label("Offload 建议");
            let rec = env.recommend_offload(32);
            ui.label(format!("总层数: {}", rec.total_layers));
            ui.label(format!("建议 GPU 层数: {}", rec.gpu_layers));
            ui.label(format!("原因: {}", rec.reason));
        }
    }
}

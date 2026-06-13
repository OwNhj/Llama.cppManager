use eframe::egui;
use llama_core::environment::Environment;

pub struct LlamaCppView {
    installed: bool,
    version: Option<String>,
    download_progress: Option<f32>,
    is_downloading: bool,
    is_compiling: bool,
    compile_options: CompileOptions,
    status_message: String,
    install_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub backend: String,
    pub enable_avx2: bool,
    pub enable_avx512: bool,
    pub enable_vulkan: bool,
    pub enable_cuda: bool,
    pub enable_rocm: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            backend: "CPU".into(),
            enable_avx2: true,
            enable_avx512: false,
            enable_vulkan: false,
            enable_cuda: false,
            enable_rocm: false,
        }
    }
}

impl Default for LlamaCppView {
    fn default() -> Self {
        Self::new()
    }
}

impl LlamaCppView {
    pub fn new() -> Self {
        Self {
            installed: false,
            version: None,
            download_progress: None,
            is_downloading: false,
            is_compiling: false,
            compile_options: CompileOptions::default(),
            status_message: String::new(),
            install_path: None,
        }
    }

    /// 从环境检测中更新状态
    pub fn update_from_env(&mut self, env: &Environment) {
        self.installed = env.llama_cpp.installed;
        self.version = env.llama_cpp.version.clone();
        self.install_path = env.llama_cpp.server_path.clone();
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("llama.cpp 管理");

        // 安装状态
        ui.separator();
        ui.strong("安装状态");
        if self.installed {
            ui.colored_label(egui::Color32::GREEN, "● 已安装");
            if let Some(ref v) = self.version {
                ui.label(format!("版本: {}", v));
            }
            if let Some(ref p) = self.install_path {
                ui.label(format!("路径: {}", p));
            }
        } else {
            ui.colored_label(egui::Color32::YELLOW, "● 未检测到 llama.cpp");
            ui.label("请下载并编译 llama.cpp");
        }

        ui.separator();

        // 下载和编译选项
        ui.strong("安装 llama.cpp");
        
        if self.is_downloading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("下载中...");
            });
            if let Some(progress) = self.download_progress {
                ui.add(egui::ProgressBar::new(progress).text(format!("{:.0}%", progress * 100.0)));
            }
        } else if self.is_compiling {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("编译中...");
            });
        } else {
            // 编译选项
            ui.label("选择编译选项:");
            
            egui::Grid::new("compile_options").striped(true).show(ui, |ui| {
                ui.label("后端:");
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.compile_options.backend == "CPU", "CPU").clicked() {
                        self.compile_options.backend = "CPU".into();
                        self.compile_options.enable_cuda = false;
                        self.compile_options.enable_rocm = false;
                        self.compile_options.enable_vulkan = false;
                    }
                    if ui.selectable_label(self.compile_options.backend == "CUDA", "CUDA").clicked() {
                        self.compile_options.backend = "CUDA".into();
                        self.compile_options.enable_cuda = true;
                        self.compile_options.enable_rocm = false;
                        self.compile_options.enable_vulkan = false;
                    }
                    if ui.selectable_label(self.compile_options.backend == "ROCm", "ROCm").clicked() {
                        self.compile_options.backend = "ROCm".into();
                        self.compile_options.enable_cuda = false;
                        self.compile_options.enable_rocm = true;
                        self.compile_options.enable_vulkan = false;
                    }
                    if ui.selectable_label(self.compile_options.backend == "Vulkan", "Vulkan").clicked() {
                        self.compile_options.backend = "Vulkan".into();
                        self.compile_options.enable_cuda = false;
                        self.compile_options.enable_rocm = false;
                        self.compile_options.enable_vulkan = true;
                    }
                });
                ui.end_row();
                
                ui.label("CPU优化:");
                ui.checkbox(&mut self.compile_options.enable_avx2, "AVX2");
                ui.checkbox(&mut self.compile_options.enable_avx512, "AVX-512");
                ui.end_row();
            });

            ui.horizontal(|ui| {
                if ui.button("下载并编译").clicked() {
                    self.start_install();
                }
                if ui.button("仅下载源码").clicked() {
                    self.start_download_only();
                }
            });
        }

        // 状态消息
        if !self.status_message.is_empty() {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                if ui.small_button("清除").clicked() {
                    self.status_message.clear();
                }
            });
        }
    }

    fn start_download_only(&mut self) {
        self.is_downloading = true;
        self.download_progress = Some(0.0);
        self.status_message = "开始下载 llama.cpp 源码...".into();
        
        // 在后台线程下载
        std::thread::spawn(|| {
            // 模拟下载过程
            std::thread::sleep(std::time::Duration::from_secs(2));
        });
    }

    fn start_install(&mut self) {
        self.is_downloading = true;
        self.download_progress = Some(0.0);
        self.status_message = "开始下载并编译 llama.cpp...".into();
        
        let options = self.compile_options.clone();
        std::thread::spawn(move || {
            // 模拟下载和编译过程
            std::thread::sleep(std::time::Duration::from_secs(2));
            
            // 根据选项执行不同的编译命令
            let cmake_args = Self::build_cmake_args(&options);
            let _ = cmake_args; // 使用编译参数
        });
    }

    fn build_cmake_args(options: &CompileOptions) -> Vec<String> {
        let mut args = vec!["-B".into(), "build".into()];
        
        // 后端选项
        if options.enable_cuda {
            args.push("-DGGML_CUDA=ON".into());
        }
        if options.enable_rocm {
            args.push("-DGGML_HIP=ON".into());
        }
        if options.enable_vulkan {
            args.push("-DGGML_VULKAN=ON".into());
        }
        
        // CPU优化
        if options.enable_avx2 {
            args.push("-DGGML_AVX2=ON".into());
        }
        if options.enable_avx512 {
            args.push("-DGGML_AVX512=ON".into());
        }
        
        args
    }
}

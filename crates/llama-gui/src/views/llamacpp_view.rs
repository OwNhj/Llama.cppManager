use eframe::egui;
use llama_core::environment::Environment;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

pub struct LlamaCppView {
    installed: bool,
    version: Option<String>,
    install_path: Option<String>,
    
    is_downloading: Arc<AtomicBool>,
    is_compiling: Arc<AtomicBool>,
    download_progress: f32,
    compile_progress: f32,
    status_message: String,
    log_output: Arc<Mutex<Vec<String>>>,
    
    backend: Backend,
    cpu_optimization: CpuOptimization,
    
    // 路径配置
    source_path: String,
    build_path: String,
    
    download_rx: Option<std::sync::mpsc::Receiver<InstallResult>>,
    compile_rx: Option<std::sync::mpsc::Receiver<InstallResult>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Backend {
    Cpu,
    Cuda,
    Rocm,
    Vulkan,
    Metal,
    Openblas,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Backend::Cpu => write!(f, "CPU"),
            Backend::Cuda => write!(f, "CUDA"),
            Backend::Rocm => write!(f, "ROCm"),
            Backend::Vulkan => write!(f, "Vulkan"),
            Backend::Metal => write!(f, "Metal"),
            Backend::Openblas => write!(f, "OpenBLAS"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CpuOptimization {
    None,
    Avx2,
    Avx512,
}

impl std::fmt::Display for CpuOptimization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuOptimization::None => write!(f, "无优化"),
            CpuOptimization::Avx2 => write!(f, "AVX2"),
            CpuOptimization::Avx512 => write!(f, "AVX-512"),
        }
    }
}

enum InstallResult {
    Log(String),
    Progress(f32),
    Complete(String),
    Error(String),
}

impl Default for LlamaCppView {
    fn default() -> Self {
        Self::new()
    }
}

impl LlamaCppView {
    pub fn new() -> Self {
        let default_path = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".into());
        
        Self {
            installed: false,
            version: None,
            install_path: None,
            is_downloading: Arc::new(AtomicBool::new(false)),
            is_compiling: Arc::new(AtomicBool::new(false)),
            download_progress: 0.0,
            compile_progress: 0.0,
            status_message: String::new(),
            log_output: Arc::new(Mutex::new(Vec::new())),
            backend: Backend::Cpu,
            cpu_optimization: CpuOptimization::Avx2,
            source_path: std::path::Path::new(&default_path).join("llama.cpp").display().to_string(),
            build_path: std::path::Path::new(&default_path).join("llama.cpp-build").display().to_string(),
            download_rx: None,
            compile_rx: None,
        }
    }

    pub fn update_from_env(&mut self, env: &Environment) {
        self.installed = env.llama_cpp.installed;
        self.version = env.llama_cpp.version.clone();
        self.install_path = env.llama_cpp.server_path.clone();
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("llama.cpp 管理");

        self.check_results();

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
        }

        ui.separator();

        let is_busy = self.is_downloading.load(Ordering::SeqCst) || 
                      self.is_compiling.load(Ordering::SeqCst);

        // 路径配置
        ui.strong("路径配置");
        ui.horizontal(|ui| {
            ui.label("源码路径:");
            ui.add_enabled(!is_busy, egui::TextEdit::singleline(&mut self.source_path));
            if ui.add_enabled(!is_busy, egui::Button::new("浏览")).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("选择 llama.cpp 源码目录")
                    .pick_folder()
                {
                    self.source_path = path.display().to_string();
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("编译输出:");
            ui.add_enabled(!is_busy, egui::TextEdit::singleline(&mut self.build_path));
            if ui.add_enabled(!is_busy, egui::Button::new("浏览")).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("选择编译输出目录")
                    .pick_folder()
                {
                    self.build_path = path.display().to_string();
                }
            }
        });

        ui.separator();

        // 编译选项
        ui.strong("编译选项");
        
        ui.horizontal(|ui| {
            ui.label("计算后端:");
            for backend in [Backend::Cpu, Backend::Cuda, Backend::Rocm, Backend::Vulkan, Backend::Metal, Backend::Openblas] {
                if ui.selectable_label(self.backend == backend, backend.to_string()).clicked() && !is_busy {
                    self.backend = backend;
                }
            }
        });

        if self.backend == Backend::Cpu {
            ui.horizontal(|ui| {
                ui.label("CPU优化:");
                for opt in [CpuOptimization::None, CpuOptimization::Avx2, CpuOptimization::Avx512] {
                    if ui.selectable_label(self.cpu_optimization == opt, opt.to_string()).clicked() && !is_busy {
                        self.cpu_optimization = opt;
                    }
                }
            });
            ui.small("AVX-512 包含 AVX2，选择 AVX-512 时自动启用 AVX2");
        }

        ui.separator();

        // 操作按钮
        if is_busy {
            ui.horizontal(|ui| {
                ui.spinner();
                if self.is_downloading.load(Ordering::SeqCst) {
                    ui.label("下载中...");
                } else if self.is_compiling.load(Ordering::SeqCst) {
                    ui.label("编译中...");
                }
            });
            
            if self.is_downloading.load(Ordering::SeqCst) {
                ui.add(egui::ProgressBar::new(self.download_progress)
                    .text(format!("下载: {:.0}%", self.download_progress * 100.0)));
            }
            if self.is_compiling.load(Ordering::SeqCst) {
                ui.add(egui::ProgressBar::new(self.compile_progress)
                    .text(format!("编译: {:.0}%", self.compile_progress * 100.0)));
            }
        } else {
            ui.horizontal(|ui| {
                if ui.button("下载并编译").clicked() {
                    self.start_full_install();
                }
                if ui.button("仅下载源码").clicked() {
                    self.start_download_only();
                }
                if ui.button("仅编译").clicked() {
                    self.start_compile_only();
                }
            });
        }

        // 日志输出
        ui.separator();
        ui.strong("输出日志");
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let logs = self.log_output.lock().unwrap();
                for line in logs.iter() {
                    ui.label(egui::RichText::new(line).small().monospace());
                }
            });

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

    fn check_results(&mut self) {
        if let Some(rx) = self.download_rx.take() {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        InstallResult::Log(msg) => {
                            self.log_output.lock().unwrap().push(msg);
                        }
                        InstallResult::Progress(p) => {
                            self.download_progress = p;
                        }
                        InstallResult::Complete(msg) => {
                            self.log_output.lock().unwrap().push(msg);
                            self.is_downloading.store(false, Ordering::SeqCst);
                            self.status_message = "下载完成".into();
                        }
                        InstallResult::Error(e) => {
                            self.log_output.lock().unwrap().push(format!("错误: {}", e));
                            self.is_downloading.store(false, Ordering::SeqCst);
                            self.status_message = format!("下载失败: {}", e);
                        }
                    }
                    self.download_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.download_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_downloading.store(false, Ordering::SeqCst);
                }
            }
        }

        if let Some(rx) = self.compile_rx.take() {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        InstallResult::Log(msg) => {
                            self.log_output.lock().unwrap().push(msg);
                        }
                        InstallResult::Progress(p) => {
                            self.compile_progress = p;
                        }
                        InstallResult::Complete(msg) => {
                            self.log_output.lock().unwrap().push(msg);
                            self.is_compiling.store(false, Ordering::SeqCst);
                            self.installed = true;
                            self.status_message = "编译完成".into();
                        }
                        InstallResult::Error(e) => {
                            self.log_output.lock().unwrap().push(format!("错误: {}", e));
                            self.is_compiling.store(false, Ordering::SeqCst);
                            self.status_message = format!("编译失败: {}", e);
                        }
                    }
                    self.compile_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.compile_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_compiling.store(false, Ordering::SeqCst);
                }
            }
        }
    }

    fn start_download_only(&mut self) {
        self.is_downloading.store(true, Ordering::SeqCst);
        self.download_progress = 0.0;
        self.log_output.lock().unwrap().clear();
        self.status_message = "开始下载 llama.cpp...".into();
        
        let dest_path = self.source_path.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.download_rx = Some(rx);
        
        std::thread::spawn(move || {
            let _ = tx.send(InstallResult::Log(format!("开始下载 llama.cpp 到: {}", dest_path)));
            let _ = tx.send(InstallResult::Progress(0.1));
            
            // 检查目标目录是否已存在
            if std::path::Path::new(&dest_path).exists() {
                let _ = tx.send(InstallResult::Log("目标目录已存在，跳过下载".into()));
                let _ = tx.send(InstallResult::Progress(1.0));
                let _ = tx.send(InstallResult::Complete(format!("目录已存在: {}", dest_path)));
                return;
            }
            
            // 创建父目录
            if let Some(parent) = std::path::Path::new(&dest_path).parent() {
                if !parent.exists() {
                    let _ = tx.send(InstallResult::Log(format!("创建目录: {}", parent.display())));
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            
            let _ = tx.send(InstallResult::Progress(0.3));
            let _ = tx.send(InstallResult::Log("正在下载...".into()));
            
            // 尝试使用 git，如果失败则使用 PowerShell 下载 zip
            let git_available = std::process::Command::new("git").arg("--version").output().is_ok();
            
            if git_available {
                let output = std::process::Command::new("git")
                    .args(["clone", "--depth=1", "https://github.com/ggerganov/llama.cpp.git", &dest_path])
                    .output();
                
                match output {
                    Ok(out) => {
                        if out.status.success() {
                            let _ = tx.send(InstallResult::Progress(1.0));
                            let _ = tx.send(InstallResult::Complete(format!("下载完成: {}", dest_path)));
                        } else {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            let _ = tx.send(InstallResult::Error(format!("git clone 失败: {}", stderr)));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(InstallResult::Error(format!("git执行失败: {}", e)));
                    }
                }
            } else {
                // 使用 PowerShell 下载 zip
                let _ = tx.send(InstallResult::Log("git 不可用，使用 PowerShell 下载...".into()));
                let _ = tx.send(InstallResult::Progress(0.5));
                
                let zip_url = "https://github.com/ggerganov/llama.cpp/archive/refs/heads/master.zip";
                let zip_path = format!("{}.zip", dest_path);
                
                let output = std::process::Command::new("powershell")
                    .args(["-Command", &format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", zip_url, zip_path)])
                    .output();
                
                match output {
                    Ok(out) => {
                        if out.status.success() {
                            let _ = tx.send(InstallResult::Log("下载完成，正在解压...".into()));
                            let _ = tx.send(InstallResult::Progress(0.8));
                            
                            // 解压
                            let extract_output = std::process::Command::new("powershell")
                                .args(["-Command", &format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", zip_path, dest_path)])
                                .output();
                            
                            match extract_output {
                                Ok(out) => {
                                    if out.status.success() {
                                        // 移动解压后的目录内容
                                        let extracted_dir = format!("{}\\llama.cpp-master", dest_path);
                                        if std::path::Path::new(&extracted_dir).exists() {
                                            let _ = std::fs::rename(&extracted_dir, &dest_path);
                                        }
                                        let _ = tx.send(InstallResult::Progress(1.0));
                                        let _ = tx.send(InstallResult::Complete(format!("下载完成: {}", dest_path)));
                                    } else {
                                        let stderr = String::from_utf8_lossy(&out.stderr);
                                        let _ = tx.send(InstallResult::Error(format!("解压失败: {}", stderr)));
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(InstallResult::Error(format!("解压执行失败: {}", e)));
                                }
                            }
                        } else {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            let _ = tx.send(InstallResult::Error(format!("下载失败: {}", stderr)));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(InstallResult::Error(format!("PowerShell执行失败: {}", e)));
                    }
                }
            }
        });
    }

    fn start_compile_only(&mut self) {
        self.is_compiling.store(true, Ordering::SeqCst);
        self.compile_progress = 0.0;
        self.log_output.lock().unwrap().clear();
        self.status_message = "开始编译 llama.cpp...".into();
        
        let backend = self.backend.clone();
        let cpu_opt = self.cpu_optimization.clone();
        let source_path = self.source_path.clone();
        let build_path = self.build_path.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.compile_rx = Some(rx);
        
        std::thread::spawn(move || {
            Self::compile_llamacpp(&backend, &cpu_opt, &source_path, &build_path, &tx);
        });
    }

    fn start_full_install(&mut self) {
        self.is_downloading.store(true, Ordering::SeqCst);
        self.download_progress = 0.0;
        self.log_output.lock().unwrap().clear();
        self.status_message = "开始下载并编译 llama.cpp...".into();
        
        let backend = self.backend.clone();
        let cpu_opt = self.cpu_optimization.clone();
        let source_path = self.source_path.clone();
        let build_path = self.build_path.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.compile_rx = Some(rx);
        
        std::thread::spawn(move || {
            let _ = tx.send(InstallResult::Log(format!("开始下载 llama.cpp 到: {}", source_path)));
            let _ = tx.send(InstallResult::Progress(0.1));
            
            // 检查源码目录是否已存在
            if std::path::Path::new(&source_path).exists() {
                let _ = tx.send(InstallResult::Log("源码目录已存在，跳过下载".into()));
                let _ = tx.send(InstallResult::Progress(0.5));
                let _ = tx.send(InstallResult::Log("开始编译...".into()));
                Self::compile_llamacpp(&backend, &cpu_opt, &source_path, &build_path, &tx);
                return;
            }
            
            // 创建父目录
            if let Some(parent) = std::path::Path::new(&source_path).parent() {
                if !parent.exists() {
                    let _ = tx.send(InstallResult::Log(format!("创建目录: {}", parent.display())));
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            
            let _ = tx.send(InstallResult::Progress(0.2));
            let _ = tx.send(InstallResult::Log("正在下载...".into()));
            
            // 尝试使用 git，如果失败则使用 PowerShell 下载 zip
            let git_available = std::process::Command::new("git").arg("--version").output().is_ok();
            
            if git_available {
                let output = std::process::Command::new("git")
                    .args(["clone", "--depth=1", "https://github.com/ggerganov/llama.cpp.git", &source_path])
                    .output();
                
                match output {
                    Ok(out) => {
                        if out.status.success() {
                            let _ = tx.send(InstallResult::Progress(0.5));
                            let _ = tx.send(InstallResult::Log("下载完成，开始编译...".into()));
                            Self::compile_llamacpp(&backend, &cpu_opt, &source_path, &build_path, &tx);
                        } else {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            let _ = tx.send(InstallResult::Error(format!("git clone 失败: {}", stderr)));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(InstallResult::Error(format!("git执行失败: {}", e)));
                    }
                }
            } else {
                // 使用 PowerShell 下载 zip
                let _ = tx.send(InstallResult::Log("git 不可用，使用 PowerShell 下载...".into()));
                let _ = tx.send(InstallResult::Progress(0.3));
                
                let zip_url = "https://github.com/ggerganov/llama.cpp/archive/refs/heads/master.zip";
                let zip_path = format!("{}.zip", source_path);
                
                let output = std::process::Command::new("powershell")
                    .args(["-Command", &format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", zip_url, zip_path)])
                    .output();
                
                match output {
                    Ok(out) => {
                        if out.status.success() {
                            let _ = tx.send(InstallResult::Log("下载完成，正在解压...".into()));
                            let _ = tx.send(InstallResult::Progress(0.6));
                            
                            let extract_output = std::process::Command::new("powershell")
                                .args(["-Command", &format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", zip_path, source_path)])
                                .output();
                            
                            match extract_output {
                                Ok(out) => {
                                    if out.status.success() {
                                        let extracted_dir = format!("{}\\llama.cpp-master", source_path);
                                        if std::path::Path::new(&extracted_dir).exists() {
                                            let _ = std::fs::rename(&extracted_dir, &source_path);
                                        }
                                        let _ = tx.send(InstallResult::Progress(0.5));
                                        let _ = tx.send(InstallResult::Log("下载完成，开始编译...".into()));
                                        Self::compile_llamacpp(&backend, &cpu_opt, &source_path, &build_path, &tx);
                                    } else {
                                        let stderr = String::from_utf8_lossy(&out.stderr);
                                        let _ = tx.send(InstallResult::Error(format!("解压失败: {}", stderr)));
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(InstallResult::Error(format!("解压执行失败: {}", e)));
                                }
                            }
                        } else {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            let _ = tx.send(InstallResult::Error(format!("下载失败: {}", stderr)));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(InstallResult::Error(format!("PowerShell执行失败: {}", e)));
                    }
                }
            }
        });
    }

    fn compile_llamacpp(backend: &Backend, cpu_opt: &CpuOptimization, source_path: &str, build_path: &str, tx: &std::sync::mpsc::Sender<InstallResult>) {
        let _ = tx.send(InstallResult::Progress(0.6));
        let _ = tx.send(InstallResult::Log(format!("源码路径: {}", source_path)));
        let _ = tx.send(InstallResult::Log(format!("输出路径: {}", build_path)));
        let _ = tx.send(InstallResult::Log("配置编译选项...".into()));
        
        let mut cmake_args: Vec<String> = vec![
            "-B".into(), build_path.to_string(), 
            "-S".into(), source_path.to_string(),
        ];
        
        match backend {
            Backend::Cpu => {
                match cpu_opt {
                    CpuOptimization::Avx2 => {
                        cmake_args.push("-DGGML_AVX2=ON".into());
                        let _ = tx.send(InstallResult::Log("启用 AVX2 优化".into()));
                    }
                    CpuOptimization::Avx512 => {
                        cmake_args.push("-DGGML_AVX512=ON".into());
                        let _ = tx.send(InstallResult::Log("启用 AVX-512 优化".into()));
                    }
                    CpuOptimization::None => {}
                }
            }
            Backend::Cuda => {
                cmake_args.push("-DGGML_CUDA=ON".into());
                let _ = tx.send(InstallResult::Log("启用 CUDA 后端".into()));
            }
            Backend::Rocm => {
                cmake_args.push("-DGGML_HIP=ON".into());
                let _ = tx.send(InstallResult::Log("启用 ROCm 后端".into()));
            }
            Backend::Vulkan => {
                cmake_args.push("-DGGML_VULKAN=ON".into());
                let _ = tx.send(InstallResult::Log("启用 Vulkan 后端".into()));
            }
            Backend::Metal => {
                cmake_args.push("-DGGML_METAL=ON".into());
                let _ = tx.send(InstallResult::Log("启用 Metal 后端".into()));
            }
            Backend::Openblas => {
                cmake_args.push("-DGGML_BLAS=ON".into());
                cmake_args.push("-DGGML_BLAS_VENDOR=OpenBLAS".into());
                let _ = tx.send(InstallResult::Log("启用 OpenBLAS 后端".into()));
            }
        }
        
        let _ = tx.send(InstallResult::Progress(0.7));
        let _ = tx.send(InstallResult::Log(format!("执行: cmake {}", cmake_args.join(" "))));
        
        let output = std::process::Command::new("cmake")
            .args(&cmake_args)
            .output();
        
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                
                for line in stdout.lines() {
                    let _ = tx.send(InstallResult::Log(line.to_string()));
                }
                for line in stderr.lines() {
                    let _ = tx.send(InstallResult::Log(format!("[stderr] {}", line)));
                }
                
                if !out.status.success() {
                    let _ = tx.send(InstallResult::Error(format!("cmake 配置失败: {}", stderr)));
                    return;
                }
            }
            Err(e) => {
                let _ = tx.send(InstallResult::Error(format!("执行cmake失败: {}", e)));
                return;
            }
        }
        
        let _ = tx.send(InstallResult::Progress(0.8));
        let _ = tx.send(InstallResult::Log("开始编译...".into()));
        
        let output = std::process::Command::new("cmake")
            .args(["--build", "build", "--config", "Release"])
            .output();
        
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                
                for line in stdout.lines() {
                    let _ = tx.send(InstallResult::Log(line.to_string()));
                }
                for line in stderr.lines() {
                    let _ = tx.send(InstallResult::Log(format!("[stderr] {}", line)));
                }
                
                if out.status.success() {
                    let _ = tx.send(InstallResult::Progress(1.0));
                    let _ = tx.send(InstallResult::Complete("编译完成!".into()));
                } else {
                    let _ = tx.send(InstallResult::Error(format!("编译失败: {}", stderr)));
                }
            }
            Err(e) => {
                let _ = tx.send(InstallResult::Error(format!("执行编译失败: {}", e)));
            }
        }
    }
}

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
    download_speed: f64,
    download_size: u64,
    status_message: String,
    log_output: Arc<Mutex<Vec<String>>>,
    
    backend: Backend,
    cpu_optimization: CpuOptimization,
    
    source_path: String,
    build_path: String,
    
    rx: Option<std::sync::mpsc::Receiver<InstallResult>>,
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
    DownloadProgress { downloaded: u64, speed: f64 },
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
            download_speed: 0.0,
            download_size: 0,
            status_message: String::new(),
            log_output: Arc::new(Mutex::new(Vec::new())),
            backend: Backend::Cpu,
            cpu_optimization: CpuOptimization::Avx2,
            source_path: std::path::Path::new(&default_path).join("llama.cpp").display().to_string(),
            build_path: std::path::Path::new(&default_path).join("llama.cpp-build").display().to_string(),
            rx: None,
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

        // CPU加速选项
        ui.horizontal(|ui| {
            ui.label("CPU加速:");
            for opt in [CpuOptimization::None, CpuOptimization::Avx2, CpuOptimization::Avx512] {
                if ui.selectable_label(self.cpu_optimization == opt, opt.to_string()).clicked() && !is_busy {
                    self.cpu_optimization = opt;
                }
            }
        });
        ui.small("当有层在CPU上运行时，AVX-512 可提供更好的性能");

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
                let speed_text = if self.download_speed > 0.0 {
                    if self.download_speed >= 1024.0 * 1024.0 {
                        format!(" ({:.1} MB/s)", self.download_speed / 1024.0 / 1024.0)
                    } else if self.download_speed >= 1024.0 {
                        format!(" ({:.1} KB/s)", self.download_speed / 1024.0)
                    } else {
                        format!(" ({:.0} B/s)", self.download_speed)
                    }
                } else {
                    String::new()
                };
                ui.add(egui::ProgressBar::new(self.download_progress)
                    .text(format!("下载: {:.0}%{}", self.download_progress * 100.0, speed_text)));
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
        if let Some(rx) = self.rx.take() {
            while let Ok(result) = rx.try_recv() {
                match result {
                    InstallResult::Log(msg) => {
                        self.log_output.lock().unwrap().push(msg);
                    }
                    InstallResult::Progress(p) => {
                        if self.is_downloading.load(Ordering::SeqCst) {
                            self.download_progress = p;
                        } else {
                            self.compile_progress = p;
                        }
                    }
                    InstallResult::DownloadProgress { downloaded, speed } => {
                        self.download_size = downloaded;
                        self.download_speed = speed;
                    }
                    InstallResult::Complete(msg) => {
                        self.log_output.lock().unwrap().push(msg);
                        if self.is_downloading.load(Ordering::SeqCst) {
                            self.is_downloading.store(false, Ordering::SeqCst);
                            self.status_message = "下载完成".into();
                        } else {
                            self.is_compiling.store(false, Ordering::SeqCst);
                            self.installed = true;
                            self.status_message = "编译完成".into();
                        }
                    }
                    InstallResult::Error(e) => {
                        self.log_output.lock().unwrap().push(format!("错误: {}", e));
                        self.is_downloading.store(false, Ordering::SeqCst);
                        self.is_compiling.store(false, Ordering::SeqCst);
                        self.status_message = format!("失败: {}", e);
                    }
                }
            }
            self.rx = Some(rx);
        }
    }

    fn start_download_only(&mut self) {
        self.is_downloading.store(true, Ordering::SeqCst);
        self.download_progress = 0.0;
        self.download_speed = 0.0;
        self.download_size = 0;
        self.log_output.lock().unwrap().clear();
        self.status_message = "开始下载 llama.cpp...".into();
        
        let dest_path = self.source_path.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.rx = Some(rx);
        
        std::thread::spawn(move || {
            Self::do_download(&dest_path, &tx);
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
        self.rx = Some(rx);
        
        std::thread::spawn(move || {
            Self::compile_llamacpp(&backend, &cpu_opt, &source_path, &build_path, &tx);
        });
    }

    fn start_full_install(&mut self) {
        self.is_downloading.store(true, Ordering::SeqCst);
        self.download_progress = 0.0;
        self.download_speed = 0.0;
        self.download_size = 0;
        self.log_output.lock().unwrap().clear();
        self.status_message = "开始下载并编译 llama.cpp...".into();
        
        let backend = self.backend.clone();
        let cpu_opt = self.cpu_optimization.clone();
        let source_path = self.source_path.clone();
        let build_path = self.build_path.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.rx = Some(rx);
        
        std::thread::spawn(move || {
            // 检查是否已存在完整源码
            let cmake_file = std::path::Path::new(&source_path).join("CMakeLists.txt");
            if cmake_file.exists() {
                let _ = tx.send(InstallResult::Log("源码已存在，跳过下载".into()));
                let _ = tx.send(InstallResult::Progress(1.0));
            } else {
                Self::do_download(&source_path, &tx);
            }
            
            // 检查下载是否成功
            let cmake_file = std::path::Path::new(&source_path).join("CMakeLists.txt");
            if cmake_file.exists() {
                Self::compile_llamacpp(&backend, &cpu_opt, &source_path, &build_path, &tx);
            } else {
                let _ = tx.send(InstallResult::Error("下载失败，无法开始编译".into()));
            }
        });
    }

    fn do_download(dest_path: &str, tx: &std::sync::mpsc::Sender<InstallResult>) {
        let _ = tx.send(InstallResult::Log(format!("下载目标: {}", dest_path)));
        
        // 检查是否已存在
        let cmake_file = std::path::Path::new(dest_path).join("CMakeLists.txt");
        if cmake_file.exists() {
            let _ = tx.send(InstallResult::Log("源码已存在".into()));
            let _ = tx.send(InstallResult::Progress(1.0));
            return;
        }
        
        // 创建父目录
        if let Some(parent) = std::path::Path::new(dest_path).parent() {
            if !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        
        // 尝试使用 git
        let git_available = std::process::Command::new("git")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok();
        
        if git_available {
            let _ = tx.send(InstallResult::Log("使用 git clone 下载...".into()));
            let _ = tx.send(InstallResult::Progress(0.1));
            
            // 先删除可能存在的不完整目录
            let _ = std::fs::remove_dir_all(dest_path);
            
            let output = std::process::Command::new("git")
                .args(["clone", "--depth=1", "--progress", "https://github.com/ggerganov/llama.cpp.git", dest_path])
                .output();
            
            match output {
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    // git clone 的进度信息在 stderr
                    for line in stderr.lines() {
                        if line.contains("Receiving objects") || line.contains("Resolving deltas") {
                            let _ = tx.send(InstallResult::Log(line.to_string()));
                        }
                    }
                    
                    if out.status.success() {
                        let _ = tx.send(InstallResult::Progress(1.0));
                        let _ = tx.send(InstallResult::Complete(format!("下载完成: {}", dest_path)));
                    } else {
                        let _ = tx.send(InstallResult::Error(format!("git clone 失败: {}", stderr)));
                    }
                }
                Err(e) => {
                    let _ = tx.send(InstallResult::Error(format!("git 执行失败: {}", e)));
                }
            }
        } else {
            // 使用 reqwest 下载 zip
            let _ = tx.send(InstallResult::Log("git 不可用，使用 HTTP 下载...".into()));
            let _ = tx.send(InstallResult::Progress(0.1));
            
            let zip_url = "https://github.com/ggerganov/llama.cpp/archive/refs/heads/master.zip";
            let zip_path = format!("{}.zip", dest_path);
            
            // 使用阻塞的 reqwest 客户端
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(600))
                .build();
            
            match client {
                Ok(client) => {
                    let _ = tx.send(InstallResult::Log(format!("正在下载: {}", zip_url)));
                    
                    match client.get(zip_url).send() {
                        Ok(resp) => {
                            if !resp.status().is_success() {
                                let _ = tx.send(InstallResult::Error(format!("HTTP {}", resp.status())));
                                return;
                            }
                            
                            let total_size = resp.content_length().unwrap_or(0);
                            let _ = tx.send(InstallResult::Log(format!("文件大小: {:.1} MB", total_size as f64 / 1024.0 / 1024.0)));
                            
                            // 流式下载
                            let mut file = match std::fs::File::create(&zip_path) {
                                Ok(f) => f,
                                Err(e) => {
                                    let _ = tx.send(InstallResult::Error(format!("创建文件失败: {}", e)));
                                    return;
                                }
                            };
                            
                            let mut downloaded: u64 = 0;
                            let start_time = std::time::Instant::now();
                            let mut last_update = std::time::Instant::now();
                            
                            let mut stream = resp;
                            let mut buffer = [0u8; 8192];
                            
                            loop {
                                use std::io::{Read, Write};
                                match stream.read(&mut buffer) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        if let Err(e) = file.write_all(&buffer[..n]) {
                                            let _ = tx.send(InstallResult::Error(format!("写入失败: {}", e)));
                                            return;
                                        }
                                        downloaded += n as u64;
                                        
                                        // 每100ms更新一次进度
                                        if last_update.elapsed() >= std::time::Duration::from_millis(100) {
                                            let progress = if total_size > 0 {
                                                downloaded as f32 / total_size as f32
                                            } else {
                                                0.5
                                            };
                                            let speed = downloaded as f64 / start_time.elapsed().as_secs_f64();
                                            
                                            let _ = tx.send(InstallResult::Progress(0.1 + progress * 0.6));
                                            let _ = tx.send(InstallResult::DownloadProgress { downloaded, speed });
                                            
                                            last_update = std::time::Instant::now();
                                        }
                                    }
                                    Err(e) => {
                                        let _ = tx.send(InstallResult::Error(format!("下载失败: {}", e)));
                                        return;
                                    }
                                }
                            }
                            
                            let _ = tx.send(InstallResult::Log("下载完成，正在解压...".into()));
                            let _ = tx.send(InstallResult::Progress(0.8));
                            
                            // 解压
                            let _ = std::fs::create_dir_all(dest_path);
                            let extract_result = std::process::Command::new("tar")
                                .args(["-xf", &zip_path, "-C", dest_path, "--strip-components=1"])
                                .status();
                            
                            match extract_result {
                                Ok(status) => {
                                    if status.success() {
                                        let _ = std::fs::remove_file(&zip_path);
                                        let _ = tx.send(InstallResult::Progress(1.0));
                                        let _ = tx.send(InstallResult::Complete(format!("下载完成: {}", dest_path)));
                                    } else {
                                        let _ = tx.send(InstallResult::Error("解压失败".into()));
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(InstallResult::Error(format!("解压执行失败: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(InstallResult::Error(format!("请求失败: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(InstallResult::Error(format!("创建HTTP客户端失败: {}", e)));
                }
            }
        }
    }

    fn compile_llamacpp(backend: &Backend, cpu_opt: &CpuOptimization, source_path: &str, build_path: &str, tx: &std::sync::mpsc::Sender<InstallResult>) {
        let _ = tx.send(InstallResult::Progress(0.0));
        let _ = tx.send(InstallResult::Log(format!("源码路径: {}", source_path)));
        let _ = tx.send(InstallResult::Log(format!("输出路径: {}", build_path)));
        
        // 检查源码目录
        if !std::path::Path::new(source_path).exists() {
            let _ = tx.send(InstallResult::Error(format!("源码目录不存在: {}", source_path)));
            return;
        }
        
        let cmake_file = std::path::Path::new(source_path).join("CMakeLists.txt");
        if !cmake_file.exists() {
            let _ = tx.send(InstallResult::Error("找不到 CMakeLists.txt".into()));
            return;
        }
        
        // 检查 cmake
        if std::process::Command::new("cmake").arg("--version").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().is_err() {
            let _ = tx.send(InstallResult::Error("cmake 未安装".into()));
            return;
        }
        
        let _ = tx.send(InstallResult::Log("配置 CMake...".into()));
        let _ = tx.send(InstallResult::Progress(0.1));
        
        let mut cmake_args: Vec<String> = vec![
            "-B".into(), build_path.into(),
            "-S".into(), source_path.into(),
            "-DCMAKE_BUILD_TYPE=Release".into(),
        ];
        
        match backend {
            Backend::Cpu => {
                match cpu_opt {
                    CpuOptimization::Avx2 => cmake_args.push("-DGGML_AVX2=ON".into()),
                    CpuOptimization::Avx512 => cmake_args.push("-DGGML_AVX512=ON".into()),
                    CpuOptimization::None => {}
                }
            }
            Backend::Cuda => cmake_args.push("-DGGML_CUDA=ON".into()),
            Backend::Rocm => cmake_args.push("-DGGML_HIP=ON".into()),
            Backend::Vulkan => cmake_args.push("-DGGML_VULKAN=ON".into()),
            Backend::Metal => cmake_args.push("-DGGML_METAL=ON".into()),
            Backend::Openblas => {
                cmake_args.push("-DGGML_BLAS=ON".into());
                cmake_args.push("-DGGML_BLAS_VENDOR=OpenBLAS".into());
            }
        }
        
        let _ = tx.send(InstallResult::Log(format!("cmake {}", cmake_args.join(" "))));
        
        let output = std::process::Command::new("cmake")
            .args(&cmake_args)
            .output();
        
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stdout.lines().take(20) {
                    let _ = tx.send(InstallResult::Log(line.to_string()));
                }
                for line in stderr.lines().take(10) {
                    let _ = tx.send(InstallResult::Log(format!("[stderr] {}", line)));
                }
                if !out.status.success() {
                    let _ = tx.send(InstallResult::Error(format!("CMake 配置失败")));
                    return;
                }
            }
            Err(e) => {
                let _ = tx.send(InstallResult::Error(format!("cmake 执行失败: {}", e)));
                return;
            }
        }
        
        let _ = tx.send(InstallResult::Log("开始编译...".into()));
        let _ = tx.send(InstallResult::Progress(0.3));
        
        let output = std::process::Command::new("cmake")
            .args(["--build", build_path, "--config", "Release", "-j", "8"])
            .output();
        
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stdout.lines().take(30) {
                    let _ = tx.send(InstallResult::Log(line.to_string()));
                }
                for line in stderr.lines().take(10) {
                    let _ = tx.send(InstallResult::Log(format!("[stderr] {}", line)));
                }
                if out.status.success() {
                    let _ = tx.send(InstallResult::Progress(1.0));
                    let _ = tx.send(InstallResult::Complete("编译完成!".into()));
                } else {
                    let _ = tx.send(InstallResult::Error("编译失败".into()));
                }
            }
            Err(e) => {
                let _ = tx.send(InstallResult::Error(format!("编译执行失败: {}", e)));
            }
        }
    }
}

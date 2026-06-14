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
    
    // 代理设置
    use_proxy: bool,
    proxy_url: String,
    
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
            source_path: default_path.clone(),
            build_path: std::path::Path::new(&default_path).join("build").display().to_string(),
            use_proxy: false,
            proxy_url: String::new(),
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

        // 代理设置
        ui.strong("代理设置");
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.use_proxy, "使用代理");
            if self.use_proxy {
                ui.label("代理地址:");
                ui.text_edit_singleline(&mut self.proxy_url);
            }
        });
        if self.use_proxy {
            ui.small("例如: http://127.0.0.1:7890 或 socks5://127.0.0.1:1080");
        }

        ui.separator();

        // 选择源码目录
        ui.strong("选择源码目录");
        ui.horizontal(|ui| {
            if ui.add_enabled(!is_busy, egui::Button::new("选择源码目录")).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("选择 llama.cpp 源码目录")
                    .pick_folder()
                {
                    // 检查是否包含CMakeLists.txt
                    let cmake_file = path.join("CMakeLists.txt");
                    if cmake_file.exists() {
                        self.source_path = path.display().to_string();
                        // 自动设置编译输出路径
                        self.build_path = path.join("build").display().to_string();
                        self.status_message = format!("已选择源码目录: {}", self.source_path);
                        self.log_output.lock().unwrap().push(format!("源码目录: {}", self.source_path));
                    } else {
                        self.status_message = "选择的目录不包含CMakeLists.txt".into();
                    }
                }
            }
            if !self.source_path.is_empty() {
                ui.label(format!("当前: {}", self.source_path));
            }
        });

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
                    .text(format!("下载: {:.0}{}", self.download_progress * 100.0, speed_text)));
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
            match rx.try_recv() {
                Ok(result) => {
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
                        InstallResult::Complete(msg) => {
                            self.log_output.lock().unwrap().push(msg);
                            self.is_downloading.store(false, Ordering::SeqCst);
                            self.is_compiling.store(false, Ordering::SeqCst);
                            self.installed = true;
                            self.status_message = "完成".into();
                        }
                        InstallResult::Error(e) => {
                            self.log_output.lock().unwrap().push(format!("错误: {}", e));
                            self.is_downloading.store(false, Ordering::SeqCst);
                            self.is_compiling.store(false, Ordering::SeqCst);
                            self.status_message = format!("失败: {}", e);
                        }
                    }
                    self.rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_downloading.store(false, Ordering::SeqCst);
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
        let use_proxy = self.use_proxy;
        let proxy_url = self.proxy_url.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.rx = Some(rx);
        
        std::thread::spawn(move || {
            Self::do_download(&dest_path, &tx, use_proxy, &proxy_url);
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
        self.log_output.lock().unwrap().clear();
        self.status_message = "开始下载并编译 llama.cpp...".into();
        
        let backend = self.backend.clone();
        let cpu_opt = self.cpu_optimization.clone();
        let source_path = self.source_path.clone();
        let build_path = self.build_path.clone();
        let use_proxy = self.use_proxy;
        let proxy_url = self.proxy_url.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.rx = Some(rx);
        
        std::thread::spawn(move || {
            // 下载
            Self::do_download(&source_path, &tx, use_proxy, &proxy_url);
            
            // 检查下载是否成功
            let cmake_file = std::path::Path::new(&source_path).join("CMakeLists.txt");
            if cmake_file.exists() {
                let _ = tx.send(InstallResult::Log("下载完成，开始编译...".into()));
                let _ = tx.send(InstallResult::Progress(0.5));
                Self::compile_llamacpp(&backend, &cpu_opt, &source_path, &build_path, &tx);
            } else {
                let _ = tx.send(InstallResult::Error("下载失败，无法开始编译".into()));
            }
        });
    }

    fn do_download(dest_path: &str, tx: &std::sync::mpsc::Sender<InstallResult>, use_proxy: bool, proxy_url: &str) {
        let _ = tx.send(InstallResult::Log(format!("下载目标: {}", dest_path)));
        if use_proxy && !proxy_url.is_empty() {
            let _ = tx.send(InstallResult::Log(format!("使用代理: {}", proxy_url)));
        }
        let _ = tx.send(InstallResult::Progress(0.1));
        
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
                let _ = tx.send(InstallResult::Log(format!("创建目录: {}", parent.display())));
                let _ = std::fs::create_dir_all(parent);
            }
        }
        
        let _ = tx.send(InstallResult::Progress(0.2));
        let _ = tx.send(InstallResult::Log("正在下载...".into()));
        
        // 检测网络是否可用
        let _ = tx.send(InstallResult::Log("检测网络连接...".into()));
        let mut curl_args = vec!["-L", "--connect-timeout", "10", "--max-time", "15"];
        if use_proxy && !proxy_url.is_empty() {
            curl_args.push("--proxy");
            curl_args.push(proxy_url);
        }
        curl_args.push("https://www.baidu.com");
        curl_args.push("-o");
        curl_args.push(if cfg!(target_os = "windows") { "NUL" } else { "/dev/null" });
        
        let curl_cmd = if cfg!(target_os = "windows") { "curl.exe" } else { "curl" };
        let network_ok = std::process::Command::new(curl_cmd)
            .args(&curl_args)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        
        if !network_ok {
            let _ = tx.send(InstallResult::Error("网络连接失败，请检查网络设置或代理配置".into()));
            return;
        }
        
        let _ = tx.send(InstallResult::Log("网络连接正常".into()));
        
        // 多个下载源
        let download_urls = vec![
            "https://github.com/ggerganov/llama.cpp/archive/refs/heads/master.zip",
            "https://gitcode.com/ggerganov/llama.cpp/-/archive/master/llama.cpp-master.zip",
        ];
        
        let zip_path = format!("{}.zip", dest_path);
        let mut download_success = false;
        
        for url in &download_urls {
            let _ = tx.send(InstallResult::Log(format!("尝试下载: {}", url)));
            
            let mut curl_args = vec![
                "-L",
                "-o", &zip_path,
                "--connect-timeout", "30",
                "--max-time", "600",
            ];
            if use_proxy && !proxy_url.is_empty() {
                curl_args.push("--proxy");
                curl_args.push(proxy_url);
            }
            curl_args.push(url);
            
            let output = std::process::Command::new(curl_cmd)
                .args(&curl_args)
                .output();
            
            match output {
                Ok(out) => {
                    if out.status.success() && std::path::Path::new(&zip_path).exists() {
                        let file_size = std::fs::metadata(&zip_path).map(|m| m.len()).unwrap_or(0);
                        if file_size > 1024 * 1024 {
                            let _ = tx.send(InstallResult::Log(format!("下载成功，文件大小: {:.1} MB", file_size as f64 / 1024.0 / 1024.0)));
                            download_success = true;
                            break;
                        } else {
                            let _ = tx.send(InstallResult::Log("文件太小，尝试下一个源...".into()));
                            let _ = std::fs::remove_file(&zip_path);
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        let _ = tx.send(InstallResult::Log(format!("下载失败: {}", stderr)));
                    }
                }
                Err(e) => {
                    let _ = tx.send(InstallResult::Log(format!("curl执行失败: {}", e)));
                }
            }
        }
        
        if !download_success {
            let _ = tx.send(InstallResult::Log("尝试使用 git clone...".into()));
            let git_available = std::process::Command::new("git").arg("--version").output().is_ok();
            
            if !git_available {
                let _ = tx.send(InstallResult::Error("所有下载方式均失败。请检查网络连接或手动下载源码。".into()));
                return;
            }
            
            let output = std::process::Command::new("git")
                .args(["clone", "--depth=1", "https://github.com/ggerganov/llama.cpp.git", dest_path])
                .output();
            
            match output {
                Ok(out) => {
                    if !out.status.success() {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        let _ = tx.send(InstallResult::Error(format!("所有下载方式均失败。git clone 失败: {}", stderr)));
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx.send(InstallResult::Error(format!("git执行失败: {}", e)));
                    return;
                }
            }
        }
        
        // 解压zip文件
        if std::path::Path::new(&zip_path).exists() {
            let _ = tx.send(InstallResult::Log("正在解压...".into()));
            let _ = tx.send(InstallResult::Progress(0.4));
            
            let parent_path = std::path::Path::new(dest_path).parent().unwrap_or(std::path::Path::new("."));
            let extract_output = std::process::Command::new("tar")
                .args(["-xf", &zip_path, "-C", &parent_path.to_str().unwrap_or(".")])
                .output();
            
            match extract_output {
                Ok(out) => {
                    if out.status.success() {
                        let _ = std::fs::remove_file(&zip_path);
                        let _ = tx.send(InstallResult::Log("解压完成".into()));
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        let _ = tx.send(InstallResult::Error(format!("解压失败: {}", stderr)));
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx.send(InstallResult::Error(format!("解压执行失败: {}", e)));
                    return;
                }
            }
        }
        
        // 验证下载是否成功
        let cmake_file = std::path::Path::new(dest_path).join("CMakeLists.txt");
        if !cmake_file.exists() {
            let _ = tx.send(InstallResult::Error("下载完成但找不到CMakeLists.txt".into()));
            return;
        }
        
        let _ = tx.send(InstallResult::Progress(1.0));
        let _ = tx.send(InstallResult::Complete(format!("下载完成: {}", dest_path)));
    }

    /// 动态检测ROCm安装路径
    fn detect_rocm_path() -> Option<String> {
        // 常见的ROCm安装路径
        let common_paths = [
            "C:\\Program Files\\AMD\\ROCM",
            "C:\\Program Files\\ROCM",
            "C:\\ROCM",
            "/opt/rocm",
            "/usr/local/rocm",
        ];
        
        // 检查环境变量
        if let Ok(path) = std::env::var("ROCM_PATH") {
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
        
        // 检查常见路径
        for base_path in &common_paths {
            let base = std::path::Path::new(base_path);
            if base.exists() {
                // 查找最新的ROCm版本
                if let Ok(entries) = std::fs::read_dir(base) {
                    let mut versions: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .filter_map(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            if name.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                                Some(name)
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    versions.sort_by(|a, b| b.cmp(a)); // 降序排序
                    
                    for version in &versions {
                        let rocm_path = base.join(version);
                        if rocm_path.exists() {
                            return Some(rocm_path.display().to_string());
                        }
                    }
                    
                    // 如果没有版本子目录，直接使用基础路径
                    return Some(base_path.to_string());
                }
            }
        }
        
        None
    }

    /// 从ROCm路径中提取版本号
    fn detect_rocm_version(rocm_path: &str) -> String {
        let path = std::path::Path::new(rocm_path);
        
        // 尝试从路径中提取版本号
        if let Some(version) = path.file_name().and_then(|n| n.to_str()) {
            // 检查是否是版本号格式
            if version.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                return version.to_string();
            }
        }
        
        // 尝试从版本文件读取
        let version_files = ["version.txt", "VERSION", "version"];
        for file in version_files {
            let version_path = path.join(file);
            if let Ok(content) = std::fs::read_to_string(&version_path) {
                if let Some(version) = content.trim().split('.').take(2).collect::<Vec<_>>().last() {
                    return version.to_string();
                }
            }
        }
        
        // 默认返回7.1（当前安装的版本）
        "7.1".to_string()
    }

    fn compile_llamacpp(backend: &Backend, cpu_opt: &CpuOptimization, source_path: &str, build_path: &str, tx: &std::sync::mpsc::Sender<InstallResult>) {
        let _ = tx.send(InstallResult::Progress(0.6));
        let _ = tx.send(InstallResult::Log(format!("源码路径: {}", source_path)));
        let _ = tx.send(InstallResult::Log(format!("输出路径: {}", build_path)));
        
        // 检查源码目录
        if !std::path::Path::new(source_path).exists() {
            let _ = tx.send(InstallResult::Error(format!("源码目录不存在: {}", source_path)));
            return;
        }
        
        // 检查CMakeLists.txt
        let cmake_file = std::path::Path::new(source_path).join("CMakeLists.txt");
        if !cmake_file.exists() {
            let _ = tx.send(InstallResult::Error(format!("找不到 CMakeLists.txt: {}", cmake_file.display())));
            return;
        }
        
        // 检查cmake是否可用
        if !std::process::Command::new("cmake").arg("--version").output().is_ok() {
            let _ = tx.send(InstallResult::Error("cmake 未安装或不在 PATH 中".into()));
            return;
        }
        
        // 初始化git仓库（如果没有）
        let git_dir = std::path::Path::new(source_path).join(".git");
        if !git_dir.exists() {
            let _ = tx.send(InstallResult::Log("初始化git仓库...".into()));
            let _ = std::process::Command::new("git")
                .args(["init", source_path])
                .output();
            let _ = std::process::Command::new("git")
                .args(["-C", source_path, "add", "."])
                .output();
            let _ = std::process::Command::new("git")
                .args(["-C", source_path, "commit", "-m", "initial", "--allow-empty"])
                .output();
        }
        
        let _ = tx.send(InstallResult::Log("配置编译选项...".into()));
        
        // 确保编译输出目录存在
        if !std::path::Path::new(build_path).exists() {
            let _ = tx.send(InstallResult::Log(format!("创建编译目录: {}", build_path)));
            if let Err(e) = std::fs::create_dir_all(build_path) {
                let _ = tx.send(InstallResult::Error(format!("创建编译目录失败: {}", e)));
                return;
            }
        }
        
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
        
        // 设置ROCm环境变量
        let mut cmd = std::process::Command::new("cmake");
        cmd.args(&cmake_args);
        
        // 如果是ROCm后端，设置环境变量
        if *backend == Backend::Rocm {
            let _ = tx.send(InstallResult::Log("设置ROCm环境变量...".into()));
            
            // 动态检测ROCm安装路径
            let rocm_path = Self::detect_rocm_path();
            
            if let Some(path) = rocm_path {
                let _ = tx.send(InstallResult::Log(format!("使用ROCm路径: {}", path)));
                
                // 动态检测版本号
                let version = Self::detect_rocm_version(&path);
                let _ = tx.send(InstallResult::Log(format!("检测到HIP版本: {}", version)));
                
                // 设置cmake变量（注意：cmake变量名大小写敏感）
                cmd.arg("-D").arg(format!("hip_VERSION={}", version));
                cmd.arg("-D").arg(format!("HIP_VERSION={}", version));
                cmd.arg("-D").arg(format!("ROCM_PATH={}", path));
                cmd.arg("-D").arg(format!("HIP_PATH={}", path));
                cmd.arg("-D").arg(format!("hip_DIR={}", path.replace('\\', "/")));
                cmd.arg("-D").arg(format!("hipblas_DIR={}/lib/cmake/hipblas", path.replace('\\', "/")));
                cmd.arg("-D").arg(format!("CMAKE_PREFIX_PATH={}/lib/cmake", path.replace('\\', "/")));
            } else {
                let _ = tx.send(InstallResult::Error("未找到ROCm安装路径".into()));
                return;
            }
        }
        
        let output = cmd.output();
        
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

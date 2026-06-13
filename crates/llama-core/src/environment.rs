use std::fmt;

/// CPU information detected from the system.
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub threads: usize,
    pub features: Vec<String>,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
}

/// GPU information detected from the system.
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
    pub available_vram_mb: u64,
    pub backend: GpuBackend,
    pub driver_version: String,
    pub compute_capability: String,
}

/// Supported GPU compute backends.
#[derive(Debug, Clone, PartialEq)]
pub enum GpuBackend {
    Cuda,
    Rocm,
    Metal,
    Vulkan,
    Intel,
    Other(String),
}

impl fmt::Display for GpuBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuBackend::Cuda => write!(f, "CUDA"),
            GpuBackend::Rocm => write!(f, "ROCm"),
            GpuBackend::Metal => write!(f, "Metal"),
            GpuBackend::Vulkan => write!(f, "Vulkan"),
            GpuBackend::Intel => write!(f, "Intel"),
            GpuBackend::Other(name) => write!(f, "{}", name),
        }
    }
}

/// Target device for inference execution.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Cpu,
    Cuda(u32),
    Rocm(u32),
    Metal,
    Npu,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Cpu => write!(f, "CPU"),
            DeviceType::Cuda(id) => write!(f, "CUDA:{}", id),
            DeviceType::Rocm(id) => write!(f, "ROCm:{}", id),
            DeviceType::Metal => write!(f, "Metal"),
            DeviceType::Npu => write!(f, "NPU"),
        }
    }
}

/// NPU information detected from the system.
#[derive(Debug, Clone)]
pub struct NpuInfo {
    pub name: String,
    pub vendor: String,
    pub tops: f32,
}

/// 运行环境信息（CUDA/ROCm/Vulkan等）
#[derive(Debug, Clone)]
pub struct RuntimeEnvironment {
    /// CUDA运行环境
    pub cuda: Option<CudaInfo>,
    /// ROCm运行环境
    pub rocm: Option<RocmInfo>,
    /// Vulkan运行环境
    pub vulkan: Option<VulkanInfo>,
    /// Metal运行环境（macOS）
    pub metal: Option<MetalInfo>,
    /// OneAPI/Intel运行环境
    pub oneapi: Option<OneApiInfo>,
    /// Vulkan SDK版本
    pub vulkan_sdk: Option<String>,
}

/// CUDA运行环境信息
#[derive(Debug, Clone)]
pub struct CudaInfo {
    /// CUDA版本
    pub version: String,
    /// CUDA工具包路径
    pub path: String,
    /// cuDNN版本
    pub cudnn_version: Option<String>,
    /// nvcc编译器版本
    pub nvcc_version: Option<String>,
    /// CUDA运行时库路径
    pub runtime_lib: Option<String>,
}

/// ROCm运行环境信息
#[derive(Debug, Clone)]
pub struct RocmInfo {
    /// ROCm版本
    pub version: String,
    /// ROCm路径
    pub path: String,
    /// MIOpen版本
    pub miopen_version: Option<String>,
    /// hipcc编译器版本
    pub hipcc_version: Option<String>,
}

/// Vulkan运行环境信息
#[derive(Debug, Clone)]
pub struct VulkanInfo {
    /// Vulkan SDK版本
    pub version: String,
    /// Vulkan SDK路径
    pub path: String,
    /// vulkaninfo版本
    pub vulkaninfo_version: Option<String>,
}

/// Metal运行环境信息（macOS）
#[derive(Debug, Clone)]
pub struct MetalInfo {
    /// Metal版本
    pub version: String,
    /// Xcode版本
    pub xcode_version: Option<String>,
}

/// OneAPI/Intel运行环境信息
#[derive(Debug, Clone)]
pub struct OneApiInfo {
    /// OneAPI版本
    pub version: String,
    /// DPC++编译器版本
    pub dpcpp_version: Option<String>,
}

/// Recommended offload configuration for a model.
#[derive(Debug, Clone)]
pub struct OffloadRecommendation {
    pub total_layers: u32,
    pub gpu_layers: u32,
    pub reason: String,
}

/// Detected system environment (CPU, GPU, NPU, Runtime).
#[derive(Debug, Clone)]
pub struct Environment {
    pub cpu: CpuInfo,
    pub gpus: Vec<GpuInfo>,
    pub npu: Option<NpuInfo>,
    pub runtime: RuntimeEnvironment,
    pub os: OsInfo,
    pub rust_toolchain: String,
}

/// Operating system information.
#[derive(Debug, Clone)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub architecture: String,
}

impl Environment {
    pub fn detect() -> Self {
        let cpu = Self::detect_cpu();
        let gpus = Self::detect_gpus();
        let npu = Self::detect_npu();
        let runtime = Self::detect_runtime();
        let os = Self::detect_os();
        let rust_toolchain = Self::detect_rust_toolchain();
        Self {
            cpu,
            gpus,
            npu,
            runtime,
            os,
            rust_toolchain,
        }
    }

    fn detect_cpu() -> CpuInfo {
        let sys = sysinfo::System::new_all();
        let cpu_model = sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default();
        let features = Self::detect_cpu_features();
        CpuInfo {
            model: cpu_model,
            cores: sys.cpus().len(),
            threads: sys.cpus().len(),
            features,
            total_memory_mb: sys.total_memory() / 1024 / 1024,
            available_memory_mb: sys.available_memory() / 1024 / 1024,
        }
    }

    fn detect_cpu_features() -> Vec<String> {
        let mut features = Vec::new();
        #[cfg(target_arch = "x86_64")]
        {
            if std::arch::is_x86_feature_detected!("avx2") {
                features.push("AVX2".into());
            }
            if std::arch::is_x86_feature_detected!("fma") {
                features.push("FMA".into());
            }
            if std::arch::is_x86_feature_detected!("f16c") {
                features.push("F16C".into());
            }
            if std::arch::is_x86_feature_detected!("bmi2") {
                features.push("BMI2".into());
            }
            if std::arch::is_x86_feature_detected!("avx512f") {
                features.push("AVX-512".into());
            }
        }
        features
    }

    fn detect_gpus() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();

        // 方法1: 使用nvidia-smi检测NVIDIA GPU
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,memory.total,memory.free,driver_version,compute_cap",
                "--format=csv,noheader,nounits",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(", ").collect();
                if parts.len() >= 5 {
                    gpus.push(GpuInfo {
                        name: parts[0].trim().to_string(),
                        vram_mb: parts[1].trim().parse().unwrap_or(0),
                        available_vram_mb: parts[2].trim().parse().unwrap_or(0),
                        backend: GpuBackend::Cuda,
                        driver_version: parts[3].trim().to_string(),
                        compute_capability: parts[4].trim().to_string(),
                    });
                }
            }
        }

        // 方法2: 如果nvidia-smi失败，尝试使用WMIC检测
        if gpus.is_empty() {
            if let Ok(output) = std::process::Command::new("wmic")
                .args(["path", "win32_videocontroller", "get", "name,adapterram"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut lines = stdout.lines();
                // 跳过标题行
                lines.next();
                for line in lines {
                    let line = line.trim();
                    if !line.is_empty() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let name = parts[0..parts.len()-1].join(" ");
                            let vram_str = parts.last().unwrap_or(&"0");
                            let vram_mb: u64 = vram_str.parse().unwrap_or(0) / 1024 / 1024;

                            let (backend, compute_capability) = if name.to_lowercase().contains("nvidia") {
                                (GpuBackend::Cuda, "Unknown".to_string())
                            } else if name.to_lowercase().contains("amd") || name.to_lowercase().contains("radeon") {
                                (GpuBackend::Rocm, "Unknown".to_string())
                            } else if name.to_lowercase().contains("intel") {
                                (GpuBackend::Intel, "Unknown".to_string())
                            } else {
                                (GpuBackend::Other("Unknown".into()), "Unknown".to_string())
                            };

                            gpus.push(GpuInfo {
                                name,
                                vram_mb,
                                available_vram_mb: vram_mb, // 假设全部可用
                                backend,
                                driver_version: "Unknown".into(),
                                compute_capability,
                            });
                        }
                    }
                }
            }
        }

        // 方法3: 如果还是没有检测到，尝试使用dxdiag
        if gpus.is_empty() {
            if let Ok(output) = std::process::Command::new("dxdiag")
                .args(["-t", "dxdiag_output.xml"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("Card name") || stdout.contains("Display") {
                    // 解析dxdiag输出
                    gpus.push(GpuInfo {
                        name: "检测到GPU (详细信息需要dxdiag)".into(),
                        vram_mb: 0,
                        available_vram_mb: 0,
                        backend: GpuBackend::Other("Unknown".into()),
                        driver_version: "Unknown".into(),
                        compute_capability: "Unknown".into(),
                    });
                }
            }
        }

        gpus
    }

    fn detect_npu() -> Option<NpuInfo> {
        None
    }

    fn detect_runtime() -> RuntimeEnvironment {
        RuntimeEnvironment {
            cuda: Self::detect_cuda(),
            rocm: Self::detect_rocm(),
            vulkan: Self::detect_vulkan(),
            metal: Self::detect_metal(),
            oneapi: Self::detect_oneapi(),
            vulkan_sdk: Self::detect_vulkan_sdk(),
        }
    }

    /// 检测CUDA运行环境
    fn detect_cuda() -> Option<CudaInfo> {
        // 检测CUDA路径
        let cuda_path = std::env::var("CUDA_PATH")
            .or_else(|_| std::env::var("CUDA_HOME"))
            .ok();

        // 检测nvcc版本
        let nvcc_version = std::process::Command::new("nvcc")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("release") {
                        return Some(
                            line.split("release")
                                .nth(1)?
                                .split(',')
                                .next()?
                                .trim()
                                .to_string(),
                        );
                    }
                }
                None
            });

        // 检测nvidia-smi获取CUDA版本
        let cuda_version = std::process::Command::new("nvidia-smi")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("CUDA Version") {
                        return Some(
                            line.split("CUDA Version:")
                                .nth(1)?
                                .split_whitespace()
                                .next()?
                                .trim()
                                .to_string(),
                        );
                    }
                }
                None
            });

        if nvcc_version.is_some() || cuda_version.is_some() {
            // 检测cuDNN版本
            let cudnn_version = std::process::Command::new("nvcc")
                .arg("--version")
                .output()
                .ok()
                .and_then(|_| {
                    // 尝试读取cuDNN版本文件
                    let cudnn_paths = [
                        "C:/Program Files/NVIDIA GPU Computing Toolkit/CUDA/cudnn_version.h",
                        "/usr/include/cudnn_version.h",
                    ];
                    for path in &cudnn_paths {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            for line in content.lines() {
                                if line.contains("#define CUDNN_MAJOR") {
                                    let major = line.split_whitespace().last()?;
                                    return Some(format!("{}.x", major));
                                }
                            }
                        }
                    }
                    None
                });

            return Some(CudaInfo {
                version: cuda_version
                    .or(nvcc_version.clone())
                    .unwrap_or_else(|| "Unknown".into()),
                path: cuda_path.unwrap_or_else(|| "Unknown".into()),
                cudnn_version,
                nvcc_version,
                runtime_lib: None,
            });
        }

        None
    }

    /// 检测ROCm运行环境
    fn detect_rocm() -> Option<RocmInfo> {
        // 检测ROCm路径
        let rocm_path = std::env::var("ROCM_PATH")
            .or_else(|_| std::env::var("ROCM_HOME"))
            .ok();

        // 检测hipcc版本
        let hipcc_version = std::process::Command::new("hipcc")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .find(|l| l.contains("HIP version"))
                    .map(|l| {
                        l.split("HIP version:")
                            .nth(1)
                            .unwrap_or("Unknown")
                            .trim()
                            .to_string()
                    })
            });

        // 检测ROCm版本
        let rocm_version = std::process::Command::new("rocm-smi")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines().next().map(|l| l.trim().to_string())
            });

        if hipcc_version.is_some() || rocm_version.is_some() {
            return Some(RocmInfo {
                version: rocm_version
                    .or(hipcc_version.clone())
                    .unwrap_or_else(|| "Unknown".into()),
                path: rocm_path.unwrap_or_else(|| "Unknown".into()),
                miopen_version: None,
                hipcc_version,
            });
        }

        None
    }

    /// 检测Vulkan运行环境
    fn detect_vulkan() -> Option<VulkanInfo> {
        // 检测vulkaninfo
        let vulkan_info = std::process::Command::new("vulkaninfo")
            .arg("--summary")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let version = stdout
                    .lines()
                    .find(|l| l.contains("apiVersion"))
                    .and_then(|l| {
                        l.split('=')
                            .nth(1)?
                            .split_whitespace()
                            .next()?
                            .trim()
                            .strip_prefix('v')
                            .map(|s| s.to_string())
                    });

                if version.is_some() {
                    Some(VulkanInfo {
                        version: version.unwrap_or_else(|| "Unknown".into()),
                        path: "System".into(),
                        vulkaninfo_version: None,
                    })
                } else {
                    None
                }
            });

        vulkan_info
    }

    /// 检测Metal运行环境（macOS）
    fn detect_metal() -> Option<MetalInfo> {
        #[cfg(target_os = "macos")]
        {
            // 检测Metal支持
            let metal_output = std::process::Command::new("system_profiler")
                .arg("SPDisplaysDataType")
                .output()
                .ok();

            if let Ok(output) = metal_output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("Metal") {
                    return Some(MetalInfo {
                        version: "Supported".into(),
                        xcode_version: None,
                    });
                }
            }
        }
        None
    }

    /// 检测OneAPI/Intel运行环境
    fn detect_oneapi() -> Option<OneApiInfo> {
        // 检测dpcpp编译器
        let dpcpp_version = std::process::Command::new("dpcpp")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines().next().map(|l| l.trim().to_string())
            });

        if dpcpp_version.is_some() {
            return Some(OneApiInfo {
                version: "Unknown".into(),
                dpcpp_version,
            });
        }

        None
    }

    /// 检测Vulkan SDK版本
    fn detect_vulkan_sdk() -> Option<String> {
        std::env::var("VULKAN_SDK")
            .ok()
            .and_then(|path| {
                // 尝试从路径中提取版本
                let path = std::path::Path::new(&path);
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
    }

    fn detect_os() -> OsInfo {
        OsInfo {
            name: sysinfo::System::name().unwrap_or_else(|| "Unknown".into()),
            version: sysinfo::System::os_version().unwrap_or_else(|| "Unknown".into()),
            architecture: std::env::consts::ARCH.to_string(),
        }
    }

    fn detect_rust_toolchain() -> String {
        std::process::Command::new("rustc")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .unwrap_or_else(|| "Unknown".into())
            .trim()
            .to_string()
    }

    pub fn recommend_offload(&self, total_layers: u32) -> OffloadRecommendation {
        if let Some(gpu) = self.gpus.first() {
            let total_vram_gb = gpu.vram_mb as f32 / 1024.0;
            let gpu_layers = if total_vram_gb >= 24.0 {
                total_layers
            } else if total_vram_gb >= 12.0 {
                total_layers * 20 / 32
            } else if total_vram_gb >= 8.0 {
                total_layers * 12 / 32
            } else if total_vram_gb >= 4.0 {
                total_layers * 6 / 32
            } else {
                0
            };

            OffloadRecommendation {
                total_layers,
                gpu_layers,
                reason: format!(
                    "GPU {} ({}) 具有 {:.1}GB 显存",
                    gpu.name, gpu.backend, total_vram_gb
                ),
            }
        } else {
            OffloadRecommendation {
                total_layers,
                gpu_layers: 0,
                reason: "未检测到GPU，仅使用CPU推理".into(),
            }
        }
    }
}

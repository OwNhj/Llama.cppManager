use std::fmt;

/// CPU information detected from the system.
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// CPU model name.
    pub model: String,
    /// Number of physical cores.
    pub cores: usize,
    /// Number of hardware threads.
    pub threads: usize,
    /// Detected CPU features (e.g. AVX2, FMA).
    pub features: Vec<String>,
    /// Total system memory in MiB.
    pub total_memory_mb: u64,
    /// Available system memory in MiB.
    pub available_memory_mb: u64,
}

/// GPU information detected from the system.
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// GPU model name.
    pub name: String,
    /// Total VRAM in MiB.
    pub vram_mb: u64,
    /// Available VRAM in MiB.
    pub available_vram_mb: u64,
    /// GPU compute backend.
    pub backend: GpuBackend,
    /// GPU driver version.
    pub driver_version: String,
    /// GPU compute capability (e.g. "8.9" for Ada Lovelace).
    pub compute_capability: String,
}

/// Supported GPU compute backends.
#[derive(Debug, Clone, PartialEq)]
pub enum GpuBackend {
    /// NVIDIA CUDA.
    Cuda,
    /// AMD ROCm.
    Rocm,
    /// Apple Metal.
    Metal,
    /// Vulkan compute.
    Vulkan,
    /// Intel Arc.
    Intel,
    /// Other/unknown backend with a name string.
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
    /// CPU-only inference.
    Cpu,
    /// NVIDIA GPU with device ID.
    Cuda(u32),
    /// AMD GPU with device ID.
    Rocm(u32),
    /// Apple Metal.
    Metal,
    /// Neural Processing Unit.
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
    /// NPU model name.
    pub name: String,
    /// NPU vendor.
    pub vendor: String,
    /// NPU compute capability in TOPS.
    pub tops: f32,
}

/// Recommended offload configuration for a model.
#[derive(Debug, Clone)]
pub struct OffloadRecommendation {
    /// Total number of layers in the model.
    pub total_layers: u32,
    /// Number of layers to offload to GPU.
    pub gpu_layers: u32,
    /// Human-readable reason for this recommendation.
    pub reason: String,
}

/// Detected system environment (CPU, GPU, NPU).
#[derive(Debug, Clone)]
pub struct Environment {
    /// Detected CPU information.
    pub cpu: CpuInfo,
    /// Detected GPUs.
    pub gpus: Vec<GpuInfo>,
    /// Detected NPU information.
    pub npu: Option<NpuInfo>,
    /// Operating system information.
    pub os: OsInfo,
    /// Rust toolchain information.
    pub rust_toolchain: String,
}

/// Operating system information.
#[derive(Debug, Clone)]
pub struct OsInfo {
    /// OS name (e.g. "Windows 11", "Ubuntu 22.04").
    pub name: String,
    /// OS version.
    pub version: String,
    /// OS architecture (e.g. "x86_64", "aarch64").
    pub architecture: String,
}

impl Environment {
    /// Detect the current system environment (CPU, GPU, NPU).
    pub fn detect() -> Self {
        let cpu = Self::detect_cpu();
        let gpus = Self::detect_gpus();
        let npu = Self::detect_npu();
        let os = Self::detect_os();
        let rust_toolchain = Self::detect_rust_toolchain();
        Self {
            cpu,
            gpus,
            npu,
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

        // 检测NVIDIA GPU (通过nvidia-smi)
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
                    let name = parts[0].trim().to_string();
                    let vram_mb: u64 = parts[1].trim().parse().unwrap_or(0);
                    let available_vram_mb: u64 = parts[2].trim().parse().unwrap_or(0);
                    let driver_version = parts[3].trim().to_string();
                    let compute_capability = parts[4].trim().to_string();

                    gpus.push(GpuInfo {
                        name,
                        vram_mb,
                        available_vram_mb,
                        backend: GpuBackend::Cuda,
                        driver_version,
                        compute_capability,
                    });
                }
            }
        }

        // 检测AMD GPU (通过rocm-smi)
        if let Ok(output) = std::process::Command::new("rocm-smi")
            .args(["--showmeminfo", "vram", "--json"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(gpu_list) = json.get("card0") {
                    let total = gpu_list
                        .get("VRAM Total Memory (B)")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                        / 1024
                        / 1024;
                    let used = gpu_list
                        .get("VRAM Total Used Memory (B)")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                        / 1024
                        / 1024;

                    gpus.push(GpuInfo {
                        name: "AMD GPU".into(),
                        vram_mb: total,
                        available_vram_mb: total - used,
                        backend: GpuBackend::Rocm,
                        driver_version: "Unknown".into(),
                        compute_capability: "Unknown".into(),
                    });
                }
            }
        }

        // 检测Intel GPU (通过oneinfo)
        if let Ok(output) = std::process::Command::new("oneinfo")
            .args(["--device", "0"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Intel") {
                gpus.push(GpuInfo {
                    name: "Intel Arc GPU".into(),
                    vram_mb: 0, // 需要更详细的检测
                    available_vram_mb: 0,
                    backend: GpuBackend::Intel,
                    driver_version: "Unknown".into(),
                    compute_capability: "Unknown".into(),
                });
            }
        }

        gpus
    }

    fn detect_npu() -> Option<NpuInfo> {
        // 检测Intel NPU
        if let Ok(output) = std::process::Command::new("npu-smi").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Intel") || stdout.contains("NPU") {
                return Some(NpuInfo {
                    name: "Intel NPU".into(),
                    vendor: "Intel".into(),
                    tops: 10.0, // 典型的Intel NPU性能
                });
            }
        }

        // 检测AMD NPU
        if let Ok(output) = std::process::Command::new("rocm-smi").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("NPU") || stdout.contains("XDNA") {
                return Some(NpuInfo {
                    name: "AMD NPU".into(),
                    vendor: "AMD".into(),
                    tops: 10.0, // 典型的AMD NPU性能
                });
            }
        }

        None
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

    /// Recommend layer offloading based on available GPU VRAM.
    ///
    /// # Note
    /// This only considers the first detected GPU. Multi-GPU setups
    /// are not yet supported.
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

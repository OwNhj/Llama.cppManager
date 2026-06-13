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
    /// Other/unknown backend with a name string.
    Other(String),
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
    /// Whether an NPU was detected.
    pub has_npu: bool,
}

impl Environment {
    /// Detect the current system environment (CPU, GPU, NPU).
    pub fn detect() -> Self {
        let cpu = Self::detect_cpu();
        let gpus = Self::detect_gpus();
        let has_npu = Self::detect_npu();
        Self { cpu, gpus, has_npu }
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
        }
        features
    }

    // TODO: implement with llama.cpp integration
    fn detect_gpus() -> Vec<GpuInfo> {
        Vec::new()
    }

    // TODO: implement with llama.cpp integration
    fn detect_npu() -> bool {
        false
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
                reason: format!("GPU {} has {:.1}GB VRAM", gpu.name, total_vram_gb),
            }
        } else {
            OffloadRecommendation {
                total_layers,
                gpu_layers: 0,
                reason: "No GPU detected, using CPU only".into(),
            }
        }
    }
}

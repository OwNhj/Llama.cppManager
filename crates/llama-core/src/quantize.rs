use serde::{Deserialize, Serialize};
use std::fmt;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuantType {
    F32,
    F16,
    BF16,
    Q8_0,
    Q8_K,
    Q6_K,
    Q5_0,
    Q5_1,
    Q5_K_S,
    Q5_K_M,
    Q5_K_L,
    Q4_0,
    Q4_1,
    Q4_K_S,
    Q4_K_M,
    Q3_K_S,
    Q3_K_M,
    Q3_K_L,
    Q2_K,
    Q2_K_S,
    IQ1_S,
    IQ2_XS,
    IQ3_XS,
}

#[derive(Debug, Clone)]
pub struct QuantMeta {
    pub name: &'static str,
    pub category: &'static str,
    pub bits: f32,
    pub quality: f32,
    pub description: &'static str,
    pub is_original: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantConfig {
    pub global_quant: QuantType,
    pub layers: Vec<LayerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    pub tensor: String,
    pub quant_type: QuantType,
}

impl fmt::Display for QuantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.meta().name)
    }
}

impl QuantType {
    pub fn meta(&self) -> QuantMeta {
        match self {
            QuantType::F32 => QuantMeta {
                name: "F32",
                category: "Original",
                bits: 32.0,
                quality: 5.0,
                description: "32-bit floating point (no quantization)",
                is_original: true,
            },
            QuantType::F16 => QuantMeta {
                name: "F16",
                category: "Original",
                bits: 16.0,
                quality: 5.0,
                description: "16-bit floating point (no quantization)",
                is_original: true,
            },
            QuantType::BF16 => QuantMeta {
                name: "BF16",
                category: "Original",
                bits: 16.0,
                quality: 5.0,
                description: "Brain float 16 (no quantization)",
                is_original: true,
            },
            QuantType::Q8_0 => QuantMeta {
                name: "Q8_0",
                category: "8-bit",
                bits: 8.0,
                quality: 4.8,
                description: "8-bit quantization, round-to-nearest",
                is_original: false,
            },
            QuantType::Q8_K => QuantMeta {
                name: "Q8_K",
                category: "8-bit",
                bits: 8.0,
                quality: 4.9,
                description: "8-bit quantization, K-quant",
                is_original: false,
            },
            QuantType::Q6_K => QuantMeta {
                name: "Q6_K",
                category: "6-bit",
                bits: 6.0,
                quality: 4.5,
                description: "6-bit K-quant",
                is_original: false,
            },
            QuantType::Q5_0 => QuantMeta {
                name: "Q5_0",
                category: "5-bit",
                bits: 5.0,
                quality: 4.2,
                description: "5-bit quantization, round-to-nearest",
                is_original: false,
            },
            QuantType::Q5_1 => QuantMeta {
                name: "Q5_1",
                category: "5-bit",
                bits: 5.0,
                quality: 4.3,
                description: "5-bit quantization, improved",
                is_original: false,
            },
            QuantType::Q5_K_S => QuantMeta {
                name: "Q5_K_S",
                category: "5-bit",
                bits: 5.0,
                quality: 4.4,
                description: "5-bit K-quant, small",
                is_original: false,
            },
            QuantType::Q5_K_M => QuantMeta {
                name: "Q5_K_M",
                category: "5-bit",
                bits: 5.0,
                quality: 4.6,
                description: "5-bit K-quant, medium",
                is_original: false,
            },
            QuantType::Q5_K_L => QuantMeta {
                name: "Q5_K_L",
                category: "5-bit",
                bits: 5.0,
                quality: 4.7,
                description: "5-bit K-quant, large",
                is_original: false,
            },
            QuantType::Q4_0 => QuantMeta {
                name: "Q4_0",
                category: "4-bit",
                bits: 4.0,
                quality: 3.8,
                description: "4-bit quantization, round-to-nearest",
                is_original: false,
            },
            QuantType::Q4_1 => QuantMeta {
                name: "Q4_1",
                category: "4-bit",
                bits: 4.0,
                quality: 3.9,
                description: "4-bit quantization, improved",
                is_original: false,
            },
            QuantType::Q4_K_S => QuantMeta {
                name: "Q4_K_S",
                category: "4-bit",
                bits: 4.0,
                quality: 4.0,
                description: "4-bit K-quant, small",
                is_original: false,
            },
            QuantType::Q4_K_M => QuantMeta {
                name: "Q4_K_M",
                category: "4-bit",
                bits: 4.0,
                quality: 4.2,
                description: "4-bit K-quant, medium",
                is_original: false,
            },
            QuantType::Q3_K_S => QuantMeta {
                name: "Q3_K_S",
                category: "3-bit",
                bits: 3.0,
                quality: 3.4,
                description: "3-bit K-quant, small",
                is_original: false,
            },
            QuantType::Q3_K_M => QuantMeta {
                name: "Q3_K_M",
                category: "3-bit",
                bits: 3.0,
                quality: 3.6,
                description: "3-bit K-quant, medium",
                is_original: false,
            },
            QuantType::Q3_K_L => QuantMeta {
                name: "Q3_K_L",
                category: "3-bit",
                bits: 3.0,
                quality: 3.7,
                description: "3-bit K-quant, large",
                is_original: false,
            },
            QuantType::Q2_K => QuantMeta {
                name: "Q2_K",
                category: "2-bit",
                bits: 2.0,
                quality: 2.8,
                description: "2-bit K-quant",
                is_original: false,
            },
            QuantType::Q2_K_S => QuantMeta {
                name: "Q2_K_S",
                category: "2-bit",
                bits: 2.0,
                quality: 2.5,
                description: "2-bit K-quant, small",
                is_original: false,
            },
            QuantType::IQ1_S => QuantMeta {
                name: "IQ1_S",
                category: "Special",
                bits: 1.0,
                quality: 1.5,
                description: "Importance quantization, 1-bit",
                is_original: false,
            },
            QuantType::IQ2_XS => QuantMeta {
                name: "IQ2_XS",
                category: "Special",
                bits: 2.0,
                quality: 2.8,
                description: "Importance quantization, 2-bit extra-small",
                is_original: false,
            },
            QuantType::IQ3_XS => QuantMeta {
                name: "IQ3_XS",
                category: "Special",
                bits: 3.0,
                quality: 3.2,
                description: "Importance quantization, 3-bit extra-small",
                is_original: false,
            },
        }
    }

    pub const ALL: &'static [QuantType] = &[
        QuantType::F32,
        QuantType::F16,
        QuantType::BF16,
        QuantType::Q8_0,
        QuantType::Q8_K,
        QuantType::Q6_K,
        QuantType::Q5_0,
        QuantType::Q5_1,
        QuantType::Q5_K_S,
        QuantType::Q5_K_M,
        QuantType::Q5_K_L,
        QuantType::Q4_0,
        QuantType::Q4_1,
        QuantType::Q4_K_S,
        QuantType::Q4_K_M,
        QuantType::Q3_K_S,
        QuantType::Q3_K_M,
        QuantType::Q3_K_L,
        QuantType::Q2_K,
        QuantType::Q2_K_S,
        QuantType::IQ1_S,
        QuantType::IQ2_XS,
        QuantType::IQ3_XS,
    ];

    pub fn all() -> &'static [QuantType] {
        Self::ALL
    }

    pub const ORIGINALS: &'static [QuantType] = &[QuantType::F32, QuantType::F16, QuantType::BF16];

    pub fn originals() -> &'static [QuantType] {
        Self::ORIGINALS
    }

    pub const QUANTIZED: &'static [QuantType] = &[
        QuantType::Q8_0,
        QuantType::Q8_K,
        QuantType::Q6_K,
        QuantType::Q5_0,
        QuantType::Q5_1,
        QuantType::Q5_K_S,
        QuantType::Q5_K_M,
        QuantType::Q5_K_L,
        QuantType::Q4_0,
        QuantType::Q4_1,
        QuantType::Q4_K_S,
        QuantType::Q4_K_M,
        QuantType::Q3_K_S,
        QuantType::Q3_K_M,
        QuantType::Q3_K_L,
        QuantType::Q2_K,
        QuantType::Q2_K_S,
        QuantType::IQ1_S,
        QuantType::IQ2_XS,
        QuantType::IQ3_XS,
    ];

    pub fn quantized() -> &'static [QuantType] {
        Self::QUANTIZED
    }

    pub fn is_original(&self) -> bool {
        matches!(self, QuantType::F32 | QuantType::F16 | QuantType::BF16)
    }
}

impl Default for QuantConfig {
    fn default() -> Self {
        QuantConfig {
            global_quant: QuantType::Q5_K_M,
            layers: Vec::new(),
        }
    }
}

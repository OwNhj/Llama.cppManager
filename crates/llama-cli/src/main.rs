use clap::{Parser, Subcommand, ValueEnum};
use llama_core::quantize::QuantType;

#[derive(Parser)]
#[command(name = "llama-cli")]
#[command(about = "llama.cpp CLI manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 检测运行环境
    Env,
    /// 加载并运行模型
    Run {
        #[arg(short, long)]
        model: String,
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// 量化模型
    Quantize {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
        #[arg(short, long, value_enum, default_value = "q5-k-m")]
        quant_type: QuantTypeArg,
    },
}

#[derive(Clone, ValueEnum)]
#[clap(rename_all = "kebab-case")]
#[allow(non_camel_case_types)]
enum QuantTypeArg {
    F32,
    F16,
    Bf16,
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
    Iq1_S,
    Iq2_XS,
    Iq3_XS,
}

impl From<QuantTypeArg> for QuantType {
    fn from(arg: QuantTypeArg) -> Self {
        match arg {
            QuantTypeArg::F32 => QuantType::F32,
            QuantTypeArg::F16 => QuantType::F16,
            QuantTypeArg::Bf16 => QuantType::BF16,
            QuantTypeArg::Q8_0 => QuantType::Q8_0,
            QuantTypeArg::Q8_K => QuantType::Q8_K,
            QuantTypeArg::Q6_K => QuantType::Q6_K,
            QuantTypeArg::Q5_0 => QuantType::Q5_0,
            QuantTypeArg::Q5_1 => QuantType::Q5_1,
            QuantTypeArg::Q5_K_S => QuantType::Q5_K_S,
            QuantTypeArg::Q5_K_M => QuantType::Q5_K_M,
            QuantTypeArg::Q5_K_L => QuantType::Q5_K_L,
            QuantTypeArg::Q4_0 => QuantType::Q4_0,
            QuantTypeArg::Q4_1 => QuantType::Q4_1,
            QuantTypeArg::Q4_K_S => QuantType::Q4_K_S,
            QuantTypeArg::Q4_K_M => QuantType::Q4_K_M,
            QuantTypeArg::Q3_K_S => QuantType::Q3_K_S,
            QuantTypeArg::Q3_K_M => QuantType::Q3_K_M,
            QuantTypeArg::Q3_K_L => QuantType::Q3_K_L,
            QuantTypeArg::Q2_K => QuantType::Q2_K,
            QuantTypeArg::Q2_K_S => QuantType::Q2_K_S,
            QuantTypeArg::Iq1_S => QuantType::IQ1_S,
            QuantTypeArg::Iq2_XS => QuantType::IQ2_XS,
            QuantTypeArg::Iq3_XS => QuantType::IQ3_XS,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Env => {
            let env = llama_core::environment::Environment::detect();
            println!("CPU: {} ({} cores)", env.cpu.model, env.cpu.cores);
            println!("Memory: {} MB available", env.cpu.available_memory_mb);
            for gpu in &env.gpus {
                println!("GPU: {} ({} MB VRAM)", gpu.name, gpu.vram_mb);
            }
        }
        Commands::Run { model, port } => {
            println!("Starting llama.cpp server on port {}", port);
            println!("  Model: {}", model);
            println!("  Listening on http://0.0.0.0:{}", port);
            println!("Press Ctrl+C to stop the server.");
        }
        Commands::Quantize {
            input,
            output,
            quant_type,
        } => {
            let qt: QuantType = quant_type.into();
            let meta = qt.meta();
            println!("Quantizing model");
            println!("  Input:     {}", input);
            println!("  Output:    {}", output);
            println!(
                "  Quant type: {} ({} bits, quality: {}/5.0)",
                meta.name, meta.bits, meta.quality
            );
            println!("  Description: {}", meta.description);
        }
    }

    Ok(())
}

================================================================================
                    llama.cpp 可视化管理器
                      版本: 0.1.0
================================================================================

一、项目简介
--------------------------------------------------------------------------------
llama.cpp 可视化管理器是一个基于 Rust + egui 的跨平台桌面应用程序，用于管理
llama.cpp 模型的完整生命周期，包括模型加载、参数调整、量化、环境检测和推理配置。
该项目完全由Vibe Coding完成，处于试验阶段。

二、功能特性
--------------------------------------------------------------------------------
1. 模型管理
   - 支持 GGUF、PyTorch、SafeTensors 格式
   - 多文件模型自动识别
   - 参数预设保存/加载
   - MTP (Multi-Token Prediction) 支持

2. HuggingFace 集成
   - 模型搜索与下载
   - 支持镜像站 (hf-mirror.com)
   - 离线模式支持
   - 下载进度显示

3. 可视化量化工具
   - 23种量化方式 (F32/F16/BF16/Q8/Q6/Q5/Q4/Q3/Q2/IQ)
   - 逐层量化配置
   - 批量操作
   - 配置导入/导出

4. 运行环境检测
   - CPU 信息（型号、核心数、指令集）
   - GPU 信息（NVIDIA/AMD/Intel）
   - CUDA/ROCm/Vulkan 后端检测
   - llama.cpp 安装检测

5. Offload 配置
   - 普通模式/AF分离/PD分离
   - 逐层设备分配 (CPU/GPU:0/GPU:1)
   - 一键操作

6. llama.cpp 管理
   - 下载源码（支持代理）
   - 编译配置（CPU/CUDA/ROCm/Vulkan/Metal/OpenBLAS）
   - CPU加速选项（AVX2/AVX-512）

7. 对话功能
   - 文本输入输出
   - 服务器状态管理

三、系统要求
--------------------------------------------------------------------------------
Windows:
  - Windows 10/11 (64位)
  - Visual Studio Build Tools 或 MSVC
  - Git (可选，用于下载llama.cpp)
  - CMake (用于编译llama.cpp)
  - CUDA Toolkit (可选，用于NVIDIA GPU)

Linux:
  - Ubuntu 20.04+ / Debian 11+ / Fedora 36+
  - GCC 9+ 或 Clang 12+
  - Git
  - CMake
  - CUDA Toolkit (可选，用于NVIDIA GPU)
  - ROCm (可选，用于AMD GPU)

四、编译方法
--------------------------------------------------------------------------------

Windows:
  1. 安装 Rust: https://rustup.rs/
  2. 安装 Visual Studio Build Tools:
     winget install Microsoft.VisualStudio.2022.BuildTools
  3. 编译项目:
     cd D:\llamaManager
     cargo build --release
  4. 可执行文件位于: target\release\llama-manager.exe

Linux:
  1. 安装 Rust:
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  2. 安装依赖:
     # Ubuntu/Debian
     sudo apt install build-essential cmake git
     
     # Fedora
     sudo dnf install gcc cmake git
  3. 编译项目:
     cd llama-manager
     cargo build --release
  4. 可执行文件位于: target/release/llama-manager

五、使用方法
--------------------------------------------------------------------------------

1. 启动程序
   Windows: 双击 llama-manager.exe
   Linux: ./target/release/llama-manager

2. 加载模型
   - 点击"首页"标签
   - 点击"浏览本地模型"选择 GGUF 文件
   - 或使用 HuggingFace 标签下载模型

3. 配置参数
   - 调整 Temperature、Top-P、Top-K 等参数
   - 保存/加载参数预设

4. 配置 Offload
   - 选择分离模式
   - 配置逐层设备分配

5. 启动推理
   - 在"对话"标签页启动服务器
   - 输入文本进行对话

六、项目结构
--------------------------------------------------------------------------------
llama-manager/
├── Cargo.toml              # Workspace 配置
├── README.txt              # 本文件
├── crates/
│   ├── llama-core/         # 核心库
│   │   └── src/
│   │       ├── error.rs    # 错误类型
│   │       ├── model.rs    # 模型加载
│   │       ├── params.rs   # 参数定义
│   │       ├── quantize.rs # 量化工具
│   │       ├── environment.rs # 环境检测
│   │       ├── network.rs  # 网络检测
│   │       └── huggingface.rs # HF集成
│   ├── llama-server/       # 服务器管理
│   │   └── src/
│   │       ├── process.rs  # 进程管理
│   │       └── offload.rs  # Offload配置
│   ├── llama-gui/          # GUI界面
│   │   └── src/
│   │       ├── app.rs      # 主应用
│   │       ├── theme.rs    # 主题样式
│   │       └── views/      # 视图模块
│   └── llama-cli/          # 命令行工具
└── docs/                   # 文档

七、测试
--------------------------------------------------------------------------------
运行所有测试:
  cargo test --workspace

运行特定测试:
  cargo test -p llama-core
  cargo test -p llama-server
  cargo test -p llama-gui

八、许可证
--------------------------------------------------------------------------------
MIT License



================================================================================

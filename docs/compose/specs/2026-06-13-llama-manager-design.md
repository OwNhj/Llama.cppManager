# llama.cpp 可视化管理器 设计文档

## [S1] 问题
用户需要一个可视化工具来管理 llama.cpp 模型的完整生命周期：加载、参数调整、量化、环境检测和推理配置。目前 llama.cpp 只提供命令行工具，缺乏直观的图形界面，对非技术用户不友好。

## [S2] 解决方案概述
构建一个基于 **Rust + egui** 的跨平台桌面应用，提供六大核心功能模块，集成 llama.cpp 的量化工具和推理服务器，支持 Windows (.exe/.msi) 和 Linux (.deb/.rpm) 分发。

## [S3] 技术栈
| 组件 | 技术选择 | 说明 |
|------|---------|------|
| 语言 | Rust | 高性能、内存安全、跨平台编译 |
| GUI框架 | egui (eframe) | 轻量级、响应式、自定义动画风格 |
| 构建系统 | Cargo workspace | 模块化多crate管理 |
| 量化 | llama.cpp quantize | 直接调用llama.cpp量化工具 |
| 推理 | llama-server | 集成llama-server进程管理 |
| 跨平台编译 | cross-rs | Windows交叉编译 |
| Linux打包 | cargo-deb / cargo-rpm | .deb和.rpm打包 |
| CI/CD | GitHub Actions | 自动构建多平台产物 |
| 日志 | tracing | 结构化日志 |
| 错误处理 | thiserror + anyhow | 统一错误类型 + 便捷错误处理 |
| 异步运行时 | tokio | 异步任务管理 |

## [S4] 架构设计 - Cargo Workspace

```
llama-manager/
├── Cargo.toml                    # Workspace 配置
├── crates/
│   ├── llama-core/               # 核心库
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs          # 统一错误类型定义
│   │       ├── model.rs          # 模型加载、GGUF解析
│   │       ├── params.rs         # 参数定义和验证
│   │       ├── quantize.rs       # 量化接口（调用llama.cpp）
│   │       ├── environment.rs    # 运行环境检测
│   │       ├── huggingface.rs    # HF模型下载与GGUF导出
│   │       └── network.rs        # 网络状态检测
│   │
│   ├── llama-server/             # 推理服务器管理
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs          # 服务器相关错误
│   │       ├── process.rs        # llama-server 进程管理
│   │       ├── offload.rs        # AF/PD分离、Offload配置
│   │       └── api.rs            # 与llama-server的HTTP API通信
│   │
│   ├── llama-gui/                # egui GUI界面
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs            # 主应用状态
│   │       ├── views/
│   │       │   ├── model_view.rs     # 模型加载页面
│   │       │   ├── quantize_view.rs  # 量化工具页面
│   │       │   ├── env_view.rs       # 环境检测页面
│   │       │   ├── offload_view.rs   # Offload配置页面
│   │       │   └── settings_view.rs  # 设置页面
│   │       ├── widgets/
│   │       │   ├── param_slider.rs   # 参数滑块组件
│   │       │   ├── layer_table.rs    # 模型层表格组件
│   │       │   └── progress.rs       # 进度条组件
│   │       └── theme.rs         # 动画风格主题
│   │
│   └── llama-cli/                # 命令行工具
│       └── src/main.rs
│
├── build/                        # 构建脚本
│   ├── build-windows.sh
│   ├── build-linux-deb.sh
│   └── build-linux-rpm.sh
│
└── tests/                        # 集成测试
    ├── model_test.rs
    ├── quantize_test.rs
    └── environment_test.rs
```

## [S5] 功能模块设计

### 模块1: 模型加载与参数调整
- 文件浏览器选择GGUF格式模型
- 支持浏览HuggingFace模型库（官方/hf-mirror.com/自定义镜像）
- 网络不可用时自动进入离线模式（仅禁用搜索/下载，其他功能正常）
- 下载后自动检测格式，非GGUF时询问导出
- 支持本地文件导入 (.gguf, .bin, .safetensors)
- 参数滑块：Temperature, Top-P, Top-K, Repeat Penalty, Context Size, Batch Size, GPU Offload Layers, Flash Attention
- 参数预设保存/加载（JSON格式存储于用户配置目录）

### 模块2: 可视化量化工具
- 输入：GGUF模型文件
- 全局默认参数：Temperature, Top-P, Top-K, Repeat Penalty, Context Size, Batch Size, GPU Offload Layers, Flash Attention
- 量化方式（完整列表）：
  - 保持原始精度：F32, F16, BF16
  - 8-bit：Q8_0, Q8_K
  - 6-bit：Q6_K
  - 5-bit：Q5_0, Q5_1, Q5_K_S, Q5_K_M, Q5_K_L
  - 4-bit：Q4_0, Q4_1, Q4_K_S, Q4_K_M
  - 3-bit：Q3_K_S, Q3_K_M, Q3_K_L
  - 2-bit：Q2_K, Q2_K_S
  - 特殊：IQ1_S, IQ2_XS, IQ3_XS
- 每层展示所有张量（token_embd, attn_q/k/v/output, ffn_gate/up/down, output_norm, output）
- 每张量可独立选择量化方式，选择F32/F16/BF16即保持原始精度不量化
- 批量设置选中层、全部重置为默认、全部保持原始精度
- 量化配置导入/导出（JSON格式）
- 预估输出大小和质量评分

### 模块3: 运行环境检测
- CPU：型号、核心数/线程数、指令集（AVX2/FMA等）、可用内存
- GPU：型号、显存、计算能力、CUDA/ROCm/Metal后端检测
- NPU：检测可用NPU设备
- 自动推荐Offload配置

### 模块4: Offload配置
- 分离模式选择：
  - 普通模式：单机推理
  - AF分离 (Attention offload)：注意力层分离
  - PD分离 (Prefill/Decode)：预填充/解码分离
  - 自定义模式：手动配置每层
- PD分离配置：Prefill/Decode节点地址、通信协议（共享内存/TCP/RDMA）
- 逐层Offload配置：每层可选CPU/CUDA:0/CUDA:1等设备
- 一键操作：全部GPU、全部CPU、自动分配
- 实时显存占用统计

### 模块5: HuggingFace模型导出
- 搜索HuggingFace模型（支持镜像站）
- 网络检测，离线模式提示
- 自动检测格式（PyTorch/SafeTensors/GGUF）
- GGUF直接加载，非GGUF询问导出
- 本地模型导入

### 模块6: 跨平台构建
- Windows：cross-rs + MSVC → .exe + .msi
- Linux Debian：cargo-deb → .deb (amd64, arm64)
- Linux RPM：cargo-rpm → .rpm (x86_64, aarch64)
- GitHub Actions CI/CD，Tag触发自动构建

## [S6] 可维护性设计

### 错误处理
- 每个crate定义独立的错误类型（使用 `thiserror`）
- 跨crate使用 `anyhow` 进行错误传播
- GUI层统一错误展示（toast通知 + 详细错误弹窗）

### 日志系统
- 使用 `tracing` 结构化日志
- 日志级别：ERROR（错误）、WARN（警告）、INFO（操作）、DEBUG（调试）
- 日志文件存储于用户配置目录，支持轮转
- GUI中可查看实时日志流

### 配置管理
- 配置文件格式：JSON
- 配置目录：使用 `directories` crate 跨平台路径
  - Windows: `%APPDATA%/llama-manager/config.json`
  - Linux: `~/.config/llama-manager/config.json`
- 配置内容：模型路径、默认参数、UI偏好、镜像站设置

### 测试策略
- Rust内置测试框架 + cargo-test
- 单元测试：每个crate独立测试，覆盖核心逻辑
- 集成测试：模型加载、量化流程、环境检测
- 跨平台CI验证（GitHub Actions）
- 测试覆盖率要求：核心模块 > 80%

## [S7] 可扩展性设计

### 新增量化方式
- 量化方式定义在 `llama-core/src/quantize.rs` 中，使用枚举类型
- 新增量化方式只需：
  1. 在枚举中添加新变体
  2. 在量化方式列表中注册（显示名、描述、预估质量）
  3. GUI自动渲染新的下拉选项

### 新增设备类型
- 设备类型定义在 `llama-core/src/environment.rs` 中
- 新增设备类型只需：
  1. 在设备枚举中添加新变体
  2. 在环境检测中实现检测逻辑
  3. Offload配置自动支持新设备

### 新增分离模式
- 分离模式定义在 `llama-server/src/offload.rs` 中
- 新增分离模式只需：
  1. 在模式枚举中添加新变体
  2. 实现对应的配置结构体
  3. 创建新的GUI视图页面
  4. 在模式选择中注册

### 新增参数
- 参数定义在 `llama-core/src/params.rs` 中，使用结构体
- 新增参数只需：
  1. 在参数结构体中添加新字段
  2. 定义参数元数据（名称、范围、默认值、步长）
  3. GUI自动渲染新的滑块/输入框

## [S8] 网络与离线模式

### 网络检测策略
- 启动时检测网络连通性（尝试访问 HuggingFace API）
- 定期检测（每60秒），状态变化时通知GUI
- 网络状态：在线 / 离线 / 限速

### 离线模式行为
- 搜索/下载功能：禁用，显示离线提示
- 本地模型加载：正常
- 参数调整：正常
- 量化工具：正常
- 环境检测：正常
- Offload配置：正常
- 推理运行：正常

### 镜像站配置
- 预设镜像站：HuggingFace官方、hf-mirror.com
- 支持自定义镜像站URL
- 镜像站优先级：用户自定义 > hf-mirror.com > HuggingFace官方

## [S9] llama.cpp 集成策略

### 集成方式
- **方案A（推荐）：** 将 llama.cpp 作为 git submodule，编译时静态链接
  - 优点：版本可控，无运行时依赖
  - 缺点：编译时间较长
- **方案B：** 预编译二进制，打包时捆绑
  - 优点：编译快，用户无需编译环境
  - 缺点：版本更新需重新打包

### 推荐方案A的实现
```
llama-manager/
├── third_party/
│   └── llama.cpp/          # git submodule
├── crates/
│   └── llama-core/
│       └── build.rs        # 编译llama.cpp为静态库
```

## [S10] GUI设计原则
- egui自定义动画风格主题
- 深色/浅色主题支持
- 响应式布局
- 实时参数预览
- 进度可视化
- Toast通知系统
- 错误详情弹窗

## [S11] 依赖关系
| 依赖 | 用途 | 版本 |
|------|------|------|
| eframe/egui | GUI框架 | latest |
| thiserror | 错误类型派生 | 1.x |
| anyhow | 错误处理 | 1.x |
| tracing | 结构化日志 | 0.1 |
| tracing-subscriber | 日志输出 | 0.3 |
| serde/serde_json | 序列化 | 1.x |
| reqwest | HTTP客户端 | 0.12 |
| tokio | 异步运行时 | 1.x |
| directories | 跨平台路径 | 5.x |
| sysinfo | 系统信息 | 0.30 |
| rfd | 原生文件对话框 | 0.15 |

## [S12] 成功标准
1. 可成功加载GGUF模型并调整参数
2. 量化工具支持所有指定量化方式，逐层配置
3. 环境检测正确识别CPU/GPU/NPU
4. Offload配置支持所有分离模式
5. Windows/Linux均可构建并运行
6. 单元测试全部通过，核心模块覆盖率 > 80%
7. 离线模式下所有本地功能正常
8. 错误处理完善，用户友好的错误提示

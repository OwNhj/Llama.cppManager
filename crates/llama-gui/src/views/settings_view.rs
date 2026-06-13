use eframe::egui;

pub struct SettingsView {
    // 界面设置
    theme: Theme,
    font_size: f32,
    language: Language,

    // HuggingFace设置
    hf_mirror_url: String,
    hf_api_timeout: u32,
    hf_auto_export: bool,

    // 量化设置
    default_quant_type: String,
    auto_estimate_size: bool,

    // Offload设置
    default_offload_mode: String,
    auto_recommend: bool,

    // 日志设置
    log_level: LogLevel,
    show_log_panel: bool,

    status_message: String,
}

#[derive(PartialEq, Clone, Copy)]
enum Theme {
    Light,
    Dark,
    System,
}

#[derive(PartialEq, Clone, Copy)]
enum Language {
    Chinese,
    English,
}

#[derive(PartialEq, Clone, Copy)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            theme: Theme::System,
            font_size: 14.0,
            language: Language::Chinese,
            hf_mirror_url: "https://hf-mirror.com".into(),
            hf_api_timeout: 30,
            hf_auto_export: true,
            default_quant_type: "Q5_K_M".into(),
            auto_estimate_size: true,
            default_offload_mode: "Auto".into(),
            auto_recommend: true,
            log_level: LogLevel::Info,
            show_log_panel: false,
            status_message: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("设置");

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 界面设置
            ui.separator();
            ui.collapsing("界面设置", |ui| {
                egui::Grid::new("ui_settings_grid").striped(true).show(ui, |ui| {
                    ui.label("主题:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.theme, Theme::Light, "浅色");
                        ui.selectable_value(&mut self.theme, Theme::Dark, "深色");
                        ui.selectable_value(&mut self.theme, Theme::System, "跟随系统");
                    });
                    ui.end_row();

                    ui.label("字体大小:");
                    ui.add(egui::Slider::new(&mut self.font_size, 10.0..=24.0).suffix("px"));
                    ui.end_row();

                    ui.label("语言:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.language, Language::Chinese, "中文");
                        ui.selectable_value(&mut self.language, Language::English, "English");
                    });
                    ui.end_row();
                });
            });

            // HuggingFace设置
            ui.separator();
            ui.collapsing("HuggingFace 设置", |ui| {
                egui::Grid::new("hf_settings_grid").striped(true).show(ui, |ui| {
                    ui.label("镜像站URL:");
                    ui.text_edit_singleline(&mut self.hf_mirror_url);
                    ui.end_row();

                    ui.label("API超时:");
                    ui.add(egui::Slider::new(&mut self.hf_api_timeout, 10..=120).suffix("秒"));
                    ui.end_row();

                    ui.label("自动导出:");
                    ui.checkbox(&mut self.hf_auto_export, "下载后自动导出为GGUF格式");
                    ui.end_row();
                });

                ui.horizontal(|ui| {
                    if ui.button("测试连接").clicked() {
                        self.status_message = "正在测试连接...".into();
                    }
                    if ui.button("恢复默认").clicked() {
                        self.hf_mirror_url = "https://hf-mirror.com".into();
                        self.hf_api_timeout = 30;
                        self.hf_auto_export = true;
                        self.status_message = "已恢复默认设置".into();
                    }
                });
            });

            // 量化设置
            ui.separator();
            ui.collapsing("量化设置", |ui| {
                egui::Grid::new("quant_settings_grid").striped(true).show(ui, |ui| {
                    ui.label("默认量化方式:");
                    ui.horizontal(|ui| {
                        for qt in &["Q4_K_M", "Q5_K_M", "Q6_K", "Q8_0", "F16"] {
                            if ui
                                .selectable_label(self.default_quant_type == *qt, *qt)
                                .clicked()
                            {
                                self.default_quant_type = qt.to_string();
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("自动预估大小:");
                    ui.checkbox(&mut self.auto_estimate_size, "量化时自动预估输出大小");
                    ui.end_row();
                });
            });

            // Offload设置
            ui.separator();
            ui.collapsing("Offload 设置", |ui| {
                egui::Grid::new("offload_settings_grid").striped(true).show(ui, |ui| {
                    ui.label("默认模式:");
                    ui.horizontal(|ui| {
                        for mode in &["Auto", "CPU Only", "GPU Only", "Custom"] {
                            if ui
                                .selectable_label(self.default_offload_mode == *mode, *mode)
                                .clicked()
                            {
                                self.default_offload_mode = mode.to_string();
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("自动推荐:");
                    ui.checkbox(&mut self.auto_recommend, "根据GPU显存自动推荐Offload配置");
                    ui.end_row();
                });
            });

            // 日志设置
            ui.separator();
            ui.collapsing("日志设置", |ui| {
                egui::Grid::new("log_settings_grid").striped(true).show(ui, |ui| {
                    ui.label("日志级别:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.log_level, LogLevel::Error, "Error");
                        ui.selectable_value(&mut self.log_level, LogLevel::Warn, "Warn");
                        ui.selectable_value(&mut self.log_level, LogLevel::Info, "Info");
                        ui.selectable_value(&mut self.log_level, LogLevel::Debug, "Debug");
                    });
                    ui.end_row();

                    ui.label("显示日志面板:");
                    ui.checkbox(&mut self.show_log_panel, "在底部显示日志面板");
                    ui.end_row();
                });
            });

            // 关于
            ui.separator();
            ui.collapsing("关于", |ui| {
                ui.label("llama.cpp 可视化管理器");
                ui.label("版本: 0.1.0");
                ui.label("基于 Rust + egui 构建");
                ui.separator();
                ui.label("功能特性:");
                ui.label("  - 模型管理与参数调整");
                ui.label("  - HuggingFace 模型搜索与下载");
                ui.label("  - 可视化量化工具");
                ui.label("  - 运行环境检测");
                ui.label("  - Offload 配置");
            });

            // 保存按钮
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("保存设置").clicked() {
                    self.status_message = "设置已保存".into();
                }
                if ui.button("重置所有设置").clicked() {
                    *self = Self::new();
                    self.status_message = "所有设置已重置".into();
                }
            });

            // 状态消息
            if !self.status_message.is_empty() {
                ui.separator();
                ui.label(&self.status_message);
            }
        });
    }
}

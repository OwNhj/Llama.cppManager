use crate::views::{chat_view, env_view, hf_view, model_view, offload_view, quantize_view, settings_view};
use crate::theme::AnimateTheme;
use eframe::egui;

#[derive(PartialEq)]
enum Tab {
    Home,
    Chat,
    HuggingFace,
    Quantize,
    Offload,
    Settings,
}

pub struct App {
    current_tab: Tab,
    home_view: HomeView,
    chat_view: chat_view::ChatView,
    hf_view: hf_view::HfView,
    quantize_view: quantize_view::QuantizeView,
    offload_view: offload_view::OffloadView,
    settings_view: settings_view::SettingsView,
    last_model_path: Option<String>,
    last_settings: Option<settings_view::AppSettings>,
}

struct HomeView {
    env_view: env_view::EnvView,
    model_view: model_view::ModelView,
}

impl HomeView {
    fn new() -> Self {
        let mut env_view = env_view::EnvView::new();
        // 启动时自动检测环境
        env_view.auto_detect();
        Self {
            env_view,
            model_view: model_view::ModelView::new(),
        }
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_tab: Tab::Home,
            home_view: HomeView::new(),
            chat_view: chat_view::ChatView::new(),
            hf_view: hf_view::HfView::new(),
            quantize_view: quantize_view::QuantizeView::new(),
            offload_view: offload_view::OffloadView::new(),
            settings_view: settings_view::SettingsView::new(),
            last_model_path: None,
            last_settings: None,
        }
    }

    /// 应用设置到egui上下文
    fn apply_settings(&self, ctx: &egui::Context, settings: &settings_view::AppSettings) {
        // 应用主题
        let theme = match settings.theme {
            settings_view::Theme::Dark => AnimateTheme::dark(),
            settings_view::Theme::Light => AnimateTheme::light(),
            settings_view::Theme::System => AnimateTheme::dark(), // 默认深色
        };
        theme.apply_to_ctx(ctx);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查设置是否变更
        let current_settings = self.settings_view.get_settings();
        if self.last_settings.as_ref() != Some(&current_settings) {
            self.apply_settings(ctx, &current_settings);
            self.last_settings = Some(current_settings);
        }

        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.current_tab == Tab::Home, "首页")
                    .clicked()
                {
                    self.current_tab = Tab::Home;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::Chat, "对话")
                    .clicked()
                {
                    self.current_tab = Tab::Chat;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::HuggingFace, "HuggingFace")
                    .clicked()
                {
                    self.current_tab = Tab::HuggingFace;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::Quantize, "量化工具")
                    .clicked()
                {
                    self.current_tab = Tab::Quantize;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::Offload, "Offload配置")
                    .clicked()
                {
                    self.current_tab = Tab::Offload;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::Settings, "设置")
                    .clicked()
                {
                    self.current_tab = Tab::Settings;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.current_tab {
            Tab::Home => {
                ui.horizontal(|ui| {
                    ui.heading("首页");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("刷新环境").clicked() {
                            self.home_view.env_view.refresh();
                        }
                    });
                });
                ui.separator();

                // 左右分栏
                ui.columns(2, |columns| {
                    // 左栏：模型管理
                    columns[0].heading("模型管理");
                    columns[0].separator();
                    self.home_view.model_view.show(&mut columns[0]);

                    // 右栏：环境检测
                    columns[1].heading("运行环境");
                    columns[1].separator();
                    self.home_view.env_view.show(&mut columns[1]);
                });

                // 检查模型是否变更，更新offload视图和对话视图
                let current_path = self.home_view.model_view.current_model_path();
                if current_path != self.last_model_path {
                    if let Some(ref name) = self.home_view.model_view.current_model_name() {
                        self.offload_view.set_model_info(name, 32);
                        // 更新对话视图
                        let params = self.home_view.model_view.current_params().clone();
                        self.chat_view.set_model_loaded(name, params);
                    } else {
                        self.offload_view.clear_model_info();
                        self.chat_view.clear_model();
                    }
                    self.last_model_path = current_path;
                }
            }
            Tab::Chat => self.chat_view.show(ui),
            Tab::HuggingFace => self.hf_view.show(ui),
            Tab::Quantize => self.quantize_view.show(ui),
            Tab::Offload => self.offload_view.show(ui),
            Tab::Settings => self.settings_view.show(ui),
        });
    }
}

use crate::views::{env_view, model_view, offload_view, quantize_view, settings_view};
use eframe::egui;

#[derive(PartialEq)]
enum Tab {
    Model,
    Quantize,
    Environment,
    Offload,
    Settings,
}

pub struct App {
    current_tab: Tab,
    model_view: model_view::ModelView,
    quantize_view: quantize_view::QuantizeView,
    env_view: env_view::EnvView,
    offload_view: offload_view::OffloadView,
    settings_view: settings_view::SettingsView,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_tab: Tab::Model,
            model_view: model_view::ModelView::new(),
            quantize_view: quantize_view::QuantizeView::new(),
            env_view: env_view::EnvView::new(),
            offload_view: offload_view::OffloadView::new(),
            settings_view: settings_view::SettingsView::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.current_tab == Tab::Model, "模型管理")
                    .clicked()
                {
                    self.current_tab = Tab::Model;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::Quantize, "量化工具")
                    .clicked()
                {
                    self.current_tab = Tab::Quantize;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::Environment, "环境检测")
                    .clicked()
                {
                    self.current_tab = Tab::Environment;
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
            Tab::Model => self.model_view.show(ui),
            Tab::Quantize => self.quantize_view.show(ui),
            Tab::Environment => self.env_view.show(ui),
            Tab::Offload => self.offload_view.show(ui),
            Tab::Settings => self.settings_view.show(ui),
        });
    }
}

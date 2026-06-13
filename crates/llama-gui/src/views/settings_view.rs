use eframe::egui;

pub struct SettingsView;

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsView {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("设置");
    }
}

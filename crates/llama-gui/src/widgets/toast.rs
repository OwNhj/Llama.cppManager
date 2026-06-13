use eframe::egui;

pub struct Toast {
    pub message: String,
}

impl Toast {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.colored_label(egui::Color32::GREEN, &self.message);
    }
}

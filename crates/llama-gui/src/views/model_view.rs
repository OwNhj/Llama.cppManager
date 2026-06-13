use eframe::egui;

pub struct ModelView {
    pub selected_path: Option<String>,
}

impl Default for ModelView {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelView {
    pub fn new() -> Self {
        Self {
            selected_path: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("模型管理");

        if ui.button("浏览本地模型").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("GGUF", &["gguf"])
                .pick_file()
            {
                self.selected_path = Some(path.display().to_string());
            }
        }

        if let Some(ref path) = self.selected_path {
            ui.label(format!("已选择: {}", path));
        }
    }
}

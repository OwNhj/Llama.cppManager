pub struct Theme {
    pub bg_primary: egui::Color32,
    pub bg_secondary: egui::Color32,
    pub text_primary: egui::Color32,
    pub text_secondary: egui::Color32,
    pub accent: egui::Color32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg_primary: egui::Color32::from_rgb(30, 30, 30),
            bg_secondary: egui::Color32::from_rgb(45, 45, 45),
            text_primary: egui::Color32::from_rgb(240, 240, 240),
            text_secondary: egui::Color32::from_rgb(180, 180, 180),
            accent: egui::Color32::from_rgb(100, 149, 237),
        }
    }
}

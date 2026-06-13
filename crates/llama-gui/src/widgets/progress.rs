use eframe::egui;

pub fn progress_bar(ui: &mut egui::Ui, progress: f32, label: &str) {
    let progress = progress.clamp(0.0, 1.0);
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::ProgressBar::new(progress).animate(true));
    });
}

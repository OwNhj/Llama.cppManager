use eframe::egui;

pub fn param_slider(ui: &mut egui::Ui, label: &str, value: &mut f32, min: f32, max: f32) -> bool {
    ui.horizontal(|ui| {
        ui.label(label);
        let changed = ui
            .add(egui::Slider::new(value, min..=max).show_value(true))
            .changed();
        changed
    })
    .inner
}

use eframe::egui;

pub fn layer_table(ui: &mut egui::Ui, layers: &[String]) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, layer) in layers.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("Layer {}: {}", i, layer));
            });
        }
    });
}

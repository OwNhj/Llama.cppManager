fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("llama.cpp Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "llama.cpp Manager",
        options,
        Box::new(|cc| Ok(Box::new(llama_gui::app::App::new(cc)))),
    )
}

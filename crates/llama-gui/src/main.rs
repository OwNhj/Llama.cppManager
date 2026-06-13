use eframe::egui;

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
        Box::new(|cc| {
            // 配置中文字体
            let mut fonts = egui::FontDefinitions::default();

            // 添加系统中文字体
            // Windows字体路径
            let font_paths = [
                "C:\\Windows\\Fonts\\msyh.ttc",    // 微软雅黑
                "C:\\Windows\\Fonts\\simsun.ttc",   // 宋体
                "C:\\Windows\\Fonts\\simhei.ttf",   // 黑体
                "C:\\Windows\\Fonts\\msyhbd.ttc",   // 微软雅黑粗体
            ];

            for font_path in &font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                    );

                    // 将中文字体添加到proportion字体族
                    if let Some(fonts) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                        fonts.insert(0, "chinese_font".to_owned());
                    }

                    // 将中文字体添加到monospace字体族
                    if let Some(fonts) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                        fonts.insert(0, "chinese_font".to_owned());
                    }

                    break; // 使用找到的第一个字体
                }
            }

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(llama_gui::app::App::new(cc)))
        }),
    )
}

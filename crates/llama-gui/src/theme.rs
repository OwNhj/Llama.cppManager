use eframe::egui::{self, Color32, Rounding, Stroke};

/// 动画风格主题
#[derive(Clone)]
#[allow(dead_code)]
pub struct AnimateTheme {
    /// 背景颜色
    pub background_color: Color32,
    /// 卡片背景颜色
    pub card_color: Color32,
    /// 主色调
    pub primary_color: Color32,
    /// 次要色调
    pub secondary_color: Color32,
    /// 强调色
    pub accent_color: Color32,
    /// 文本颜色
    pub text_color: Color32,
    /// 次要文本颜色
    pub text_secondary_color: Color32,
    /// 边框颜色
    pub border_color: Color32,
    /// 阴影颜色
    pub shadow_color: Color32,
    /// 圆角大小
    pub rounding: Rounding,
    /// 动画速度
    pub animation_speed: f32,
}

impl Default for AnimateTheme {
    fn default() -> Self {
        Self::dark()
    }
}

#[allow(dead_code)]
impl AnimateTheme {
    /// 深色主题
    pub fn dark() -> Self {
        Self {
            background_color: Color32::from_rgb(25, 25, 35),
            card_color: Color32::from_rgb(35, 35, 50),
            primary_color: Color32::from_rgb(100, 149, 237),
            secondary_color: Color32::from_rgb(70, 130, 180),
            accent_color: Color32::from_rgb(255, 165, 0),
            text_color: Color32::from_rgb(240, 240, 240),
            text_secondary_color: Color32::from_rgb(160, 160, 180),
            border_color: Color32::from_rgb(60, 60, 80),
            shadow_color: Color32::from_rgba_premultiplied(0, 0, 0, 50),
            rounding: Rounding::same(8.0),
            animation_speed: 0.15,
        }
    }

    /// 浅色主题
    pub fn light() -> Self {
        Self {
            background_color: Color32::from_rgb(245, 245, 250),
            card_color: Color32::from_rgb(255, 255, 255),
            primary_color: Color32::from_rgb(70, 130, 180),
            secondary_color: Color32::from_rgb(100, 149, 237),
            accent_color: Color32::from_rgb(255, 140, 0),
            text_color: Color32::from_rgb(30, 30, 30),
            text_secondary_color: Color32::from_rgb(100, 100, 120),
            border_color: Color32::from_rgb(220, 220, 230),
            shadow_color: Color32::from_rgba_premultiplied(0, 0, 0, 20),
            rounding: Rounding::same(8.0),
            animation_speed: 0.15,
        }
    }

    /// 应用主题到egui上下文
    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // 设置背景颜色
        let mut visuals = style.visuals.clone();
        visuals.panel_fill = self.background_color;
        visuals.window_fill = self.card_color;
        visuals.extreme_bg_color = self.background_color;
        visuals.faint_bg_color = self.card_color;

        // 设置窗口样式
        visuals.window_rounding = self.rounding;
        visuals.window_shadow = egui::epaint::Shadow::NONE;
        visuals.window_stroke = Stroke::new(1.0, self.border_color);

        // 设置选择框样式
        visuals.selection.bg_fill = self.primary_color.linear_multiply(0.3);
        visuals.selection.stroke = Stroke::new(1.0, self.primary_color);

        // 设置滑块样式
        visuals.widgets.noninteractive.bg_fill = self.card_color;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.noninteractive.rounding = self.rounding;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border_color);

        visuals.widgets.inactive.bg_fill = self.card_color;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.inactive.rounding = self.rounding;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.border_color);

        visuals.widgets.hovered.bg_fill = self.primary_color.linear_multiply(0.2);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.hovered.rounding = self.rounding;
        visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, self.primary_color);

        visuals.widgets.active.bg_fill = self.primary_color.linear_multiply(0.4);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.active.rounding = self.rounding;
        visuals.widgets.active.bg_stroke = Stroke::new(2.0, self.primary_color);

        visuals.widgets.open.bg_fill = self.primary_color.linear_multiply(0.2);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.open.rounding = self.rounding;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, self.primary_color);

        // 设置超链接样式
        visuals.hyperlink_color = self.accent_color;

        // 设置分割线颜色
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border_color);

        // 设置文本编辑框样式 - 确保文字清晰可见
        visuals.widgets.noninteractive.bg_fill = self.background_color;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_color);
        
        visuals.widgets.inactive.bg_fill = self.background_color;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.border_color);
        
        visuals.widgets.hovered.bg_fill = self.background_color;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.primary_color);
        
        visuals.widgets.active.bg_fill = self.background_color;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.active.bg_stroke = Stroke::new(2.0, self.primary_color);

        style.visuals = visuals;

        // 设置字体大小
        let mut text_styles = style.text_styles.clone();
        text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::proportional(20.0),
        );
        text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::proportional(14.0),
        );
        text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::monospace(13.0),
        );
        text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::proportional(14.0),
        );
        text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::proportional(11.0),
        );
        style.text_styles = text_styles;

        ctx.set_style(style);
    }

    /// 绘制卡片背景
    pub fn paint_card(ui: &mut egui::Ui, theme: &AnimateTheme, content: impl FnOnce(&mut egui::Ui)) {
        let frame = egui::Frame::none()
            .fill(theme.card_color)
            .rounding(theme.rounding)
            .stroke(Stroke::new(1.0, theme.border_color))
            .inner_margin(egui::Margin::same(12.0));

        frame.show(ui, |ui| {
            content(ui);
        });
    }

    /// 绘制动画按钮
    pub fn animate_button(
        ui: &mut egui::Ui,
        _text: &str,
        theme: &AnimateTheme,
        response: &egui::Response,
    ) {
        if response.hovered() {
            let rect = response.rect;
            ui.painter().rect_filled(
                rect,
                theme.rounding,
                theme.primary_color.linear_multiply(0.1),
            );
        }
    }
}

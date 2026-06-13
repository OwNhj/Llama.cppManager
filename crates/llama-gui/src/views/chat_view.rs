use eframe::egui;
use llama_core::params::ModelParams;

pub struct ChatView {
    messages: Vec<ChatMessage>,
    input_text: String,
    model_loaded: bool,
    model_name: Option<String>,
    is_generating: bool,
    params: ModelParams,
    is_gguf: bool,
    server_running: bool,
}

#[derive(Debug, Clone)]
struct ChatMessage {
    role: MessageRole,
    content: String,
    timestamp: String,
}

#[derive(Debug, Clone, PartialEq)]
enum MessageRole {
    User,
    Assistant,
    System,
}

impl Default for ChatView {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatView {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_text: String::new(),
            model_loaded: false,
            model_name: None,
            is_generating: false,
            params: ModelParams::default(),
            is_gguf: false,
            server_running: false,
        }
    }

    /// 设置模型信息
    pub fn set_model_loaded(&mut self, name: &str, params: ModelParams, is_gguf: bool) {
        self.model_loaded = true;
        self.model_name = Some(name.to_string());
        self.params = params;
        self.is_gguf = is_gguf;
        
        if is_gguf {
            self.messages.push(ChatMessage {
                role: MessageRole::System,
                content: format!("模型 {} 已加载。请点击\"启动服务器\"开始对话。", name),
                timestamp: Self::current_time(),
            });
        } else {
            self.messages.push(ChatMessage {
                role: MessageRole::System,
                content: format!("模型 {} 是非GGUF格式，需要先导出为GGUF格式才能使用对话功能。", name),
                timestamp: Self::current_time(),
            });
        }
    }

    /// 设置服务器状态
    pub fn set_server_running(&mut self, running: bool) {
        self.server_running = running;
        if running {
            self.messages.push(ChatMessage {
                role: MessageRole::System,
                content: "服务器已启动，可以开始对话。".into(),
                timestamp: Self::current_time(),
            });
        }
    }

    /// 清除模型信息
    pub fn clear_model(&mut self) {
        self.model_loaded = false;
        self.model_name = None;
        self.is_gguf = false;
        self.server_running = false;
        self.messages.clear();
        self.input_text.clear();
    }

    /// 获取当前时间
    fn current_time() -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs();
        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        format!("{:02}:{:02}", hours, minutes)
    }

    /// 发送消息
    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() || !self.model_loaded || !self.server_running {
            return;
        }

        // 添加用户消息
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: self.input_text.clone(),
            timestamp: Self::current_time(),
        });

        let user_input = self.input_text.clone();
        self.input_text.clear();

        // 模拟AI回复（实际应该调用llama-server API）
        self.is_generating = true;
        
        // 这里应该异步调用llama-server API
        // 目前模拟一个回复
        let response = format!(
            "收到您的消息：\"{}\"\n\n注意：此功能需要连接到llama-server。请确保llama-server正在运行。",
            user_input
        );
        
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: response,
            timestamp: Self::current_time(),
        });

        self.is_generating = false;
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("对话");

        if !self.model_loaded {
            ui.separator();
            ui.label("请先在首页加载模型");
            ui.label("加载模型后可以在此进行对话。");
            return;
        }

        // 检查是否是GGUF格式
        if !self.is_gguf {
            ui.separator();
            ui.colored_label(egui::Color32::YELLOW, "⚠ 当前模型不是GGUF格式");
            ui.label("对话功能仅支持GGUF格式模型。");
            ui.label("请先导出模型为GGUF格式，或加载已有的GGUF模型。");
            return;
        }

        // 模型信息
        if let Some(ref name) = self.model_name {
            ui.horizontal(|ui| {
                ui.label("当前模型:");
                ui.strong(name);
                ui.separator();
                ui.label(format!("Temp: {:.2}", self.params.temperature));
                ui.separator();
                ui.label(format!("Top-P: {:.2}", self.params.top_p));
            });
        }

        // 服务器状态
        if !self.server_running {
            ui.separator();
            ui.colored_label(egui::Color32::YELLOW, "⚠ 服务器未运行");
            ui.label("请点击下方按钮启动llama-server");
            if ui.button("启动服务器").clicked() {
                self.set_server_running(true);
            }
            return;
        }

        ui.separator();

        // 消息列表
        let available_height = ui.available_height() - 80.0; // 留出输入框空间
        
        egui::ScrollArea::vertical()
            .max_height(available_height)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in &self.messages {
                    self.show_message(ui, message);
                }
                
                if self.is_generating {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("正在生成回复...");
                    });
                }
            });

        ui.separator();

        // 输入框
        ui.horizontal(|ui| {
            let input_width = ui.available_width() - 100.0;
            
            ui.add_sized(
                [input_width, 30.0],
                egui::TextEdit::singleline(&mut self.input_text)
                    .hint_text("输入消息...")
                    .desired_width(input_width),
            );

            if ui.button("发送").clicked() || 
               (ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.input_text.trim().is_empty()) {
                self.send_message();
            }
        });

        // 状态栏
        ui.horizontal(|ui| {
            ui.label(format!("消息数: {}", self.messages.len()));
            if self.is_generating {
                ui.spinner();
                ui.label("生成中...");
            }
        });
    }

    fn show_message(&self, ui: &mut egui::Ui, message: &ChatMessage) {
        let (bg_color, label) = match message.role {
            MessageRole::User => (egui::Color32::from_rgb(70, 130, 180), "用户"),
            MessageRole::Assistant => (egui::Color32::from_rgb(60, 179, 113), "助手"),
            MessageRole::System => (egui::Color32::from_rgb(128, 128, 128), "系统"),
        };

        ui.horizontal(|ui| {
            // 角色标签
            ui.colored_label(bg_color, format!("[{}]", label));
            ui.label(&message.timestamp);
        });

        // 消息内容
        let frame = egui::Frame::none()
            .fill(egui::Color32::from_rgb(45, 45, 55))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(8.0));

        frame.show(ui, |ui| {
            ui.label(&message.content);
        });

        ui.add_space(4.0);
    }
}

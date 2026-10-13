use crate::assets;
use crate::engine::EngineManager;
use crate::ui::ZapretApp;
use eframe::egui::{self, Color32, CornerRadius, RichText};

impl ZapretApp {
    pub(crate) fn render_logs_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Журнал работы (Логи)");
        ui.label(RichText::new("В реальном времени фиксируются перехваченные пакеты, сигналы DPI и сетевые события").weak());
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.enable_logging, " Включить логирование").changed() {
                self.engine.set_enable_logging(self.enable_logging);
                if self.engine.is_running() {
                    self.engine.restart();
                    self.status_message = if self.enable_logging {
                        "Логирование включено (движок перезапущен)".to_string()
                    } else {
                        "Логирование выключено (движок перезапущен)".to_string()
                    };
                }
            }

            ui.add_space(10.0);

            if ui.button("🔄 Обновить").clicked() {
                self.log_content = EngineManager::read_log_tail(250);
            }

            if ui.button("📂 Папка logs/").clicked() {
                let log_path = assets::get_log_path();
                if let Some(parent) = log_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                    let _ = std::process::Command::new("explorer").arg(parent).spawn();
                }
            }

            if ui.button("🗑️ Очистить лог").clicked() {
                let log_path = assets::get_log_path();
                let _ = std::fs::write(&log_path, "");
                self.log_content = "Лог очищен.".to_string();
            }
        });

        if !self.enable_logging {
            ui.add_space(4.0);
            ui.label(
                RichText::new("ℹ️ Логирование движка winws2 выключено (по умолчанию). Для записи отладочных сообщений в файл включите чекбокс выше.")
                    .weak()
                    .size(12.0),
            );
        }

        ui.add_space(8.0);

        if self.log_content.is_empty() {
            self.log_content = EngineManager::read_log_tail(250);
        }

        egui::Frame::new()
            .fill(Color32::from_rgb(15, 18, 22))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                egui::ScrollArea::both()
                    .max_height(280.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.log_content.as_str())
                                .font(egui::TextStyle::Monospace)
                                .text_color(Color32::from_rgb(180, 220, 180))
                                .desired_width(f32::INFINITY),
                        );
                    });
            });
    }
}

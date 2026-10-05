use crate::assets;
use crate::ui::components::card_frame;
use crate::ui::ZapretApp;
use eframe::egui::{self, Color32, RichText};

impl ZapretApp {
    pub(crate) fn render_lists_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📄 Списки доменов и исключений");
        ui.label(RichText::new("Все списки хранятся в формате обычного текста в папке lists/").weak());
        ui.add_space(10.0);

        let base = assets::resolve_working_directory().join("lists");

        card_frame(ui, |ui| {
            let files = [
                ("hosts-youtube.txt", "Домены YouTube, CDN и превью"),
                ("hosts-discord.txt", "Шлюзы, медиа и голосовые сервера Discord"),
                ("hosts-general.txt", "Игры (Battle.net, GTA5RP, RageMP), VoIP, соцсети и DoH"),
                ("hosts-exclude.txt", "Исключения: Steam, Twitch, банки, госуслуги"),
                ("hosts-user.txt", "Пользовательские домены (для ваших сайтов)"),
            ];

            for (filename, desc) in files {
                let path = base.join(filename);
                let count = if path.exists() {
                    std::fs::read_to_string(&path)
                        .unwrap_or_default()
                        .lines()
                        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                        .count()
                } else {
                    0
                };

                ui.horizontal(|ui| {
                    ui.label(RichText::new(filename).strong());
                    ui.label(RichText::new(format!("({} хостов)", count)).color(Color32::from_rgb(100, 180, 255)));
                    ui.label(RichText::new(format!("— {}", desc)).weak());
                });
            }
        });

        ui.add_space(14.0);

        ui.horizontal(|ui| {
            if ui.button("📂 Открыть папку lists/ в проводнике").clicked() {
                let _ = std::process::Command::new("explorer").arg(&base).spawn();
            }

            if ui.button("🔄 Применить изменения списков (Перезапуск)").clicked() {
                if self.engine.is_running() {
                    self.engine.restart();
                    self.status_message = "Zapret2 перезапущен с обновленными списками!".to_string();
                } else {
                    self.status_message = "Списки будут применены при следующем включении".to_string();
                }
            }
        });
    }
}

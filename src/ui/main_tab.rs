use crate::engine::{EngineManager, Preset};
use crate::ui::components::card_frame;
use crate::ui::ZapretApp;
use eframe::egui::{self, Color32, CornerRadius, RichText, Vec2};

impl ZapretApp {
    pub(crate) fn relaunch_as_admin() {
        if let Ok(exe_path) = std::env::current_exe() {
            let _ = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!("Start-Process -FilePath '{}' -Verb RunAs", exe_path.to_string_lossy()),
                ])
                .spawn();
            std::process::exit(0);
        }
    }

    pub(crate) fn render_main_tab(&mut self, ui: &mut egui::Ui) {
        let is_running = self.engine.is_running();

        // Предупреждение об отсутствии прав Администратора
        if !self.is_admin {
            egui::Frame::new()
                .fill(Color32::from_rgb(60, 20, 20))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("⚠️ Нет прав Администратора. Драйвер WinDivert не сможет запуститься!")
                                .color(Color32::from_rgb(255, 180, 50))
                                .strong(),
                        );
                        if ui.button("🛡️ Перезапустить как Админ").clicked() {
                            Self::relaunch_as_admin();
                        }
                    });
                });
            ui.add_space(8.0);
        }

        // Карточка статуса защиты
        egui::Frame::new()
            .fill(if is_running {
                Color32::from_rgb(20, 50, 30)
            } else {
                Color32::from_rgb(50, 20, 20)
            })
            .corner_radius(CornerRadius::same(10))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    if is_running {
                        ui.label(
                            RichText::new("🟢 ЗАЩИТА АКТИВНА")
                                .size(22.0)
                                .strong()
                                .color(Color32::from_rgb(80, 240, 120)),
                        );
                        ui.label(RichText::new("YouTube 4K, Discord и сайты открываются без ограничений").weak());
                    } else {
                        ui.label(
                            RichText::new("🔴 ОБХОД ОСТАНОВЛЕН")
                                .size(22.0)
                                .strong()
                                .color(Color32::from_rgb(255, 90, 90)),
                        );
                        ui.label(RichText::new("Нажмите кнопку ниже для включения обхода").weak());
                    }

                    ui.add_space(10.0);

                    let (btn_text, btn_color) = if is_running {
                        ("ОСТАНОВИТЬ ОБХОД", Color32::from_rgb(180, 40, 40))
                    } else {
                        ("ВКЛЮЧИТЬ ОБХОД", Color32::from_rgb(40, 160, 60))
                    };

                    let button = egui::Button::new(
                        RichText::new(btn_text)
                            .size(16.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(btn_color)
                    .min_size(Vec2::new(260.0, 44.0))
                    .corner_radius(CornerRadius::same(6));

                    if ui.add(button).clicked() {
                        if is_running {
                            self.engine.stop();
                            self.status_message = "Обход успешно остановлен".to_string();
                        } else if !self.is_admin {
                            self.status_message =
                                "Ошибка: требуются права Администратора для загрузки драйвера!".to_string();
                        } else {
                            match self.engine.start() {
                                Ok(_) => self.status_message = "Обход успешно запущен!".to_string(),
                                Err(err) => self.status_message = err,
                            }
                        }
                    }
                });
            });

        ui.add_space(14.0);

        // Блок выбора режима работы
        card_frame(ui, |ui| {
            ui.label(RichText::new("⚙️ Выбор режима работы:").strong().size(14.0));
            ui.add_space(6.0);

            let mut current = self.engine.current_preset();

            ui.radio_value(&mut current, Preset::AutoAll, Preset::AutoAll.name());
            ui.label(
                RichText::new("   (Рекомендуется: полный комплект + автосмена стратегий при блокировке)")
                    .weak()
                    .size(11.0),
            );

            ui.add_space(4.0);
            ui.radio_value(&mut current, Preset::YouTubeOnly, Preset::YouTubeOnly.name());
            ui.label(
                RichText::new("   (Ультра-легкий профиль, минимальная нагрузка на процессор)")
                    .weak()
                    .size(11.0),
            );

            ui.add_space(4.0);
            ui.radio_value(&mut current, Preset::DiscordOnly, Preset::DiscordOnly.name());
            ui.label(RichText::new("   (Шлюз + голосовые сервера)").weak().size(11.0));

            if current != self.engine.current_preset() {
                self.engine.set_preset(current);
                self.status_message = format!("Выбран режим: {}", current.name());
            }
        });

        ui.add_space(10.0);

        // Блок фоновой службы Windows
        card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut self.autostart_service,
                        " Запускать как службу Windows при включении ПК",
                    )
                    .changed()
                {
                    if self.autostart_service {
                        match self.engine.install_service() {
                            Ok(_) => {
                                self.status_message =
                                    "Служба winws2 успешно установлена в автозагрузку!".to_string();
                            }
                            Err(e) => {
                                self.autostart_service = false;
                                self.status_message = e;
                            }
                        }
                    } else {
                        match EngineManager::remove_service() {
                            Ok(_) => {
                                self.status_message = "Служба winws2 удалена из автозагрузки".to_string();
                            }
                            Err(e) => self.status_message = e,
                        }
                    }
                }
            });
            ui.label(
                RichText::new("Служба работает в фоне даже без входа в аккаунт и не показывает окон командной строки")
                    .weak()
                    .size(11.0),
            );
        });

        ui.add_space(10.0);

        // Блок отладочного логирования
        card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.enable_logging, " Включить логирование движка (отладка)")
                    .changed()
                {
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
            });
            ui.label(
                RichText::new("   (По умолчанию выключено для экономии ресурсов диска и процессора)")
                    .weak()
                    .size(11.0),
            );
        });
    }
}

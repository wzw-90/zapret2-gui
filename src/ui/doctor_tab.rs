use crate::diagnostics::Diagnostics;
use crate::ui::components::card_frame;
use crate::ui::ZapretApp;
use eframe::egui::{self, Color32, RichText};
use std::sync::Arc;

impl ZapretApp {
    pub(crate) fn render_doctor_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔍 Сетевой Доктор (Самодиагностика)");
        ui.label(RichText::new("Проверка системы, совместимости оборудования и доступности сервисов").weak());
        ui.add_space(10.0);

        let in_progress = *self.diagnostic_in_progress.lock().unwrap();

        ui.horizontal_wrapped(|ui| {
            if ui
                .button(RichText::new("🔍 Запустить полную диагностику").strong())
                .clicked()
                && !in_progress
            {
                *self.diagnostic_in_progress.lock().unwrap() = true;
                let flag = Arc::clone(&self.diagnostic_in_progress);
                let rep_dest = Arc::clone(&self.diagnostic_report);

                std::thread::spawn(move || {
                    let report = Diagnostics::run_full_report();
                    *flag.lock().unwrap() = false;
                    *rep_dest.lock().unwrap() = Some(report);
                });
            }

            if ui.button("⚡ Оптимизировать адаптеры (LSO)").clicked() {
                match Diagnostics::optimize_adapters() {
                    Ok(msg) => self.status_message = format!("✓ {}", msg),
                    Err(e) => self.status_message = format!("Ошибка оптимизации: {}", e),
                }
            }

            if ui.button("🛡️ Добавить в исключения Защитника").clicked() {
                match Diagnostics::add_defender_exclusion() {
                    Ok(_) => {
                        self.status_message = "Успешно добавлено в исключения Windows Defender!".to_string();
                    }
                    Err(e) => self.status_message = format!("Ошибка добавления в исключения: {}", e),
                }
            }
        });

        ui.add_space(12.0);

        let report_opt = self.diagnostic_report.lock().unwrap().clone();
        if let Some(report) = report_opt {
            card_frame(ui, |ui| {
                ui.label(RichText::new("Результаты сканирования системы и совместимости:").strong());
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Операционная система:");
                    ui.label(RichText::new(format!("{} ({})", report.os_name, report.architecture)).strong());
                });

                if report.is_arm64 {
                    ui.label(
                        RichText::new(
                            "⚠️ Архитектура ARM64: для запуска драйвера может потребоваться режим Test Signing",
                        )
                        .color(Color32::YELLOW),
                    );
                }

                if report.is_win7_or_8 {
                    ui.horizontal(|ui| {
                        ui.label("Поддержка SHA-256 (Windows 7/8):");
                        if report.win7_sha2_ok == Some(true) {
                            ui.label(RichText::new("✓ Патч KB4474419 установлен").color(Color32::GREEN));
                        } else {
                            ui.label(
                                RichText::new("✗ KB4474419 не найден! Драйвер может блокироваться (ошибка 1275)")
                                    .color(Color32::RED),
                            );
                            if ui.button("🌐 Скачать KB4474419").clicked() {
                                let _ = std::process::Command::new("explorer")
                                    .arg("https://www.catalog.update.microsoft.com/search.aspx?q=kb4474419")
                                    .spawn();
                            }
                        }
                    });
                }

                ui.horizontal(|ui| {
                    ui.label("Изоляция ядра (HVCI / Memory Integrity):");
                    if report.hvci_active {
                        ui.label(
                            RichText::new("✓ Активна (Драйвер WinDivert 2.2 EV совместим)").color(Color32::GREEN),
                        );
                    } else {
                        ui.label(RichText::new("✓ Не активна / Стандартный режим").weak());
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Сетевая разгрузка (LSO / Checksum Offload):");
                    if report.lso_enabled {
                        ui.label(
                            RichText::new(
                                "⚠️ LSO включен (рекомендуется нажать кнопку «Оптимизировать адаптеры»)",
                            )
                            .color(Color32::YELLOW),
                        );
                    } else {
                        ui.label(
                            RichText::new("✓ Оптимизировано (LSO выключен, нет потерь пакетов)")
                                .color(Color32::GREEN),
                        );
                    }
                });

                if !report.active_adapters.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Активные адаптеры:");
                        ui.label(RichText::new(report.active_adapters.join(" | ")).weak());
                    });
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Права Администратора:");
                    if report.is_admin {
                        ui.label(RichText::new("✓ Имеются").color(Color32::GREEN));
                    } else {
                        ui.label(
                            RichText::new("✗ Отсутствуют (Перезапустите от Администратора)").color(Color32::RED),
                        );
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Драйвер WinDivert (Файлы):");
                    if report.windivert_driver_present {
                        ui.label(
                            RichText::new("✓ Найдены (WinDivert.dll + WinDivert64.sys)").color(Color32::GREEN),
                        );
                    } else {
                        ui.label(RichText::new("✗ Не найдены в bin/").color(Color32::RED));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Конфликтные сетевые драйверы:");
                    if report.conflicting_software.is_empty() {
                        ui.label(RichText::new("✓ Конфликтов не обнаружено").color(Color32::GREEN));
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "⚠️ Обнаружены: {}",
                                report.conflicting_software.join(", ")
                            ))
                            .color(Color32::YELLOW),
                        );
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Пинг YouTube (HTTPS):");
                    ui.label(RichText::new(&report.youtube_status).strong());
                    if let Some(ms) = report.youtube_latency_ms {
                        ui.label(RichText::new(format!("({} мс)", ms)).weak());
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Пинг Discord (HTTPS):");
                    ui.label(RichText::new(&report.discord_status).strong());
                    if let Some(ms) = report.discord_latency_ms {
                        ui.label(RichText::new(format!("({} мс)", ms)).weak());
                    }
                });
            });
        } else {
            ui.label(
                RichText::new("Нажмите 'Запустить полную диагностику' для комплексной проверки системы и сети.")
                    .weak(),
            );
        }
    }
}

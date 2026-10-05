pub mod components;
pub mod doctor_tab;
pub mod lists_tab;
pub mod logs_tab;
pub mod main_tab;

use crate::diagnostics::{DiagnosticReport, Diagnostics};
use crate::engine::EngineManager;
use crate::ICON_GREEN_RGBA_32;
use crate::ICON_RGBA_32;
use components::tab_button;
use eframe::egui::{self, Color32, RichText};
use std::sync::{Arc, Mutex};
use tray_icon::{
    menu::{Menu, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

#[derive(PartialEq, Clone, Copy)]
pub enum ActiveTab {
    Main,
    Doctor,
    Lists,
    Logs,
}

pub struct ZapretApp {
    pub(crate) engine: EngineManager,
    pub(crate) active_tab: ActiveTab,
    pub(crate) status_message: String,
    pub(crate) autostart_service: bool,
    pub(crate) enable_logging: bool,
    pub(crate) is_admin: bool,
    pub(crate) log_content: String,
    pub(crate) diagnostic_report: Arc<Mutex<Option<DiagnosticReport>>>,
    pub(crate) diagnostic_in_progress: Arc<Mutex<bool>>,
    pub(crate) tray_icon: Option<TrayIcon>,
    pub(crate) last_tray_status: Option<bool>,
    pub(crate) should_exit: bool,
}

impl ZapretApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let is_admin = Diagnostics::is_admin();
        let mut engine = EngineManager::new();
        let is_service = EngineManager::is_service_installed();
        let enable_logging = engine.enable_logging();
        let is_running = engine.is_running();

        let tray_icon = Self::create_tray_icon(is_running);

        Self {
            engine,
            active_tab: ActiveTab::Main,
            status_message: "Готов к работе".to_string(),
            autostart_service: is_service,
            enable_logging,
            is_admin,
            log_content: String::new(),
            diagnostic_report: Arc::new(Mutex::new(None)),
            diagnostic_in_progress: Arc::new(Mutex::new(false)),
            tray_icon,
            last_tray_status: Some(is_running),
            should_exit: false,
        }
    }

    pub fn create_tray_icon(is_running: bool) -> Option<TrayIcon> {
        let rgba = if is_running {
            ICON_GREEN_RGBA_32.to_vec()
        } else {
            ICON_RGBA_32.to_vec()
        };
        if let Ok(icon) = Icon::from_rgba(rgba, 32, 32) {
            let tray_menu = Menu::new();
            let _ = tray_menu.append(&MenuItem::with_id("open", "Открыть Zapret2", true, None));
            let _ = tray_menu.append(&MenuItem::with_id("exit", "Выход", true, None));

            let tooltip = if is_running {
                "Zapret2: Защита АКТИВНА"
            } else {
                "Zapret2: Обход остановлен"
            };

            TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_menu_on_left_click(false)
                .with_menu_on_right_click(true)
                .with_tooltip(tooltip)
                .with_icon(icon)
                .build()
                .ok()
        } else {
            None
        }
    }
}

impl eframe::App for ZapretApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().set_visuals(egui::Visuals::dark());

        // Периодическое пробуждение для отклика на события трея при скрытом окне
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(150));

        // Если запрошен полный выход из приложения
        if self.should_exit {
            return;
        }

        let is_running = self.engine.is_running();

        // Динамическое переключение цвета иконки в трее (зеленый - активен, синий - остановлен)
        if self.last_tray_status != Some(is_running) {
            self.last_tray_status = Some(is_running);
            let rgba = if is_running {
                ICON_GREEN_RGBA_32.to_vec()
            } else {
                ICON_RGBA_32.to_vec()
            };

            if let Some(tray) = &mut self.tray_icon {
                if let Ok(icon) = Icon::from_rgba(rgba.clone(), 32, 32) {
                    let _ = tray.set_icon(Some(icon));
                }
                let tooltip = if is_running {
                    "Zapret2: Защита АКТИВНА"
                } else {
                    "Zapret2: Обход остановлен"
                };
                let _ = tray.set_tooltip(Some(tooltip));
            }

            let icon_data = egui::IconData {
                rgba,
                width: 32,
                height: 32,
            };
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(icon_data))));
        }

        // Обработка пунктов меню в трее ("Открыть Zapret2", "Выход")
        while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id.as_ref() == "open" {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            } else if event.id.as_ref() == "exit" {
                self.should_exit = true;
                self.engine.stop();
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }

        // Обработка клика по иконке в трее: только двойной клик ЛКМ открывает окно
        while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
            match event {
                tray_icon::TrayIconEvent::DoubleClick { button: tray_icon::MouseButton::Left, .. } => {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                }
                _ => {}
            }
        }

        // Перехват нажатия на крестик (X) окна: отменяем закрытие и прячем окно в трей
        if ui.input(|i| i.viewport().close_requested()) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(false));
            return;
        }

        // Заголовок
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("⚡ Zapret2 Control").strong().color(Color32::from_rgb(100, 160, 255)));
            ui.label(RichText::new("v1.0.5.2").weak());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.is_admin {
                    ui.label(RichText::new("🛡️ Администратор").color(Color32::from_rgb(100, 220, 120)));
                } else {
                    ui.label(RichText::new("⚠️ Обычный пользователь").color(Color32::from_rgb(255, 180, 50)));
                }
            });
        });
        ui.add_space(4.0);

        // Навигация (Segmented Control)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            let tabs = [
                (ActiveTab::Main, "⚡ Главная"),
                (ActiveTab::Doctor, "🔍 Сетевой Доктор"),
                (ActiveTab::Lists, "📄 Списки доменов"),
                (ActiveTab::Logs, "📋 Логи"),
            ];

            for (tab, title) in tabs {
                if tab_button(ui, self.active_tab == tab, title).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();
        ui.add_space(4.0);

        // Вычисляем высоту под контент с резервом под нижнюю панель статуса
        let footer_height = 28.0;
        let content_height = (ui.available_height() - footer_height).max(100.0);

        // Основной контент со скроллом
        egui::ScrollArea::vertical()
            .max_height(content_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                match self.active_tab {
                    ActiveTab::Main => self.render_main_tab(ui),
                    ActiveTab::Doctor => self.render_doctor_tab(ui),
                    ActiveTab::Lists => self.render_lists_tab(ui),
                    ActiveTab::Logs => self.render_logs_tab(ui),
                }
            });

        // Нижняя строка статуса (всегда видна и зафиксирована внизу окна)
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new(&self.status_message).weak().size(12.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("zapret2 github", "https://github.com/bol-van/zapret2");
            });
        });
    }
}

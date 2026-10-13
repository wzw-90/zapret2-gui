#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod cmd_utils;
mod diagnostics;
mod engine;
mod ui;

use diagnostics::Diagnostics;
use ui::ZapretApp;

pub const ICON_RGBA_32: &[u8] = include_bytes!("icon_32.bin");
pub const ICON_GREEN_RGBA_32: &[u8] = include_bytes!("icon_green_32.bin");

fn main() -> eframe::Result {
    // 1. Устанавливаем текущую директорию равной папке исполняемого файла
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let _ = std::env::set_current_dir(parent);
        }
    }

    // 2. Если запуск выполнен без прав Администратора, автоматически запрашиваем UAC повышение
    if !Diagnostics::is_admin() {
        let args: Vec<String> = std::env::args().collect();
        if !args.iter().any(|a| a == "--no-elevation") {
            if let Ok(exe_path) = std::env::current_exe() {
                let status = std::process::Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-Command",
                        &format!("Start-Process -FilePath '{}' -Verb RunAs", exe_path.to_string_lossy()),
                    ])
                    .status();
                if status.map(|s| s.success()).unwrap_or(false) {
                    std::process::exit(0);
                }
            }
        }
    }

    // 3. Подготавливаем рабочее окружение (распаковываем встроенные компоненты при необходимости)
    let _ = assets::resolve_working_directory();

    let icon_data = eframe::egui::IconData {
        rgba: ICON_RGBA_32.to_vec(),
        width: 32,
        height: 32,
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([560.0, 640.0])
            .with_min_inner_size([460.0, 420.0])
            .with_title("Zapret2 Control Panel")
            .with_icon(icon_data)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Zapret2 Control Panel",
        options,
        Box::new(|cc| Ok(Box::new(ZapretApp::new(cc)))),
    )
}

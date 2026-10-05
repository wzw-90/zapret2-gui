use crate::cmd_utils::silent_command;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    AutoAll,     // zapret2.conf (YouTube + Discord + General Auto)
    YouTubeOnly, // youtube_only.conf
    DiscordOnly, // discord_only.conf
}

impl Preset {
    pub fn name(&self) -> &'static str {
        match self {
            Preset::AutoAll => "Автоматический (YouTube + Discord + Сайты)",
            Preset::YouTubeOnly => "Только YouTube (4K без тормозов)",
            Preset::DiscordOnly => "Только Discord (Голос + Шлюз)",
        }
    }

    pub fn config_file(&self) -> &'static str {
        match self {
            Preset::AutoAll => "config/zapret2.conf",
            Preset::YouTubeOnly => "config/youtube_only.conf",
            Preset::DiscordOnly => "config/discord_only.conf",
        }
    }
}

pub struct EngineManager {
    child: Option<Child>,
    current_preset: Preset,
    running: Arc<AtomicBool>,
    last_status_check: Instant,
    last_known_status: bool,
    enable_logging: bool,
}

impl EngineManager {
    pub fn new() -> Self {
        let is_running = Self::is_service_running() || Self::is_process_running();
        Self {
            child: None,
            current_preset: Preset::AutoAll,
            running: Arc::new(AtomicBool::new(is_running)),
            last_status_check: Instant::now(),
            last_known_status: is_running,
            enable_logging: false,
        }
    }

    pub fn enable_logging(&self) -> bool {
        self.enable_logging
    }

    pub fn set_enable_logging(&mut self, enable: bool) {
        self.enable_logging = enable;
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(None) => return true,
                Ok(Some(_)) => {
                    self.child = None;
                }
                Err(_) => {
                    self.child = None;
                }
            }
        }

        // Проверяем внешнюю службу/процесс не чаще 1 раза в 2 секунды, чтобы не грузить систему и не вызывать консоль
        if self.last_status_check.elapsed().as_millis() > 2000 {
            self.last_status_check = Instant::now();
            self.last_known_status = Self::is_service_running() || Self::is_process_running();
        }

        self.last_known_status
    }

    pub fn current_preset(&self) -> Preset {
        self.current_preset
    }

    pub fn set_preset(&mut self, preset: Preset) {
        if self.current_preset != preset {
            self.current_preset = preset;
            if self.is_running() {
                self.restart();
            }
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        self.stop();

        let base_dir = crate::assets::resolve_working_directory();
        let bin_path = base_dir.join("bin").join("winws2.exe");
        let conf_path = base_dir.join(self.current_preset.config_file());

        if !bin_path.exists() {
            return Err(format!("Файл winws2.exe не найден по пути {:?}", bin_path));
        }
        if !conf_path.exists() {
            return Err(format!("Конфигурационный файл не найден: {:?}", conf_path));
        }

        let arg_conf = format!("@{}", conf_path.to_string_lossy().replace('\\', "/"));

        let mut cmd = silent_command(bin_path.to_string_lossy().as_ref());
        cmd.current_dir(&base_dir);

        if self.enable_logging {
            let log_path = crate::assets::get_log_path();
            if let Some(parent) = log_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let log_file_arg = format!("--debug=@{}", log_path.to_string_lossy().replace('\\', "/"));
            cmd.arg(&log_file_arg);

            if let Ok(file_out) = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&log_path) {
                if let Ok(file_err) = file_out.try_clone() {
                    cmd.stdout(std::process::Stdio::from(file_out));
                    cmd.stderr(std::process::Stdio::from(file_err));
                }
            }
        } else {
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());
        }

        cmd.arg(&arg_conf);

        match cmd.spawn() {
            Ok(mut child) => {
                // Даем 350 мс на инициализацию WinDivert и проверяем, не завершился ли процесс
                std::thread::sleep(std::time::Duration::from_millis(350));
                match child.try_wait() {
                    Ok(Some(status)) => {
                        self.running.store(false, Ordering::SeqCst);
                        self.last_known_status = false;
                        let code = status.code().unwrap_or(-1);
                        if code == 5 {
                            Err("Ошибка: доступ запрещен (код 5). Запустите приложение от Администратора!".to_string())
                        } else {
                            Err(format!("Процесс winws2 завершился с ошибкой (код {}). Проверьте вкладку 'Логи'!", code))
                        }
                    }
                    Ok(None) => {
                        self.child = Some(child);
                        self.running.store(true, Ordering::SeqCst);
                        self.last_known_status = true;
                        self.last_status_check = Instant::now();
                        Ok(())
                    }
                    Err(e) => Err(format!("Ошибка мониторинга процесса: {}", e)),
                }
            }
            Err(e) => Err(format!("Не удалось запустить winws2.exe: {}", e)),
        }
    }

    pub fn read_log_tail(lines_count: usize) -> String {
        let log_path = crate::assets::get_log_path();
        if let Ok(content) = std::fs::read_to_string(&log_path) {
            let lines: Vec<&str> = content.lines().collect();
            if lines.is_empty() {
                "Журнал пуст. Включите логирование и запустите обход для записи логов.".to_string()
            } else if lines.len() > lines_count {
                lines[lines.len() - lines_count..].join("\n")
            } else {
                content
            }
        } else {
            "Файл логов winws2.log еще не создан (логирование выключено). Отметьте чекбокс «Включить логирование».".to_string()
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Останавливаем процесс winws2, службу и выгружаем драйвер WinDivert из памяти ядра
        let _ = silent_command("taskkill")
            .args(["/F", "/IM", "winws2.exe"])
            .output();

        let _ = silent_command("net")
            .args(["stop", "winws2"])
            .output();

        let _ = silent_command("net")
            .args(["stop", "WinDivert"])
            .output();

        self.running.store(false, Ordering::SeqCst);
        self.last_known_status = false;
        self.last_status_check = Instant::now();
    }

    pub fn restart(&mut self) {
        self.stop();
        std::thread::sleep(std::time::Duration::from_millis(300));
        let _ = self.start();
    }

    pub fn is_service_running() -> bool {
        if let Ok(output) = silent_command("sc").args(["query", "winws2"]).output() {
            let s = String::from_utf8_lossy(&output.stdout);
            s.contains("RUNNING")
        } else {
            false
        }
    }

    pub fn is_process_running() -> bool {
        if let Ok(output) = silent_command("tasklist")
            .args(["/FI", "IMAGENAME eq winws2.exe"])
            .output()
        {
            let s = String::from_utf8_lossy(&output.stdout);
            s.contains("winws2.exe")
        } else {
            false
        }
    }

    pub fn is_service_installed() -> bool {
        if let Ok(output) = silent_command("sc").args(["query", "winws2"]).output() {
            let s = String::from_utf8_lossy(&output.stdout);
            !s.contains("1060") // 1060 = ERROR_SERVICE_DOES_NOT_EXIST
        } else {
            false
        }
    }

    pub fn install_service(&self) -> Result<(), String> {
        let base_dir = crate::assets::resolve_working_directory();
        let bin_path = base_dir.join("bin").join("winws2.exe");
        let conf_path = base_dir.join(self.current_preset.config_file());

        if !bin_path.exists() {
            return Err(format!("Файл winws2.exe не найден: {:?}", bin_path));
        }
        if !conf_path.exists() {
            return Err(format!("Файл конфигурации не найден: {:?}", conf_path));
        }

        // Если служба уже существовала или была запущена - останавливаем и удаляем
        let _ = silent_command("cmd").args(["/c", "net stop winws2 >nul 2>&1"]).output();
        let _ = silent_command("cmd").args(["/c", "sc delete winws2 >nul 2>&1"]).output();
        std::thread::sleep(std::time::Duration::from_millis(200));

        let log_arg = if self.enable_logging {
            let log_path = crate::assets::get_log_path();
            if let Some(parent) = log_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            format!(" --debug=@\"{}\"", log_path.to_string_lossy().replace('\\', "/"))
        } else {
            String::new()
        };

        let binary_path = format!(
            "\"{}\" --chdir=\"{}\"{} @\"{}\"",
            bin_path.to_string_lossy(),
            base_dir.to_string_lossy(),
            log_arg,
            conf_path.to_string_lossy()
        );

        let ps_script = format!(
            "New-Service -Name 'winws2' -BinaryPathName '{}' -DisplayName 'Zapret2 DPI Bypass Service' -StartupType Automatic -Description 'Фоновая служба zapret2 для обхода блокировок YouTube и Discord'",
            binary_path
        );

        let create = silent_command("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .output()
            .map_err(|e| e.to_string())?;

        if !create.status.success() {
            let err_out = String::from_utf8_lossy(&create.stderr);
            let std_out = String::from_utf8_lossy(&create.stdout);
            let msg = if !err_out.trim().is_empty() {
                err_out.trim().to_string()
            } else if !std_out.trim().is_empty() {
                std_out.trim().to_string()
            } else {
                "Не удалось создать службу".to_string()
            };
            return Err(format!("Ошибка службы: {}", msg));
        }

        let _ = silent_command("cmd").args(["/c", "sc start winws2"]).output();

        Ok(())
    }

    pub fn remove_service() -> Result<(), String> {
        let _ = silent_command("cmd").args(["/c", "net stop winws2 >nul 2>&1"]).output();
        let out = silent_command("cmd")
            .args(["/c", "sc delete winws2"])
            .output()
            .map_err(|e| e.to_string())?;

        if out.status.success() {
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            let out_str = String::from_utf8_lossy(&out.stdout);
            let msg = if !err.trim().is_empty() {
                err.trim()
            } else {
                out_str.trim()
            };
            Err(format!("Не удалось удалить службу: {}", msg))
        }
    }
}

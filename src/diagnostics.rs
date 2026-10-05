use crate::cmd_utils::silent_command;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub is_admin: bool,
    pub os_name: String,
    pub architecture: String,
    pub is_arm64: bool,
    pub is_win7_or_8: bool,
    pub win7_sha2_ok: Option<bool>,
    pub hvci_active: bool,
    pub lso_enabled: bool,
    pub active_adapters: Vec<String>,
    pub windivert_driver_present: bool,
    pub conflicting_software: Vec<String>,
    pub youtube_status: String,
    pub youtube_latency_ms: Option<u128>,
    pub discord_status: String,
    pub discord_latency_ms: Option<u128>,
}

pub struct Diagnostics;

impl Diagnostics {
    pub fn is_admin() -> bool {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = silent_command("net").arg("session").output() {
                output.status.success()
            } else {
                false
            }
        }
        #[cfg(not(target_os = "windows"))]
        true
    }

    pub fn detect_os_and_arch() -> (String, String, bool, bool, Option<bool>) {
        let arch = std::env::var("PROCESSOR_ARCHITEW6432")
            .or_else(|_| std::env::var("PROCESSOR_ARCHITECTURE"))
            .unwrap_or_else(|_| "x64".to_string());
        let is_arm64 = arch.to_uppercase().contains("ARM64");

        let mut os_name = "Windows".to_string();
        let mut is_win7_or_8 = false;
        let mut win7_sha2_ok = None;

        // Получаем информацию об ОС через реестр Windows бесшумно
        let ps_script = "(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion').ProductName + ' ' + (Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion').DisplayVersion";
        if let Ok(output) = silent_command("powershell")
            .args(["-NoProfile", "-Command", ps_script])
            .output()
        {
            let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !raw.is_empty() {
                os_name = raw;
            }
        }

        // Проверяем версию ядра
        let ver_script = "[System.Environment]::OSVersion.Version.Major.ToString() + '.' + [System.Environment]::OSVersion.Version.Minor.ToString()";
        if let Ok(output) = silent_command("powershell")
            .args(["-NoProfile", "-Command", ver_script])
            .output()
        {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if ver.starts_with("6.") {
                is_win7_or_8 = true;
                // Проверяем наличие KB4474419 для SHA-256
                let kb_check = "if (Get-HotFix -Id 'KB4474419' -ErrorAction SilentlyContinue) { 'INSTALLED' } else { 'MISSING' }";
                if let Ok(kb_out) = silent_command("powershell")
                    .args(["-NoProfile", "-Command", kb_check])
                    .output()
                {
                    let kb_str = String::from_utf8_lossy(&kb_out.stdout);
                    win7_sha2_ok = Some(kb_str.contains("INSTALLED"));
                }
            }
        }

        (os_name, arch, is_arm64, is_win7_or_8, win7_sha2_ok)
    }

    pub fn check_hvci() -> bool {
        let ps_cmd = "(Get-CimInstance -ClassName Win32_DeviceGuard -Namespace root\\Microsoft\\Windows\\DeviceGuard -ErrorAction SilentlyContinue).SecurityServicesRunning -contains 2";
        if let Ok(output) = silent_command("powershell")
            .args(["-NoProfile", "-Command", ps_cmd])
            .output()
        {
            String::from_utf8_lossy(&output.stdout).to_lowercase().contains("true")
        } else {
            false
        }
    }

    pub fn check_lso_and_adapters() -> (bool, Vec<String>) {
        let mut lso_enabled = false;
        let mut adapters = Vec::new();

        // Проверка LSO
        let lso_cmd = "(Get-NetAdapterLso -ErrorAction SilentlyContinue | Where-Object { $_.IPv4Enabled -eq $true -or $_.IPv6Enabled -eq $true }).Count -gt 0";
        if let Ok(output) = silent_command("powershell")
            .args(["-NoProfile", "-Command", lso_cmd])
            .output()
        {
            if String::from_utf8_lossy(&output.stdout).to_lowercase().contains("true") {
                lso_enabled = true;
            }
        }

        // Список активных адаптеров
        let adapt_cmd = "Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object { $_.Status -eq 'Up' } | ForEach-Object { $_.Name + ' (' + $_.InterfaceDescription + ')' }";
        if let Ok(output) = silent_command("powershell")
            .args(["-NoProfile", "-Command", adapt_cmd])
            .output()
        {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    adapters.push(trimmed.to_string());
                }
            }
        }

        (lso_enabled, adapters)
    }

    pub fn optimize_adapters() -> Result<String, String> {
        let ps_cmd = "Disable-NetAdapterLso -Name * -IPv4 -IPv6 -ErrorAction SilentlyContinue; netsh int tcp set global timestamps=enabled | Out-Null; netsh int tcp set global autotuninglevel=normal | Out-Null";
        let output = silent_command("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", ps_cmd])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok("LSO успешно отключен, стек TCP/IP оптимизирован!".to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn check_conflicts() -> Vec<String> {
        let mut conflicts = Vec::new();
        let suspicious_services = [
            ("Killer Network Service", "KNDBFR"),
            ("CFosSpeed Traffic Shaper", "cfosspeed"),
            ("ASUS ROG GameFirst", "GameFirst"),
            ("SmartByte Network Service", "SmartByte"),
        ];

        for (name, service_name) in suspicious_services {
            if let Ok(output) = silent_command("sc").args(["query", service_name]).output() {
                let s = String::from_utf8_lossy(&output.stdout);
                if s.contains("RUNNING") || s.contains("STOPPED") {
                    conflicts.push(name.to_string());
                }
            }
        }

        // Проверяем запущенные VPN / Прокси клиенты, которые могут перехватывать сетевой трафик
        let suspicious_processes = [
            ("Xray Core / V2Ray", "xray"),
            ("Happ Proxy (TUN/VPN)", "happ"),
            ("Sing-Box Core", "sing-box"),
            ("NekoRay / NekoBox", "nekoray"),
            ("Clash VPN/Proxy", "clash"),
        ];

        if let Ok(output) = silent_command("tasklist").output() {
            let s = String::from_utf8_lossy(&output.stdout).to_lowercase();
            for (display, proc_name) in suspicious_processes {
                if s.contains(proc_name) {
                    conflicts.push(format!("{} (активен)", display));
                }
            }
        }

        conflicts
    }

    pub fn check_windivert_files() -> bool {
        let base = std::env::current_dir().unwrap_or_default();
        let dll = base.join("bin").join("WinDivert.dll");
        let sys = base.join("bin").join("WinDivert64.sys");
        dll.exists() && sys.exists()
    }

    pub fn add_defender_exclusion() -> Result<(), String> {
        let dir = std::env::current_dir().map_err(|e| e.to_string())?;
        let path_str = dir.to_string_lossy().to_string();

        let ps_cmd = format!(
            "Add-MpPreference -ExclusionPath '{}'; Add-MpPreference -ExclusionProcess 'winws2.exe'",
            path_str
        );

        let output = silent_command("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn run_full_report() -> DiagnosticReport {
        let is_admin = Self::is_admin();
        let (os_name, architecture, is_arm64, is_win7_or_8, win7_sha2_ok) = Self::detect_os_and_arch();
        let hvci_active = Self::check_hvci();
        let (lso_enabled, active_adapters) = Self::check_lso_and_adapters();
        let windivert_driver_present = Self::check_windivert_files();
        let conflicting_software = Self::check_conflicts();

        // Тест доступности YouTube
        let (youtube_status, youtube_latency_ms) = Self::test_http("https://www.youtube.com");

        // Тест доступности Discord
        let (discord_status, discord_latency_ms) = Self::test_http("https://discord.com");

        DiagnosticReport {
            is_admin,
            os_name,
            architecture,
            is_arm64,
            is_win7_or_8,
            win7_sha2_ok,
            hvci_active,
            lso_enabled,
            active_adapters,
            windivert_driver_present,
            conflicting_software,
            youtube_status,
            youtube_latency_ms,
            discord_status,
            discord_latency_ms,
        }
    }

    fn test_http(url: &str) -> (String, Option<u128>) {
        let start = Instant::now();
        let config = ureq::config::Config::builder()
            .timeout_global(Some(std::time::Duration::from_secs(7)))
            .build();
        let agent = ureq::Agent::new_with_config(config);

        match agent.get(url).call() {
            Ok(response) => {
                let ms = start.elapsed().as_millis();
                let status = format!("Доступен (HTTP {})", response.status().as_u16());
                (status, Some(ms))
            }
            Err(e) => {
                let ms = start.elapsed().as_millis();
                (format!("Сбой: {}", e), Some(ms))
            }
        }
    }
}

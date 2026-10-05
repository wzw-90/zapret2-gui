// Auto-generated embedded assets module for Zapret2 single-file binary
use std::path::{Path, PathBuf};

pub struct EmbeddedFile {
    pub path: &'static str,
    pub content: &'static [u8],
}

pub const EMBEDDED_FILES: &[EmbeddedFile] = &[
    EmbeddedFile { path: "bin/WinDivert.dll", content: include_bytes!("../bin/WinDivert.dll") },
    EmbeddedFile { path: "bin/WinDivert64.sys", content: include_bytes!("../bin/WinDivert64.sys") },
    EmbeddedFile { path: "bin/blobs/ACTIVE_DISCORD_UDP.bin", content: include_bytes!("../bin/blobs/ACTIVE_DISCORD_UDP.bin") },
    EmbeddedFile { path: "bin/blobs/ACTIVE_GAME_UDP.bin", content: include_bytes!("../bin/blobs/ACTIVE_GAME_UDP.bin") },
    EmbeddedFile { path: "bin/blobs/quic_initial_4pda_to.bin", content: include_bytes!("../bin/blobs/quic_initial_4pda_to.bin") },
    EmbeddedFile { path: "bin/blobs/quic_initial_5ka_ru.bin", content: include_bytes!("../bin/blobs/quic_initial_5ka_ru.bin") },
    EmbeddedFile { path: "bin/blobs/quic_initial_rutube_ru.bin", content: include_bytes!("../bin/blobs/quic_initial_rutube_ru.bin") },
    EmbeddedFile { path: "bin/blobs/quic_initial_steamcommunity_com.bin", content: include_bytes!("../bin/blobs/quic_initial_steamcommunity_com.bin") },
    EmbeddedFile { path: "bin/blobs/quic_initial_tencent_com.bin", content: include_bytes!("../bin/blobs/quic_initial_tencent_com.bin") },
    EmbeddedFile { path: "bin/blobs/quic_initial_www_google_com.bin", content: include_bytes!("../bin/blobs/quic_initial_www_google_com.bin") },
    EmbeddedFile { path: "bin/blobs/stun.bin", content: include_bytes!("../bin/blobs/stun.bin") },
    EmbeddedFile { path: "bin/blobs/stun2.bin", content: include_bytes!("../bin/blobs/stun2.bin") },
    EmbeddedFile { path: "bin/blobs/tls_clienthello_4pda_to.bin", content: include_bytes!("../bin/blobs/tls_clienthello_4pda_to.bin") },
    EmbeddedFile { path: "bin/blobs/tls_clienthello_5ka_ru.bin", content: include_bytes!("../bin/blobs/tls_clienthello_5ka_ru.bin") },
    EmbeddedFile { path: "bin/blobs/tls_clienthello_max_ru.bin", content: include_bytes!("../bin/blobs/tls_clienthello_max_ru.bin") },
    EmbeddedFile { path: "bin/blobs/tls_clienthello_sochi_park.bin", content: include_bytes!("../bin/blobs/tls_clienthello_sochi_park.bin") },
    EmbeddedFile { path: "bin/blobs/tls_clienthello_www_google_com.bin", content: include_bytes!("../bin/blobs/tls_clienthello_www_google_com.bin") },
    EmbeddedFile { path: "bin/blobs/tls_clienthello_www_sferum_ru.bin", content: include_bytes!("../bin/blobs/tls_clienthello_www_sferum_ru.bin") },
    EmbeddedFile { path: "bin/cygwin1.dll", content: include_bytes!("../bin/cygwin1.dll") },
    EmbeddedFile { path: "bin/ip2net.exe", content: include_bytes!("../bin/ip2net.exe") },
    EmbeddedFile { path: "bin/killall.exe", content: include_bytes!("../bin/killall.exe") },
    EmbeddedFile { path: "bin/mdig.exe", content: include_bytes!("../bin/mdig.exe") },
    EmbeddedFile { path: "bin/winws.exe", content: include_bytes!("../bin/winws.exe") },
    EmbeddedFile { path: "bin/winws2.exe", content: include_bytes!("../bin/winws2.exe") },
    EmbeddedFile { path: "config/discord_only.conf", content: include_bytes!("../config/discord_only.conf") },
    EmbeddedFile { path: "config/youtube_only.conf", content: include_bytes!("../config/youtube_only.conf") },
    EmbeddedFile { path: "config/zapret2.conf", content: include_bytes!("../config/zapret2.conf") },
    EmbeddedFile { path: "lists/hosts-discord.txt", content: include_bytes!("../lists/hosts-discord.txt") },
    EmbeddedFile { path: "lists/hosts-exclude.txt", content: include_bytes!("../lists/hosts-exclude.txt") },
    EmbeddedFile { path: "lists/hosts-general.txt", content: include_bytes!("../lists/hosts-general.txt") },
    EmbeddedFile { path: "lists/hosts-user.txt", content: include_bytes!("../lists/hosts-user.txt") },
    EmbeddedFile { path: "lists/hosts-youtube.txt", content: include_bytes!("../lists/hosts-youtube.txt") },
    EmbeddedFile { path: "lua/zapret-antidpi.lua", content: include_bytes!("../lua/zapret-antidpi.lua") },
    EmbeddedFile { path: "lua/zapret-auto.lua", content: include_bytes!("../lua/zapret-auto.lua") },
    EmbeddedFile { path: "lua/zapret-lib.lua", content: include_bytes!("../lua/zapret-lib.lua") },
    EmbeddedFile { path: "lua/zapret-obfs.lua", content: include_bytes!("../lua/zapret-obfs.lua") },
    EmbeddedFile { path: "lua/zapret-pcap.lua", content: include_bytes!("../lua/zapret-pcap.lua") },
    EmbeddedFile { path: "lua/zapret-tests.lua", content: include_bytes!("../lua/zapret-tests.lua") },
    EmbeddedFile { path: "utils/optimize_network_adapters.cmd", content: include_bytes!("../utils/optimize_network_adapters.cmd") },
];

pub fn get_exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

pub fn get_log_path() -> PathBuf {
    get_exe_dir().join("logs").join("winws2.log")
}

pub fn resolve_working_directory() -> PathBuf {
    let exe_dir = get_exe_dir();
    
    // 1. If bin/winws2.exe and config/zapret2.conf exist next to the executable, use local portable folder
    if exe_dir.join("bin").join("winws2.exe").exists() && exe_dir.join("config").join("zapret2.conf").exists() {
        return exe_dir;
    }

    // 2. Otherwise, use %LOCALAPPDATA%\Zapret2 for self-contained single-file mode
    let target_dir = if let Ok(local) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local).join("Zapret2")
    } else if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("Zapret2")
    } else {
        std::env::temp_dir().join("Zapret2")
    };

    unpack_embedded_assets(&target_dir);
    target_dir
}

pub fn unpack_embedded_assets(target_dir: &Path) {
    let _ = std::fs::create_dir_all(target_dir);

    for file in EMBEDDED_FILES {
        let dest = target_dir.join(file.path);
        if let Some(parent) = dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let should_write = if dest.exists() {
            if file.path.starts_with("lists/") {
                // Preserve user-edited hostlists
                false
            } else if let Ok(meta) = dest.metadata() {
                meta.len() != file.content.len() as u64
            } else {
                true
            }
        } else {
            true
        };

        if should_write {
            let _ = std::fs::write(&dest, file.content);
        }
    }
}

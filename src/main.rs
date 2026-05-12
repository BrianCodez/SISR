#![windows_subsystem = "windows"]

use std::{env, process::ExitCode};

use sisr::{
    app::{runner::AppRunner, steam},
    config::{CONFIG, Config},
    logging,
};

fn main() -> ExitCode {
    logging::setup();

    #[cfg(target_os = "linux")]
    if !setup_linux_webview_backend() {
        return ExitCode::from(1);
    }

    #[cfg(windows)]
    {
        sisr::win_console::alloc();
    }

    unsafe {
        // TODO: does this do anything?
        env::set_var("SteamStreamingVideo", "0");
        env::set_var("SteamStreaming", "0");

        env::set_var("SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD", "1");
        env::set_var("SDL_JOYSTICK_HIDAPI_STEAMXBOX", "1");
        // this specific SDL_Hint doesn't work when Steam is injected.
        // Envar does...
        env::set_var("SDL_GAMECONTROLLER_IGNORE_DEVICES", "");
        env::set_var("SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT", "");
    }

    let config = Config::parse();
    *CONFIG.write().unwrap() = Some(config.clone());

    logging::set_level(config.log.level.as_ref().unwrap().parse().unwrap());

    if let Some(log_file) = &config.log.log_file
        && let Some(path) = &log_file.path
    {
        match log_file
            .file_level
            .as_ref()
            .unwrap_or(&config.log.level.as_ref().unwrap().parse().unwrap())
            .parse()
        {
            Ok(level) => logging::add_file(path, level),
            Err(e) => {
                tracing::error!("Failed to parse log file level: {}", e);
            }
        }
    }
    tracing::trace!("merged config: {:?}", config);

    tracing::trace!(
        viiper_min_version = sisr::viiper_metadata::VIIPER_MIN_VERSION,
        viiper_allow_dev = sisr::viiper_metadata::VIIPER_ALLOW_DEV,
        viiper_fetch_prelease = sisr::viiper_metadata::VIIPER_FETCH_PRELEASE,
        "VIIPER metadata"
    );

    tracing::trace!("Environment variables:");
    for (key, value) in env::vars() {
        tracing::trace!("  {}={}", key, value);
    }

    #[cfg(windows)]
    {
        if config.console.unwrap_or(false) {
            sisr::win_console::show();
        }
    }

    // just fill onceLock if we are started via Steam or not.
    steam::util::init();

    let mut app = AppRunner::new();
    let res = app.run();

    #[cfg(windows)]
    {
        sisr::win_console::cleanup();
    }

    res
}

#[cfg(target_os = "linux")]
fn setup_linux_webview_backend() -> bool {
    fn can_open_x11_display(display: Option<&str>) -> bool {
        x11rb::connect(display).is_ok()
    }

    fn x11_socket_candidates() -> Vec<String> {
        use std::path::Path;

        let mut out = Vec::new();
        let x11_sock_dir = Path::new("/tmp/.X11-unix");
        if let Ok(entries) = std::fs::read_dir(x11_sock_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let Some(name) = name.to_str() else {
                    continue;
                };
                if !name.starts_with('X') {
                    continue;
                }

                let display_num = &name[1..];
                if display_num.is_empty() || !display_num.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }
                out.push(format!(":{display_num}"));
            }
        }

        out.sort();
        out.reverse();
        out
    }

    let wayland = env::var_os("WAYLAND_DISPLAY").is_some();
    if !wayland {
        return true;
    }

    let mut candidates: Vec<String> = Vec::new();
    if let Ok(display) = env::var("DISPLAY") {
        candidates.push(display);
    }
    if let Ok(xwayland_display) = env::var("XWAYLAND_DISPLAY")
        && !candidates.iter().any(|c| c == &xwayland_display)
    {
        candidates.push(xwayland_display);
    }
    for candidate in x11_socket_candidates() {
        if !candidates.iter().any(|c| c == &candidate) {
            candidates.push(candidate);
        }
    }

    let usable_display = candidates
        .iter()
        .find(|candidate| can_open_x11_display(Some(candidate.as_str())))
        .cloned();

    let Some(x11_display) = usable_display else {
        let current_display = env::var("DISPLAY").unwrap_or_else(|_| "<unset>".to_string());
        let current_xwayland_display =
            env::var("XWAYLAND_DISPLAY").unwrap_or_else(|_| "<unset>".to_string());
        tracing::error!(
            "Wayland session detected, but no authorized X11 display is accessible. \
             DISPLAY={}, XWAYLAND_DISPLAY={}. \
             SISR requires XWayland for the integrated webview.",
            current_display,
            current_xwayland_display
        );
        return false;
    };

    unsafe {
        env::set_var("GDK_BACKEND", "x11");
        env::set_var("WINIT_UNIX_BACKEND", "x11");
        env::set_var("DISPLAY", &x11_display);
    }
    tracing::info!(
        "Wayland session detected; enabling automatic X11 compatibility mode (DISPLAY={})",
        x11_display
    );

    true
}

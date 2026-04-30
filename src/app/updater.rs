use std::sync::Mutex;

use tracing::{error, info, warn};

use crate::config::{self, UpdateNotify};

use super::gui::dialogs::{self, Dialog};

static REMIND_LATER_VERSION: Mutex<Option<String>> = Mutex::new(None);

macro_rules! current_version {
    () => {
        match option_env!("SISR_VERSION") {
            Some(v) => v,
            None => env!("CARGO_PKG_VERSION"),
        }
    };
}
const CURRENT_VERSION: &str = current_version!();

#[derive(Debug)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    commits: u64,
}

fn parse_version(s: &str) -> Option<Version> {
    let s = s.trim().strip_prefix('v').unwrap_or(s);

    let mut main_and_rest = s.splitn(2, '-');
    let main_part = main_and_rest.next()?;

    let commits_part = main_and_rest.next().and_then(|rest| {
        let first_segment = rest.split('-').next()?;
        first_segment.parse::<u64>().ok()
    });

    let mut parts = main_part.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;

    Some(Version {
        major,
        minor,
        patch,
        commits: commits_part.unwrap_or(0),
    })
}

impl Version {
    fn is_newer_than(&self, other: &Version) -> bool {
        (self.major, self.minor, self.patch, self.commits)
            > (other.major, other.minor, other.patch, other.commits)
    }
}

fn dismissed_file_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("", "", "SISR")
        .map(|proj| proj.data_dir().join("update-dismissed"))
}

fn is_dismissed(ver: &str) -> bool {
    let Some(path) = dismissed_file_path() else {
        return false;
    };
    std::fs::read_to_string(&path)
        .map(|content| content.trim() == ver)
        .unwrap_or(false)
}

fn write_dismissed(ver: &str) {
    let Some(path) = dismissed_file_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&path, ver) {
        error!("Failed to write update-dismissed file: {}", e);
    }
}

fn is_remind_later(ver: &str) -> bool {
    REMIND_LATER_VERSION
        .lock()
        .ok()
        .and_then(|g| g.as_deref().map(|v| v == ver))
        .unwrap_or(false)
}

fn set_remind_later(ver: &str) {
    if let Ok(mut g) = REMIND_LATER_VERSION.lock() {
        *g = Some(ver.to_string());
    }
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    prerelease: bool,
    html_url: String,
}

pub async fn check_update() {
    let notify = config::get_config()
        .update_notify
        .unwrap_or(UpdateNotify::Stable);

    if notify == UpdateNotify::None {
        return;
    }

    let cur = match parse_version(CURRENT_VERSION) {
        Some(v) => v,
        None => {
            if CURRENT_VERSION != "dev" {
                warn!("Failed to parse current SISR version: {}", CURRENT_VERSION);
            }
            return;
        }
    };

    let release = match fetch_release(notify).await {
        Some(r) => r,
        None => return,
    };

    let version_source = if release.prerelease {
        release.name.as_deref().unwrap_or(&release.tag_name)
    } else {
        &release.tag_name
    };

    let remote = match parse_version(version_source) {
        Some(v) => v,
        None => {
            warn!("Failed to parse remote version: {}", version_source);
            return;
        }
    };

    if !remote.is_newer_than(&cur) {
        return;
    }

    let matched = version_source
        .trim()
        .strip_prefix('v')
        .map(|s| format!("v{}", s))
        .unwrap_or_else(|| format!("v{}", version_source.trim()));

    if is_dismissed(&matched) || is_remind_later(&matched) {
        tracing::debug!("Update {} is dismissed or remind later, skipping notification", matched);
        return;
    }

    info!(
        "SISR update available: current={}, available={}",
        CURRENT_VERSION, matched
    );

    let install_channel = if notify == UpdateNotify::Prerelease {
        "main"
    } else {
        "stable"
    };

    show_update_dialog(&matched, &release.html_url, install_channel);
}

async fn fetch_release(notify: UpdateNotify) -> Option<GithubRelease> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("SISR-updater")
        .build()
        .ok()?;

    if notify == UpdateNotify::Prerelease {
        let resp = client
            .get("https://api.github.com/repos/Alia5/SISR/releases?per_page=1")
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            warn!(
                "GitHub API returned status {} when fetching releases",
                resp.status()
            );
            return None;
        }
        let releases: Vec<GithubRelease> = resp.json().await.ok()?;
        releases.into_iter().next()
    } else {
        let resp = client
            .get("https://api.github.com/repos/Alia5/SISR/releases/latest")
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            warn!(
                "GitHub API returned status {} when fetching latest release",
                resp.status()
            );
            return None;
        }
        resp.json().await.ok()
    }
}

fn show_update_dialog(version: &str, html_url: &str, install_channel: &str) {
    let version = version.to_string();
    let html_url = html_url.to_string();
    let install_channel = install_channel.to_string();

    let ver_update = version.clone();
    let ver_dismiss = version.clone();
    let ver_remind = version.clone();
    let html_url_view = html_url.clone();
    let channel = install_channel.clone();

    let message = format!("A new version of SISR is available: {}", version);
    let message_for_cb = message.clone();

    let _ver_update2 = ver_update.clone();
    let ver_dismiss2 = ver_dismiss.clone();
    let ver_remind2 = ver_remind.clone();
    let html_url_view2 = html_url_view.clone();
    let channel2 = channel.clone();

    let dialog = Dialog {
        title: "SISR Update Available".to_string(),
        message: String::new(),
        buttons_hidden: true,
        draw_callback: Some(Box::new(move |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.label(&message_for_cb);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let update_btn = egui::Button::new("Update Now")
                    .fill(ui.visuals().selection.bg_fill);
                if ui.add(update_btn).clicked() {
                    let ch = channel2.clone();
                    run_install_script(&ch);
                    _ = dialogs::pop_dialog();
                }

                if ui.button("View on GitHub").clicked() {
                    let url = html_url_view2.clone();
                    open_browser(&url);
                    _ = dialogs::pop_dialog();
                }

                if ui.button("Remind Me Later").clicked() {
                    set_remind_later(&ver_remind2);
                    _ = dialogs::pop_dialog();
                }

                if ui.button("Skip This Version").clicked() {
                    write_dismissed(&ver_dismiss2);
                    _ = dialogs::pop_dialog();
                }
            });
        })),
        ..Default::default()
    };

    if let Err(e) = dialogs::push_dialog(dialog) {
        error!("Failed to push update dialog: {}", e);
    }
}

fn open_browser(url: &str) {
    let result = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()
    } else {
        std::process::Command::new("xdg-open").arg(url).spawn()
    };
    if let Err(e) = result {
        error!("Failed to open browser: {}", e);
    }
}

fn run_install_script(channel: &str) {
    let base_url = format!("https://alia5.github.io/SISR/{}/install", channel);

    if cfg!(target_os = "windows") {
        let url = format!("{}.ps1", base_url);
        let result = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!(
                    "Start-Process powershell -ArgumentList '-NoExit -NoProfile -ExecutionPolicy Bypass -Command \"iwr -useb {} | iex\"' -Verb RunAs",
                    url
                ),
            ])
            .spawn();
        if let Err(e) = result {
            error!("Failed to run install script: {}", e);
        }
    } else {
        let url = format!("{}.sh", base_url);
        let result = std::process::Command::new("sh")
            .args(["-c", &format!("curl -fsSL '{}' | sh", url)])
            .spawn();
        if let Err(e) = result {
            error!("Failed to run install script: {}", e);
        }
    }
}

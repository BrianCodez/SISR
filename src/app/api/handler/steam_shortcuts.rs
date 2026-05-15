use std::path::PathBuf;

use axum::{Json, extract::State, response::IntoResponse};
use problem_details::ProblemDetails;
use reqwest::StatusCode;

use crate::{
    app::{actions, api::AppState, steam},
    config::{get_config, update_config},
};

/// List Steam Shortcuts
///
/// Lists the active Steam user's non-Steam shortcuts so SISR can switch Steam Input profiles.
#[utoipa::path(
    get,
    path = "/api/v1/steam_shortcuts",
    tag = "steam",
    responses(
        (status = 200, description = "OK", body = SteamShortcutsResponse),
        (status = 409, description = "Steam shortcut data unavailable"),
    )
)]
pub async fn get_steam_shortcuts(State(_state): State<AppState>) -> impl IntoResponse {
    let Some(shortcuts_path) = shortcuts_path() else {
        return ProblemDetails::from_status_code(StatusCode::CONFLICT)
            .with_detail("Steam shortcuts.vdf path not found".to_string())
            .into_response();
    };

    let selected_app_id = selected_app_id();
    let shortcuts = steam::util::steam_shortcuts(&shortcuts_path)
        .into_iter()
        .map(|shortcut| SteamShortcutProfile {
            app_id: shortcut.app_id,
            game_id: shortcut.game_id,
            name: shortcut.name,
            exe: shortcut.exe,
            start_dir: shortcut.start_dir,
            launch_options: shortcut.launch_options,
            selected: Some(shortcut.app_id) == selected_app_id,
        })
        .collect();

    (
        StatusCode::OK,
        Json(SteamShortcutsResponse {
            shortcuts_path: shortcuts_path.to_string_lossy().to_string(),
            selected_app_id,
            shortcuts,
        }),
    )
        .into_response()
}

/// Launch Steam Shortcut
///
/// Persists the selected shortcut app id, asks Steam to launch it, then shuts this SISR instance down.
#[utoipa::path(
    post,
    path = "/api/v1/steam_shortcuts/launch",
    tag = "steam",
    request_body = SelectSteamShortcutRequest,
    responses(
        (status = 200, description = "OK"),
        (status = 400, description = "Unknown shortcut app id"),
        (status = 409, description = "Steam shortcut data unavailable"),
    )
)]
pub async fn launch_steam_shortcut(
    State(_state): State<AppState>,
    Json(payload): Json<SelectSteamShortcutRequest>,
) -> StatusCode {
    let Some(shortcuts_path) = shortcuts_path() else {
        return StatusCode::CONFLICT;
    };

    if !steam::util::steam_shortcuts(&shortcuts_path)
        .iter()
        .any(|shortcut| shortcut.app_id == payload.app_id)
    {
        return StatusCode::BAD_REQUEST;
    }

    update_config(|c| c.controller_emulation.steam_input_profile_app_id = Some(payload.app_id));
    if persist_selected_app_id(payload.app_id).is_err() {
        tracing::warn!("Failed to persist selected Steam Input profile app id");
    }

    let game_id = steam::util::shortcut_game_id(payload.app_id);
    if let Err(e) = steam::util::open_url(&format!("steam://rungameid/{}", game_id)) {
        tracing::error!("Failed to launch Steam shortcut {}: {}", payload.app_id, e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    actions::shutdown();
    StatusCode::OK
}

fn selected_app_id() -> Option<u32> {
    get_config()
        .controller_emulation
        .steam_input_profile_app_id
        .or_else(|| {
            steam::binding_enforcer::binding_enforcer()
                .lock()
                .ok()
                .and_then(|e| e.app_id())
        })
}

fn shortcuts_path() -> Option<PathBuf> {
    let steam_path = std::env::var("SteamPath")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(steam::util::steam_path)?;
    let active_user_id = steam::util::active_user_id()?;
    steam::util::get_shortcuts_path(&steam_path, active_user_id)
}

fn persist_selected_app_id(app_id: u32) -> Result<(), std::io::Error> {
    let Some(config_dir) = directories::ProjectDirs::from("", "", "SISR") else {
        return Ok(());
    };
    let path = config_dir.config_dir().join("SISR.json");
    let mut current = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .unwrap_or_else(|| serde_json::Value::Object(Default::default()))
    } else {
        serde_json::Value::Object(Default::default())
    };

    if !current.is_object() {
        current = serde_json::Value::Object(Default::default());
    }

    let Some(root) = current.as_object_mut() else {
        return Ok(());
    };
    let controller = root
        .entry("controller_emulation")
        .or_insert_with(|| serde_json::json!({}));
    if !controller.is_object() {
        *controller = serde_json::json!({});
    }
    if let Some(controller) = controller.as_object_mut() {
        controller.insert(
            "steam_input_profile_app_id".to_string(),
            serde_json::json!(app_id),
        );
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&current).unwrap_or_default())
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SteamShortcutsResponse {
    pub shortcuts_path: String,
    pub selected_app_id: Option<u32>,
    pub shortcuts: Vec<SteamShortcutProfile>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SteamShortcutProfile {
    pub app_id: u32,
    pub game_id: u64,
    pub name: String,
    pub exe: String,
    pub start_dir: Option<String>,
    pub launch_options: Option<String>,
    pub selected: bool,
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct SelectSteamShortcutRequest {
    pub app_id: u32,
}

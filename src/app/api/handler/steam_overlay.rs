use axum::{Json, extract::State};
use reqwest::StatusCode;

use crate::app::{actions, api::AppState};

/// Set Steam Overlay
///
/// Enables or disables the SISR Steam overlay window.
#[utoipa::path(
    post,
    path = "/api/v1/steam_overlay",
    tag = "ui",
    request_body = SetSteamOverlayRequest,
    responses(
        (status = 200, description = "OK"),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn set_steam_overlay(
    State(_state): State<AppState>,
    Json(payload): Json<SetSteamOverlayRequest>,
) -> StatusCode {
    tracing::debug!(
        "Received request to set Steam overlay enabled: {}",
        payload.enabled
    );
    if actions::set_steam_overlay_enabled(payload.enabled) {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct SetSteamOverlayRequest {
    pub enabled: bool,
}

use axum::{Json, extract::State};
use reqwest::StatusCode;

use crate::app::{actions, api::AppState};

/// Set Force Controller Config
///
/// Enables or disables Steam input config enforcement.
/// If `app_id` is omitted, defaults to the currently active app id.
#[utoipa::path(
    post,
    path = "/api/v1/force_controller_config",
    tag = "steam",
    request_body = SetForceControllerConfigRequest,
    responses(
        (status = 200, description = "OK"),
    )
)]
pub async fn set_force_controller_config(
    State(_state): State<AppState>,
    Json(payload): Json<SetForceControllerConfigRequest>,
) -> StatusCode {
    actions::set_desktop_config_allowed(!payload.enforce, payload.app_id);

    StatusCode::OK
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct SetForceControllerConfigRequest {
    pub enforce: bool,
    pub app_id: Option<u32>,
}

use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;

use crate::app::{actions, api::AppState};

/// Shutdown Application
///
/// Shuts down the application gracefully
#[utoipa::path(
    post,
    path = "/api/v1/shutdown",
    tag = "ui",
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn shutdown(State(_state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("Received request to shutdown application");

    actions::shutdown();

    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}

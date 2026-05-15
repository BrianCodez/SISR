use crate::{
    app::{
        runner::AppRunner,
        steam::{self, binding_enforcer::binding_enforcer},
        window::event::{WindowRunnerEvent, get_event_sender},
    },
    config::{get_config, update_config},
};

use super::runner::get_tokio_handle;

pub fn set_steam_overlay_enabled(enabled: bool) -> bool {
    if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::SetFullscreen(enabled)) {
        tracing::error!("Failed to send SetFullscreen event: {:?}", e);
        return false;
    }

    let visibility_event = if enabled {
        WindowRunnerEvent::ShowWindow()
    } else {
        WindowRunnerEvent::HideWindow()
    };

    if let Err(e) = get_event_sender().send_event(visibility_event) {
        tracing::error!("Failed to send window visibility event: {:?}", e);
        return false;
    }

    true
}

pub fn toggle_steam_overlay_enabled() -> bool {
    let cfg = get_config();
    let currently_enabled =
        cfg.window.create.unwrap_or(false) && cfg.window.fullscreen.unwrap_or(true);
    let enabled = !currently_enabled;
    let _ = set_steam_overlay_enabled(enabled);
    enabled
}

pub fn set_ui_visible(show: bool) -> bool {
    if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::ToggleUi(Some(show))) {
        tracing::error!("Failed to send ToggleUi event: {:?}", e);
        return false;
    }

    true
}

pub fn toggle_ui_visible() {
    if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::ToggleUi(None)) {
        tracing::error!("Failed to send ToggleUi event: {e}");
    }
}

pub fn open_controller_config() -> bool {
    let app_id = get_config()
        .controller_emulation
        .steam_input_profile_app_id
        .or_else(|| binding_enforcer().lock().ok().and_then(|e| e.app_id()));
    open_controller_config_for_app_id(app_id)
}

pub fn open_controller_config_for_app_id(app_id: Option<u32>) -> bool {
    let Some(app_id) = app_id else {
        tracing::warn!("Cannot open Steam controller config: no app id available");
        return false;
    };

    get_tokio_handle().spawn(async move {
        steam::util::open_controller_config(app_id).await;
    });
    true
}

pub fn set_desktop_config_allowed(allow: bool, app_id: Option<u32>) {
    let Ok(mut enforcer) = binding_enforcer().lock() else {
        tracing::error!("Failed to lock binding enforcer");
        return;
    };

    update_config(|c| {
        c.controller_emulation.allow_desktop_config = Some(allow);
        if let Some(app_id) = app_id {
            c.controller_emulation.steam_input_profile_app_id = Some(app_id);
        }
    });
    if allow {
        enforcer.deactivate();
    } else if let Some(app_id) =
        app_id.or_else(|| get_config().controller_emulation.steam_input_profile_app_id)
    {
        enforcer.activate_with_appid(app_id);
    } else {
        enforcer.activate();
    }
}

pub fn toggle_desktop_config_allowed() -> Option<bool> {
    let Ok(mut enforcer) = binding_enforcer().lock() else {
        tracing::error!("Failed to lock binding enforcer");
        return None;
    };

    let allow = enforcer.is_active();
    update_config(|c| c.controller_emulation.allow_desktop_config = Some(allow));
    if allow {
        enforcer.deactivate();
    } else {
        enforcer.activate();
    }
    Some(allow)
}

pub fn shutdown() {
    let _ = get_event_sender().send_event(WindowRunnerEvent::HideWindow());
    AppRunner::shutdown();
}

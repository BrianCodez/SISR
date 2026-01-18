use sdl3_sys::events::{SDL_EVENT_GAMEPAD_UPDATE_COMPLETE, SDL_Event};
use std::sync::{Arc, Mutex};

use crate::app::input::context::Context;
use crate::app::input::device::Device;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::config::get_config;

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
    viiper_bridge: Arc<Mutex<ViiperBridge>>,
    _gyro_passthrough: bool,
    no_steam: bool,
}

impl Handler {
    pub fn new(ctx: Arc<Mutex<Context>>, viiper_bridge: Arc<Mutex<ViiperBridge>>) -> Self {
        Self {
            ctx,
            viiper_bridge,
            _gyro_passthrough: get_config()
                .controller_emulation
                .gyro_passthrough
                .unwrap_or(true),
            no_steam: get_config().steam.no_steam.unwrap_or(false),
        }
    }

    fn update_device_gamepad_state(&self, device: &mut Device, which: u32) {
        let Some(viiper_device) = device.viiper_device.as_mut() else {
            tracing::warn!(
                "No Viiper device found for SDL id {} in device id {}",
                which,
                device.id
            );
            return;
        };
        let Some(gp) = device.sdl_devices.iter().find_map(|d| {
            if d.id == which && d.gamepad.is_some() {
                d.gamepad.as_ref()
            } else {
                None
            }
        }) else {
            tracing::warn!(
                "No gamepad found for SDL id {} in device id {}",
                which,
                device.id
            );
            return;
        };

        if viiper_device.state.viiper_type() != Some(viiper_device.device.r#type.as_str()) {
            tracing::warn!(
                "Viiper device state type mismatch for device id {}. Reinitializing state.",
                device.id
            );
            viiper_device.init_state();
        }
        viiper_device.state.update_from_sdl_gamepad(gp);

        let Ok(viiper) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge mutex");
            return;
        };
        let Some(viiper_device_state_boxed) = viiper_device.state.boxed() else {
            tracing::error!("Failed to get boxed state for device id {}", device.id);
            return;
        };
        viiper.update_device_state_boxed(device.id, viiper_device_state_boxed);
    }
}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        _event: &Option<RoutedEvent>,
        sdl_event: &SDL_Event,
    ) {
        // let event_type = SDL_EventType(unsafe { sdl_event.r#type });
        // tracing::trace!(event = ?event_type); // TODO: log only if enabled via flag
        let which = unsafe { sdl_event.gdevice.which }.0;
        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };

        if ctx.steam_overlay_open {
            // tracing::debug!(
            //     "Ignoring gamepad update complete for SDL id {} because overlay is open",
            //     which
            // );
            return;
        }
        let Some(device_mtx) = ctx.device_for_sdl_id(which) else {
            tracing::warn!("No device found for SDL id {}", which);
            return;
        };
        let Ok(mut device) = device_mtx.lock() else {
            tracing::error!("Failed to lock Device mutex for SDL id {}", which);
            return;
        };
        drop(ctx);
        if device.steam_handle == 0 && !self.no_steam {
            // tracing::trace!(
            //     "Ignoring gamepad update complete for SDL id {} because no Steam handle",
            //     which
            // );
            return;
        }

        self.update_device_gamepad_state(&mut device, which);
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::SdlEventType(SDL_EVENT_GAMEPAD_UPDATE_COMPLETE)]
    }
}

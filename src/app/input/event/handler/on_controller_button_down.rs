use std::mem::discriminant;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sdl3::event::Event;
use sdl3_sys::events::SDL_Event;

use crate::app::actions;
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::{
    context::Context,
    event::router::{EventHandler, ListenEvent, RoutedEvent},
};
use crate::app::window;
use crate::app::window::event::{UiControllerAction, WindowRunnerEvent, get_event_sender};

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
    last_ui_toggle_time: Arc<Mutex<Option<Instant>>>,
    last_desktop_config_toggle_time: Arc<Mutex<Option<Instant>>>,
    last_ui_action_time: Arc<Mutex<Option<Instant>>>,
}
impl Handler {
    pub fn new(ctx: Arc<Mutex<Context>>) -> Self {
        Self {
            ctx,
            last_ui_toggle_time: Arc::new(Mutex::new(None)),
            last_desktop_config_toggle_time: Arc::new(Mutex::new(None)),
            last_ui_action_time: Arc::new(Mutex::new(None)),
        }
    }
}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        let event = match event {
            Some(RoutedEvent::SdlEvent(event)) => event,
            _ => {
                tracing::warn!("Received non-SDL event ");
                return;
            }
        };
        let (which, button) = match event {
            Event::ControllerButtonDown { which, button, .. } => (*which, button),
            _ => {
                tracing::warn!("Received non-ControllerButtonDown event ");
                return;
            }
        };

        if *button == sdl3::gamepad::Button::Guide {
            // draw frames for a a second for overlay-spawn...
            tracing::debug!("HACK: Rending for a second to allow Steam overlay to spawn...");
            thread::spawn(|| {
                for _ in 0..60 {
                    thread::sleep(std::time::Duration::from_millis(16));
                    window::event::request_redraw_without_webview();
                }
            });
        }

        let ui_action = match *button {
            sdl3::gamepad::Button::DPadDown | sdl3::gamepad::Button::DPadRight => {
                Some(UiControllerAction::Next)
            }
            sdl3::gamepad::Button::DPadUp | sdl3::gamepad::Button::DPadLeft => {
                Some(UiControllerAction::Previous)
            }
            sdl3::gamepad::Button::South => Some(UiControllerAction::Activate),
            sdl3::gamepad::Button::East => Some(UiControllerAction::Back),
            _ => None,
        };

        if ui_action.is_none() && *button != sdl3::gamepad::Button::North {
            return;
        }

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };

        let Some(device) = ctx.device_for_sdl_id(which) else {
            return;
        };
        drop(ctx);

        let Ok(device) = device.lock() else {
            tracing::error!("Failed to lock Device mutex for SDL id {}", which);
            return;
        };

        let Some(gp) = device.sdl_devices.iter().find_map(|d| {
            if d.id == which && d.gamepad.is_some() {
                d.gamepad.as_ref()
            } else {
                None
            }
        }) else {
            return;
        };

        if gp.button(sdl3::gamepad::Button::LeftShoulder)
            && gp.button(sdl3::gamepad::Button::RightShoulder)
            && gp.button(sdl3::gamepad::Button::Back)
        {
            if *button == sdl3::gamepad::Button::South {
                tracing::trace!("UI toggle controller chord detected on SDL ID {}", which);
                if should_handle_chord(&self.last_ui_toggle_time, "UI toggle", which) {
                    actions::toggle_ui_visible();
                }
            } else if *button == sdl3::gamepad::Button::North {
                tracing::trace!(
                    "Desktop config toggle controller chord detected on SDL ID {}",
                    which
                );
                if should_handle_chord(
                    &self.last_desktop_config_toggle_time,
                    "desktop config toggle",
                    which,
                ) {
                    if let Some(allow) = actions::toggle_desktop_config_allowed() {
                        tracing::info!("Steam Input Desktop Layout allowed: {}", allow);
                    }
                }
            }
            return;
        }

        let Some(action) = ui_action else {
            return;
        };

        if should_handle_chord(&self.last_ui_action_time, "UI controller action", which) {
            if let Err(e) =
                get_event_sender().send_event(WindowRunnerEvent::UiControllerAction(action))
            {
                tracing::trace!("Failed to send UI controller action: {e}");
            }
        }
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![
            ListenEvent::SdlEvent(discriminant(&Event::ControllerButtonDown {
                timestamp: 0,
                which: 0,
                button: sdl3::gamepad::Button::South,
            })),
            ListenEvent::SdlEvent(discriminant(&Event::ControllerButtonDown {
                timestamp: 0,
                which: 0,
                button: sdl3::gamepad::Button::North,
            })),
            ListenEvent::SdlEvent(discriminant(&Event::ControllerButtonDown {
                timestamp: 0,
                which: 0,
                button: sdl3::gamepad::Button::East,
            })),
            ListenEvent::SdlEvent(discriminant(&Event::ControllerButtonDown {
                timestamp: 0,
                which: 0,
                button: sdl3::gamepad::Button::DPadUp,
            })),
            ListenEvent::SdlEvent(discriminant(&Event::ControllerButtonDown {
                timestamp: 0,
                which: 0,
                button: sdl3::gamepad::Button::DPadDown,
            })),
            ListenEvent::SdlEvent(discriminant(&Event::ControllerButtonDown {
                timestamp: 0,
                which: 0,
                button: sdl3::gamepad::Button::DPadLeft,
            })),
            ListenEvent::SdlEvent(discriminant(&Event::ControllerButtonDown {
                timestamp: 0,
                which: 0,
                button: sdl3::gamepad::Button::DPadRight,
            })),
        ]
    }
}

fn should_handle_chord(
    last_action_time: &Arc<Mutex<Option<Instant>>>,
    action_name: &str,
    sdl_id: u32,
) -> bool {
    const DEBOUNCE_DURATION: Duration = Duration::from_millis(200);

    let Ok(mut last_time) = last_action_time.lock() else {
        tracing::error!("Failed to lock debounce timestamp for {}", action_name);
        return false;
    };

    let now = Instant::now();
    let should_handle = match *last_time {
        Some(last) => now.duration_since(last) >= DEBOUNCE_DURATION,
        None => true,
    };

    if should_handle {
        *last_time = Some(now);
    } else {
        tracing::trace!(
            "Ignoring duplicate {} within debounce window on SDL ID {}",
            action_name,
            sdl_id
        );
    }

    should_handle
}

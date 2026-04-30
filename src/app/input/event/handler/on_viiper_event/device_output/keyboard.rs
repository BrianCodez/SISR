use std::sync::{Arc, Mutex};

use crate::app::input::context::Context;

pub fn handle_output(_ctx: Arc<Mutex<Context>>, _device_id: &u64, _leds: &u8) {
    // TODO: set leds on host
    tracing::warn!("Ignoreing Keyboard output! Not implemented");
}

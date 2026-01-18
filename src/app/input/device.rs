use std::fmt::Debug;

use viiper_client::devices::{
    dualshock4::{Dualshock4Input, Dualshock4Output},
    keyboard::{KeyboardInput, KeyboardOutput},
    mouse::MouseInput,
    xbox360::{Xbox360Input, Xbox360Output},
};

use crate::app::input::{context::Context, device_info::SDLDeviceInfo, device_state::DeviceState};

#[derive(Debug, Default)]
pub struct Device {
    pub id: u64,
    pub sdl_devices: Vec<SDLDevice>,
    pub steam_handle: u64,
    pub viiper_type: Option<String>,
    pub viiper_device: Option<ViiperDevice>,
    ///  holds virtual_device created by steam for real devices
    /// or real_device for virtual_steam_devices
    pub corresponding_device_id: Option<u64>, // TODO: map on connect !!!
}

impl Device {
    pub fn sdl_device(&mut self, sdl_id: u32) -> Option<&mut SDLDevice> {
        self.sdl_devices
            .iter_mut()
            .find(|sdl_device| sdl_device.id == sdl_id)
    }

    pub fn steam_device_id(&self, ctx: &Context) -> Option<u64> {
        // TODO: better correlation // maybe add gui; whatever, this works for an initial bringup...
        // TODO: fix this mess, but whatever for now... fuck this

        if self.steam_handle != 0 {
            tracing::debug!(
                "Device id {} has no steam handle, cannot find steam SDL controller",
                self.id
            );
            return None;
        }

        let Some(gamepad) = self
            .sdl_devices
            .iter()
            .find_map(|sdl_device| sdl_device.gamepad.as_ref())
        else {
            tracing::debug!("No SDL gamepad found for device id {}", self.id);
            return None;
        };

        let Some(name) = gamepad.name() else {
            tracing::debug!("SDL gamepad has no name for device id {}", self.id);
            return None;
        };

        // search for device with SteamHandle != 0!!! but with a gamepad that has the same name
        let mut devices: Vec<_> = ctx.devices.iter_mut().collect();

        let Some(steam_device_id) = devices.iter_mut().find_map(|r| {
            if r.key() == &self.id {
                return None;
            }
            let Ok(mut dev) = r.value().lock() else {
                return None;
            };
            if dev.steam_handle == 0 {
                return None;
            }
            dev.sdl_devices.iter_mut().find_map(|sdl_device| {
                // Now this is silly that it ahs to be mut
                let gp = sdl_device.gamepad.as_mut()?;
                let gp_name = gp.name()?;
                // TODO: hack! steamdeck
                if gp_name.starts_with("Steam Deck") && name.starts_with("Steam Deck") {
                    tracing::trace!(
                        "Found steam SDL controller '{}' for device id {}",
                        gp_name,
                        self.id
                    );

                    return Some(*r.key());
                }
                if gp_name == name {
                    tracing::trace!(
                        "Found steam SDL controller '{}' for device id {}",
                        gp_name,
                        self.id
                    );
                    return Some(*r.key());
                }
                None
            })
        }) else {
            tracing::debug!(
                "No matching steam SDL controller found for device id {}",
                self.id
            );
            return None;
        };

        Some(steam_device_id)
    }

    pub fn non_steam_sdl_controller(&self, ctx: &Context) -> Option<sdl3::gamepad::Gamepad> {
        // TODO: better correlation // maybe add gui; whatever, this works for an initial bringup...

        if self.steam_handle == 0 {
            tracing::debug!(
                "Device id {} has no steam handle, cannot find non-steam SDL controller",
                self.id
            );
            return None;
        }

        let Some(gamepad) = self
            .sdl_devices
            .iter()
            .find_map(|sdl_device| sdl_device.gamepad.as_ref())
        else {
            tracing::debug!("No SDL gamepad found for device id {}", self.id);
            return None;
        };

        let Some(name) = gamepad.name() else {
            tracing::debug!("SDL gamepad has no name for device id {}", self.id);
            return None;
        };

        // search for device with SteamHandle 0!!! but with a gamepad that has the same name
        let Some(real_sdl_gamepad) = ctx.devices.iter().find_map(|r| {
            if r.key() == &self.id {
                return None;
            }
            let Ok(dev) = r.value().lock() else {
                return None;
            };
            if dev.steam_handle != 0 {
                return None;
            }
            dev.sdl_devices.iter().find_map(|sdl_device| {
                let gp = sdl_device.gamepad.as_ref()?;
                let gp_name = gp.name()?;
                if gp_name == name {
                    tracing::debug!(
                        "Found non-steam SDL controller '{}' for device id {}",
                        gp_name,
                        self.id
                    );
                    let Ok(gp_id) = gp.id() else {
                        return None;
                    };
                    let Ok(real_gp) = gp.subsystem().get(gp_id) else {
                        return None;
                    };
                    return Some(real_gp);
                }
                None
            })
        }) else {
            tracing::debug!(
                "No matching non-steam SDL controller found for device id {}",
                self.id
            );
            return None;
        };

        Some(real_sdl_gamepad)
    }
}
pub struct ViiperDevice {
    pub device: viiper_client::Device,
    pub state: DeviceState,
}

impl ViiperDevice {
    pub fn init_state(&mut self) {
        match self.device.r#type.as_str() {
            "xbox360" => {
                self.state = DeviceState::Xbox360 {
                    input_state: Xbox360Input::default(),
                    output_state: Xbox360Output::default(),
                };
            }
            "dualshock4" => {
                self.state = DeviceState::Dualshock4 {
                    input_state: Dualshock4Input::default(),
                    output_state: Dualshock4Output::default(),
                };
            }
            "keyboard" => {
                self.state = DeviceState::Keyboard {
                    input_state: KeyboardInput::default(),
                    output_state: KeyboardOutput::default(),
                };
            }
            "mouse" => {
                self.state = DeviceState::Mouse {
                    input_state: MouseInput::default(),
                };
            }
            _ => {
                tracing::warn!(
                    "Unknown Viiper device type '{}' for device",
                    self.device.r#type,
                );
                self.state = DeviceState::Empty;
            }
        }
    }
}

impl Debug for ViiperDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.device)
    }
}

pub struct SDLDevice {
    pub id: u32,
    pub infos: SDLDeviceInfo,
    pub joystick: Option<sdl3::joystick::Joystick>,
    pub gamepad: Option<sdl3::gamepad::Gamepad>,
}

impl SDLDevice {
    pub fn new(
        id: u32,
        joystick: Option<sdl3::joystick::Joystick>,
        gamepad: Option<sdl3::gamepad::Gamepad>,
    ) -> Self {
        let mut res = Self {
            id,
            infos: SDLDeviceInfo::default(),
            joystick,
            gamepad,
        };
        res.update_info();
        res
    }
    pub fn update_info(&mut self) {
        self.infos.update(&self.joystick, &self.gamepad);
    }
}

impl Debug for SDLDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (&self.joystick, &self.gamepad) {
            (Some(js), None) => f
                .debug_struct("SDLDevice::Joystick")
                .field("name", &js.name())
                .field("id", &js.id())
                .finish(),
            (None, Some(gp)) => f
                .debug_struct("SDLDevice::Gamepad")
                .field("name", &gp.name())
                .field("id", &gp.id())
                .finish(),
            (Some(js), Some(gp)) => f
                .debug_struct("SDLDevice::Joystick+Gamepad")
                .field("id", &self.id)
                .field("joystick_name", &js.name())
                .field("gamepad_name", &gp.name().unwrap_or("N/A".to_string()))
                .finish(),
            _ => write!(f, "SDLDevice::Unknown"),
        }
    }
}

// is thread safe, fuck this
unsafe impl Send for SDLDevice {}
unsafe impl Sync for SDLDevice {}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

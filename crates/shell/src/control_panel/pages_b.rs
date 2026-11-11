//! The second half of the Control Panel's pages: Screen Resolution, Devices and Printers, Sound,
//! Network and Sharing Center, Default Programs, Startup Programs, Date and Time, Ease of Access and System. Each page is a
//! column the window puts in its scrolling pane; each change applies at once, as in the QML
//! shell's `ControlPanel.qml`.

#[path = "b/about.rs"]
mod about;
#[path = "b/common.rs"]
mod common;
#[path = "b/date_time.rs"]
mod date_time;
#[path = "b/defaults.rs"]
mod defaults;
#[path = "b/devices.rs"]
mod devices;
#[path = "b/display.rs"]
mod display;
#[path = "b/ease.rs"]
mod ease;
#[path = "b/network.rs"]
mod network;
#[path = "b/network_edit.rs"]
mod network_edit;
#[path = "b/sound.rs"]
mod sound;
#[path = "b/startup.rs"]
mod startup;

use anti_webui::prelude::*;

/// Screen Resolution: the screens' layout, and each screen's mode, refresh rate and scale.
pub fn display_page() -> Div {
    display::page()
}

/// Devices and Printers: the computer, the screens, the USB devices and the printers.
pub fn devices_page() -> Div {
    devices::page()
}

/// Sound: the playback and recording devices, and the default of each.
pub fn sound_page() -> Div {
    sound::page()
}

/// Network and Sharing Center: the networks in use, the adapters and the saved connections.
pub fn network_page() -> Div {
    network::page()
}

/// Default Programs: the programs that open each kind of file, and those the desktop starts.
pub fn default_programs_page() -> Div {
    defaults::page()
}

/// Startup Programs: the XDG autostart entries.
pub fn startup_page() -> Div {
    startup::page()
}

/// Date and Time: the clock's format, the additional clocks, and the system's time zone.
pub fn date_time_page() -> Div {
    date_time::page()
}

/// Ease of Access: the animations and their speed.
pub fn ease_page() -> Div {
    ease::page()
}

/// System: the facts about the computer and this desktop.
pub fn about_page() -> Div {
    about::page()
}

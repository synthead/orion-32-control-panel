// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod control_panel;
mod control_panel_ad_phase;
mod control_panel_helpers;
mod control_panel_mixer;
mod control_panel_routing;

mod drag_value;
mod text_editor;

mod orion_hid;
mod orion_hid_poller;
mod orion_state;

use crate::control_panel::ControlPanel;

fn main() -> eframe::Result {
    env_logger::init();
    ControlPanel::run()
}

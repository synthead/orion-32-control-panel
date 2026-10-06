use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

// use crate::control_panel_helpers::ControlPanelHelpers;

// use crate::render_asset;

pub(crate) struct ControlPanelMixer {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,
}

impl ControlPanelMixer {
    pub(crate) fn new(
        orion_state: Arc<Mutex<OrionState>>,
        orion_hid: Arc<Mutex<OrionHid>>,
    ) -> Self {
        Self {
            orion_state,
            orion_hid,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {}
}

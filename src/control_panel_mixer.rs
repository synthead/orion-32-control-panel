use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

// use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

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

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        Self::draw_base_graphics(ui);
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        render_asset!(ui, 0, 222, "Mixer/default/background", 1.0);

        for mixer_channel in 0..32 {
            let pos_x = 7 + mixer_channel * 35;

            render_asset!(ui, pos_x, 604, "SoloButton/default/released", 1.0);
            render_asset!(ui, pos_x, 622, "MuteButton/default/released", 1.0);

            render_asset!(ui, pos_x + 4, 366, "MixerSlider/default/layers/handle", 1.0);

            if mixer_channel % 2 == 0 {
                render_asset!(ui, pos_x, 640, "LinkButton/default/released", 1.0);
            }
        }

        render_asset!(
            ui,
            1136,
            356,
            "MasterMixerSlider/default/layers/handle",
            1.0
        );
        render_asset!(ui, 1136, 639, "MuteButton/default/released", 1.0);
    }
}

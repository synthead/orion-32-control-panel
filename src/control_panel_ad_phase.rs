use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionStates;

use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

struct ControlPanelAdPhaseDimensions;

impl ControlPanelAdPhaseDimensions {
    const BACKGROUND_X: u16 = 0;
    const BACKGROUND_Y: u16 = 226;

    const STEP_X: u16 = 34;
    const BUTTON_X: u16 = 51;
    const BUTTON_Y: u16 = 411;
}

pub(crate) struct ControlPanelAdPhase {
    orion_states: Arc<Mutex<OrionStates>>,
    orion_hid: Arc<Mutex<OrionHid>>,
}

impl ControlPanelAdPhase {
    pub(crate) fn new(
        orion_states: Arc<Mutex<OrionStates>>,
        orion_hid: Arc<Mutex<OrionHid>>,
    ) -> Self {
        Self {
            orion_states,
            orion_hid,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        let Some(orion_state) = self.orion_states.lock().unwrap().current().cloned() else {
            return;
        };

        Self::draw_base_graphics(ui);
        self.draw_buttons(ui, &orion_state.ad_phases);
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        render_asset!(
            ui,
            ControlPanelAdPhaseDimensions::BACKGROUND_X,
            ControlPanelAdPhaseDimensions::BACKGROUND_Y,
            "PhaseInvertView/default/background",
            1.0,
            0.0,
            None
        );
    }

    fn draw_buttons(&mut self, ui: &mut egui::Ui, ad_phases: &[u8; 4]) {
        for ad_phase_i in 0..32 {
            let byte_i = ad_phase_i / 8;
            let ad_phase_byte = ad_phases[byte_i];

            let bit_i = ad_phase_i % 8;
            let ad_phase = ad_phase_byte & 1 << bit_i;

            let pos_x = ControlPanelAdPhaseDimensions::BUTTON_X
                + ControlPanelAdPhaseDimensions::STEP_X * ad_phase_i as u16;

            let ad_button_state = ad_phase != 0;

            let ad_phase_response = if ad_phase != 0 {
                render_asset!(
                    ui,
                    pos_x,
                    ControlPanelAdPhaseDimensions::BUTTON_Y,
                    "PhaseInvertButton/default/pressed",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            } else {
                render_asset!(
                    ui,
                    pos_x,
                    ControlPanelAdPhaseDimensions::BUTTON_Y,
                    "PhaseInvertButton/default/released",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            };

            if ControlPanelHelpers::mouse_down_on(&ad_phase_response) {
                let ad_phase_new = if ad_button_state { 0x00 } else { 0x80 } | ad_phase_i as u8;

                self.orion_hid.lock().unwrap().queue_ad_phase(ad_phase_new);
            }
        }
    }
}

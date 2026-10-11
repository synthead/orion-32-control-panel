// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

mod control_panel;
mod control_panel_ad_phase;
mod control_panel_helpers;
mod control_panel_mixer;
mod control_panel_routing;

mod text_editor;
mod drag_value;

mod orion_hid;
mod orion_hid_poller;
mod orion_state;

use crate::control_panel::ControlPanel;

use crate::text_editor::TextEditor;
use crate::drag_value::DragValue;

use crate::orion_hid::OrionHid;
use crate::orion_hid_poller::OrionHidPoller;
use crate::orion_state::OrionState;

fn main() -> eframe::Result {
    // Log to stderr with RUST_LOG=debug env var.
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 700.0]),
        ..Default::default()
    };

    let orion_state = Arc::new(Mutex::new(OrionState::default()));
    let (hid_write_tx, hid_write_rx) = mpsc::channel::<[u8; 64]>();
    let mut orion_hid_poller = OrionHidPoller::new(Arc::clone(&orion_state), hid_write_rx);
    let mut orion_hid = OrionHid::new(hid_write_tx, Arc::clone(&orion_state));

    orion_hid.queue_init();

    let text_editor = TextEditor::new();
    let drag_value = DragValue::new();

    let control_panel = ControlPanel::new(
        Arc::clone(&orion_state),
        Arc::new(Mutex::new(orion_hid)),
        Arc::new(Mutex::new(text_editor)),
        Arc::new(Mutex::new(drag_value)),
    );

    eframe::run_native(
        "Orion 32 Control Panel",
        options,
        Box::new(|cc| {
            cc.egui_ctx.style_mut_of(cc.egui_ctx.theme(), |style| {
                style.interaction.interact_radius = 0.0;
            });

            egui_extras::install_image_loaders(&cc.egui_ctx);

            let ctx_thread = cc.egui_ctx.clone();

            thread::spawn(move || loop {
                orion_hid_poller.poll_and_update_state();
                ctx_thread.request_repaint();
            });

            Ok(Box::new(control_panel))
        }),
    )
}

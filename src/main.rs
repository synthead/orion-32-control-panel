// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

mod control_panel;
mod control_panel_helpers;
mod control_panel_routing;

mod orion_hid;
mod orion_hid_poller;
mod orion_state;

use crate::control_panel::ControlPanel;

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
    let mut orion_hid = OrionHid::new(hid_write_tx);

    orion_hid.queue_init();

    let control_panel =
        ControlPanel::new(Arc::clone(&orion_state), Arc::new(Mutex::new(orion_hid)));

    eframe::run_native(
        "Orion 32 Control Panel",
        options,
        Box::new(|cc| {
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

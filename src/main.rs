// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

mod control_panel;
mod orion_hid;
mod orion_state;

use crate::control_panel::ControlPanel;
use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

fn main() -> eframe::Result {
    // Log to stderr with RUST_LOG=debug env var.
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 700.0]),
        ..Default::default()
    };

    let orion_state = Arc::new(Mutex::new(OrionState::default()));
    let orion_hid = Arc::new(Mutex::new(OrionHid::new(Arc::clone(&orion_state))));

    let control_panel = ControlPanel {
        orion_state: Arc::clone(&orion_state),
        orion_hid: Arc::clone(&orion_hid),
    };

    eframe::run_native(
        "Orion 32 Control Panel",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let ctx_thread = cc.egui_ctx.clone();
            let orion_hid_thread = Arc::clone(&orion_hid);

            thread::spawn(move || loop {
                let mut orion_hid_locked = orion_hid_thread.lock().unwrap();

                orion_hid_locked.poll_and_update_state();
                ctx_thread.request_repaint();
            });

            {
                let mut orion_hid_locked = orion_hid.lock().unwrap();
                orion_hid_locked.write_init();
            }

            Ok(Box::new(control_panel))
        }),
    )
}

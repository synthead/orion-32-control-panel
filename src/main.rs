// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::sync::Arc;
use std::thread;

mod control_panel;
mod orion_hid;
mod orion_state;

use crate::control_panel::ControlPanel;
use crate::orion_hid::OrionHid;

fn main() -> eframe::Result {
    // Log to stderr with RUST_LOG=debug env var.
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 700.0]),
        ..Default::default()
    };

    let control_panel = ControlPanel::new();

    let orion_hid = Arc::clone(&control_panel.orion_hid);
    let orion_state = Arc::clone(&control_panel.orion_state);

    eframe::run_native(
        "Orion 32 Control Panel",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let ctx_copy = cc.egui_ctx.clone();

            thread::spawn(move || {
                {
                    let mut orion_hid_locked = orion_hid.lock().unwrap();
                    orion_hid_locked.write_init();
                }

                loop {
                    let hid_data = {
                        let mut orion_hid_locked = orion_hid.lock().unwrap();
                        orion_hid_locked.read_packet()
                    };

                    let updated = {
                        let mut orion_state_locked = orion_state.lock().unwrap();

                        OrionHid::update_orion_state(&hid_data, &mut orion_state_locked)
                    };

                    if updated {
                        ctx_copy.request_repaint();
                    }
                }
            });

            Ok(Box::new(control_panel))
        }),
    )
}

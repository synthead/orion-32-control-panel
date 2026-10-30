#[macro_export]
macro_rules! render_asset {
    ($ui:expr, $x:expr, $y:expr, $subpath:literal, $scale:expr, $rotation:expr, $sense:expr) => {
        $ui.scope_builder(
            egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                egui::pos2($x as f32, $y as f32),
                $ui.available_size(),
            )),
            |ui| {
                let image = egui::Image::new(egui::include_image!(concat!(
                    "../assets/images/",
                    $subpath,
                    ".png"
                )))
                .fit_to_original_size($scale)
                .show_loading_spinner(false);

                let image = if $rotation == 0.0 {
                    image
                } else {
                    image.rotate($rotation, egui::Vec2::splat(0.5))
                };

                let image = if let Some(sense) = $sense {
                    image.sense(sense)
                } else {
                    image
                };

                ui.add(image)
            },
        )
        .inner
    };
}

pub(crate) struct ControlPanelHelpers;

impl ControlPanelHelpers {
    const COMBOBOX_TEXT_SIZE: f32 = 10.5;

    const COMBOBOX_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x95, 0x95, 0x95);
    const COMBOBOX_ARROW_COLOR: egui::Color32 = egui::Color32::from_rgb(0x85, 0xc8, 0x26);
    const COMBOBOX_BG_COLOR: egui::Color32 = egui::Color32::BLACK;

    pub(crate) fn draw_combobox(
        ui: &mut egui::Ui,
        x: u16,
        y: u16,
        width: u8,
        selected: u8,
        selections: &[&str],
        mut changed_callback: impl FnMut(u8),
    ) {
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                egui::pos2(x as f32, y as f32),
                egui::vec2(width as f32, 17.0),
            )),
            |ui| {
                ui.visuals_mut().widgets.inactive.weak_bg_fill = Self::COMBOBOX_BG_COLOR;
                ui.visuals_mut().widgets.hovered.weak_bg_fill = Self::COMBOBOX_BG_COLOR;
                ui.visuals_mut().widgets.active.weak_bg_fill = Self::COMBOBOX_BG_COLOR;
                ui.visuals_mut().widgets.open.weak_bg_fill = Self::COMBOBOX_BG_COLOR;

                ui.visuals_mut().widgets.inactive.fg_stroke.color = Self::COMBOBOX_ARROW_COLOR;
                ui.visuals_mut().widgets.hovered.fg_stroke.color = Self::COMBOBOX_ARROW_COLOR;
                ui.visuals_mut().widgets.active.fg_stroke.color = Self::COMBOBOX_ARROW_COLOR;
                ui.visuals_mut().widgets.open.fg_stroke.color = Self::COMBOBOX_ARROW_COLOR;

                let inactive_stroke = ui.visuals().widgets.inactive.bg_stroke;

                ui.visuals_mut().widgets.hovered.bg_stroke = inactive_stroke;
                ui.visuals_mut().widgets.active.bg_stroke = inactive_stroke;
                ui.visuals_mut().widgets.open.bg_stroke = inactive_stroke;

                ui.spacing_mut().interact_size.y -= 2.0;

                let mut selected_mut = selected;

                egui::ComboBox::from_id_salt((x, y))
                    .width(width as f32)
                    .selected_text(
                        egui::RichText::new(selections[selected as usize])
                            .size(Self::COMBOBOX_TEXT_SIZE)
                            .color(Self::COMBOBOX_TEXT_COLOR),
                    )
                    .popup_style(egui::style::StyleModifier::new(|style| {
                        style.visuals.window_fill = Self::COMBOBOX_BG_COLOR;
                    }))
                    .show_ui(ui, |ui| {
                        for (i, selection) in selections.iter().enumerate() {
                            let response = ui.selectable_value(
                                &mut selected_mut,
                                i as u8,
                                egui::RichText::new(*selection)
                                    .size(Self::COMBOBOX_TEXT_SIZE)
                                    .color(Self::COMBOBOX_TEXT_COLOR),
                            );

                            if response.changed() {
                                changed_callback(selected_mut);
                            }
                        }
                    });
            },
        );
    }

    pub(crate) fn mouse_down_on(response: &egui::Response) -> bool {
        response.is_pointer_button_down_on() && response.ctx.input(|i| i.pointer.primary_pressed())
    }
}

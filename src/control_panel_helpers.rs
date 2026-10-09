use crate::orion_hid::OrionHid;
use std::sync::Arc;
use std::sync::Mutex;

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
    pub(crate) fn draw_text(
        ui: &mut egui::Ui,
        center_x: u16,
        pos_y: u16,
        color: (u8, u8, u8),
        text: &str,
        size: f32,
    ) {
        let color = egui::Color32::from_rgb(color.0, color.1, color.2);
        let label = egui::Label::new(egui::RichText::new(text).strong().size(size).color(color));
        let (_, galley, _) = label.layout_in_ui(ui);

        let pos_x = center_x as f32 - galley.size().x / 2.0;

        ui.painter().galley(
            egui::pos2(pos_x, pos_y as f32),
            galley,
            egui::Color32::PLACEHOLDER,
        );
    }

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
                ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::BLACK;
                ui.visuals_mut().widgets.hovered.weak_bg_fill = egui::Color32::BLACK;
                ui.visuals_mut().widgets.active.weak_bg_fill = egui::Color32::BLACK;
                ui.visuals_mut().widgets.open.weak_bg_fill = egui::Color32::BLACK;

                let mut selected_mut = selected;

                egui::ComboBox::from_id_salt((x, y))
                    .width(width as f32)
                    .selected_text(egui::RichText::new(selections[selected as usize]).size(10.5))
                    .show_ui(ui, |ui| {
                        for (i, selection) in selections.iter().enumerate() {
                            let response =
                                ui.selectable_value(&mut selected_mut, i as u8, *selection);

                            if response.changed() {
                                changed_callback(selected_mut);
                            }
                        }
                    });
            },
        );
    }

    pub(crate) fn draw_text_editor(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        id: egui::Id,
        text: &str,
        edit_string: &mut String,
        editing: &mut bool,
        font_size: f32,
        color: egui::Color32,
        orion_hid: &Arc<Mutex<OrionHid>>,
        mut on_commit: impl FnMut(&mut OrionHid, &String),
        mut on_tab: impl FnMut(),
    ) {
        let text_response = ui.interact(rect, id, egui::Sense::click());

        if !*editing {
            if text_response.double_clicked() {
                edit_string.clear();
                edit_string.push_str(text);
                *editing = true;
            } else {
                let galley = ui.painter().layout_no_wrap(
                    text.to_owned(),
                    egui::FontId::proportional(font_size),
                    color,
                );

                let pos = rect.center() - galley.size() / 2.0;
                ui.painter().galley(pos, galley, color);

                return;
            }
        }

        let response = ui.put(
            rect,
            egui::TextEdit::singleline(edit_string)
                .id(id)
                .font(egui::FontId::proportional(font_size))
                .horizontal_align(egui::Align::Center)
                .vertical_align(egui::Align::Center)
                .text_color(color)
                .frame(egui::Frame::NONE),
        );

        if response.has_focus() && response.gained_focus() {
            if let Some(mut state) = egui::widgets::text_edit::TextEditState::load(ui.ctx(), id) {
                let len = edit_string.chars().count();

                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::two(
                        egui::text::CCursor::new(0),
                        egui::text::CCursor::new(len),
                    )));

                state.store(ui.ctx(), id);
            }
        }

        let outside_click = ui.input(|i| {
            i.pointer.any_click()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !rect.contains(pos))
        });

        if ui.input(|i| i.key_pressed(egui::Key::Enter)) || outside_click {
            let mut orion_hid = orion_hid.lock().unwrap();

            on_commit(&mut orion_hid, edit_string);

            *editing = false;
        } else if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            let mut orion_hid = orion_hid.lock().unwrap();

            on_commit(&mut orion_hid, edit_string);
            on_tab();
        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            *editing = false;
        }
    }
}

pub(crate) struct TextEditor {
    editing_id: Option<String>,
    edit_string: String,
}

impl TextEditor {
    pub(crate) fn new() -> Self {
        Self {
            editing_id: None,
            edit_string: String::new(),
        }
    }

    pub(crate) fn draw(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        field_id: &str,
        text: &str,
        font_size: f32,
        color: egui::Color32,
    ) -> Option<(String, Option<isize>)> {
        let id = ui.id().with(field_id);
        let is_editing = self.editing_id.as_deref() == Some(field_id);

        if !is_editing {
            let text_response = ui.interact(rect, id, egui::Sense::click());

            if text_response.double_clicked() {
                self.edit_string = text.to_owned();
                self.editing_id = Some(field_id.to_owned());
            } else {
                let galley = ui.painter().layout_no_wrap(
                    text.to_owned(),
                    egui::FontId::proportional(font_size),
                    color,
                );

                let pos = rect.center() - galley.size() / 2.0;
                ui.painter().galley(pos, galley, color);

                return None;
            }
        }

        let response = ui.put(
            rect,
            egui::TextEdit::singleline(&mut self.edit_string)
                .id(id)
                .font(egui::FontId::proportional(font_size))
                .horizontal_align(egui::Align::Center)
                .vertical_align(egui::Align::Center)
                .text_color(color)
                .frame(egui::Frame::NONE)
                .lock_focus(true),
        );

        if response.has_focus() && response.gained_focus() {
            if let Some(mut state) = egui::widgets::text_edit::TextEditState::load(ui.ctx(), id) {
                let len = self.edit_string.chars().count();

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

        let (enter, esc, tab, shift) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Tab),
                i.modifiers.shift,
            )
        });

        if esc {
            self.editing_id = None;
            return None;
        }

        if tab {
            return Some((self.edit_string.clone(), Some(if shift { -1 } else { 1 })));
        }

        if enter || outside_click {
            self.editing_id = None;
            return Some((self.edit_string.clone(), None));
        }

        None
    }

    pub(crate) fn set_editing_id(&mut self, editing_id: &str) {
        self.editing_id = Some(editing_id.to_owned());
    }
}

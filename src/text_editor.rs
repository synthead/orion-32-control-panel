pub(crate) struct TextEditor {
    editing_id: Option<egui::Id>,
    edit_string: Option<String>,
    select_all_on_focus: bool,
}

impl TextEditor {
    pub(crate) fn new() -> Self {
        Self {
            editing_id: None,
            edit_string: None,
            select_all_on_focus: false,
        }
    }

    pub(crate) fn draw(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        field_id: egui::Id,
        text: &str,
        font_size: f32,
        color: egui::Color32,
    ) -> Option<(String, Option<isize>)> {
        let id = ui.id().with(field_id);
        let is_editing = self.editing_id == Some(field_id);

        if !is_editing {
            let text_response = ui.interact(rect, id, egui::Sense::click());

            if text_response.double_clicked() {
                self.editing_id = Some(field_id);
                self.select_all_on_focus = true;
            } else {
                let galley = ui.painter().layout_no_wrap(
                    text.to_owned(),
                    egui::FontId::proportional(font_size),
                    color,
                );

                let pos = rect.center() - galley.size() / 2.0;

                ui.painter().with_clip_rect(rect).galley(pos, galley, color);

                return None;
            }
        }

        let edit_string = self.edit_string.get_or_insert_with(|| text.to_owned());

        let response = ui.put(
            rect,
            egui::TextEdit::singleline(edit_string)
                .id(id)
                .font(egui::FontId::proportional(font_size))
                .horizontal_align(egui::Align::Center)
                .vertical_align(egui::Align::Center)
                .text_color(color)
                .frame(egui::Frame::NONE)
                .lock_focus(true)
                .event_filter(egui::EventFilter {
                    tab: true,
                    horizontal_arrows: true,
                    ..Default::default()
                }),
        );

        if !ui.memory(|mem| mem.has_focus(id)) {
            ui.memory_mut(|mem| mem.request_focus(id));
        }

        if self.select_all_on_focus && response.has_focus() {
            let mut state =
                egui::widgets::text_edit::TextEditState::load(ui.ctx(), id).unwrap_or_default();

            let len = edit_string.chars().count();

            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(0),
                    egui::text::CCursor::new(len),
                )));

            state.store(ui.ctx(), id);
            self.select_all_on_focus = false;
        }

        let outside_click = ui.input(|i| {
            i.pointer.button_pressed(egui::PointerButton::Primary)
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
            self.edit_string = None;
            return None;
        }

        if tab {
            let text = self.edit_string.take().unwrap_or_default();
            return Some((text, Some(if shift { -1 } else { 1 })));
        }

        if enter || outside_click {
            self.editing_id = None;
            let text = self.edit_string.take().unwrap_or_default();
            return Some((text, None));
        }

        None
    }

    pub(crate) fn set_editing_id(&mut self, editing_id: egui::Id) {
        self.editing_id = Some(editing_id);
        self.edit_string = None;
        self.select_all_on_focus = true;
    }
}

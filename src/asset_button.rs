pub(crate) struct AssetButton {
    clicked_id: Option<egui::Id>,
}

impl AssetButton {
    pub(crate) fn new() -> Self {
        Self { clicked_id: None }
    }

    pub(crate) fn pressed_when_clicked(
        &mut self,
        ui: &mut egui::Ui,
        response: egui::Response,
        state: bool,
    ) -> (bool, bool, bool) {
        let contains_pointer = response.contains_pointer();

        let (primary_down, primary_pressed, primary_released) = ui.input(|i| {
            (
                i.pointer.primary_down(),
                i.pointer.primary_pressed(),
                i.pointer.primary_released(),
            )
        });

        if contains_pointer && primary_pressed {
            self.clicked_id = Some(response.id);
        }

        if self.clicked_id != Some(response.id) {
            return (contains_pointer, state, false);
        }

        if primary_released {
            self.clicked_id = None;

            return (contains_pointer, state, contains_pointer);
        }

        (
            contains_pointer,
            state || contains_pointer && primary_down,
            false,
        )
    }

    pub(crate) fn pressed_when_down(
        &mut self,
        ui: &mut egui::Ui,
        response: egui::Response,
        state: bool,
    ) -> (bool, bool, bool) {
        let contains_pointer = response.contains_pointer();
        let primary_pressed = ui.input(|i| i.pointer.primary_pressed());

        if contains_pointer && primary_pressed {
            return (true, true, true);
        }

        return (contains_pointer, state, false);
    }
}

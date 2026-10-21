pub(crate) struct DragValue {
    start_value: u8,
    drag_value: u8,
    drag_offset: f32,
}

impl DragValue {
    pub(crate) fn new() -> Self {
        Self {
            start_value: 0,
            drag_value: 0,
            drag_offset: 0.0,
        }
    }

    pub(crate) fn value_with_drag(
        &mut self,
        ui: &mut egui::Ui,
        hitbox_rect: egui::Rect,
        value: u8,
        value_min: u8,
        value_max: u8,
        drag_travel: u16,
        drag_scale: f32,
        drag_direction_x: bool,
    ) -> (u8, bool) {
        let response = ui.allocate_rect(hitbox_rect, egui::Sense::click_and_drag());

        if response.drag_started() {
            self.start_value = value;
            self.drag_value = value;
            self.drag_offset = 0.0;
        }

        if response.dragged() {
            let drag_delta_unwrap = response.total_drag_delta().unwrap();

            let drag_delta_xy = if drag_direction_x {
                drag_delta_unwrap.x
            } else {
                drag_delta_unwrap.y
            };

            let value_span = (value_max - value_min) as f32;
            let pixels_per_value = drag_travel as f32 / value_span;
            let value_offset = (self.start_value - value_min) as f32;

            let current_pos = value_offset * pixels_per_value;
            let drag_delta = drag_delta_xy * drag_scale;
            let raw_pos = current_pos - drag_delta;
            let clamped_pos = raw_pos.clamp(0.0, drag_travel as f32);

            let value_unclamped = (value_min as f32 + clamped_pos / pixels_per_value).round() as u8;

            let value_new = value_unclamped.clamp(value_min, value_max);
            let should_write = value_new != self.drag_value;

            self.drag_value = value_new;

            return (self.drag_value, should_write);
        }

        (value, false)
    }
}

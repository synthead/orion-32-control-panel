struct ControlPanelVuMetersDimensions;

impl ControlPanelVuMetersDimensions {
    const VU_METER_LEFT_X: f32 = 135.0;
    const VU_METER_RIGHT_X: f32 = 716.0;
    const VU_METER_X_STEP: f32 = 11.0;
    const VU_METER_Y: f32 = 60.0;

    const VU_METER_CURVE_STRENGTH: f32 = 1.5;
    const VU_METER_FALL_DB_PER_SECOND: f32 = 80.0;
    const VU_METER_PEAK_HEIGHT: f32 = 2.0;
    const VU_METER_PEAK_HOLD_SECONDS: f32 = 1.0;

    const VU_METER_MAX_DB: u8 = 0x60;
    const VU_METER_DB_SPAN: f32 = 96.0;
}

const SILENT_DB: f32 = ControlPanelVuMetersDimensions::VU_METER_MAX_DB as f32;

#[derive(Clone, Copy)]
struct MeterState {
    level_db: f32,
    peak_db: f32,
    peak_hold_left: f32,
}

impl MeterState {
    const SILENT: Self = Self {
        level_db: SILENT_DB,
        peak_db: SILENT_DB,
        peak_hold_left: 0.0,
    };

    fn is_active(&self) -> bool {
        self.level_db < SILENT_DB || self.peak_db < SILENT_DB
    }

    fn update(&mut self, db: u8, dt: f32) {
        let input = (db as f32).min(SILENT_DB);

        if input <= self.level_db {
            self.level_db = input;
        } else {
            self.level_db = (self.level_db
                + ControlPanelVuMetersDimensions::VU_METER_FALL_DB_PER_SECOND * dt)
                .min(input);
        }

        if input < SILENT_DB && input <= self.peak_db {
            self.peak_db = input;
            self.peak_hold_left = ControlPanelVuMetersDimensions::VU_METER_PEAK_HOLD_SECONDS;
        } else if self.peak_db < SILENT_DB {
            self.peak_hold_left -= dt;
            if self.peak_hold_left <= 0.0 {
                self.peak_db = SILENT_DB;
            }
        }
    }
}

pub(crate) struct ControlPanelVuMeters {
    texture: egui::TextureHandle,
    meter_width: f32,
    meter_height: f32,
    meter_state_left: [MeterState; 32],
    meter_state_right: [MeterState; 32],
}

impl ControlPanelVuMeters {
    pub(crate) fn new(ctx: &egui::Context) -> Self {
        let image = image::load_from_memory(include_bytes!(
            "../assets/images/MonitorPeakMeter/default/background.png"
        ))
        .unwrap()
        .into_rgba8();

        let (width, height) = image.dimensions();

        let texture = ctx.load_texture(
            "vu_meter",
            egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                image.as_raw(),
            ),
            Default::default(),
        );

        Self {
            texture,
            meter_width: width as f32,
            meter_height: height as f32,
            meter_state_left: [MeterState::SILENT; 32],
            meter_state_right: [MeterState::SILENT; 32],
        }
    }

    pub(crate) fn draw(
        &mut self,
        ui: &mut egui::Ui,
        left_vu_values: &[u8; 32],
        right_vu_values: &[u8; 32],
    ) {
        let dt = ui.input(|i| i.stable_dt).min(0.1);
        let mut animating = false;

        for index in 0..32 {
            self.meter_state_left[index].update(left_vu_values[index], dt);
            self.meter_state_right[index].update(right_vu_values[index], dt);

            animating |= self.meter_state_left[index].is_active()
                || self.meter_state_right[index].is_active();
        }

        let painter = ui.painter();

        for index in 0..32 {
            let offset_x = index as f32 * ControlPanelVuMetersDimensions::VU_METER_X_STEP;
            let pos_x_left = ControlPanelVuMetersDimensions::VU_METER_LEFT_X + offset_x;
            let pos_x_right = ControlPanelVuMetersDimensions::VU_METER_RIGHT_X + offset_x;

            self.draw_meter(painter, pos_x_left, &self.meter_state_left[index]);
            self.draw_meter(painter, pos_x_right, &self.meter_state_right[index]);
        }

        if animating {
            ui.ctx().request_repaint();
        }
    }

    fn y_for_db(&self, db: f32) -> f32 {
        let normalized_level =
            1.0 - (db / ControlPanelVuMetersDimensions::VU_METER_DB_SPAN).clamp(0.0, 1.0);

        let curve_strength = ControlPanelVuMetersDimensions::VU_METER_CURVE_STRENGTH;
        let curved_level = if curve_strength > 0.0 {
            (curve_strength * normalized_level).exp_m1() / curve_strength.exp_m1()
        } else {
            normalized_level
        };

        (ControlPanelVuMetersDimensions::VU_METER_Y + self.meter_height * (1.0 - curved_level))
            .round()
    }

    fn draw_slice(&self, painter: &egui::Painter, meter_x: f32, top_y: f32, bottom_y: f32) {
        if bottom_y <= top_y {
            return;
        }

        let meter_top_y = ControlPanelVuMetersDimensions::VU_METER_Y;
        let uv_top = (top_y - meter_top_y) / self.meter_height;
        let uv_bottom = (bottom_y - meter_top_y) / self.meter_height;

        painter.image(
            self.texture.id(),
            egui::Rect::from_min_max(
                egui::pos2(meter_x, top_y),
                egui::pos2(meter_x + self.meter_width, bottom_y),
            ),
            egui::Rect::from_min_max(egui::pos2(0.0, uv_top), egui::pos2(1.0, uv_bottom)),
            egui::Color32::WHITE,
        );
    }

    fn draw_meter(&self, painter: &egui::Painter, meter_x: f32, state: &MeterState) {
        let meter_bottom_y = ControlPanelVuMetersDimensions::VU_METER_Y + self.meter_height;

        if state.level_db < SILENT_DB {
            self.draw_slice(
                painter,
                meter_x,
                self.y_for_db(state.level_db),
                meter_bottom_y,
            );
        }

        if state.peak_db < SILENT_DB {
            let peak_top = self.y_for_db(state.peak_db);
            let peak_bottom = (peak_top + ControlPanelVuMetersDimensions::VU_METER_PEAK_HEIGHT)
                .min(meter_bottom_y);

            self.draw_slice(painter, meter_x, peak_top, peak_bottom);
        }
    }
}

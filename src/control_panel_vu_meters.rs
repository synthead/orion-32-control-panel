struct ControlPanelVuMetersDimensions;

impl ControlPanelVuMetersDimensions {
    const VU_METER_LEFT_X: usize = 135;
    const VU_METER_RIGHT_X: usize = 716;
    const VU_METER_X_STEP: usize = 11;
    const VU_METER_Y: f32 = 60.0;

    const VU_METER_MAX_DB: u8 = 0x60;
    const VU_METER_DB_SPAN: f32 = 96.0;
}

pub(crate) struct ControlPanelVuMeters {
    texture: egui::TextureHandle,
    meter_width: f32,
    meter_height: f32,
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
        }
    }

    pub(crate) fn draw(
        &self,
        ui: &mut egui::Ui,
        left_vu_values: &[u8; 32],
        right_vu_values: &[u8; 32],
    ) {
        let painter = ui.painter();

        for index in 0..32 {
            let offset_x = index * ControlPanelVuMetersDimensions::VU_METER_X_STEP;

            self.draw_meter(
                painter,
                ControlPanelVuMetersDimensions::VU_METER_LEFT_X + offset_x,
                left_vu_values[index],
            );

            self.draw_meter(
                painter,
                ControlPanelVuMetersDimensions::VU_METER_RIGHT_X + offset_x,
                right_vu_values[index],
            );
        }
    }

    fn draw_meter(&self, painter: &egui::Painter, meter_x: usize, db: u8) {
        if db >= ControlPanelVuMetersDimensions::VU_METER_MAX_DB {
            return;
        }

        let meter_left_x = meter_x as f32;
        let meter_top_y = ControlPanelVuMetersDimensions::VU_METER_Y;
        let meter_bottom_y = meter_top_y + self.meter_height;

        let visible_top_y = (meter_top_y
            + self.meter_height * db as f32 / ControlPanelVuMetersDimensions::VU_METER_DB_SPAN)
            .round();

        let visible_top_uv_y = (visible_top_y - meter_top_y) / self.meter_height;

        painter.image(
            self.texture.id(),
            egui::Rect::from_min_max(
                egui::pos2(meter_left_x, visible_top_y),
                egui::pos2(meter_left_x + self.meter_width, meter_bottom_y),
            ),
            egui::Rect::from_min_max(egui::pos2(0.0, visible_top_uv_y), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}

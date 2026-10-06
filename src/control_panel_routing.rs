use std::collections::BTreeSet;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

struct ControlPanelRouteColors;

impl ControlPanelRouteColors {
    const ADC_DAC: egui::Color32 = egui::Color32::from_rgb(0x20, 0xa4, 0xf7);
    const USB: egui::Color32 = egui::Color32::from_rgb(0x00, 0x73, 0xff);
    const MADI: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb8, 0x00);
    const ADAT: egui::Color32 = egui::Color32::from_rgb(0xff, 0x00, 0x37);
    const SPDIF: egui::Color32 = egui::Color32::from_rgb(0xd9, 0x24, 0x94);
    const MIX_LR: egui::Color32 = egui::Color32::from_rgb(0x72, 0xab, 0x20);
    const OSC_1: egui::Color32 = egui::Color32::from_rgb(0x38, 0x90, 0xa7);
    const OSC_2: egui::Color32 = egui::Color32::from_rgb(0x90, 0x38, 0xa7);
    const MUTE: egui::Color32 = egui::Color32::from_rgb(0x7b, 0x7b, 0x7b);
    const SELECT: egui::Color32 = egui::Color32::from_rgb(0x91, 0xb4, 0xdf);
}

struct ControlPanelRouteDimensions;

impl ControlPanelRouteDimensions {
    const START_X: f32 = 127.0;
    const START_Y_SOURCE: f32 = 284.0;
    const START_Y_TARGET: f32 = 448.0;

    const STEP_X: f32 = 30.0;
    const STEP_Y: f32 = 15.0;

    const SIZE_X: f32 = 29.0;
    const SIZE_Y: f32 = 14.0;
}

pub(crate) struct ControlPanelRouting {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,

    route_labels: [[String; 32]; 7],

    route_source_counts: [usize; 7],
    route_target_counts: [usize; 7],

    route_label_edit_cell: Option<[u8; 2]>,
    route_label_edit_string: String,

    route_source_select_range_row: Option<u8>,
    route_source_select_range_column: u8,

    route_source_select_toggle_columns: HashSet<u8>,

    route_source_select_row: u8,
    route_source_select_columns: BTreeSet<u8>,

    route_target_select_cell: Option<[u8; 2]>,

    route_source_mousedown: bool,
    route_target_hover_cell: Option<[u8; 2]>,
}

impl ControlPanelRouting {
    const SOURCE_ROW_LABELS: [&str; 7] = [
        "ADC",
        "USB PLAY",
        "MADI IN 1-32",
        "MADI IN 33-64",
        "ADAT IN",
        "S/PDIF IN",
        "MIX 1 L/R",
    ];

    pub(crate) fn new(
        orion_state: Arc<Mutex<OrionState>>,
        orion_hid: Arc<Mutex<OrionHid>>,
    ) -> Self {
        let route_labels = std::array::from_fn(|group| {
            let start_at = if group == 3 { 33 } else { 1 };

            std::array::from_fn(|i| (i + start_at).to_string())
        });

        Self {
            orion_state,
            orion_hid,

            route_labels,

            route_source_counts: [0; 7],
            route_target_counts: [0; 7],

            route_label_edit_cell: None,
            route_label_edit_string: String::new(),

            route_source_select_range_row: None,
            route_source_select_range_column: 0,

            route_source_select_toggle_columns: HashSet::new(),

            route_source_select_row: 0,
            route_source_select_columns: BTreeSet::new(),

            route_target_select_cell: None,

            route_source_mousedown: false,
            route_target_hover_cell: None,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        let orion_state = self.orion_state.lock().unwrap().clone();

        let madi_1_count = match orion_state.sample_rate {
            5 | 6 => 16,
            _ => 32,
        };

        let madi_2_count = match orion_state.sample_rate {
            0 | 1 | 2 => 32,
            _ => 0,
        };

        let adat_count = match orion_state.sample_rate {
            3 | 4 => 8,
            5 | 6 => 4,
            _ => 16,
        };

        self.route_source_counts = [
            32,
            32,
            madi_1_count.try_into().unwrap(),
            madi_2_count.try_into().unwrap(),
            adat_count.try_into().unwrap(),
            2,
            2,
        ];

        self.route_target_counts = [
            32,
            32,
            madi_1_count.try_into().unwrap(),
            madi_2_count.try_into().unwrap(),
            adat_count.try_into().unwrap(),
            2,
            32,
        ];

        self.draw_route_sources(ui);
        self.draw_route_targets(ui, &orion_state.routes);

        self.draw_route_selections(ui);
        self.draw_route_drag(ui);
        self.draw_route_text(ui);

        self.draw_signals(ui, orion_state.signals);
        self.draw_route_navbar(ui);

        self.clear_target_selection_when_outside(ui);
        self.update_route_targets_on_mouseup(ui, &orion_state.routes);

        if self.route_label_edit_cell.is_some() {
            self.draw_route_editor(ui);
        }
    }

    fn draw_route_sources(&mut self, ui: &mut egui::Ui) {
        for row in 0..self.route_source_counts.len() {
            for column in 0..self.route_source_counts[row] as u8 {
                let label = &self.route_labels[row][column as usize].clone();
                let color = Self::route_color_for_route(row as u8);

                let rect =
                    Self::draw_route_rect_with_label(ui, column, row as u8, true, color, label);

                let id = ui.id().with(("route_source", column, row as u8));
                let response = ui.interact(rect, id, egui::Sense::click());

                if response.hovered() && ui.input(|i| i.pointer.primary_pressed()) {
                    self.route_source_mousedown = true;

                    self.update_source_selections(ui, column, row as u8);
                }

                if response.double_clicked() {
                    self.route_label_edit_string = label.clone();
                    self.route_label_edit_cell = Some([column, row as u8]);
                }
            }
        }

        if ui.input(|i| i.pointer.any_pressed()) && !self.route_source_mousedown {
            self.clear_selections();
        }
    }

    fn update_source_selections(&mut self, ui: &mut egui::Ui, column: u8, row: u8) {
        let shift_held = ui.input(|i| i.modifiers.shift);
        let ctrl_held = ui.input(|i| i.modifiers.ctrl);

        self.route_target_select_cell = None;

        if self.route_source_select_row != row {
            self.route_source_select_row = row;

            self.route_source_select_toggle_columns.clear();
            self.route_source_select_columns.clear();
        }

        'done: {
            if shift_held {
                self.route_source_select_columns.clear();

                if self.route_source_select_range_row == Some(row) {
                    self.route_source_select_columns.extend(
                        self.route_source_select_range_column.min(column)
                            ..=self.route_source_select_range_column.max(column),
                    );
                } else {
                    self.route_source_select_columns.insert(column);
                }

                break 'done;
            }

            self.route_source_select_range_row = Some(row);
            self.route_source_select_range_column = column;

            self.route_source_select_toggle_columns =
                self.route_source_select_columns.iter().copied().collect();

            if !ctrl_held {
                if self.route_source_select_toggle_columns.contains(&column) {
                    break 'done;
                }

                self.route_source_select_toggle_columns.clear();
            }

            self.route_source_select_columns.clear();

            if !self.route_source_select_toggle_columns.remove(&column) {
                self.route_source_select_toggle_columns.insert(column);
            }
        }

        self.route_source_select_columns
            .extend(self.route_source_select_toggle_columns.iter().copied());
    }

    fn draw_route_targets(&mut self, ui: &mut egui::Ui, routes: &[[u8; 32]; 7]) {
        self.route_target_hover_cell = None;

        for row in 0..self.route_target_counts.len() {
            for column in 0..self.route_target_counts[row] {
                let route = routes[row][column];
                let color = Self::route_color_for_route(route);
                let label = &self.route_label_for_route(route);

                let rect = Self::draw_route_rect_with_label(
                    ui,
                    column as u8,
                    row as u8,
                    false,
                    color,
                    label,
                );

                let id = ui.id().with(("route_target", column, row as u8));
                let response = ui.interact(rect, id, egui::Sense::click());

                if response.contains_pointer() {
                    self.route_target_hover_cell = Some([column as u8, row as u8]);

                    if ui.input(|i| i.pointer.primary_pressed()) {
                        self.update_target_selections(ui, column as u8, row as u8, route);
                    }
                }
            }
        }
    }

    fn update_target_selections(&mut self, ui: &mut egui::Ui, column: u8, row: u8, route: u8) {
        self.route_target_select_cell = Some([column, row]);

        self.route_source_select_toggle_columns.clear();
        self.route_source_select_columns.clear();

        if let Some([source_column, source_row]) = Self::route_source_cell_for_route(route) {
            self.route_source_select_row = source_row;
            self.route_source_select_columns.insert(source_column);
        }
    }

    fn draw_route_selection(&self, ui: &mut egui::Ui, column: u8, row: u8, is_source: bool) {
        let label = &self.route_labels[row as usize][column as usize];

        Self::draw_route_rect_with_label(
            ui,
            column,
            row,
            is_source,
            ControlPanelRouteColors::SELECT,
            label,
        );
    }

    fn draw_route_selections(&self, ui: &mut egui::Ui) {
        for &column in &self.route_source_select_columns {
            self.draw_route_selection(ui, column, self.route_source_select_row, true);
        }

        if let Some([target_column, target_row]) = self.route_target_select_cell {
            self.draw_route_selection(ui, target_column, target_row, false);
        }
    }

    fn draw_route_text(&self, ui: &mut egui::Ui) {
        if self.route_source_select_columns.is_empty() {
            return;
        }

        let text = format!(
            "{}: {}",
            Self::SOURCE_ROW_LABELS[self.route_source_select_row as usize],
            self.route_source_select_columns
                .iter()
                .map(
                    |&column| self.route_labels[self.route_source_select_row as usize]
                        [column as usize]
                        .as_str()
                )
                .collect::<Vec<_>>()
                .join(", ")
        );

        ui.painter().text(
            egui::pos2(128.0, 409.0),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
    }

    fn draw_signals(&self, ui: &mut egui::Ui, signals: u8) {
        // TODO: Figure out the rest of these signals.

        if signals == 0x09 {
            render_asset!(ui, 1095, 301, "PresenceLed/default/pressed");
        } else {
            render_asset!(ui, 1095, 301, "PresenceLed/default/released");
        }

        render_asset!(ui, 1095, 323, "PresenceLed/default/released");
        render_asset!(ui, 1095, 346, "PresenceLed/default/released");
        render_asset!(ui, 1112, 346, "PresenceLed/default/released");
        render_asset!(ui, 1095, 361, "PresenceLed/default/released");
    }

    fn draw_route_navbar(&self, ui: &mut egui::Ui) {
        render_asset!(ui, 1106, 410, "UndoButton/default/released");
        render_asset!(ui, 1106, 436, "RedoButton/default/released");
        render_asset!(ui, 1106, 462, "SaveRoutingButton/default/released");
        render_asset!(ui, 1106, 488, "LoadRoutingButton/default/released");
    }

    fn clear_target_selection_when_outside(&mut self, ui: &mut egui::Ui) {
        let rect = egui::Rect::from_min_max(egui::pos2(127.0, 448.0), egui::pos2(1086.0, 552.0));
        let hover = ui.rect_contains_pointer(rect);

        if self.route_target_select_cell.is_some() && !hover {
            self.clear_selections();
        }
    }

    fn update_route_targets_on_mouseup(&mut self, ui: &mut egui::Ui, routes: &[[u8; 32]; 7]) {
        if self.route_source_mousedown && ui.input(|i| i.pointer.primary_released()) {
            if let Some([target_column, target_row]) = self.route_target_hover_cell {
                let mut new_routes = routes[target_row as usize];
                let target_row_count = self.route_target_counts[target_row as usize];

                for (column_index, &source_column) in
                    self.route_source_select_columns.iter().enumerate()
                {
                    if target_column as usize + column_index >= target_row_count {
                        break;
                    }

                    let route = source_column * 0x08 + self.route_source_select_row;

                    new_routes[target_column as usize + column_index] = route;
                }

                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_routes(target_row, &new_routes);
            }

            self.route_source_mousedown = false;
        }
    }

    fn clear_selections(&mut self) {
        self.route_target_select_cell = None;
        self.route_source_select_columns.clear();
    }

    fn draw_route_drag(&mut self, ui: &mut egui::Ui) {
        if !self.route_source_mousedown || self.route_target_hover_cell.is_none() {
            return;
        }

        let [target_column, target_row] = self.route_target_hover_cell.unwrap();
        let target_row_count = self.route_target_counts[target_row as usize];

        for (column_index, &source_column) in self.route_source_select_columns.iter().enumerate() {
            if target_column as usize + column_index >= target_row_count {
                return;
            }

            let label =
                &self.route_labels[self.route_source_select_row as usize][source_column as usize];
            let color = Self::route_color_for_route(self.route_source_select_row);

            Self::draw_route_rect_with_label(
                ui,
                target_column + column_index as u8,
                target_row,
                false,
                color,
                label,
            );
        }
    }

    fn draw_route_editor(&mut self, ui: &mut egui::Ui) {
        let [column, row] = self.route_label_edit_cell.unwrap();
        let (pos_x, pos_y) = Self::route_cell_pos(column, row, true);
        let color = Self::route_color_for_route(row);

        let rect = egui::Rect::from_min_size(
            egui::pos2(pos_x - 14.0, pos_y - 2.0),
            egui::vec2(58.0, 18.0),
        );

        ui.painter().rect(
            rect,
            egui::CornerRadius::same(5),
            color,
            egui::Stroke::new(1.0, egui::Color32::BLACK),
            egui::StrokeKind::Inside,
        );

        let hitbox_rect = egui::Rect::from_min_size(
            egui::pos2(pos_x, pos_y),
            egui::vec2(
                ControlPanelRouteDimensions::SIZE_X,
                ControlPanelRouteDimensions::SIZE_Y,
            ),
        );

        if ui.input(|i| {
            i.pointer.any_click()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !hitbox_rect.contains(pos))
        }) {
            self.route_labels[row as usize][column as usize] = self.route_label_edit_string.clone();
            self.route_label_edit_cell = None;
        }

        let text_edit_id = ui.id().with(("label_route_edit", column, row));

        if !ui.memory(|mem| mem.has_focus(text_edit_id)) {
            let mut state = egui::widgets::text_edit::TextEditState::load(ui.ctx(), text_edit_id)
                .unwrap_or_default();

            let label_chars = self.route_label_edit_string.chars().count();

            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(0),
                    egui::text::CCursor::new(label_chars),
                )));

            state.store(ui.ctx(), text_edit_id);

            ui.memory_mut(|mem| mem.request_focus(text_edit_id));
        }

        ui.put(
            rect,
            egui::TextEdit::singleline(&mut self.route_label_edit_string)
                .id(text_edit_id)
                .font(egui::FontId::proportional(12.0))
                .horizontal_align(egui::Align::Center)
                .vertical_align(egui::Align::Center)
                .text_color(egui::Color32::WHITE)
                .frame(egui::Frame::NONE)
                .event_filter(egui::EventFilter {
                    tab: true,
                    ..Default::default()
                }),
        );

        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.route_labels[row as usize][column as usize] = self.route_label_edit_string.clone();
            self.route_label_edit_cell = None;
        }

        if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.route_labels[row as usize][column as usize] = self.route_label_edit_string.clone();

            let mut column_next = column as usize;
            let mut row_next = row as usize;

            column_next += 1;

            if column_next >= self.route_source_counts[row_next] {
                column_next = 0;
                row_next += 1;
            }

            if row_next >= self.route_source_counts.len() {
                row_next = 0;
            }

            self.route_label_edit_string = self.route_labels[row_next][column_next].clone();
            self.route_label_edit_cell = Some([column_next as u8, row_next as u8]);
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.route_label_edit_cell = None;
        }
    }

    fn route_source_cell_for_route(route: u8) -> Option<[u8; 2]> {
        match route {
            0x07 | 0x0f | 0x17 => None,

            _ => {
                let row = route % 0x08;
                let column = route / 0x08;

                Some([column, row])
            }
        }
    }

    fn route_label_for_route(&self, route: u8) -> String {
        match route {
            0x07 => "O1".to_string(),
            0x0f => "O2".to_string(),
            0x17 => "M".to_string(),

            _ => {
                let route_type = route % 0x08;
                let route_index = route / 0x08;

                self.route_labels[route_type as usize][route_index as usize].clone()
            }
        }
    }

    fn route_color_for_route(route: u8) -> egui::Color32 {
        match route {
            0x07 => ControlPanelRouteColors::OSC_1,
            0x0f => ControlPanelRouteColors::OSC_2,
            0x17 => ControlPanelRouteColors::MUTE,

            _ => match route % 0x08 {
                0x00 => ControlPanelRouteColors::ADC_DAC,
                0x01 => ControlPanelRouteColors::USB,
                0x02 => ControlPanelRouteColors::MADI,
                0x03 => ControlPanelRouteColors::MADI,
                0x04 => ControlPanelRouteColors::ADAT,
                0x05 => ControlPanelRouteColors::SPDIF,
                0x06 => ControlPanelRouteColors::MIX_LR,

                _ => egui::Color32::BLACK,
            },
        }
    }

    fn route_cell_pos(column: u8, row: u8, is_source: bool) -> (f32, f32) {
        let base_pos_y = if is_source {
            ControlPanelRouteDimensions::START_Y_SOURCE
        } else {
            ControlPanelRouteDimensions::START_Y_TARGET
        };

        let pos_x = ControlPanelRouteDimensions::START_X
            + column as f32 * ControlPanelRouteDimensions::STEP_X;
        let pos_y = base_pos_y + row as f32 * ControlPanelRouteDimensions::STEP_Y;

        (pos_x, pos_y)
    }

    fn draw_route_rect_with_label(
        ui: &mut egui::Ui,
        column: u8,
        row: u8,
        is_source: bool,
        color: egui::Color32,
        label: &String,
    ) -> egui::Rect {
        let (pos_x, pos_y) = Self::route_cell_pos(column, row, is_source);

        let rect = egui::Rect::from_min_size(
            egui::pos2(pos_x, pos_y),
            egui::vec2(
                ControlPanelRouteDimensions::SIZE_X,
                ControlPanelRouteDimensions::SIZE_Y,
            ),
        );

        ui.painter().rect_filled(rect, 0.0, color);

        ControlPanelHelpers::draw_text(
            ui,
            pos_x as u16 + 15,
            pos_y as u16,
            (0xff, 0xff, 0xff),
            label,
        );

        rect
    }
}

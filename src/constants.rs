pub(crate) struct FilePaths;

impl FilePaths {
    pub(crate) const CONFIG_DIR: &str = "Orion 32 Control Panel";
    pub(crate) const PERSISTENCE_FILE: &str = "persistence.json";
    pub(crate) const ROUTING_FILE_DESCRIPTION: &str = "Orion 32 Routing File";
    pub(crate) const ROUTING_FILE_EXTENSIONS: &[&str] = &["o32r"];
}

pub(crate) struct WidgetColors;

impl WidgetColors {
    pub(crate) const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0xa0, 0xa0, 0xa0);
    pub(crate) const TEXT_COLOR_DEVICE: egui::Color32 = egui::Color32::WHITE;
    pub(crate) const TEXT_COLOR_DEVICE_STATUS: egui::Color32 = egui::Color32::RED;
    pub(crate) const TEXT_COLOR_LABEL: egui::Color32 = egui::Color32::WHITE;
    pub(crate) const TEXT_COLOR_PRESET: egui::Color32 = egui::Color32::from_rgb(0x64, 0xe7, 0x1d);
    pub(crate) const TEXT_COLOR_ABOUT: egui::Color32 = egui::Color32::WHITE;
    pub(crate) const TEXT_COLOR_DISABLED: egui::Color32 = egui::Color32::from_rgb(0xd0, 0xd0, 0xd0);
    pub(crate) const BG_COLOR: egui::Color32 = egui::Color32::BLACK;
    pub(crate) const ARROW_COLOR: egui::Color32 = egui::Color32::from_rgb(0x85, 0xc8, 0x26);
}

pub(crate) struct WidgetDimensions;

impl WidgetDimensions {
    pub(crate) const TEXT_SIZE: f32 = 10.5;
    pub(crate) const TEXT_SIZE_MIXER_VALUE: f32 = 12.0;
    pub(crate) const TEXT_SIZE_DEVICE_STATUS: f32 = 15.0;
}

pub(crate) struct RoutingLabelColors;

impl RoutingLabelColors {
    pub(crate) const ADC_DAC: egui::Color32 = egui::Color32::from_rgb(0x20, 0xa4, 0xf7);
    pub(crate) const USB: egui::Color32 = egui::Color32::from_rgb(0x00, 0x73, 0xff);
    pub(crate) const MADI: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb8, 0x00);
    pub(crate) const ADAT: egui::Color32 = egui::Color32::from_rgb(0xff, 0x00, 0x37);
    pub(crate) const SPDIF: egui::Color32 = egui::Color32::from_rgb(0xd9, 0x24, 0x94);
    pub(crate) const MIX_LR: egui::Color32 = egui::Color32::from_rgb(0x72, 0xab, 0x20);
    pub(crate) const OSC_1: egui::Color32 = egui::Color32::from_rgb(0x38, 0x90, 0xa7);
    pub(crate) const OSC_2: egui::Color32 = egui::Color32::from_rgb(0x90, 0x38, 0xa7);
    pub(crate) const MUTE: egui::Color32 = egui::Color32::from_rgb(0x7b, 0x7b, 0x7b);
    pub(crate) const SELECT: egui::Color32 = egui::Color32::from_rgb(0x91, 0xb4, 0xdf);
}

pub(crate) struct RoutingRows;

impl RoutingRows {
    pub(crate) const ADC_DAC: u8 = 0x00;
    pub(crate) const USB: u8 = 0x01;
    pub(crate) const MADI_1: u8 = 0x02;
    pub(crate) const MADI_2: u8 = 0x03;
    pub(crate) const ADAT: u8 = 0x04;
    pub(crate) const SPDIF: u8 = 0x05;
    pub(crate) const MIX_LR: u8 = 0x06;
    pub(crate) const SPECIAL: u8 = 0x07;
}

pub(crate) struct RoutingColumns;

impl RoutingColumns {
    pub(crate) const OSC_1: u8 = 0x00;
    pub(crate) const OSC_2: u8 = 0x01;
    pub(crate) const MUTE: u8 = 0x02;
    pub(crate) const STEP: u8 = 0x08;
}

pub(crate) struct RoutingLabels;

impl RoutingLabels {
    pub(crate) const OSC_1: &str = "O1";
    pub(crate) const OSC_2: &str = "O2";
    pub(crate) const MUTE: &str = "M";

    pub(crate) const DEFAULT_LABEL_NUMBERS: [[u8; 32]; 7] = {
        let mut defaults_1_32 = [0; 32];
        let mut defaults_33_64 = [0; 32];

        let mut column = 0;

        while column < 32 {
            defaults_1_32[column] = (column + 1) as u8;
            defaults_33_64[column] = (column + 33) as u8;

            column += 1;
        }

        [
            defaults_1_32,
            defaults_1_32,
            defaults_1_32,
            defaults_33_64,
            defaults_1_32,
            defaults_1_32,
            defaults_1_32,
        ]
    };
}

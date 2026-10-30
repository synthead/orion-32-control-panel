pub(crate) struct WidgetColors;

impl WidgetColors {
    pub(crate) const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x95, 0x95, 0x95);
    pub(crate) const BG_COLOR: egui::Color32 = egui::Color32::BLACK;
    pub(crate) const ARROW_COLOR: egui::Color32 = egui::Color32::from_rgb(0x85, 0xc8, 0x26);
}

pub(crate) struct WidgetDimensions;

impl WidgetDimensions {
    pub(crate) const TEXT_SIZE: f32 = 10.5;
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
}

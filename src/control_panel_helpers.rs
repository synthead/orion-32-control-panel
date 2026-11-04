#[macro_export]
macro_rules! preload_assets {
    ($ctx:expr, $($subpath:literal),* $(,)?) => {{
        let mut ready = true;

        $(
            ready &= !matches!(
                egui::include_image!(concat!("../assets/images/", $subpath, ".png"))
                    .load($ctx, Default::default(), Default::default()),
                Ok(egui::load::TexturePoll::Pending { .. })
            );
        )*

        ready
    }};
}

#[macro_export]
macro_rules! preload_assets_blocking {
    ($ctx:expr) => {
        $crate::preload_assets!(
            $ctx,
            "AboutButton/default/released",
            "AvailableDevicesSlide/default/background-edited",
            "CloseButton/default/released",
            "ControlPanel/default/background-edited",
            "LineOutVolumePot/default/layers/pointer",
            "LoadRoutingButton/default/released",
            "LockLed/default/background",
            "MinimizeButton/default/released",
            "MixerNavButton/default/released@2x",
            "PhaseInvNavButton/default/released",
            "PresenceLed/default/pressed",
            "PresenceLed/default/released",
            "RedoButton/default/released",
            "Routing/default/background",
            "RoutingNavButton/default/pressed@2x",
            "SaveRoutingButton/default/released",
            "SettingsButton/default/released",
            "StandbyButton/default/pressed",
            "StandbyButton/default/released",
            "UndoButton/default/released",
        )
    };
}

#[macro_export]
macro_rules! preload_assets_nonblocking {
    ($ctx:expr) => {
        $crate::preload_assets!(
            $ctx,
            "AboutBox/default/background-edited",
            "AboutButton/default/pressed",
            "BrightnessSlider/default/layers/handle",
            "CloseButton/default/pressed",
            "LinkButton/default/pressed",
            "LinkButton/default/released",
            "LoadRoutingButton/default/pressed",
            "MasterMixerSlider/default/layers/handle",
            "MinimizeButton/default/pressed",
            "Mixer/default/background",
            "MixerNavButton/default/pressed@2x",
            "MixerPan/default/layers/pointer",
            "MixerSlider/default/layers/handle",
            "MuteButton/default/pressed",
            "MuteButton/default/released",
            "PhaseInvNavButton/default/pressed",
            "PhaseInvertButton/default/pressed",
            "PhaseInvertButton/default/released",
            "PhaseInvertView/default/background",
            "RedoButton/default/pressed",
            "RoutingNavButton/default/released@2x",
            "SaveRoutingButton/default/pressed",
            "SettingsButton/default/pressed",
            "SettingsLabel/default/background",
            "SettingsPanel/default/background-edited",
            "SettingsPanelButton/default/pressed",
            "SettingsPanelButton/default/released",
            "SoloButton/default/pressed",
            "SoloButton/default/released",
            "UndoButton/default/pressed",
        )
    };
}

#[macro_export]
macro_rules! render_asset {
    ($ui:expr, $x:expr, $y:expr, $subpath:literal, $scale:expr, $rotation:expr, $sense:expr) => {
        $ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt(egui::Id::new(($subpath, $x as i32, $y as i32)))
                .max_rect(egui::Rect::from_min_size(
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

use crate::constants::{WidgetColors, WidgetDimensions};

pub(crate) struct ControlPanelHelpers;

impl ControlPanelHelpers {
    pub(crate) fn draw_combobox(
        ui: &mut egui::Ui,
        x: u16,
        y: u16,
        width: u8,
        selected: u8,
        selected_text: Option<&str>,
        selections: &[&str],
        enabled: bool,
        mut changed_callback: impl FnMut(u8),
    ) {
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                egui::pos2(x as f32, y as f32),
                egui::vec2(width as f32, 17.0),
            )),
            |ui| {
                ui.visuals_mut().widgets.inactive.weak_bg_fill = WidgetColors::BG_COLOR;
                ui.visuals_mut().widgets.hovered.weak_bg_fill = WidgetColors::BG_COLOR;
                ui.visuals_mut().widgets.active.weak_bg_fill = WidgetColors::BG_COLOR;
                ui.visuals_mut().widgets.open.weak_bg_fill = WidgetColors::BG_COLOR;

                ui.visuals_mut().widgets.inactive.fg_stroke.color = WidgetColors::ARROW_COLOR;
                ui.visuals_mut().widgets.hovered.fg_stroke.color = WidgetColors::ARROW_COLOR;
                ui.visuals_mut().widgets.active.fg_stroke.color = WidgetColors::ARROW_COLOR;
                ui.visuals_mut().widgets.open.fg_stroke.color = WidgetColors::ARROW_COLOR;

                let inactive_stroke = ui.visuals().widgets.inactive.bg_stroke;

                ui.visuals_mut().widgets.hovered.bg_stroke = inactive_stroke;
                ui.visuals_mut().widgets.active.bg_stroke = inactive_stroke;
                ui.visuals_mut().widgets.open.bg_stroke = inactive_stroke;

                ui.spacing_mut().interact_size.y -= 2.0;
                ui.spacing_mut().combo_height = 1000.0;

                let mut selected_mut = selected;

                ui.add_enabled_ui(enabled, |ui| {
                    egui::ComboBox::from_id_salt((x, y))
                        .width(width as f32)
                        .selected_text(
                            egui::RichText::new(
                                selected_text.unwrap_or(selections[selected as usize]),
                            )
                            .size(WidgetDimensions::TEXT_SIZE)
                            .color(if enabled {
                                WidgetColors::TEXT_COLOR
                            } else {
                                WidgetColors::TEXT_COLOR_DISABLED
                            }),
                        )
                        .popup_style(egui::style::StyleModifier::new(|style| {
                            style.visuals.window_fill = WidgetColors::BG_COLOR;
                        }))
                        .show_ui(ui, |ui| {
                            for (i, selection) in selections.iter().enumerate() {
                                let response = ui.selectable_value(
                                    &mut selected_mut,
                                    i as u8,
                                    egui::RichText::new(*selection)
                                        .size(WidgetDimensions::TEXT_SIZE)
                                        .color(WidgetColors::TEXT_COLOR),
                                );

                                if response.changed() {
                                    changed_callback(selected_mut);
                                }
                            }
                        });
                });
            },
        );
    }

    pub(crate) fn mouse_down_on(response: &egui::Response) -> bool {
        response.is_pointer_button_down_on() && response.ctx.input(|i| i.pointer.primary_pressed())
    }
}

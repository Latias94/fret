pub(crate) mod autocomplete;
pub(crate) mod badge;
pub(crate) mod bottom_sheet;
pub(crate) mod carousel_item;
pub(crate) mod chip_set;
pub(crate) mod controls;
pub(crate) mod date_picker;
pub(crate) mod divider;
pub(crate) mod exposed_dropdown;
pub(crate) mod fab;
pub(crate) mod icon_button;
pub(crate) mod list;
pub(crate) mod menu_dialog_style;
pub(crate) mod navigation;
pub(crate) mod overlays;
pub(crate) mod progress_indicator;
pub(crate) mod radio;
pub(crate) mod search_bar;
pub(crate) mod search_view;
pub(crate) mod segmented_button;
pub(crate) mod slider;
pub(crate) mod snackbar;
pub(crate) mod tabs;
pub(crate) mod text_field;
pub(crate) mod time_picker;
pub(crate) mod top_app_bar;

use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Material3HeadlessSchemeV1 {
    pub(crate) mode: SchemeMode,
    pub(crate) variant: DynamicVariant,
    pub(crate) label: &'static str,
}

pub(crate) const MATERIAL3_HEADLESS_SCHEMES_V1: [Material3HeadlessSchemeV1; 4] = [
    Material3HeadlessSchemeV1 {
        mode: SchemeMode::Dark,
        variant: DynamicVariant::TonalSpot,
        label: "dark.tonal_spot",
    },
    Material3HeadlessSchemeV1 {
        mode: SchemeMode::Light,
        variant: DynamicVariant::TonalSpot,
        label: "light.tonal_spot",
    },
    Material3HeadlessSchemeV1 {
        mode: SchemeMode::Dark,
        variant: DynamicVariant::Expressive,
        label: "dark.expressive",
    },
    Material3HeadlessSchemeV1 {
        mode: SchemeMode::Light,
        variant: DynamicVariant::Expressive,
        label: "light.expressive",
    },
];

pub(crate) const MATERIAL3_HEADLESS_SCALE_FACTORS_V1: [f32; 3] = [1.0, 1.25, 2.0];

pub(crate) fn scale_segment(scale_factor: f32) -> &'static str {
    if (scale_factor - 1.0).abs() < 1e-6 {
        "scale1_0"
    } else if (scale_factor - 1.25).abs() < 1e-6 {
        "scale1_25"
    } else if (scale_factor - 2.0).abs() < 1e-6 {
        "scale2_0"
    } else {
        panic!("unsupported scale factor: {scale_factor}");
    }
}

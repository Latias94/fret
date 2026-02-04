//! Typed token access for Material 3 date picker primitives.
//!
//! Reference: Material Web v30 `md.comp.date-picker.{docked,modal}.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DatePickerTokenVariant {
    #[default]
    Docked,
    Modal,
}

fn token_key(variant: DatePickerTokenVariant, suffix: &str) -> String {
    match variant {
        DatePickerTokenVariant::Docked => format!("md.comp.date-picker.docked.{suffix}"),
        DatePickerTokenVariant::Modal => format!("md.comp.date-picker.modal.{suffix}"),
    }
}

pub(crate) fn container_width(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    theme
        .metric_by_key(&token_key(variant, "container.width"))
        .unwrap_or(Px(360.0))
}

pub(crate) fn container_height(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    theme
        .metric_by_key(&token_key(variant, "container.height"))
        .unwrap_or(Px(456.0))
}

pub(crate) fn container_elevation(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    theme
        .metric_by_key(&token_key(variant, "container.elevation"))
        .unwrap_or(Px(3.0))
}

pub(crate) fn container_shape(theme: &Theme, variant: DatePickerTokenVariant) -> Corners {
    theme
        .corners_by_key(&token_key(variant, "container.shape"))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.large"))
        .unwrap_or(Corners::all(Px(16.0)))
}

pub(crate) fn container_color(theme: &Theme, variant: DatePickerTokenVariant) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(variant, "container.color"),
        "md.sys.color.surface-container-high",
    )
}

pub(crate) fn weekdays_label_text_style(
    theme: &Theme,
    variant: DatePickerTokenVariant,
) -> TextStyle {
    theme
        .text_style_by_key(&token_key(variant, "weekdays.label-text"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"))
        .unwrap_or_default()
}

pub(crate) fn weekdays_label_text_color(theme: &Theme, variant: DatePickerTokenVariant) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(variant, "weekdays.label-text.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn header_headline_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key("md.comp.date-picker.modal.header.headline")
        .or_else(|| theme.text_style_by_key("md.sys.typescale.headline-large"))
        .unwrap_or_default()
}

pub(crate) fn header_headline_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.date-picker.modal.header.headline.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn date_cell_width(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    theme
        .metric_by_key(&token_key(variant, "date.container.width"))
        .unwrap_or(Px(40.0))
}

pub(crate) fn date_cell_height(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    theme
        .metric_by_key(&token_key(variant, "date.container.height"))
        .unwrap_or(Px(40.0))
}

pub(crate) fn date_cell_shape(theme: &Theme, variant: DatePickerTokenVariant) -> Corners {
    theme
        .corners_by_key(&token_key(variant, "date.container.shape"))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Corners::all(Px(9999.0)))
}

pub(crate) fn date_today_outline_width(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    theme
        .metric_by_key(&token_key(variant, "date.today.container.outline.width"))
        .unwrap_or(Px(1.0))
}

pub(crate) fn date_today_outline_color(theme: &Theme, variant: DatePickerTokenVariant) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(variant, "date.today.container.outline.color"),
        "md.sys.color.primary",
    )
}

pub(crate) fn date_label_text_style(theme: &Theme, variant: DatePickerTokenVariant) -> TextStyle {
    theme
        .text_style_by_key(&token_key(variant, "date.label-text"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"))
        .unwrap_or_default()
}

pub(crate) fn date_unselected_label_text_color(
    theme: &Theme,
    variant: DatePickerTokenVariant,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(variant, "date.unselected.label-text.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn date_selected_container_color(
    theme: &Theme,
    variant: DatePickerTokenVariant,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(variant, "date.selected.container.color"),
        "md.sys.color.primary",
    )
}

pub(crate) fn date_selected_label_text_color(
    theme: &Theme,
    variant: DatePickerTokenVariant,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(variant, "date.selected.label-text.color"),
        "md.sys.color.on-primary",
    )
}

pub(crate) fn date_outside_month_opacity(theme: &Theme, variant: DatePickerTokenVariant) -> f32 {
    theme
        .number_by_key(&token_key(
            variant,
            "date.unselected.outside-month.label-text.opacity",
        ))
        .unwrap_or(0.38)
}

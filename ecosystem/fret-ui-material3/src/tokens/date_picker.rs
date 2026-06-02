//! Typed token access for Material 3 date picker primitives.
//!
//! Reference: Material Web v30 `md.comp.date-picker.{docked,modal}.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::{Theme, theme::CubicBezier};
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::typography;

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

fn date_picker_metric(
    theme: &Theme,
    variant: DatePickerTokenVariant,
    suffix: &str,
    fallback: Px,
) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(&token_key(variant, suffix)), fallback)
}

fn material_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

pub(crate) fn container_width(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    date_picker_metric(theme, variant, "container.width", Px(360.0))
}

pub(crate) fn container_height(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    date_picker_metric(theme, variant, "container.height", Px(456.0))
}

pub(crate) fn container_elevation(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    date_picker_metric(theme, variant, "container.elevation", Px(3.0))
}

pub(crate) fn container_shape(theme: &Theme, variant: DatePickerTokenVariant) -> Corners {
    let key = token_key(variant, "container.shape");
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[key.as_str(), "md.sys.shape.corner.large"],
        Corners::all(Px(16.0)),
    )
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
    typography::text_style(
        theme,
        Some(&token_key(variant, "weekdays.label-text")),
        "md.sys.typescale.body-large",
        TextIntent::Control,
    )
}

pub(crate) fn weekdays_label_text_color(theme: &Theme, variant: DatePickerTokenVariant) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(variant, "weekdays.label-text.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn calendar_horizontal_padding(theme: &Theme, _variant: DatePickerTokenVariant) -> Px {
    material_metric(
        theme,
        "md.sys.fret.material.date-picker.calendar.horizontal-padding",
        Px(12.0),
    )
}

pub(crate) fn header_headline_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some("md.comp.date-picker.modal.header.headline"),
        "md.sys.typescale.headline-large",
        TextIntent::Control,
    )
}

pub(crate) fn header_headline_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.date-picker.modal.header.headline.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn month_nav_title_text_style(theme: &Theme) -> Option<TextStyle> {
    typography::text_style_chain_optional(
        theme,
        &[
            "md.sys.typescale.title-large",
            "md.sys.typescale.title-medium",
        ],
        TextIntent::Control,
    )
}

pub(crate) fn month_nav_title_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface")
}

pub(crate) fn date_cell_width(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    date_picker_metric(theme, variant, "date.container.width", Px(40.0))
}

pub(crate) fn date_cell_height(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    date_picker_metric(theme, variant, "date.container.height", Px(40.0))
}

pub(crate) fn date_cell_shape(theme: &Theme, variant: DatePickerTokenVariant) -> Corners {
    let key = token_key(variant, "date.container.shape");
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[key.as_str(), "md.sys.shape.corner.full"],
        Corners::all(Px(9999.0)),
    )
}

pub(crate) fn date_today_outline_width(theme: &Theme, variant: DatePickerTokenVariant) -> Px {
    date_picker_metric(
        theme,
        variant,
        "date.today.container.outline.width",
        Px(1.0),
    )
}

pub(crate) fn date_today_outline_color(theme: &Theme, variant: DatePickerTokenVariant) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(variant, "date.today.container.outline.color"),
        "md.sys.color.primary",
    )
}

pub(crate) fn date_label_text_style(theme: &Theme, variant: DatePickerTokenVariant) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key(variant, "date.label-text")),
        "md.sys.typescale.body-large",
        TextIntent::Control,
    )
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
    MaterialTokenResolver::new(theme).number_optional(
        Some(&token_key(
            variant,
            "date.unselected.outside-month.label-text.opacity",
        )),
        0.38,
    )
}

pub(crate) fn modal_open_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys("md.sys.motion.duration.medium2", 300)
}

pub(crate) fn modal_close_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys("md.sys.motion.duration.medium2", 300)
}

pub(crate) fn modal_easing(theme: &Theme, easing_key: Option<&str>) -> CubicBezier {
    MaterialTokenResolver::new(theme).easing_optional_or_linear(Some(
        easing_key.unwrap_or("md.sys.motion.easing.emphasized"),
    ))
}

pub(crate) fn modal_scrim_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_sys("md.sys.color.scrim")
}

pub(crate) fn modal_scrim_opacity(theme: &Theme, fallback: f32) -> f32 {
    MaterialTokenResolver::new(theme)
        .number_sys(
            "md.sys.fret.material.date-picker.modal.scrim.opacity",
            fallback,
        )
        .clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn date_picker_metrics_keep_material_defaults() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(
            container_width(theme, DatePickerTokenVariant::Docked),
            Px(360.0)
        );
        assert_eq!(
            container_height(theme, DatePickerTokenVariant::Modal),
            Px(456.0)
        );
        assert_eq!(
            container_elevation(theme, DatePickerTokenVariant::Docked),
            Px(3.0)
        );
        assert_eq!(
            calendar_horizontal_padding(theme, DatePickerTokenVariant::Docked),
            Px(12.0)
        );
        assert_eq!(
            date_cell_width(theme, DatePickerTokenVariant::Modal),
            Px(40.0)
        );
        assert_eq!(
            date_today_outline_width(theme, DatePickerTokenVariant::Docked),
            Px(1.0)
        );
    }

    #[test]
    fn date_picker_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.date-picker.docked.container.width".to_string(),
            380.0,
        );
        patch.metrics.insert(
            "md.comp.date-picker.modal.container.elevation".to_string(),
            6.0,
        );
        patch.metrics.insert(
            "md.comp.date-picker.modal.date.container.height".to_string(),
            44.0,
        );
        patch.metrics.insert(
            "md.sys.fret.material.date-picker.calendar.horizontal-padding".to_string(),
            16.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_width(&theme, DatePickerTokenVariant::Docked),
            Px(380.0)
        );
        assert_eq!(
            container_elevation(&theme, DatePickerTokenVariant::Modal),
            Px(6.0)
        );
        assert_eq!(
            date_cell_height(&theme, DatePickerTokenVariant::Modal),
            Px(44.0)
        );
        assert_eq!(
            calendar_horizontal_padding(&theme, DatePickerTokenVariant::Docked),
            Px(16.0)
        );
    }

    #[test]
    fn date_picker_modal_tokens_prefer_theme_values() {
        let mut patch = ThemeConfig::default();
        patch
            .durations_ms
            .insert("md.sys.motion.duration.medium2".to_string(), 180);
        patch.easings.insert(
            "md.sys.motion.easing.test-date-picker".to_string(),
            CubicBezier {
                x1: 0.1,
                y1: 0.2,
                x2: 0.3,
                y2: 0.4,
            },
        );
        patch.numbers.insert(
            "md.sys.fret.material.date-picker.modal.scrim.opacity".to_string(),
            0.24,
        );
        let scrim = Color::from_srgb_hex_rgb(0x11_22_33);
        patch
            .colors
            .insert("md.sys.color.scrim".to_string(), "#112233".to_string());
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(modal_open_duration_ms(&theme), 180);
        assert_eq!(modal_close_duration_ms(&theme), 180);
        assert_eq!(
            modal_easing(&theme, Some("md.sys.motion.easing.test-date-picker")),
            CubicBezier {
                x1: 0.1,
                y1: 0.2,
                x2: 0.3,
                y2: 0.4,
            }
        );
        assert_eq!(modal_scrim_color(&theme), scrim);
        assert_eq!(modal_scrim_opacity(&theme, 0.32), 0.24);
    }
}

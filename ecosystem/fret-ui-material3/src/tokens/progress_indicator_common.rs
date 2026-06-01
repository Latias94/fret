//! Shared token fallback helpers for Material 3 progress indicators.
//!
//! Material Web v30 ships a merged token set:
//! - shared colors/shapes: `md.comp.progress-indicator.*`
//! - linear metrics: `md.comp.progress-indicator.linear.*`
//! - circular metrics: `md.comp.progress-indicator.circular.*`

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::shape;

const DEFAULT_FULL_SHAPE: Corners = Corners::all(Px(9999.0));
const DEFAULT_LINEAR_HEIGHT: Px = Px(4.0);
const DEFAULT_TRACK_THICKNESS: Px = Px(4.0);
const DEFAULT_ACTIVE_THICKNESS: Px = Px(4.0);
const DEFAULT_CIRCULAR_SIZE: Px = Px(40.0);

pub(crate) fn track_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.progress-indicator.track.color",
        "md.sys.color.secondary-container",
    )
}

pub(crate) fn active_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.progress-indicator.active-indicator.color",
        "md.sys.color.primary",
    )
}

pub(crate) fn four_color_palette(theme: &Theme) -> [Color; 4] {
    let tokens = MaterialTokenResolver::new(theme);
    [
        tokens.color_sys("md.sys.color.primary"),
        tokens.color_sys("md.sys.color.primary-container"),
        tokens.color_sys("md.sys.color.tertiary"),
        tokens.color_sys("md.sys.color.tertiary-container"),
    ]
}

pub(crate) fn track_shape(theme: &Theme) -> Corners {
    shape::corners_or_metric(theme, "md.comp.progress-indicator.track.shape")
        .or_else(|| shape::corners_or_metric(theme, "md.sys.shape.corner.full"))
        .unwrap_or(DEFAULT_FULL_SHAPE)
}

pub(crate) fn active_shape(theme: &Theme) -> Corners {
    shape::corners_or_metric(theme, "md.comp.progress-indicator.active-indicator.shape")
        .or_else(|| shape::corners_or_metric(theme, "md.sys.shape.corner.full"))
        .unwrap_or(DEFAULT_FULL_SHAPE)
}

pub(crate) fn linear_height(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(
        Some("md.comp.progress-indicator.linear.height"),
        DEFAULT_LINEAR_HEIGHT,
    )
}

pub(crate) fn linear_track_thickness(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(
        &[
            "md.comp.progress-indicator.linear.track.thickness",
            "md.comp.progress-indicator.track.thickness",
        ],
        DEFAULT_TRACK_THICKNESS,
    )
}

pub(crate) fn linear_active_thickness(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(
        &[
            "md.comp.progress-indicator.linear.active-indicator.thickness",
            "md.comp.progress-indicator.active-indicator.thickness",
        ],
        DEFAULT_ACTIVE_THICKNESS,
    )
}

pub(crate) fn circular_size(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(
        Some("md.comp.progress-indicator.circular.size"),
        DEFAULT_CIRCULAR_SIZE,
    )
}

pub(crate) fn circular_track_thickness(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(
        &[
            "md.comp.progress-indicator.circular.track.thickness",
            "md.comp.progress-indicator.track.thickness",
        ],
        DEFAULT_TRACK_THICKNESS,
    )
}

pub(crate) fn circular_active_thickness(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(
        &[
            "md.comp.progress-indicator.circular.active-indicator.thickness",
            "md.comp.progress-indicator.active-indicator.thickness",
        ],
        DEFAULT_ACTIVE_THICKNESS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn empty_theme() -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config_patch(&ThemeConfig::default());
        });
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn progress_indicator_metrics_keep_material_defaults() {
        let (_app, theme) = empty_theme();

        assert_eq!(linear_height(&theme), Px(4.0));
        assert_eq!(linear_track_thickness(&theme), Px(4.0));
        assert_eq!(linear_active_thickness(&theme), Px(4.0));
        assert_eq!(circular_size(&theme), Px(40.0));
        assert_eq!(circular_track_thickness(&theme), Px(4.0));
        assert_eq!(circular_active_thickness(&theme), Px(4.0));
    }

    #[test]
    fn progress_indicator_variant_metrics_fall_back_to_shared_metrics() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.progress-indicator.track.thickness".to_string(),
            6.0,
        );
        patch.metrics.insert(
            "md.comp.progress-indicator.active-indicator.thickness".to_string(),
            8.0,
        );
        patch
            .metrics
            .insert("md.comp.progress-indicator.circular.size".to_string(), 44.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(linear_track_thickness(&theme), Px(6.0));
        assert_eq!(circular_track_thickness(&theme), Px(6.0));
        assert_eq!(linear_active_thickness(&theme), Px(8.0));
        assert_eq!(circular_active_thickness(&theme), Px(8.0));
        assert_eq!(circular_size(&theme), Px(44.0));
    }

    #[test]
    fn progress_indicator_shapes_fall_back_to_system_full_shape() {
        let mut patch = ThemeConfig::default();
        patch.corners.insert(
            "md.sys.shape.corner.full".to_string(),
            Corners::all(Px(80.0)),
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(track_shape(&theme), Corners::all(Px(80.0)));
        assert_eq!(active_shape(&theme), Corners::all(Px(80.0)));
    }

    #[test]
    fn progress_indicator_colors_prefer_component_tokens_and_system_palette() {
        let mut patch = ThemeConfig::default();
        patch.colors.insert(
            "md.comp.progress-indicator.active-indicator.color".to_string(),
            "#112233".to_string(),
        );
        patch
            .colors
            .insert("md.sys.color.primary".to_string(), "#334455".to_string());
        patch.colors.insert(
            "md.sys.color.primary-container".to_string(),
            "#445566".to_string(),
        );
        patch
            .colors
            .insert("md.sys.color.tertiary".to_string(), "#556677".to_string());
        patch.colors.insert(
            "md.sys.color.tertiary-container".to_string(),
            "#667788".to_string(),
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            active_color(&theme),
            theme
                .color_by_key("md.comp.progress-indicator.active-indicator.color")
                .expect("patched active indicator color")
        );
        assert_eq!(
            four_color_palette(&theme),
            [
                theme
                    .color_by_key("md.sys.color.primary")
                    .expect("patched primary"),
                theme
                    .color_by_key("md.sys.color.primary-container")
                    .expect("patched primary container"),
                theme
                    .color_by_key("md.sys.color.tertiary")
                    .expect("patched tertiary"),
                theme
                    .color_by_key("md.sys.color.tertiary-container")
                    .expect("patched tertiary container"),
            ]
        );
    }
}

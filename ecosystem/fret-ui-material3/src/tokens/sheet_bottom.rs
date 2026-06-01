//! Typed token access for Material 3 bottom sheets.
//!
//! Reference: Material Web v30 `md.comp.sheet.bottom.*` tokens.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::focus_ring::{
    material_focus_indicator_color, material_focus_indicator_outline_offset,
    material_focus_indicator_thickness,
};
use crate::foundation::token_resolver::MaterialTokenResolver;

const SHEET_BOTTOM_PREFIX: &str = "md.comp.sheet.bottom";

fn sheet_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

pub(crate) fn docked_container_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.sheet.bottom.docked.container.color",
        "md.sys.color.surface-container-low",
    )
}

pub(crate) fn modal_scrim_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_sys("md.sys.color.scrim")
}

pub(crate) fn modal_scrim_opacity(theme: &Theme, fallback: f32) -> f32 {
    MaterialTokenResolver::new(theme)
        .number_optional(
            Some("md.sys.fret.material.sheet.bottom.docked.modal.scrim.opacity"),
            fallback,
        )
        .clamp(0.0, 1.0)
}

pub(crate) fn docked_container_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.sheet.bottom.docked.container.shape")
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-large.top"))
        .unwrap_or(Corners {
            top_left: Px(28.0),
            top_right: Px(28.0),
            bottom_right: Px(0.0),
            bottom_left: Px(0.0),
        })
}

pub(crate) fn docked_modal_elevation(theme: &Theme) -> Px {
    sheet_metric(
        theme,
        "md.comp.sheet.bottom.docked.modal.container.elevation",
        Px(1.0),
    )
}

pub(crate) fn docked_standard_elevation(theme: &Theme) -> Px {
    sheet_metric(
        theme,
        "md.comp.sheet.bottom.docked.standard.container.elevation",
        Px(1.0),
    )
}

pub(crate) fn docked_drag_handle_width(theme: &Theme) -> Px {
    sheet_metric(
        theme,
        "md.comp.sheet.bottom.docked.drag-handle.width",
        Px(32.0),
    )
}

pub(crate) fn docked_drag_handle_height(theme: &Theme) -> Px {
    sheet_metric(
        theme,
        "md.comp.sheet.bottom.docked.drag-handle.height",
        Px(4.0),
    )
}

pub(crate) fn docked_drag_handle_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme)
        .number_optional(Some("md.comp.sheet.bottom.docked.drag-handle.opacity"), 0.4)
}

pub(crate) fn docked_drag_handle_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.sheet.bottom.docked.drag-handle.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn focus_indicator_color(theme: &Theme) -> Color {
    material_focus_indicator_color(theme, SHEET_BOTTOM_PREFIX)
}

pub(crate) fn focus_indicator_thickness(theme: &Theme) -> Px {
    material_focus_indicator_thickness(theme, SHEET_BOTTOM_PREFIX)
}

pub(crate) fn focus_indicator_outline_offset(theme: &Theme) -> Px {
    material_focus_indicator_outline_offset(theme, SHEET_BOTTOM_PREFIX)
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
    fn bottom_sheet_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(docked_modal_elevation(theme), Px(1.0));
        assert_eq!(docked_standard_elevation(theme), Px(1.0));
        assert_eq!(docked_drag_handle_width(theme), Px(32.0));
        assert_eq!(docked_drag_handle_height(theme), Px(4.0));
    }

    #[test]
    fn bottom_sheet_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.sheet.bottom.docked.modal.container.elevation".to_string(),
            3.0,
        );
        patch.metrics.insert(
            "md.comp.sheet.bottom.docked.standard.container.elevation".to_string(),
            2.0,
        );
        patch.metrics.insert(
            "md.comp.sheet.bottom.docked.drag-handle.width".to_string(),
            40.0,
        );
        patch.metrics.insert(
            "md.comp.sheet.bottom.docked.drag-handle.height".to_string(),
            6.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(docked_modal_elevation(&theme), Px(3.0));
        assert_eq!(docked_standard_elevation(&theme), Px(2.0));
        assert_eq!(docked_drag_handle_width(&theme), Px(40.0));
        assert_eq!(docked_drag_handle_height(&theme), Px(6.0));
    }
}

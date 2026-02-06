//! Typed token access for Material 3 bottom sheets.
//!
//! Reference: Material Web v30 `md.comp.sheet.bottom.*` tokens.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) fn docked_container_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.sheet.bottom.docked.container.color",
        "md.sys.color.surface-container-low",
    )
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
    theme
        .metric_by_key("md.comp.sheet.bottom.docked.modal.container.elevation")
        .unwrap_or(Px(1.0))
}

pub(crate) fn docked_standard_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.sheet.bottom.docked.standard.container.elevation")
        .unwrap_or(Px(1.0))
}

pub(crate) fn docked_drag_handle_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.sheet.bottom.docked.drag-handle.width")
        .unwrap_or(Px(32.0))
}

pub(crate) fn docked_drag_handle_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.sheet.bottom.docked.drag-handle.height")
        .unwrap_or(Px(4.0))
}

pub(crate) fn docked_drag_handle_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key("md.comp.sheet.bottom.docked.drag-handle.opacity")
        .unwrap_or(0.4)
}

pub(crate) fn docked_drag_handle_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.sheet.bottom.docked.drag-handle.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn focus_indicator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.sheet.bottom.focus.indicator.color",
        "md.sys.color.secondary",
    )
}

pub(crate) fn focus_indicator_thickness(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.sheet.bottom.focus.indicator.thickness")
        .or_else(|| theme.metric_by_key("md.sys.state.focus-indicator.thickness"))
        .unwrap_or(Px(3.0))
}

pub(crate) fn focus_indicator_outline_offset(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.sheet.bottom.focus.indicator.outline.offset")
        .or_else(|| theme.metric_by_key("md.sys.state.focus-indicator.outer-offset"))
        .unwrap_or(Px(2.0))
}

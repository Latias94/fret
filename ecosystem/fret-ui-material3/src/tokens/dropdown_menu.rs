//! Typed token access for Material 3 dropdown menus.
//!
//! This module centralizes token key mapping and fallback chains so dropdown menu outcomes remain
//! stable and drift-resistant during refactors.

use fret_core::{Edges, Px};
use fret_ui::Theme;
use fret_ui::theme::CubicBezier;

pub(crate) fn open_duration_ms(theme: &Theme) -> u32 {
    theme
        .duration_ms_by_key("md.sys.motion.duration.short4")
        .unwrap_or(200)
}

pub(crate) fn close_duration_ms(theme: &Theme) -> u32 {
    theme
        .duration_ms_by_key("md.sys.motion.duration.short2")
        .unwrap_or(100)
}

pub(crate) fn easing(theme: &Theme) -> CubicBezier {
    theme
        .easing_by_key("md.sys.motion.easing.emphasized")
        .or_else(|| theme.easing_by_key("md.sys.motion.easing.standard"))
        .unwrap_or(CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        })
}

pub(crate) fn divider_margin_total(theme: &Theme) -> Px {
    let _ = theme;
    Px(8.0)
}

pub(crate) fn collision_padding(theme: &Theme) -> Edges {
    let _ = theme;
    Edges::all(Px(8.0))
}

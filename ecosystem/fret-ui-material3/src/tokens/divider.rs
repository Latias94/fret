//! Typed token access for Material 3 dividers.
//!
//! This module centralizes token key mapping and fallback chains so divider visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Px};
use fret_ui::Theme;

pub(crate) fn thickness(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.divider.thickness")
        .unwrap_or(Px(1.0))
}

pub(crate) fn color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.divider.color")
        .or_else(|| theme.color_by_key("md.sys.color.outline-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.outline-variant"))
}

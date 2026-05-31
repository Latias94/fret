//! Typed token access for Material 3 dividers.
//!
//! This module centralizes token key mapping and fallback chains so divider visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) fn thickness(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.divider.thickness")
        .unwrap_or(Px(1.0))
}

pub(crate) fn color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys("md.comp.divider.color", "md.sys.color.outline-variant")
}

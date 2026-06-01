//! Typed token access for Material 3 dropdown menus.
//!
//! This module centralizes token key mapping and fallback chains so dropdown menu outcomes remain
//! stable and drift-resistant during refactors.

use fret_core::{Edges, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) fn close_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys("md.sys.motion.duration.short2", 100)
}

pub(crate) fn divider_margin_total(theme: &Theme) -> Px {
    let _ = theme;
    Px(8.0)
}

pub(crate) fn collision_padding(theme: &Theme) -> Edges {
    let _ = theme;
    Edges::all(Px(8.0))
}

pub(crate) fn max_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.container.max-height")
        .or_else(|| theme.metric_by_key("component.dropdown_menu.max_height"))
        .unwrap_or(Px(320.0))
}

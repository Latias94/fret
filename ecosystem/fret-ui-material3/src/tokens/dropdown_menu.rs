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
    MaterialTokenResolver::new(theme)
        .metric_optional(Some("md.comp.menu.container.max-height"), Px(320.0))
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
    fn dropdown_menu_max_height_defaults_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(max_height(theme), Px(320.0));
    }

    #[test]
    fn dropdown_menu_max_height_prefers_material_token() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.menu.container.max-height".to_string(), 280.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(max_height(&theme), Px(280.0));
    }

    #[test]
    fn dropdown_menu_max_height_ignores_non_material_legacy_token() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("component.dropdown_menu.max_height".to_string(), 240.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(max_height(&theme), Px(320.0));
    }
}

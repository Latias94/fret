//! Typed token access for Material 3 dividers.
//!
//! This module centralizes token key mapping and fallback chains so divider visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) fn thickness(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some("md.comp.divider.thickness"), Px(1.0))
}

pub(crate) fn color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys("md.comp.divider.color", "md.sys.color.outline-variant")
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
    fn divider_thickness_defaults_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(thickness(theme), Px(1.0));
    }

    #[test]
    fn divider_thickness_prefers_material_token() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.divider.thickness".to_string(), 2.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(thickness(&theme), Px(2.0));
    }
}

//! Shared helpers for component shape token access.

use fret_core::Corners;
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) fn uniform_corners_from_metric(theme: &Theme, key: &str) -> Option<Corners> {
    MaterialTokenResolver::new(theme)
        .metric_value(key)
        .map(Corners::all)
}

pub(crate) fn corners_or_metric(theme: &Theme, key: &str) -> Option<Corners> {
    theme
        .corners_by_key(key)
        .or_else(|| uniform_corners_from_metric(theme, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::Px;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn corners_or_metric_prefers_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert("md.comp.test.shape".to_string(), 12.0);
        patch.corners.insert(
            "md.comp.test.shape".to_string(),
            Corners {
                top_left: Px(2.0),
                top_right: Px(4.0),
                bottom_right: Px(6.0),
                bottom_left: Px(8.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            corners_or_metric(&theme, "md.comp.test.shape"),
            Some(Corners {
                top_left: Px(2.0),
                top_right: Px(4.0),
                bottom_right: Px(6.0),
                bottom_left: Px(8.0),
            })
        );
    }

    #[test]
    fn corners_or_metric_maps_uniform_metric_to_all_corners() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert("md.comp.test.shape".to_string(), 14.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            corners_or_metric(&theme, "md.comp.test.shape"),
            Some(Corners::all(Px(14.0)))
        );
    }
}

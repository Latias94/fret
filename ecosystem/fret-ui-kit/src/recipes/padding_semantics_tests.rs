#[cfg(test)]
mod tests {
    use crate::recipes::control::{ControlFallbacks, ControlTokenKeys, resolve_control_chrome};
    use crate::recipes::input::{InputTokenKeys, resolve_input_chrome};
    use crate::recipes::surface::{SurfaceTokenKeys, resolve_surface_chrome};
    use crate::{ChromeRefinement, Size, Space};
    use fret_core::Px;
    use fret_ui::{Theme, ThemeConfig};

    #[test]
    fn surface_padding_pr_behaves_like_px() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([("component.space.2".to_string(), 20.0)]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app);

        let style = ChromeRefinement::default().pr(Space::N2);
        let resolved = resolve_surface_chrome(
            theme,
            &style,
            SurfaceTokenKeys {
                padding_x: None,
                padding_y: None,
                radius: None,
                border_width: None,
                bg: None,
                border: None,
            },
        );
        assert_eq!(resolved.padding_x, Px(20.0));
    }

    #[test]
    fn surface_padding_pb_behaves_like_py() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([("component.space.2".to_string(), 18.0)]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app);

        let style = ChromeRefinement::default().pb(Space::N2);
        let resolved = resolve_surface_chrome(
            theme,
            &style,
            SurfaceTokenKeys {
                padding_x: None,
                padding_y: None,
                radius: None,
                border_width: None,
                bg: None,
                border: None,
            },
        );
        assert_eq!(resolved.padding_y, Px(18.0));
    }

    #[test]
    fn input_padding_per_edge_overrides_axis_defaults() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("component.space.2".to_string(), 20.0),
                    ("component.space.3".to_string(), 24.0),
                    ("component.input.padding_x".to_string(), 6.0),
                    ("component.input.padding_y".to_string(), 4.0),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app);

        let style = ChromeRefinement::default().pb(Space::N3).pr(Space::N3);
        let resolved = resolve_input_chrome(theme, Size::Medium, &style, InputTokenKeys::none());
        assert_eq!(resolved.padding.left, Px(6.0));
        assert_eq!(resolved.padding.right, Px(24.0));
        assert_eq!(resolved.padding.top, Px(4.0));
        assert_eq!(resolved.padding.bottom, Px(24.0));
    }

    #[test]
    fn control_padding_pr_behaves_like_px() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([("component.space.2".to_string(), 16.0)]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app);

        let style = ChromeRefinement::default().pr(Space::N2);
        let resolved = resolve_control_chrome(
            theme,
            &style,
            ControlTokenKeys {
                padding_x: None,
                padding_y: None,
                min_height: None,
                radius: None,
                border_width: None,
                background: None,
                border_color: None,
                text_color: None,
                text_px: None,
            },
            ControlFallbacks {
                padding_x: Px(0.0),
                padding_y: Px(0.0),
                min_height: Px(0.0),
                radius: Px(0.0),
                border_width: Px(0.0),
                background: fret_core::Color::TRANSPARENT,
                border_color: fret_core::Color::TRANSPARENT,
                text_color: fret_core::Color::TRANSPARENT,
                text_px: Px(0.0),
            },
        );
        assert_eq!(resolved.padding_x, Px(16.0));
    }
}

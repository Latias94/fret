#[cfg(test)]
mod tests {
    use super::super::style::container_props;
    use crate::{ChromeRefinement, LayoutRefinement, Space};
    use fret_core::Px;
    use fret_ui::{Theme, ThemeConfig};

    #[test]
    fn declarative_chrome_padding_pt_overrides_only_top() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("component.space.0".to_string(), 0.0),
                    ("component.space.6".to_string(), 32.0),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).clone();

        let chrome = ChromeRefinement::default().p(Space::N6).pt(Space::N0);
        let props = container_props(&theme, chrome, LayoutRefinement::default());

        assert_eq!(props.padding.top, Px(0.0).into());
        assert_eq!(props.padding.bottom, Px(32.0).into());
        assert_eq!(props.padding.left, Px(32.0).into());
        assert_eq!(props.padding.right, Px(32.0).into());
    }

    #[test]
    fn declarative_chrome_padding_pr_overrides_only_right() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("component.space.0".to_string(), 0.0),
                    ("component.space.6".to_string(), 32.0),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).clone();

        let chrome = ChromeRefinement::default().p(Space::N6).pr(Space::N0);
        let props = container_props(&theme, chrome, LayoutRefinement::default());

        assert_eq!(props.padding.top, Px(32.0).into());
        assert_eq!(props.padding.bottom, Px(32.0).into());
        assert_eq!(props.padding.left, Px(32.0).into());
        assert_eq!(props.padding.right, Px(0.0).into());
    }
}

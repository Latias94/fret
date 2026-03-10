use fret_core::Color;
use fret_ui::Theme;

pub(crate) mod keys {
    pub(crate) const FRAME_BG: &str = "workspace.frame.bg";
    pub(crate) const TOP_BAR_BG: &str = "workspace.top_bar.bg";
    pub(crate) const TOP_BAR_BORDER: &str = "workspace.top_bar.border";
    pub(crate) const STATUS_BAR_BG: &str = "workspace.status_bar.bg";
    pub(crate) const STATUS_BAR_BORDER: &str = "workspace.status_bar.border";
    pub(crate) const TABSTRIP_BG: &str = "workspace.tabstrip.bg";
    pub(crate) const TABSTRIP_BG_LEGACY: &str = "workspace.tab_strip.bg";
    pub(crate) const TABSTRIP_BORDER: &str = "workspace.tabstrip.border";
    pub(crate) const TABSTRIP_SCROLL_FG: &str = "workspace.tabstrip.scroll_fg";
    pub(crate) const TABSTRIP_SCROLL_FG_LEGACY: &str = "workspace.tab_strip.scroll_fg";
}

fn color_by_keys(theme: &Theme, keys: &[&str]) -> Option<Color> {
    keys.iter().find_map(|key| theme.color_by_key(key))
}

pub(crate) fn workspace_frame_background(theme: &Theme) -> Option<Color> {
    color_by_keys(theme, &[keys::FRAME_BG, "background"])
}

pub(crate) fn workspace_top_bar_background(theme: &Theme) -> Option<Color> {
    color_by_keys(theme, &[keys::TOP_BAR_BG, "muted", "background"])
}

pub(crate) fn workspace_top_bar_border(theme: &Theme) -> Option<Color> {
    color_by_keys(theme, &[keys::TOP_BAR_BORDER, "border"])
}

pub(crate) fn workspace_status_bar_background(theme: &Theme) -> Option<Color> {
    color_by_keys(theme, &[keys::STATUS_BAR_BG, "muted", "background"])
}

pub(crate) fn workspace_status_bar_border(theme: &Theme) -> Option<Color> {
    color_by_keys(theme, &[keys::STATUS_BAR_BORDER, "border"])
}

pub(crate) fn workspace_tabstrip_background(theme: &Theme) -> Option<Color> {
    color_by_keys(
        theme,
        &[
            keys::TABSTRIP_BG,
            keys::TABSTRIP_BG_LEGACY,
            "muted",
            "background",
        ],
    )
}

pub(crate) fn workspace_tabstrip_border(theme: &Theme) -> Option<Color> {
    color_by_keys(theme, &[keys::TABSTRIP_BORDER, "border"])
}

pub(crate) fn workspace_tabstrip_scroll_foreground(theme: &Theme) -> Option<Color> {
    color_by_keys(
        theme,
        &[
            keys::TABSTRIP_SCROLL_FG,
            keys::TABSTRIP_SCROLL_FG_LEGACY,
            "muted-foreground",
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use fret_app::App;
    use fret_ui::{Theme, ThemeConfig};

    fn test_app(colors: &[(&str, &str)]) -> App {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Workspace theme token tests".to_string(),
                colors: colors
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect::<HashMap<_, _>>(),
                ..ThemeConfig::default()
            });
        });
        app
    }

    #[test]
    fn workspace_shell_namespace_tokens_fall_back_to_generic_semantics() {
        let app = test_app(&[
            ("background", "#101214"),
            ("muted", "#202224"),
            ("border", "#303234"),
            ("muted-foreground", "#404244"),
        ]);
        let theme = Theme::global(&app);

        assert_eq!(
            workspace_frame_background(theme),
            theme.color_by_key("background")
        );
        assert_eq!(
            workspace_top_bar_background(theme),
            theme.color_by_key("muted")
        );
        assert_eq!(
            workspace_top_bar_border(theme),
            theme.color_by_key("border")
        );
        assert_eq!(
            workspace_status_bar_background(theme),
            theme.color_by_key("muted")
        );
        assert_eq!(
            workspace_status_bar_border(theme),
            theme.color_by_key("border")
        );
        assert_eq!(
            workspace_tabstrip_background(theme),
            theme.color_by_key("muted")
        );
        assert_eq!(
            workspace_tabstrip_border(theme),
            theme.color_by_key("border")
        );
        assert_eq!(
            workspace_tabstrip_scroll_foreground(theme),
            theme.color_by_key("muted-foreground")
        );
    }

    #[test]
    fn workspace_shell_namespace_tokens_override_generic_semantics() {
        let app = test_app(&[
            ("workspace.frame.bg", "#111111"),
            ("workspace.top_bar.bg", "#222222"),
            ("workspace.top_bar.border", "#333333"),
            ("workspace.status_bar.bg", "#444444"),
            ("workspace.status_bar.border", "#555555"),
            ("background", "#666666"),
            ("muted", "#777777"),
            ("border", "#888888"),
        ]);
        let theme = Theme::global(&app);

        assert_eq!(
            workspace_frame_background(theme),
            theme.color_by_key("workspace.frame.bg")
        );
        assert_eq!(
            workspace_top_bar_background(theme),
            theme.color_by_key("workspace.top_bar.bg")
        );
        assert_eq!(
            workspace_top_bar_border(theme),
            theme.color_by_key("workspace.top_bar.border")
        );
        assert_eq!(
            workspace_status_bar_background(theme),
            theme.color_by_key("workspace.status_bar.bg")
        );
        assert_eq!(
            workspace_status_bar_border(theme),
            theme.color_by_key("workspace.status_bar.border")
        );
    }

    #[test]
    fn workspace_tabstrip_prefers_canonical_keys_and_keeps_legacy_compatibility() {
        let app = test_app(&[
            ("workspace.tabstrip.bg", "#111111"),
            ("workspace.tab_strip.bg", "#222222"),
            ("workspace.tabstrip.scroll_fg", "#333333"),
            ("workspace.tab_strip.scroll_fg", "#444444"),
            ("workspace.tabstrip.border", "#555555"),
            ("muted", "#666666"),
            ("muted-foreground", "#777777"),
            ("border", "#888888"),
        ]);
        let theme = Theme::global(&app);

        assert_eq!(
            workspace_tabstrip_background(theme),
            theme.color_by_key("workspace.tabstrip.bg")
        );
        assert_eq!(
            workspace_tabstrip_scroll_foreground(theme),
            theme.color_by_key("workspace.tabstrip.scroll_fg")
        );
        assert_eq!(
            workspace_tabstrip_border(theme),
            theme.color_by_key("workspace.tabstrip.border")
        );

        let app = test_app(&[
            ("workspace.tab_strip.bg", "#222222"),
            ("workspace.tab_strip.scroll_fg", "#444444"),
            ("muted", "#666666"),
            ("muted-foreground", "#777777"),
        ]);
        let theme = Theme::global(&app);

        assert_eq!(
            workspace_tabstrip_background(theme),
            theme.color_by_key("workspace.tab_strip.bg")
        );
        assert_eq!(
            workspace_tabstrip_scroll_foreground(theme),
            theme.color_by_key("workspace.tab_strip.scroll_fg")
        );
    }
}

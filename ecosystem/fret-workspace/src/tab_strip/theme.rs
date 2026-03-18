use fret_core::{Color, Px, TextStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::theme_tokens::{
    workspace_tabstrip_background, workspace_tabstrip_border, workspace_tabstrip_scroll_foreground,
};

use super::utils::tab_text_style;

#[derive(Debug, Clone)]
pub(super) struct WorkspaceTabStripTheme {
    pub(super) bar_bg: Option<Color>,
    pub(super) bar_border: Option<Color>,
    pub(super) active_bg: Option<Color>,
    pub(super) active_fg: Color,
    pub(super) inactive_fg: Color,
    pub(super) dirty_fg: Color,
    pub(super) hover_bg: Color,
    pub(super) indicator_color: Option<Color>,
    pub(super) text_style: TextStyle,
    pub(super) tab_radius: Px,
    pub(super) scroll_button_fg: Color,
    pub(super) tab_max_width: Px,
}

impl WorkspaceTabStripTheme {
    fn from_theme(theme: &Theme) -> Self {
        let bar_bg = workspace_tabstrip_background(theme);
        let bar_border = workspace_tabstrip_border(theme);

        let active_bg = theme
            .color_by_key("workspace.tab.active_bg")
            .or_else(|| theme.color_by_key("background"));
        let active_fg = theme.color_token("foreground");
        let inactive_fg = theme.color_by_key("muted-foreground").unwrap_or(active_fg);
        let dirty_fg = theme
            .color_by_key("workspace.tab.dirty_fg")
            .or_else(|| theme.color_by_key("ring"))
            .or_else(|| theme.color_by_key("primary"))
            .unwrap_or(active_fg);
        let hover_bg = theme
            .color_by_key("workspace.tab.hover_bg")
            .or_else(|| theme.color_by_key("accent"))
            .unwrap_or(Color::TRANSPARENT);

        let indicator_color = theme
            .color_by_key("workspace.tab.drop_indicator")
            .or_else(|| theme.color_by_key("ring"))
            .or_else(|| theme.color_by_key("accent"));

        let text_style = tab_text_style(theme);
        let tab_radius = theme.metric_by_key("radius").unwrap_or(Px(6.0));
        let scroll_button_fg = workspace_tabstrip_scroll_foreground(theme).unwrap_or(active_fg);
        let tab_max_width = theme
            .metric_by_key("workspace.tab.max_width")
            .unwrap_or(Px(240.0));

        Self {
            bar_bg,
            bar_border,
            active_bg,
            active_fg,
            inactive_fg,
            dirty_fg,
            hover_bg,
            indicator_color,
            text_style,
            tab_radius,
            scroll_button_fg,
            tab_max_width,
        }
    }

    pub(super) fn resolve<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {
        Self::from_theme(Theme::global(cx.app))
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceTabStripTheme;
    use fret_app::App;
    use fret_core::Color;
    use fret_ui::{Theme, ThemeConfig};

    #[test]
    fn workspace_tab_hover_background_prefers_owner_token_over_accent() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors
                .insert("accent".to_string(), "#335577".to_string());
            cfg.colors
                .insert("workspace.tab.hover_bg".to_string(), "#112233".to_string());
            theme.apply_config_patch(&cfg);
        });

        let resolved = WorkspaceTabStripTheme::from_theme(Theme::global(&app));
        assert_eq!(resolved.hover_bg, Color::from_srgb_hex_rgb(0x11_22_33));
    }
}

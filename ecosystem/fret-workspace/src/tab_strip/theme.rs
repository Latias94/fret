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
    pub(super) fn resolve<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {
        let theme = Theme::global(cx.app);

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
            .color_by_key("accent")
            .or_else(|| theme.color_by_key("workspace.tab.hover_bg"))
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
}

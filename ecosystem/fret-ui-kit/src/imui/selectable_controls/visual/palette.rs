use fret_core::Color;
use fret_ui::Theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::imui::selectable_controls) struct SelectablePalette {
    pub(in crate::imui::selectable_controls) bg: Option<Color>,
    pub(in crate::imui::selectable_controls) fg: Color,
}

pub(in crate::imui::selectable_controls) fn resolve_selectable_palette(
    theme: &Theme,
    enabled: bool,
    selected: bool,
    hovered: bool,
    pressed: bool,
) -> SelectablePalette {
    let hovered_or_pressed = enabled && (hovered || pressed);
    let selected_bg = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("list.row.selected"))
        .or_else(|| theme.color_by_key("selection.background"))
        .unwrap_or_else(|| theme.color_token("selection.background"));
    let hover_bg = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    let fg = if !enabled {
        theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"))
    } else if !selected && hovered_or_pressed {
        theme
            .color_by_key("accent-foreground")
            .unwrap_or_else(|| theme.color_token("accent-foreground"))
    } else {
        theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"))
    };

    let bg = if selected {
        Some(selected_bg)
    } else if hovered_or_pressed {
        Some(hover_bg)
    } else {
        None
    };

    SelectablePalette { bg, fg }
}

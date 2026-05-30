use fret_core::Color;

pub(super) struct ImuiTextInputPalette {
    pub(super) background: Color,
    pub(super) foreground: Color,
    pub(super) muted_foreground: Color,
    pub(super) border_idle: Color,
    pub(super) ring: Color,
    pub(super) primary: Color,
    pub(super) selection_color: Color,
    pub(super) preedit_bg_color: Color,
}

pub(super) fn imui_text_input_palette(theme: &fret_ui::Theme) -> ImuiTextInputPalette {
    let background = theme
        .color_by_key("card")
        .or_else(|| theme.color_by_key("muted"))
        .or_else(|| theme.color_by_key("background"))
        .unwrap_or_else(|| theme.color_token("background"));
    let foreground = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"));
    let muted_foreground = theme
        .color_by_key("muted-foreground")
        .unwrap_or_else(|| theme.color_token("muted-foreground"));
    let border_idle = theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("input"));
    let ring = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_token("ring"));
    let primary = theme
        .color_by_key("primary")
        .unwrap_or_else(|| theme.color_token("primary"));
    let selection = theme
        .color_by_key("component.input.selection")
        .unwrap_or_else(|| theme.color_token("selection.background"));
    let selection_color = Color {
        a: 1.0,
        ..selection
    };
    let mut preedit_bg_color = selection_color;
    preedit_bg_color.a = (preedit_bg_color.a * 0.35).clamp(0.0, 1.0);

    ImuiTextInputPalette {
        background,
        foreground,
        muted_foreground,
        border_idle,
        ring,
        primary,
        selection_color,
        preedit_bg_color,
    }
}

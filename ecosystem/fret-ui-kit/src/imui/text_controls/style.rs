use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

pub(super) fn imui_text_input_style_from_theme(theme: &fret_ui::Theme) -> fret_ui::TextInputStyle {
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

    fret_ui::TextInputStyle {
        padding: Edges {
            left: Px(8.0),
            right: Px(8.0),
            top: Px(3.0),
            bottom: Px(3.0),
        },
        background,
        border: Edges::all(Px(1.0)),
        border_color: border_idle,
        border_color_focused: ring,
        focus_ring: None,
        corner_radii: Corners::all(super::super::control_chrome::CONTROL_RADIUS),
        text_color: foreground,
        placeholder_color: muted_foreground,
        selection_color,
        caret_color: foreground,
        preedit_bg_color,
        preedit_color: primary,
        preedit_underline_color: primary,
    }
}

pub(super) fn imui_text_area_style_from_theme(theme: &fret_ui::Theme) -> fret_ui::TextAreaStyle {
    let input_style = imui_text_input_style_from_theme(theme);

    fret_ui::TextAreaStyle {
        padding_x: input_style.padding.left,
        padding_y: input_style.padding.top,
        background: input_style.background,
        border: input_style.border,
        border_color: input_style.border_color,
        border_color_focused: input_style.border_color_focused,
        focus_ring: None,
        corner_radii: input_style.corner_radii,
        text_color: input_style.text_color,
        placeholder_color: input_style.placeholder_color,
        selection_color: input_style.selection_color,
        caret_color: input_style.caret_color,
        preedit_bg_color: input_style.preedit_bg_color,
        preedit_underline_color: input_style.preedit_underline_color,
    }
}

pub(super) fn default_input_text_style_from_theme(theme: &fret_ui::Theme) -> fret_core::TextStyle {
    crate::typography::control_text_style_for_font_size(
        theme,
        fret_core::FontId::ui(),
        theme
            .metric_by_key("font.size")
            .unwrap_or_else(|| theme.metric_token("font.size")),
    )
}

pub(super) fn input_text_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT),
            min_height: Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT)),
            max_height: Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT)),
            ..Default::default()
        },
        ..Default::default()
    }
}

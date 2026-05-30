use fret_ui::element::LayoutStyle;

mod chrome;
mod palette;

pub(super) fn imui_text_input_style_from_theme(theme: &fret_ui::Theme) -> fret_ui::TextInputStyle {
    let palette = palette::imui_text_input_palette(theme);

    fret_ui::TextInputStyle {
        padding: chrome::input_padding(),
        background: palette.background,
        border: chrome::input_border(),
        border_color: palette.border_idle,
        border_color_focused: palette.ring,
        focus_ring: None,
        corner_radii: chrome::input_corner_radii(),
        text_color: palette.foreground,
        placeholder_color: palette.muted_foreground,
        selection_color: palette.selection_color,
        caret_color: palette.foreground,
        preedit_bg_color: palette.preedit_bg_color,
        preedit_color: palette.primary,
        preedit_underline_color: palette.primary,
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
    chrome::input_text_layout()
}

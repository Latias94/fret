use fret_core::Color;

pub(super) struct ThemePresetRowVisual {
    pub(super) background: Color,
    pub(super) text_color: Color,
    pub(super) border_color: Color,
    pub(super) status_color: Color,
}

pub(super) struct ThemePresetRowVisualInput {
    pub(super) selected: bool,
    pub(super) enabled: bool,
    pub(super) hovered: bool,
    pub(super) hovered_raw: bool,
    pub(super) pressed: bool,
    pub(super) fg: Color,
    pub(super) muted_fg: Color,
    pub(super) subtle_bg: Color,
    pub(super) accent: Color,
    pub(super) border: Color,
}

pub(super) fn theme_preset_row_visual(input: ThemePresetRowVisualInput) -> ThemePresetRowVisual {
    let active_bg = mix_color(input.subtle_bg, input.accent, 0.42);
    let hover_bg = mix_color(input.subtle_bg, input.accent, 0.18);
    let pressed_bg = mix_color(input.subtle_bg, input.accent, 0.32);
    let background = if input.selected {
        active_bg
    } else if input.pressed {
        pressed_bg
    } else if input.hovered || input.hovered_raw {
        hover_bg
    } else {
        input.subtle_bg
    };
    let text_color = if input.enabled {
        input.fg
    } else {
        mix_color(input.muted_fg, input.subtle_bg, 0.35)
    };
    let border_color = if input.selected {
        input.accent
    } else {
        input.border
    };
    let status_color = if input.selected {
        input.accent
    } else {
        input.muted_fg
    };

    ThemePresetRowVisual {
        background,
        text_color,
        border_color,
        status_color,
    }
}

fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

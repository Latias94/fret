use fret_core::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ThemePresetRowVisual {
    pub(super) background: Color,
    pub(super) text_color: Color,
    pub(super) border_color: Color,
    pub(super) status_color: Color,
}

#[derive(Debug, Clone, Copy)]
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

#[cfg(test)]
mod tests {
    use fret_core::Color;

    use super::{ThemePresetRowVisualInput, mix_color, theme_preset_row_visual};

    const EPSILON: f32 = 0.000_001;

    fn palette_input() -> ThemePresetRowVisualInput {
        ThemePresetRowVisualInput {
            selected: false,
            enabled: true,
            hovered: false,
            hovered_raw: false,
            pressed: false,
            fg: color(0.80, 0.82, 0.84, 1.0),
            muted_fg: color(0.45, 0.47, 0.49, 1.0),
            subtle_bg: color(0.10, 0.12, 0.14, 1.0),
            accent: color(0.70, 0.45, 0.20, 1.0),
            border: color(0.18, 0.20, 0.22, 1.0),
        }
    }

    fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    fn assert_color_close(actual: Color, expected: Color) {
        assert!(
            (actual.r - expected.r).abs() <= EPSILON,
            "red channel differs: {actual:?} != {expected:?}"
        );
        assert!(
            (actual.g - expected.g).abs() <= EPSILON,
            "green channel differs: {actual:?} != {expected:?}"
        );
        assert!(
            (actual.b - expected.b).abs() <= EPSILON,
            "blue channel differs: {actual:?} != {expected:?}"
        );
        assert!(
            (actual.a - expected.a).abs() <= EPSILON,
            "alpha channel differs: {actual:?} != {expected:?}"
        );
    }

    #[test]
    fn theme_preset_row_visual_uses_selected_state_as_top_priority() {
        let input = ThemePresetRowVisualInput {
            selected: true,
            hovered: true,
            hovered_raw: true,
            pressed: true,
            ..palette_input()
        };

        let visual = theme_preset_row_visual(input);

        assert_color_close(
            visual.background,
            mix_color(input.subtle_bg, input.accent, 0.42),
        );
        assert_eq!(visual.border_color, input.accent);
        assert_eq!(visual.status_color, input.accent);
    }

    #[test]
    fn theme_preset_row_visual_projects_pressed_before_hover() {
        let input = ThemePresetRowVisualInput {
            pressed: true,
            hovered: true,
            hovered_raw: true,
            ..palette_input()
        };

        let visual = theme_preset_row_visual(input);

        assert_color_close(
            visual.background,
            mix_color(input.subtle_bg, input.accent, 0.32),
        );
        assert_eq!(visual.border_color, input.border);
        assert_eq!(visual.status_color, input.muted_fg);
    }

    #[test]
    fn theme_preset_row_visual_projects_hover_and_raw_hover_equally() {
        let hovered = ThemePresetRowVisualInput {
            hovered: true,
            ..palette_input()
        };
        let raw_hovered = ThemePresetRowVisualInput {
            hovered_raw: true,
            ..palette_input()
        };

        let expected = mix_color(hovered.subtle_bg, hovered.accent, 0.18);

        assert_color_close(theme_preset_row_visual(hovered).background, expected);
        assert_color_close(theme_preset_row_visual(raw_hovered).background, expected);
    }

    #[test]
    fn theme_preset_row_visual_dims_disabled_text_without_changing_status() {
        let input = ThemePresetRowVisualInput {
            enabled: false,
            ..palette_input()
        };

        let visual = theme_preset_row_visual(input);

        assert_color_close(
            visual.text_color,
            mix_color(input.muted_fg, input.subtle_bg, 0.35),
        );
        assert_eq!(visual.status_color, input.muted_fg);
    }
}

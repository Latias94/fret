use fret_core::Color;
use fret_core::window::ColorScheme;
use fret_ui::ThemeSnapshot;
use fret_ui_kit::ColorRef;

pub(crate) fn invalid_control_ring_color(theme: &ThemeSnapshot, default_color: Color) -> Color {
    theme
        .color_by_key("component.control.invalid_ring")
        .or_else(|| {
            let ring_key = if theme.color_scheme == Some(ColorScheme::Dark) {
                "destructive/40"
            } else {
                "destructive/20"
            };
            theme
                .color_by_key(ring_key)
                .or_else(|| theme.color_by_key("destructive/20"))
        })
        .unwrap_or(default_color)
}

pub(crate) fn tabs_trigger_inactive_fg(theme: &ThemeSnapshot) -> ColorRef {
    theme
        .color_by_key("component.tabs.trigger.fg_inactive")
        .map(ColorRef::Color)
        .unwrap_or_else(|| {
            if theme.color_scheme == Some(ColorScheme::Dark) {
                ColorRef::Color(theme.color_token("muted-foreground"))
            } else {
                ColorRef::Color(theme.color_token("foreground"))
            }
        })
}

pub(crate) fn radio_group_choice_card_checked_bg(theme: &ThemeSnapshot, primary: Color) -> Color {
    theme
        .color_by_key("component.radio_group.choice_card.checked_bg")
        .unwrap_or_else(|| {
            let bg_alpha = if theme.color_scheme == Some(ColorScheme::Dark) {
                0.10
            } else {
                0.05
            };
            alpha_mul(primary, bg_alpha)
        })
}

pub(crate) fn menu_destructive_focus_bg(theme: &ThemeSnapshot, destructive_fg: Color) -> Color {
    theme
        .color_by_key("component.menu.destructive_focus_bg")
        .unwrap_or_else(|| {
            // Fallback for non-shadcn themes: approximate the upstream `/10` and `dark:/20` alphas.
            let alpha = if theme.color_scheme == Some(ColorScheme::Dark) {
                0.2
            } else {
                0.1
            };
            alpha_mul(destructive_fg, alpha)
        })
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

use fret_core::Color;
use fret_ui::Theme;

use crate::primitives::EditorTokenKeys;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ResolvedCheckboxChrome {
    pub(super) bg_unchecked: Color,
    pub(super) bg_checked: Color,
    pub(super) fg_checked: Color,
    pub(super) ring_color: Color,
}

pub(super) fn resolve_checkbox_chrome(theme: &Theme, fallback_bg: Color) -> ResolvedCheckboxChrome {
    let bg_unchecked = theme
        .color_by_key(EditorTokenKeys::CHECKBOX_BG)
        .or_else(|| theme.color_by_key("component.checkbox.bg"))
        .or_else(|| theme.color_by_key("component.input.bg"))
        .unwrap_or(fallback_bg);
    let bg_checked = theme
        .color_by_key(EditorTokenKeys::CHECKBOX_CHECKED_BG)
        .unwrap_or_else(|| theme.color_token("primary"));
    let fg_checked = theme
        .color_by_key(EditorTokenKeys::CHECKBOX_CHECKED_FG)
        .unwrap_or_else(|| theme.color_token("primary-foreground"));
    let ring_color = theme
        .color_by_key(EditorTokenKeys::CHECKBOX_RING)
        .or_else(|| theme.color_by_key("ring"))
        .unwrap_or_else(|| theme.color_token("primary"));

    ResolvedCheckboxChrome {
        bg_unchecked,
        bg_checked,
        fg_checked,
        ring_color,
    }
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::Color;
    use fret_ui::{Theme, ThemeConfig};

    use super::resolve_checkbox_chrome;
    use crate::primitives::EditorTokenKeys;

    #[test]
    fn checkbox_chrome_prefers_editor_owned_tokens_over_generic_palette() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors.insert(
                EditorTokenKeys::CHECKBOX_BG.to_string(),
                "#141b24".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::CHECKBOX_CHECKED_BG.to_string(),
                "#355a86".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::CHECKBOX_CHECKED_FG.to_string(),
                "#edf3fa".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::CHECKBOX_RING.to_string(),
                "#7faee8".to_string(),
            );
            cfg.colors
                .insert("component.checkbox.bg".to_string(), "#ff0000".to_string());
            cfg.colors
                .insert("component.input.bg".to_string(), "#00ff00".to_string());
            cfg.colors
                .insert("primary".to_string(), "#123456".to_string());
            cfg.colors
                .insert("primary-foreground".to_string(), "#654321".to_string());
            cfg.colors.insert("ring".to_string(), "#888888".to_string());
            theme.apply_config_patch(&cfg);
        });

        let theme = Theme::global(&app);
        let chrome = resolve_checkbox_chrome(theme, Color::from_srgb_hex_rgb(0x0c_11_18));

        assert_eq!(chrome.bg_unchecked, Color::from_srgb_hex_rgb(0x14_1b_24));
        assert_eq!(chrome.bg_checked, Color::from_srgb_hex_rgb(0x35_5a_86));
        assert_eq!(chrome.fg_checked, Color::from_srgb_hex_rgb(0xed_f3_fa));
        assert_eq!(chrome.ring_color, Color::from_srgb_hex_rgb(0x7f_ae_e8));
    }
}

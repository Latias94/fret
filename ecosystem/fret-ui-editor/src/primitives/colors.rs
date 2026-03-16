//! Shared editor semantic color helpers.
//!
//! These helpers keep editor-owned surfaces on the editor token lane first while preserving
//! generic app-theme fallback for compatibility.

use fret_core::Color;
use fret_ui::Theme;

use super::EditorTokenKeys;

pub(crate) fn editor_foreground(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::TEXT_FIELD_FG)
        .or_else(|| theme.color_by_key("foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

pub(crate) fn editor_muted_foreground(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::CHROME_MUTED_FG)
        .or_else(|| theme.color_by_key("muted-foreground"))
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| editor_foreground(theme))
}

pub(crate) fn editor_accent(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::CHROME_ACCENT)
        .unwrap_or_else(|| theme.color_token("accent"))
}

pub(crate) fn editor_focus_ring(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::CHROME_RING)
        .or_else(|| theme.color_by_key(EditorTokenKeys::TEXT_FIELD_BORDER_FOCUS))
        .or_else(|| theme.color_by_key("ring"))
        .unwrap_or_else(|| theme.color_token("primary"))
}

pub(crate) fn editor_border(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::TEXT_FIELD_BORDER)
        .or_else(|| theme.color_by_key("component.text_field.border"))
        .or_else(|| theme.color_by_key("component.input.border"))
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"))
}

pub(crate) fn editor_subtle_bg(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::TEXT_FIELD_BG)
        .or_else(|| theme.color_by_key("component.text_field.bg"))
        .or_else(|| theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BG))
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or_else(|| theme.color_token("muted"))
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::Color;
    use fret_ui::{Theme, ThemeConfig};

    use super::{
        editor_accent, editor_border, editor_focus_ring, editor_foreground,
        editor_muted_foreground, editor_subtle_bg,
    };
    use crate::primitives::EditorTokenKeys;

    #[test]
    fn editor_semantic_color_helpers_prefer_editor_owned_keys() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors.insert(
                EditorTokenKeys::TEXT_FIELD_BG.to_string(),
                "#141b24".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::TEXT_FIELD_FG.to_string(),
                "#f0f4f8".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::CHROME_MUTED_FG.to_string(),
                "#8aa1b7".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::CHROME_ACCENT.to_string(),
                "#355a86".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::CHROME_RING.to_string(),
                "#7faee8".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::TEXT_FIELD_BORDER.to_string(),
                "#3b4758".to_string(),
            );
            cfg.colors
                .insert("foreground".to_string(), "#ffffff".to_string());
            cfg.colors
                .insert("border".to_string(), "#ff8800".to_string());
            cfg.colors
                .insert("muted-foreground".to_string(), "#00ff00".to_string());
            cfg.colors
                .insert("accent".to_string(), "#ff00ff".to_string());
            cfg.colors.insert("ring".to_string(), "#ff8800".to_string());
            theme.apply_config_patch(&cfg);
        });

        let theme = Theme::global(&app);
        assert_eq!(
            editor_foreground(theme),
            Color::from_srgb_hex_rgb(0xf0_f4_f8)
        );
        assert_eq!(
            editor_muted_foreground(theme),
            Color::from_srgb_hex_rgb(0x8a_a1_b7)
        );
        assert_eq!(editor_accent(theme), Color::from_srgb_hex_rgb(0x35_5a_86));
        assert_eq!(
            editor_focus_ring(theme),
            Color::from_srgb_hex_rgb(0x7f_ae_e8)
        );
        assert_eq!(editor_border(theme), Color::from_srgb_hex_rgb(0x3b_47_58));
        assert_eq!(
            editor_subtle_bg(theme),
            Color::from_srgb_hex_rgb(0x14_1b_24)
        );
    }

    #[test]
    fn editor_focus_ring_color_falls_back_to_text_field_focus_border() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors.insert(
                EditorTokenKeys::TEXT_FIELD_BORDER_FOCUS.to_string(),
                "#6ea8e0".to_string(),
            );
            cfg.colors.insert("ring".to_string(), "#ff8800".to_string());
            theme.apply_config_patch(&cfg);
        });

        let theme = Theme::global(&app);
        assert_eq!(
            editor_focus_ring(theme),
            Color::from_srgb_hex_rgb(0x6e_a8_e0)
        );
    }

    #[test]
    fn editor_muted_foreground_keeps_shared_palette_fallback() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors
                .insert("muted-foreground".to_string(), "#9eabbc".to_string());
            theme.apply_config_patch(&cfg);
        });

        let theme = Theme::global(&app);
        assert_eq!(
            editor_muted_foreground(theme),
            Color::from_srgb_hex_rgb(0x9e_ab_bc)
        );
    }
}

//! Shared helpers for component typography token access.

use fret_core::{FontWeight, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::{self, TextIntent};

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) fn text_style(
    theme: &Theme,
    primary_key: Option<&str>,
    source_key: &str,
    intent: TextIntent,
) -> TextStyle {
    let tokens = MaterialTokenResolver::new(theme);
    let style = primary_key
        .and_then(|key| tokens.text_style_value(key))
        .or_else(|| tokens.text_style_value(source_key))
        .unwrap_or_default();

    typography::with_intent(style, intent)
}

pub(crate) fn text_style_chain(theme: &Theme, keys: &[&str], intent: TextIntent) -> TextStyle {
    text_style_chain_optional(theme, keys, intent).unwrap_or_default()
}

pub(crate) fn text_style_chain_optional(
    theme: &Theme,
    keys: &[&str],
    intent: TextIntent,
) -> Option<TextStyle> {
    MaterialTokenResolver::new(theme)
        .text_style_chain(keys)
        .map(|style| typography::with_intent(style, intent))
}

pub(crate) fn text_style_value(theme: &Theme, key: &str, intent: TextIntent) -> Option<TextStyle> {
    MaterialTokenResolver::new(theme)
        .text_style_value(key)
        .map(|style| typography::with_intent(style, intent))
}

pub(crate) fn text_style_with_weight(
    theme: &Theme,
    primary_key: Option<&str>,
    source_key: &str,
    weight_key: Option<&str>,
    intent: TextIntent,
) -> TextStyle {
    let mut style = text_style(theme, primary_key, source_key, intent);

    if let Some(weight) =
        weight_key.and_then(|key| MaterialTokenResolver::new(theme).number_value(key))
    {
        style.weight = FontWeight(weight.round().clamp(1.0, 1000.0) as u16);
    }

    style
}

pub(crate) fn text_style_with_weight_fallback(
    theme: &Theme,
    primary_key: Option<&str>,
    source_key: &str,
    weight_key: &str,
    fallback_weight: f32,
    intent: TextIntent,
) -> TextStyle {
    let mut style = text_style(theme, primary_key, source_key, intent);
    let weight = MaterialTokenResolver::new(theme)
        .number_value(weight_key)
        .unwrap_or(fallback_weight);
    style.weight = FontWeight(weight.round().clamp(1.0, 1000.0) as u16);
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn text_style_with_weight_uses_material_number_token() {
        let mut patch = ThemeConfig::default();
        patch
            .numbers
            .insert("md.comp.test.label-text.weight".to_string(), 651.0);
        let (_app, theme) = theme_with_patch(patch);

        let style = text_style_with_weight(
            &theme,
            None,
            "md.sys.typescale.label-large",
            Some("md.comp.test.label-text.weight"),
            TextIntent::Control,
        );

        assert_eq!(style.weight, FontWeight(651));
    }

    #[test]
    fn text_style_with_weight_fallback_uses_fallback_when_token_is_missing() {
        let app = App::new();
        let theme = Theme::global(&app);

        let style = text_style_with_weight_fallback(
            theme,
            None,
            "md.sys.typescale.label-large",
            "md.comp.test.missing.label-text.weight",
            500.0,
            TextIntent::Control,
        );

        assert_eq!(style.weight, FontWeight(500));
    }
}

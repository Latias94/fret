//! Typed token access for Material 3 text fields.
//!
//! This module centralizes token key mapping and fallback chains so text field visuals remain
//! stable and drift-resistant during refactors.

use fret_core::{Color, Px};
use fret_ui::{TextInputStyle, Theme};

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::text_field::TextFieldVariant;
use crate::tokens::field_common::{self, FieldIconRole, FieldState, FieldTokenSet, FieldVariant};

const TEXT_FIELD_TOKENS: FieldTokenSet =
    FieldTokenSet::new("md.comp.outlined-text-field", "md.comp.filled-text-field");

pub(crate) fn container_height(theme: &Theme, variant: TextFieldVariant) -> Px {
    field_common::container_height(theme, field_prefix(variant))
}

pub(crate) fn initial_input_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_sys("md.sys.color.surface")
}

pub(crate) fn text_input_style(
    theme: &Theme,
    variant: TextFieldVariant,
    focused: bool,
    hovered: bool,
    disabled: bool,
    error: bool,
) -> TextInputStyle {
    let prefix = field_prefix(variant);
    let state = field_state(hovered, disabled, error, focused);
    match variant {
        TextFieldVariant::Outlined => field_common::outlined_text_input_style(theme, prefix, state),
        TextFieldVariant::Filled => field_common::filled_text_input_style(
            theme,
            prefix,
            state,
            "md.sys.color.on-surface-variant",
        ),
    }
}

pub(crate) fn label_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    field_common::role_color(
        theme,
        field_prefix(variant),
        "label-text",
        field_state(hovered, disabled, error, focused),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn supporting_text_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    field_common::role_color(
        theme,
        field_prefix(variant),
        "supporting-text",
        field_state(hovered, disabled, error, focused),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn leading_icon_size(theme: &Theme, variant: TextFieldVariant) -> Px {
    field_common::icon_size(theme, field_prefix(variant), FieldIconRole::Leading)
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> (Color, f32) {
    field_common::role_color_with_opacity(
        theme,
        field_prefix(variant),
        "leading-icon",
        field_state(hovered, disabled, error, focused),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn trailing_icon_size(theme: &Theme, variant: TextFieldVariant) -> Px {
    field_common::icon_size(theme, field_prefix(variant), FieldIconRole::Trailing)
}

pub(crate) fn trailing_icon_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> (Color, f32) {
    field_common::role_color_with_opacity(
        theme,
        field_prefix(variant),
        "trailing-icon",
        field_state(hovered, disabled, error, focused),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn hover_state_layer(
    theme: &Theme,
    variant: TextFieldVariant,
    error: bool,
) -> Option<(Color, f32)> {
    match variant {
        TextFieldVariant::Outlined => None,
        TextFieldVariant::Filled => Some(field_common::hover_state_layer(
            theme,
            field_prefix(variant),
            error,
        )),
    }
}

fn field_prefix(variant: TextFieldVariant) -> &'static str {
    TEXT_FIELD_TOKENS.prefix(field_variant(variant))
}

fn field_variant(variant: TextFieldVariant) -> FieldVariant {
    match variant {
        TextFieldVariant::Outlined => FieldVariant::Outlined,
        TextFieldVariant::Filled => FieldVariant::Filled,
    }
}

fn field_state(hovered: bool, disabled: bool, error: bool, focused: bool) -> FieldState {
    FieldState::new(hovered, disabled, error, focused)
}

#[cfg(test)]
mod tests {
    use super::{label_color, supporting_text_color};
    use crate::text_field::TextFieldVariant;
    use crate::tokens::v30::{TypographyOptions, theme_config};
    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn apply_patch_color(cfg: &mut ThemeConfig, key: &str, hex: &str) {
        cfg.colors.insert(key.to_string(), hex.to_string());
    }

    #[test]
    fn outlined_hover_label_and_supporting_use_hover_tokens_when_present() {
        let mut app = App::new();
        let base = theme_config(TypographyOptions::default());
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&base));

        let mut patch = ThemeConfig::default();
        apply_patch_color(
            &mut patch,
            "md.comp.outlined-text-field.label-text.color",
            "#00ff00",
        );
        apply_patch_color(
            &mut patch,
            "md.comp.outlined-text-field.hover.label-text.color",
            "#ff0000",
        );
        apply_patch_color(
            &mut patch,
            "md.comp.outlined-text-field.supporting-text.color",
            "#00ff00",
        );
        apply_patch_color(
            &mut patch,
            "md.comp.outlined-text-field.hover.supporting-text.color",
            "#ff0000",
        );
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));

        let theme = Theme::global(&app);

        let base_label = label_color(
            theme,
            TextFieldVariant::Outlined,
            false,
            false,
            false,
            false,
        );
        let hover_label = label_color(theme, TextFieldVariant::Outlined, true, false, false, false);
        assert_ne!(base_label, hover_label);
        assert_eq!(
            base_label,
            theme
                .color_by_key("md.comp.outlined-text-field.label-text.color")
                .expect("expected patched base label color"),
        );
        assert_eq!(
            hover_label,
            theme
                .color_by_key("md.comp.outlined-text-field.hover.label-text.color")
                .expect("expected patched hover label color"),
        );

        let base_supporting = supporting_text_color(
            theme,
            TextFieldVariant::Outlined,
            false,
            false,
            false,
            false,
        );
        let hover_supporting =
            supporting_text_color(theme, TextFieldVariant::Outlined, true, false, false, false);
        assert_ne!(base_supporting, hover_supporting);
        assert_eq!(
            base_supporting,
            theme
                .color_by_key("md.comp.outlined-text-field.supporting-text.color")
                .expect("expected patched base supporting color"),
        );
        assert_eq!(
            hover_supporting,
            theme
                .color_by_key("md.comp.outlined-text-field.hover.supporting-text.color")
                .expect("expected patched hover supporting color"),
        );
    }

    #[test]
    fn outlined_error_hover_label_prefers_error_hover_token() {
        let mut app = App::new();
        let base = theme_config(TypographyOptions::default());
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&base));

        let mut patch = ThemeConfig::default();
        apply_patch_color(
            &mut patch,
            "md.comp.outlined-text-field.error.hover.label-text.color",
            "#112233",
        );
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));

        let theme = Theme::global(&app);
        let c = label_color(theme, TextFieldVariant::Outlined, true, false, true, false);
        assert_eq!(
            c,
            theme
                .color_by_key("md.comp.outlined-text-field.error.hover.label-text.color")
                .expect("expected patched error hover label color"),
        );
    }
}

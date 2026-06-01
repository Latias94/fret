//! Typed token access for Material 3 autocomplete.
//!
//! This module centralizes token key mapping and fallback chains so autocomplete visuals remain
//! stable and drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::{TextInputStyle, Theme};

use crate::text_field::TextFieldVariant;
use crate::tokens::{
    field_common::{self, FieldIconRole, FieldState, FieldTokenSet, FieldVariant},
    field_menu_common::{self, FieldMenuTokenSet},
    selectable_menu_item as selectable_item_tokens,
};

const AUTOCOMPLETE_FIELD_TOKENS: FieldTokenSet = FieldTokenSet::new(
    "md.comp.outlined-autocomplete.text-field",
    "md.comp.filled-autocomplete.text-field",
);
const AUTOCOMPLETE_MENU_TOKENS: FieldMenuTokenSet = FieldMenuTokenSet::new(
    "md.comp.outlined-autocomplete.menu",
    "md.comp.filled-autocomplete.menu",
);

pub(crate) fn text_field_container_height(theme: &Theme, variant: TextFieldVariant) -> Px {
    field_common::container_height(theme, field_prefix(variant))
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

pub(crate) fn trailing_icon_size(theme: &Theme, variant: TextFieldVariant) -> Px {
    field_common::icon_size(theme, field_prefix(variant), FieldIconRole::Trailing)
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
    Some(field_common::hover_state_layer(
        theme,
        field_prefix(variant),
        error,
    ))
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

fn field_prefix(variant: TextFieldVariant) -> &'static str {
    AUTOCOMPLETE_FIELD_TOKENS.prefix(field_variant(variant))
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

pub(crate) fn menu_container_background(theme: &Theme, variant: TextFieldVariant) -> Color {
    field_menu_common::container_background(theme, AUTOCOMPLETE_MENU_TOKENS, field_variant(variant))
}

pub(crate) fn menu_container_elevation(theme: &Theme, variant: TextFieldVariant) -> Px {
    field_menu_common::container_elevation(theme, AUTOCOMPLETE_MENU_TOKENS, field_variant(variant))
}

pub(crate) fn menu_container_shadow_color(theme: &Theme, variant: TextFieldVariant) -> Color {
    field_menu_common::container_shadow_color(
        theme,
        AUTOCOMPLETE_MENU_TOKENS,
        field_variant(variant),
    )
}

pub(crate) fn menu_container_shape(theme: &Theme, variant: TextFieldVariant) -> Corners {
    field_menu_common::container_shape(theme, AUTOCOMPLETE_MENU_TOKENS, field_variant(variant))
}

pub(crate) fn menu_list_item_height(theme: &Theme, variant: TextFieldVariant) -> Px {
    field_menu_common::list_item_height(theme, AUTOCOMPLETE_MENU_TOKENS, field_variant(variant))
}

pub(crate) fn menu_selectable_item_outer_horizontal_padding(
    theme: &Theme,
    _variant: TextFieldVariant,
) -> Px {
    selectable_item_tokens::outer_horizontal_padding(theme)
}

pub(crate) fn menu_selectable_item_outer_vertical_padding(
    theme: &Theme,
    _variant: TextFieldVariant,
    has_secondary_text: bool,
) -> Px {
    selectable_item_tokens::outer_vertical_padding(theme, has_secondary_text)
}

pub(crate) fn menu_list_item_content_horizontal_padding(
    theme: &Theme,
    _variant: TextFieldVariant,
) -> Px {
    selectable_item_tokens::content_horizontal_padding(theme)
}

pub(crate) fn menu_list_item_container_shape(
    theme: &Theme,
    _variant: TextFieldVariant,
    selected: bool,
) -> Corners {
    selectable_item_tokens::container_shape(theme, selected)
}

pub(crate) fn menu_list_item_label_text_style(
    theme: &Theme,
    _variant: TextFieldVariant,
) -> Option<TextStyle> {
    field_menu_common::list_item_label_text_style(theme)
}

pub(crate) fn menu_list_item_label_text_color(
    theme: &Theme,
    variant: TextFieldVariant,
    enabled: bool,
    selected: bool,
) -> Color {
    field_menu_common::list_item_label_text_color(
        theme,
        AUTOCOMPLETE_MENU_TOKENS,
        field_variant(variant),
        enabled,
        selected,
    )
}

pub(crate) fn menu_list_item_selected_container_color(
    theme: &Theme,
    variant: TextFieldVariant,
) -> Color {
    field_menu_common::list_item_selected_container_color(
        theme,
        AUTOCOMPLETE_MENU_TOKENS,
        field_variant(variant),
    )
}

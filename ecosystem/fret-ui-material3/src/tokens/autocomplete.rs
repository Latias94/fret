//! Typed token access for Material 3 autocomplete.
//!
//! This module centralizes token key mapping and fallback chains so autocomplete visuals remain
//! stable and drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::{TextInputStyle, Theme};
use fret_ui_kit::typography::{self, TextIntent};

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::text_field::TextFieldVariant;
use crate::tokens::{
    field_common::{self, FieldIconRole, FieldState, FieldTokenSet, FieldVariant},
    selectable_menu_item as selectable_item_tokens, shape,
};

const AUTOCOMPLETE_FIELD_TOKENS: FieldTokenSet = FieldTokenSet::new(
    "md.comp.outlined-autocomplete.text-field",
    "md.comp.filled-autocomplete.text-field",
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
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.menu.container.color",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.container.color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.surface-container")
}

pub(crate) fn menu_container_elevation(theme: &Theme, variant: TextFieldVariant) -> Px {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.menu.container.elevation",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.container.elevation",
    };
    theme.metric_by_key(key).unwrap_or(Px(3.0))
}

pub(crate) fn menu_container_shadow_color(theme: &Theme, variant: TextFieldVariant) -> Color {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.menu.container.shadow-color",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.container.shadow-color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.shadow")
}

pub(crate) fn menu_container_shape(theme: &Theme, variant: TextFieldVariant) -> Corners {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.menu.container.shape",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.container.shape",
    };
    shape::corners_or_metric(theme, key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-small"))
        .unwrap_or_else(|| Corners::all(Px(4.0)))
}

pub(crate) fn menu_list_item_height(theme: &Theme, variant: TextFieldVariant) -> Px {
    let key = match variant {
        TextFieldVariant::Outlined => {
            "md.comp.outlined-autocomplete.menu.list-item.container.height"
        }
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.list-item.container.height",
    };
    theme.metric_by_key(key).unwrap_or(Px(48.0))
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
    theme
        .text_style_by_key("md.sys.typescale.label-large")
        .map(|style| typography::with_intent(style, TextIntent::Control))
}

pub(crate) fn menu_list_item_label_text_color(
    theme: &Theme,
    variant: TextFieldVariant,
    enabled: bool,
    selected: bool,
) -> Color {
    if let Some(label) =
        selectable_item_tokens::selected_or_disabled_label_color(theme, selected, enabled)
    {
        return label;
    }

    let key = match variant {
        TextFieldVariant::Outlined => {
            "md.comp.outlined-autocomplete.menu.list-item.label-text.color"
        }
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.list-item.label-text.color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface")
}

pub(crate) fn menu_list_item_selected_container_color(
    theme: &Theme,
    variant: TextFieldVariant,
) -> Color {
    let key = match variant {
        TextFieldVariant::Outlined => {
            "md.comp.outlined-autocomplete.menu.list-item.selected.container.color"
        }
        TextFieldVariant::Filled => {
            "md.comp.filled-autocomplete.menu.list-item.selected.container.color"
        }
    };
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(key, "md.sys.color.surface-container-highest")
}

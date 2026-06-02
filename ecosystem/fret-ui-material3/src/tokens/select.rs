//!
//! Centralized token key mapping and fallback chains for Material 3 Select.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{MaterialTokenResolver, blend_over};
use crate::select::SelectVariant;
use crate::tokens::{
    field_common::{self, FieldIconRole, FieldState, FieldTokenSet, FieldVariant},
    field_menu_common::{self, FieldMenuTokenSet, MenuItemIconRole},
    selectable_menu_item as selectable_item_tokens, typography as material_typography,
};

const SELECT_FIELD_TOKENS: FieldTokenSet = FieldTokenSet::new(
    "md.comp.outlined-select.text-field",
    "md.comp.filled-select.text-field",
);
const SELECT_MENU_TOKENS: FieldMenuTokenSet =
    FieldMenuTokenSet::new("md.comp.outlined-select.menu", "md.comp.filled-select.menu");
const INPUT_TEXT_STYLE_KEY: &str = "md.sys.typescale.body-large";
const SUPPORTING_TEXT_STYLE_KEY: &str = "md.sys.typescale.body-small";

pub(crate) fn container_height(theme: &Theme, variant: SelectVariant) -> Px {
    field_common::container_height(theme, field_prefix(variant))
}

pub(crate) fn container_corner(theme: &Theme, variant: SelectVariant) -> Corners {
    field_common::container_shape(theme, field_prefix(variant), field_variant(variant))
}

pub(crate) fn container_background(theme: &Theme, variant: SelectVariant, disabled: bool) -> Color {
    let key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.text-field.container.color",
        SelectVariant::Filled => "md.comp.filled-select.text-field.container.color",
    };
    let tokens = MaterialTokenResolver::new(theme);
    let color = tokens.color_comp_or_sys(key, "md.sys.color.surface-container-highest");

    if disabled && variant == SelectVariant::Filled {
        let (overlay, opacity) = tokens.color_comp_or_sys_with_opacity(
            "md.comp.filled-select.text-field.disabled.container.color",
            "md.sys.color.on-surface",
            Some("md.comp.filled-select.text-field.disabled.container.opacity"),
            0.04,
        );
        return blend_over(color, overlay, opacity);
    }

    color
}

pub(crate) fn hover_state_layer(
    theme: &Theme,
    variant: SelectVariant,
    error: bool,
) -> (Color, f32) {
    field_common::hover_state_layer(theme, field_prefix(variant), error)
}

pub(crate) fn outline(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Option<(Px, Color, f32)> {
    if variant != SelectVariant::Outlined {
        return None;
    }

    Some(field_common::outline(
        theme,
        field_prefix(variant),
        field_state(hovered, disabled, error, focused),
    ))
}

pub(crate) fn active_indicator(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Option<(Px, Color, f32)> {
    if variant != SelectVariant::Filled {
        return None;
    }

    Some(field_common::active_indicator(
        theme,
        field_prefix(variant),
        field_state(hovered, disabled, error, focused),
        "md.sys.color.on-surface-variant",
    ))
}

pub(crate) fn input_text_style(theme: &Theme, variant: SelectVariant) -> Option<TextStyle> {
    let _ = variant;
    // Material Web v30 models Select typography via `*.font/size/weight/tracking/line-height` and
    // a `*.type` mixin token (not a scalar key). For now, use the canonical sys typescale and keep
    // component-specific typography as a future import step.
    material_typography::text_style_value(theme, INPUT_TEXT_STYLE_KEY, TextIntent::Control)
}

pub(crate) fn supporting_text_style(theme: &Theme, variant: SelectVariant) -> Option<TextStyle> {
    let _ = variant;
    material_typography::text_style_value(theme, SUPPORTING_TEXT_STYLE_KEY, TextIntent::Content)
}

pub(crate) fn input_text_color(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> (Color, f32) {
    field_common::role_color_with_opacity(
        theme,
        field_prefix(variant),
        "input-text",
        field_state(hovered, disabled, error, focused),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn leading_icon_size(theme: &Theme, variant: SelectVariant) -> Px {
    field_common::icon_size(theme, field_prefix(variant), FieldIconRole::Leading)
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    variant: SelectVariant,
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

pub(crate) fn trailing_icon_size(theme: &Theme, variant: SelectVariant) -> Px {
    field_common::icon_size(theme, field_prefix(variant), FieldIconRole::Trailing)
}

pub(crate) fn trailing_icon_color(
    theme: &Theme,
    variant: SelectVariant,
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

pub(crate) fn placeholder_color(
    theme: &Theme,
    variant: SelectVariant,
    disabled: bool,
    _error: bool,
) -> Color {
    field_common::placeholder_color(
        theme,
        field_prefix(variant),
        FieldState::new(false, disabled, false, false),
    )
}

pub(crate) fn label_color(
    theme: &Theme,
    variant: SelectVariant,
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
    variant: SelectVariant,
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

fn field_prefix(variant: SelectVariant) -> &'static str {
    SELECT_FIELD_TOKENS.prefix(field_variant(variant))
}

fn field_variant(variant: SelectVariant) -> FieldVariant {
    match variant {
        SelectVariant::Outlined => FieldVariant::Outlined,
        SelectVariant::Filled => FieldVariant::Filled,
    }
}

fn field_state(hovered: bool, disabled: bool, error: bool, focused: bool) -> FieldState {
    FieldState::new(hovered, disabled, error, focused)
}

pub(crate) fn menu_container_background(theme: &Theme, variant: SelectVariant) -> Color {
    field_menu_common::container_background(theme, SELECT_MENU_TOKENS, field_variant(variant))
}

pub(crate) fn menu_container_elevation(theme: &Theme, variant: SelectVariant) -> Px {
    field_menu_common::container_elevation(theme, SELECT_MENU_TOKENS, field_variant(variant))
}

pub(crate) fn menu_container_shadow_color(theme: &Theme, variant: SelectVariant) -> Color {
    field_menu_common::container_shadow_color(theme, SELECT_MENU_TOKENS, field_variant(variant))
}

pub(crate) fn menu_container_shape(theme: &Theme, variant: SelectVariant) -> Corners {
    field_menu_common::container_shape(theme, SELECT_MENU_TOKENS, field_variant(variant))
}

pub(crate) fn menu_list_item_height(theme: &Theme, variant: SelectVariant) -> Px {
    field_menu_common::list_item_height(theme, SELECT_MENU_TOKENS, field_variant(variant))
}

pub(crate) fn menu_selectable_item_outer_horizontal_padding(
    _theme: &Theme,
    _variant: SelectVariant,
) -> Px {
    selectable_item_tokens::outer_horizontal_padding(_theme)
}

pub(crate) fn menu_selectable_item_outer_vertical_padding(
    _theme: &Theme,
    _variant: SelectVariant,
    has_secondary_text: bool,
) -> Px {
    selectable_item_tokens::outer_vertical_padding(_theme, has_secondary_text)
}

pub(crate) fn menu_list_item_content_horizontal_padding(
    _theme: &Theme,
    _variant: SelectVariant,
) -> Px {
    selectable_item_tokens::content_horizontal_padding(_theme)
}

pub(crate) fn menu_list_item_icon_text_gap(theme: &Theme, _variant: SelectVariant) -> Px {
    selectable_item_tokens::icon_text_gap(theme)
}

pub(crate) fn menu_list_item_container_shape(
    theme: &Theme,
    _variant: SelectVariant,
    selected: bool,
) -> Corners {
    selectable_item_tokens::container_shape(theme, selected)
}

pub(crate) fn menu_list_item_label_text_style(
    theme: &Theme,
    variant: SelectVariant,
) -> Option<TextStyle> {
    let _ = variant;
    // Material Web v30 `menu.list-item.label-text.type` is a mixin. The underlying scalars map to
    // sys `label-large`, so use that as the stable default in v1.
    field_menu_common::list_item_label_text_style(theme)
}

pub(crate) fn menu_list_item_label_text_style_or_fallback(
    theme: &Theme,
    variant: SelectVariant,
) -> TextStyle {
    menu_list_item_label_text_style(theme, variant).unwrap_or_else(|| TextStyle {
        size: Px(14.0),
        ..Default::default()
    })
}

pub(crate) fn menu_list_item_supporting_text_estimate_style(
    theme: &Theme,
    variant: SelectVariant,
) -> TextStyle {
    let _ = variant;
    material_typography::text_style_value(theme, SUPPORTING_TEXT_STYLE_KEY, TextIntent::Content)
        .unwrap_or_else(|| TextStyle {
            size: Px(12.0),
            ..Default::default()
        })
}

pub(crate) fn menu_list_item_label_text_color(
    theme: &Theme,
    variant: SelectVariant,
    enabled: bool,
    selected: bool,
) -> Color {
    field_menu_common::list_item_label_text_color(
        theme,
        SELECT_MENU_TOKENS,
        field_variant(variant),
        enabled,
        selected,
    )
}

pub(crate) fn menu_list_item_leading_icon_size(theme: &Theme, variant: SelectVariant) -> Px {
    field_menu_common::list_item_icon_size(
        theme,
        SELECT_MENU_TOKENS,
        field_variant(variant),
        MenuItemIconRole::Leading,
    )
}

pub(crate) fn menu_list_item_leading_icon_color(
    theme: &Theme,
    variant: SelectVariant,
    enabled: bool,
    selected: bool,
) -> Color {
    field_menu_common::list_item_icon_color(
        theme,
        SELECT_MENU_TOKENS,
        field_variant(variant),
        MenuItemIconRole::Leading,
        enabled,
        selected,
    )
}

pub(crate) fn menu_list_item_trailing_icon_size(theme: &Theme, variant: SelectVariant) -> Px {
    field_menu_common::list_item_icon_size(
        theme,
        SELECT_MENU_TOKENS,
        field_variant(variant),
        MenuItemIconRole::Trailing,
    )
}

pub(crate) fn menu_list_item_trailing_icon_color(
    theme: &Theme,
    variant: SelectVariant,
    enabled: bool,
    selected: bool,
) -> Color {
    field_menu_common::list_item_icon_color(
        theme,
        SELECT_MENU_TOKENS,
        field_variant(variant),
        MenuItemIconRole::Trailing,
        enabled,
        selected,
    )
}

pub(crate) fn menu_list_item_selected_container_color(
    theme: &Theme,
    variant: SelectVariant,
) -> Color {
    field_menu_common::list_item_selected_container_color(
        theme,
        SELECT_MENU_TOKENS,
        field_variant(variant),
    )
}

pub(crate) fn menu_list_item_state_layer_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface")
}

pub(crate) fn menu_list_item_pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_sys("md.sys.state.pressed.state-layer-opacity", 0.1)
}

pub(crate) fn menu_list_item_hover_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_sys("md.sys.state.hover.state-layer-opacity", 0.08)
}

pub(crate) fn menu_list_item_focus_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_sys("md.sys.state.focus.state-layer-opacity", 0.1)
}

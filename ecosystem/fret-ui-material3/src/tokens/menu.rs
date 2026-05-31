//! Typed token access for Material 3 menus.
//!
//! This module centralizes token key mapping and fallback chains so menu visuals remain stable and
//! drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::content::MaterialContentDefaults;
use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul};
use crate::tokens::typography;

pub(crate) const ITEM_MIN_WIDTH_FALLBACK: Px = Px(112.0);
pub(crate) const ITEM_MAX_WIDTH_FALLBACK: Px = Px(280.0);
pub(crate) const CONTAINER_VERTICAL_PADDING_FALLBACK: Px = Px(8.0);
pub(crate) const ITEM_HORIZONTAL_PADDING_FALLBACK: Px = Px(12.0);
pub(crate) const ITEM_TWO_LINE_HEIGHT_FALLBACK: Px = Px(64.0);
pub(crate) const ITEM_ICON_SIZE_FALLBACK: Px = Px(24.0);
pub(crate) const ITEM_SLOT_GAP_FALLBACK: Px = Px(12.0);
pub(crate) const SECTION_LABEL_HEIGHT_FALLBACK: Px = Px(32.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuItemInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn list_item_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.container.height")
        .unwrap_or(Px(48.0))
}

pub(crate) fn list_item_two_line_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.two-line-container.height")
        .or_else(|| theme.metric_by_key("md.comp.menu.list-item.supporting-text.container.height"))
        .unwrap_or(ITEM_TWO_LINE_HEIGHT_FALLBACK)
}

pub(crate) fn list_item_height_for_supporting(theme: &Theme, has_supporting_text: bool) -> Px {
    if has_supporting_text {
        list_item_two_line_height(theme)
    } else {
        list_item_height(theme)
    }
}

pub(crate) fn item_min_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.container.min-width")
        .unwrap_or(ITEM_MIN_WIDTH_FALLBACK)
}

pub(crate) fn item_max_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.container.max-width")
        .unwrap_or(ITEM_MAX_WIDTH_FALLBACK)
}

pub(crate) fn container_vertical_padding(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.container.vertical-padding")
        .unwrap_or(CONTAINER_VERTICAL_PADDING_FALLBACK)
}

pub(crate) fn item_horizontal_padding(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.content.horizontal-padding")
        .unwrap_or(ITEM_HORIZONTAL_PADDING_FALLBACK)
}

pub(crate) fn item_slot_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.content.gap")
        .or_else(|| theme.metric_by_key("md.comp.menu.list-item.leading-icon.trailing-space"))
        .unwrap_or(ITEM_SLOT_GAP_FALLBACK)
}

pub(crate) fn item_icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.icon.size")
        .or_else(|| theme.metric_by_key("md.comp.menu.list-item.leading-icon.size"))
        .unwrap_or(ITEM_ICON_SIZE_FALLBACK)
}

pub(crate) fn section_label_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.section-label.container.height")
        .unwrap_or(SECTION_LABEL_HEIGHT_FALLBACK)
}

pub(crate) fn item_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.menu.list-item.label-text"),
        "md.sys.typescale.label-large",
        Some("md.comp.menu.list-item.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn item_supporting_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.menu.list-item.supporting-text"),
        "md.sys.typescale.body-medium",
        Some("md.comp.menu.list-item.supporting-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn item_trailing_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.menu.list-item.trailing-text"),
        "md.sys.typescale.label-large",
        Some("md.comp.menu.list-item.trailing-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn section_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.menu.section-label.label-text"),
        "md.sys.typescale.label-small",
        Some("md.comp.menu.section-label.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn container_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.menu.container.color",
        "md.sys.color.surface-container",
    )
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.container.elevation")
        .unwrap_or(Px(0.0))
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys("md.comp.menu.container.shadow-color", "md.sys.color.shadow")
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .metric_by_key("md.comp.menu.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.extra-small"))
        .map(Corners::all)
        .unwrap_or_else(|| Corners::all(Px(4.0)))
}

pub(crate) fn divider_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.divider.height")
        .unwrap_or(Px(1.0))
}

pub(crate) fn divider_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys("md.comp.menu.divider.color", "md.sys.color.surface-variant")
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_optional(
        Some("md.comp.menu.list-item.pressed.state-layer.opacity"),
        0.1,
    )
}

pub(crate) fn item_outcomes(
    theme: &Theme,
    enabled: bool,
    interaction: MenuItemInteraction,
) -> (Color, Color, f32) {
    let (label_key, state_layer_key, opacity_key) = match interaction {
        MenuItemInteraction::Pressed => (
            "md.comp.menu.list-item.pressed.label-text.color",
            "md.comp.menu.list-item.pressed.state-layer.color",
            "md.comp.menu.list-item.pressed.state-layer.opacity",
        ),
        MenuItemInteraction::Focused => (
            "md.comp.menu.list-item.focus.label-text.color",
            "md.comp.menu.list-item.focus.state-layer.color",
            "md.comp.menu.list-item.focus.state-layer.opacity",
        ),
        MenuItemInteraction::Hovered => (
            "md.comp.menu.list-item.hover.label-text.color",
            "md.comp.menu.list-item.hover.state-layer.color",
            "md.comp.menu.list-item.hover.state-layer.opacity",
        ),
        MenuItemInteraction::Default => (
            "md.comp.menu.list-item.label-text.color",
            // Keep the default state-layer token aligned to hover for this MVP.
            "md.comp.menu.list-item.hover.state-layer.color",
            "md.comp.menu.list-item.hover.state-layer.opacity",
        ),
    };

    let defaults = MaterialContentDefaults::on_surface(theme);
    let tokens = MaterialTokenResolver::new(theme);
    let mut label = tokens.color_comp_or_fallback(label_key, defaults.content_color);
    let state_layer = tokens.color_comp_or_fallback(state_layer_key, defaults.content_color);
    let mut opacity = tokens.number_optional(Some(opacity_key), 0.0);

    if !enabled {
        let label_opacity = tokens.number_optional(
            Some("md.comp.menu.list-item.disabled.label-text.opacity"),
            defaults.disabled_opacity,
        );
        label = alpha_mul(label, label_opacity);
        opacity = 0.0;
    }

    (label, state_layer, opacity)
}

pub(crate) fn item_icon_color(theme: &Theme, enabled: bool) -> Color {
    item_content_color(
        theme,
        enabled,
        &[
            "md.comp.menu.list-item.leading-icon.color",
            "md.comp.menu.list-item.trailing-icon.color",
            "md.comp.menu.list-item.icon.color",
        ],
        "md.comp.menu.list-item.disabled.leading-icon.opacity",
    )
}

pub(crate) fn item_supporting_text_color(theme: &Theme, enabled: bool) -> Color {
    item_content_color(
        theme,
        enabled,
        &["md.comp.menu.list-item.supporting-text.color"],
        "md.comp.menu.list-item.disabled.supporting-text.opacity",
    )
}

pub(crate) fn item_trailing_text_color(theme: &Theme, enabled: bool) -> Color {
    item_content_color(
        theme,
        enabled,
        &[
            "md.comp.menu.list-item.trailing-text.color",
            "md.comp.menu.list-item.shortcut.color",
        ],
        "md.comp.menu.list-item.disabled.trailing-text.opacity",
    )
}

pub(crate) fn section_label_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_chain_or_sys(
        &[
            "md.comp.menu.section-label.label-text.color",
            "md.comp.menu.list-item.supporting-text.color",
        ],
        "md.sys.color.on-surface-variant",
    )
}

fn item_content_color(
    theme: &Theme,
    enabled: bool,
    comp_keys: &[&str],
    disabled_opacity_key: &str,
) -> Color {
    let defaults = MaterialContentDefaults::on_surface(theme);
    let tokens = MaterialTokenResolver::new(theme);
    let mut color = tokens.color_comp_chain_or_sys(comp_keys, "md.sys.color.on-surface-variant");
    if !enabled {
        let opacity = tokens.number_optional(Some(disabled_opacity_key), defaults.disabled_opacity);
        color = alpha_mul(color, opacity);
    }
    color
}

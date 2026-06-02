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

fn menu_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn menu_metric_chain(theme: &Theme, keys: &[&'static str], fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(keys, fallback)
}

pub(crate) fn list_item_height(theme: &Theme) -> Px {
    menu_metric(theme, "md.comp.menu.list-item.container.height", Px(48.0))
}

pub(crate) fn list_item_two_line_height(theme: &Theme) -> Px {
    menu_metric_chain(
        theme,
        &[
            "md.comp.menu.list-item.two-line-container.height",
            "md.comp.menu.list-item.supporting-text.container.height",
        ],
        ITEM_TWO_LINE_HEIGHT_FALLBACK,
    )
}

pub(crate) fn list_item_height_for_supporting(theme: &Theme, has_supporting_text: bool) -> Px {
    if has_supporting_text {
        list_item_two_line_height(theme)
    } else {
        list_item_height(theme)
    }
}

pub(crate) fn item_min_width(theme: &Theme) -> Px {
    menu_metric(
        theme,
        "md.comp.menu.list-item.container.min-width",
        ITEM_MIN_WIDTH_FALLBACK,
    )
}

pub(crate) fn item_max_width(theme: &Theme) -> Px {
    menu_metric(
        theme,
        "md.comp.menu.list-item.container.max-width",
        ITEM_MAX_WIDTH_FALLBACK,
    )
}

pub(crate) fn container_vertical_padding(theme: &Theme) -> Px {
    menu_metric(
        theme,
        "md.comp.menu.container.vertical-padding",
        CONTAINER_VERTICAL_PADDING_FALLBACK,
    )
}

pub(crate) fn item_horizontal_padding(theme: &Theme) -> Px {
    menu_metric(
        theme,
        "md.comp.menu.list-item.content.horizontal-padding",
        ITEM_HORIZONTAL_PADDING_FALLBACK,
    )
}

pub(crate) fn item_slot_gap(theme: &Theme) -> Px {
    menu_metric_chain(
        theme,
        &[
            "md.comp.menu.list-item.content.gap",
            "md.comp.menu.list-item.leading-icon.trailing-space",
        ],
        ITEM_SLOT_GAP_FALLBACK,
    )
}

pub(crate) fn item_icon_size(theme: &Theme) -> Px {
    menu_metric_chain(
        theme,
        &[
            "md.comp.menu.list-item.icon.size",
            "md.comp.menu.list-item.leading-icon.size",
        ],
        ITEM_ICON_SIZE_FALLBACK,
    )
}

pub(crate) fn section_label_height(theme: &Theme) -> Px {
    menu_metric(
        theme,
        "md.comp.menu.section-label.container.height",
        SECTION_LABEL_HEIGHT_FALLBACK,
    )
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
    menu_metric(theme, "md.comp.menu.container.elevation", Px(0.0))
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys("md.comp.menu.container.shadow-color", "md.sys.color.shadow")
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[
            "md.comp.menu.container.shape",
            "md.sys.shape.corner.extra-small",
        ],
        Corners::all(Px(4.0)),
    )
}

pub(crate) fn divider_height(theme: &Theme) -> Px {
    menu_metric(theme, "md.comp.menu.divider.height", Px(1.0))
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
        label = tokens.color_comp_or_fallback(
            "md.comp.menu.list-item.disabled.label-text.color",
            defaults.content_color,
        );
        let label_opacity = tokens.number_chain(
            &["md.comp.menu.list-item.disabled.label-text.opacity"],
            defaults.disabled_opacity,
        );
        label = alpha_mul(label, label_opacity);
        opacity = 0.0;
    }

    (label, state_layer, opacity)
}

pub(crate) fn item_icon_color(
    theme: &Theme,
    enabled: bool,
    interaction: MenuItemInteraction,
) -> Color {
    let comp_keys: &[&str] = if !enabled {
        &[
            "md.comp.menu.list-item.with-leading-icon.disabled.leading-icon.color",
            "md.comp.menu.list-item.with-trailing-icon.disabled.trailing-icon.color",
            "md.comp.menu.list-item.leading-icon.color",
            "md.comp.menu.list-item.trailing-icon.color",
            "md.comp.menu.list-item.icon.color",
        ]
    } else {
        match interaction {
            MenuItemInteraction::Pressed => &[
                "md.comp.menu.list-item.with-leading-icon.pressed.icon.color",
                "md.comp.menu.list-item.with-trailing-icon.pressed.icon.color",
                "md.comp.menu.list-item.leading-icon.color",
                "md.comp.menu.list-item.trailing-icon.color",
                "md.comp.menu.list-item.icon.color",
            ],
            MenuItemInteraction::Focused => &[
                "md.comp.menu.list-item.with-leading-icon.focus.icon.color",
                "md.comp.menu.list-item.with-trailing-icon.focus.icon.color",
                "md.comp.menu.list-item.leading-icon.color",
                "md.comp.menu.list-item.trailing-icon.color",
                "md.comp.menu.list-item.icon.color",
            ],
            MenuItemInteraction::Hovered => &[
                "md.comp.menu.list-item.with-leading-icon.hover.icon.color",
                "md.comp.menu.list-item.with-trailing-icon.hover.icon.color",
                "md.comp.menu.list-item.leading-icon.color",
                "md.comp.menu.list-item.trailing-icon.color",
                "md.comp.menu.list-item.icon.color",
            ],
            MenuItemInteraction::Default => &[
                "md.comp.menu.list-item.with-leading-icon.leading-icon.color",
                "md.comp.menu.list-item.with-trailing-icon.trailing-icon.color",
                "md.comp.menu.list-item.leading-icon.color",
                "md.comp.menu.list-item.trailing-icon.color",
                "md.comp.menu.list-item.icon.color",
            ],
        }
    };

    item_content_color(
        theme,
        enabled,
        comp_keys,
        &[
            "md.comp.menu.list-item.with-leading-icon.disabled.leading-icon.opacity",
            "md.comp.menu.list-item.with-trailing-icon.disabled.trailing-icon.opacity",
            "md.comp.menu.list-item.disabled.leading-icon.opacity",
        ],
    )
}

pub(crate) fn item_supporting_text_color(theme: &Theme, enabled: bool) -> Color {
    item_content_color(
        theme,
        enabled,
        &["md.comp.menu.list-item.supporting-text.color"],
        &["md.comp.menu.list-item.disabled.supporting-text.opacity"],
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
        &["md.comp.menu.list-item.disabled.trailing-text.opacity"],
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
    disabled_opacity_keys: &[&str],
) -> Color {
    let defaults = MaterialContentDefaults::on_surface(theme);
    let tokens = MaterialTokenResolver::new(theme);
    let mut color = tokens.color_comp_chain_or_sys(comp_keys, "md.sys.color.on-surface-variant");
    if !enabled {
        let opacity = tokens.number_chain(disabled_opacity_keys, defaults.disabled_opacity);
        color = alpha_mul(color, opacity);
    }
    color
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
    fn menu_metric_chains_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.menu.list-item.supporting-text.container.height".to_string(),
            72.0,
        );
        patch.metrics.insert(
            "md.comp.menu.list-item.leading-icon.trailing-space".to_string(),
            16.0,
        );
        patch
            .metrics
            .insert("md.comp.menu.list-item.leading-icon.size".to_string(), 28.0);
        patch
            .metrics
            .insert("md.sys.shape.corner.extra-small".to_string(), 6.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(list_item_two_line_height(&theme), Px(72.0));
        assert_eq!(item_slot_gap(&theme), Px(16.0));
        assert_eq!(item_icon_size(&theme), Px(28.0));
        assert_eq!(container_shape(&theme), Corners::all(Px(6.0)));
    }

    #[test]
    fn menu_disabled_label_prefers_disabled_color_and_opacity() {
        let mut patch = ThemeConfig::default();
        patch.colors.insert(
            "md.comp.menu.list-item.label-text.color".to_string(),
            "#ff0000".to_string(),
        );
        patch.colors.insert(
            "md.comp.menu.list-item.disabled.label-text.color".to_string(),
            "#0000ff".to_string(),
        );
        patch.numbers.insert(
            "md.comp.menu.list-item.disabled.label-text.opacity".to_string(),
            0.5,
        );
        let (_app, theme) = theme_with_patch(patch);

        let (label, _, state_opacity) = item_outcomes(&theme, false, MenuItemInteraction::Default);
        let expected = alpha_mul(
            theme
                .color_by_key("md.comp.menu.list-item.disabled.label-text.color")
                .expect("patched disabled label color"),
            0.5,
        );

        assert_eq!(label, expected);
        assert_eq!(state_opacity, 0.0);
    }

    #[test]
    fn menu_icon_color_prefers_material_web_interaction_keys() {
        let mut patch = ThemeConfig::default();
        patch.colors.insert(
            "md.comp.menu.list-item.icon.color".to_string(),
            "#ff0000".to_string(),
        );
        patch.colors.insert(
            "md.comp.menu.list-item.with-leading-icon.pressed.icon.color".to_string(),
            "#00aa00".to_string(),
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            item_icon_color(&theme, true, MenuItemInteraction::Pressed),
            theme
                .color_by_key("md.comp.menu.list-item.with-leading-icon.pressed.icon.color")
                .expect("patched pressed icon color")
        );
    }

    #[test]
    fn menu_shape_prefers_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.menu.container.shape".to_string(), 4.0);
        patch.corners.insert(
            "md.comp.menu.container.shape".to_string(),
            Corners {
                top_left: Px(3.0),
                top_right: Px(5.0),
                bottom_right: Px(7.0),
                bottom_left: Px(9.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme),
            Corners {
                top_left: Px(3.0),
                top_right: Px(5.0),
                bottom_right: Px(7.0),
                bottom_left: Px(9.0),
            }
        );
    }
}

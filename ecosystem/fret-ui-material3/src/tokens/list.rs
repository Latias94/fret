//! Typed token access for Material 3 lists.
//!
//! This module centralizes token key mapping and fallback chains so list visuals remain stable and
//! drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::content::MaterialContentDefaults;
use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};
use crate::tokens::typography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListItemInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn one_line_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.one-line.container.height")
        .unwrap_or(Px(56.0))
}

pub(crate) fn two_line_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.two-line.container.height")
        .unwrap_or(Px(72.0))
}

pub(crate) fn three_line_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.three-line.container.height")
        .unwrap_or(Px(88.0))
}

pub(crate) fn item_container_shape_with_variant(
    theme: &Theme,
    selected: bool,
    expressive: bool,
) -> Corners {
    let (expressive_key, standard_key, fallback) = if selected {
        (
            "md.comp.list.list-item.selected.container.expressive.shape",
            "md.comp.list.list-item.selected.container.shape",
            Corners::all(Px(16.0)),
        )
    } else {
        (
            "md.comp.list.list-item.container.expressive.shape",
            "md.comp.list.list-item.container.shape",
            Corners::all(Px(0.0)),
        )
    };

    if expressive {
        theme
            .corners_by_key(expressive_key)
            .or_else(|| theme.corners_by_key(standard_key))
            .unwrap_or(fallback)
    } else {
        theme.corners_by_key(standard_key).unwrap_or(fallback)
    }
}

pub(crate) fn item_container_shape_for_interaction(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: ListItemInteraction,
    expressive: bool,
) -> Corners {
    if !expressive {
        return item_container_shape_with_variant(theme, selected, false);
    }

    let key = match (selected, enabled, interaction) {
        (true, false, _) => "md.comp.list.list-item.selected.disabled.container.expressive.shape",
        (true, true, ListItemInteraction::Pressed) => {
            "md.comp.list.list-item.selected.pressed.container.expressive.shape"
        }
        (true, true, ListItemInteraction::Focused) => {
            "md.comp.list.list-item.selected.focused.container.expressive.shape"
        }
        (true, true, ListItemInteraction::Hovered) => {
            "md.comp.list.list-item.selected.hovered.container.expressive.shape"
        }
        (true, true, ListItemInteraction::Default) => {
            "md.comp.list.list-item.selected.container.expressive.shape"
        }
        (false, false, _) => "md.comp.list.list-item.disabled.container.expressive.shape",
        (false, true, ListItemInteraction::Pressed) => {
            "md.comp.list.list-item.pressed.container.expressive.shape"
        }
        (false, true, ListItemInteraction::Focused) => {
            "md.comp.list.list-item.focused.container.expressive.shape"
        }
        (false, true, ListItemInteraction::Hovered) => {
            "md.comp.list.list-item.hovered.container.expressive.shape"
        }
        (false, true, ListItemInteraction::Default) => {
            "md.comp.list.list-item.container.expressive.shape"
        }
    };

    theme
        .corners_by_key(key)
        .unwrap_or_else(|| item_container_shape_with_variant(theme, selected, true))
}

pub(crate) fn item_between_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.between-space")
        .unwrap_or(Px(12.0))
}

pub(crate) fn item_leading_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.leading-space")
        .unwrap_or(Px(16.0))
}

pub(crate) fn item_trailing_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.trailing-space")
        .unwrap_or(Px(16.0))
}

pub(crate) fn item_top_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.top-space")
        .unwrap_or(Px(10.0))
}

pub(crate) fn item_bottom_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.bottom-space")
        .unwrap_or(Px(10.0))
}

pub(crate) fn leading_icon_size_with_variant(theme: &Theme, expressive: bool) -> Px {
    if expressive {
        theme
            .metric_by_key("md.comp.list.list-item.leading-icon.expressive.size")
            .or_else(|| theme.metric_by_key("md.comp.list.list-item.leading-icon.size"))
            .unwrap_or(Px(24.0))
    } else {
        theme
            .metric_by_key("md.comp.list.list-item.leading-icon.size")
            .unwrap_or(Px(24.0))
    }
}

pub(crate) fn trailing_icon_size_with_variant(theme: &Theme, expressive: bool) -> Px {
    if expressive {
        theme
            .metric_by_key("md.comp.list.list-item.trailing-icon.expressive.size")
            .or_else(|| theme.metric_by_key("md.comp.list.list-item.trailing-icon.size"))
            .unwrap_or(Px(24.0))
    } else {
        theme
            .metric_by_key("md.comp.list.list-item.trailing-icon.size")
            .unwrap_or(Px(24.0))
    }
}

fn supporting_text_opacity(theme: &Theme, enabled: bool, selected: bool) -> f32 {
    if enabled {
        return 1.0;
    }

    MaterialTokenResolver::new(theme)
        .number_comp_or_sys(
            if selected {
                "md.comp.list.list-item.selected.disabled.supporting-text.opacity"
            } else {
                "md.comp.list.list-item.disabled.supporting-text.opacity"
            },
            "md.sys.state.disabled.state-layer-opacity",
            0.38,
        )
        .clamp(0.0, 1.0)
}

fn overline_text_opacity(theme: &Theme, enabled: bool, selected: bool) -> f32 {
    if enabled {
        return 1.0;
    }

    MaterialTokenResolver::new(theme)
        .number_comp_or_sys(
            if selected {
                "md.comp.list.list-item.selected.disabled.overline.opacity"
            } else {
                "md.comp.list.list-item.disabled.overline.opacity"
            },
            "md.sys.state.disabled.state-layer-opacity",
            0.38,
        )
        .clamp(0.0, 1.0)
}

fn trailing_supporting_text_opacity(theme: &Theme, enabled: bool, selected: bool) -> f32 {
    if enabled {
        return 1.0;
    }

    let tokens = MaterialTokenResolver::new(theme);
    if selected {
        tokens
            .number_comp_or_sys(
                "md.comp.list.list-item.selected.disabled.trailing-supporting-text.opacity",
                "md.sys.state.disabled.state-layer-opacity",
                0.38,
            )
            .clamp(0.0, 1.0)
    } else {
        // Material Web v30 does not define a dedicated non-selected trailing supporting opacity
        // token; fall back to the sys disabled opacity.
        tokens
            .number_sys("md.sys.state.disabled.state-layer-opacity", 0.38)
            .clamp(0.0, 1.0)
    }
}

pub(crate) fn supporting_text_style(theme: &Theme, _selected: bool) -> Option<TextStyle> {
    theme.text_style_by_key("md.sys.typescale.body-medium")
}

pub(crate) fn trailing_supporting_text_style(theme: &Theme, _selected: bool) -> Option<TextStyle> {
    theme.text_style_by_key("md.sys.typescale.label-small")
}

pub(crate) fn overline_text_style(theme: &Theme, _selected: bool) -> Option<TextStyle> {
    theme.text_style_by_key("md.sys.typescale.label-small")
}

pub(crate) fn label_text_style(theme: &Theme, _selected: bool) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        None,
        "md.sys.typescale.body-large",
        Some("md.comp.list.list-item.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn supporting_text_color(theme: &Theme, enabled: bool, selected: bool) -> Color {
    let key = match (selected, enabled) {
        (true, true) => "md.comp.list.list-item.selected.supporting-text.color",
        (true, false) => "md.comp.list.list-item.selected.disabled.supporting-text.color",
        (false, true) => "md.comp.list.list-item.supporting-text.color",
        (false, false) => "md.comp.list.list-item.disabled.supporting-text.color",
    };
    let mut color =
        MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface-variant");
    color = alpha_mul(color, supporting_text_opacity(theme, enabled, selected));
    color
}

pub(crate) fn trailing_supporting_text_color(
    theme: &Theme,
    enabled: bool,
    selected: bool,
) -> Color {
    let key = match (selected, enabled) {
        (true, true) => "md.comp.list.list-item.selected.trailing-supporting-text.color",
        (true, false) => "md.comp.list.list-item.selected.disabled.trailing-supporting-text.color",
        // Material Web v30 does not define a dedicated non-selected disabled trailing supporting
        // color token; use the enabled color with disabled opacity applied.
        (false, _) => "md.comp.list.list-item.trailing-supporting-text.color",
    };
    let mut color =
        MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface-variant");
    color = alpha_mul(
        color,
        trailing_supporting_text_opacity(theme, enabled, selected),
    );
    color
}

pub(crate) fn overline_text_color(theme: &Theme, enabled: bool, selected: bool) -> Color {
    let key = match (selected, enabled) {
        (true, true) => "md.comp.list.list-item.selected.overline.color",
        (true, false) => "md.comp.list.list-item.selected.disabled.overline.color",
        (false, true) => "md.comp.list.list-item.overline.color",
        (false, false) => "md.comp.list.list-item.disabled.overline.color",
    };
    let mut color =
        MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface-variant");
    color = alpha_mul(color, overline_text_opacity(theme, enabled, selected));
    color
}

pub(crate) fn selected_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        return MaterialTokenResolver::new(theme).color_comp_or_sys(
            "md.comp.list.list-item.selected.container.color",
            "md.sys.color.secondary-container",
        );
    }

    let tokens = MaterialTokenResolver::new(theme);
    let bg = tokens.color_comp_or_sys(
        "md.comp.list.list-item.selected.disabled.container.color",
        "md.sys.color.on-surface",
    );
    let opacity = tokens.number_comp_or_sys(
        "md.comp.list.list-item.selected.disabled.container.opacity",
        "md.sys.state.disabled.state-layer-opacity",
        0.38,
    );
    alpha_mul(bg, opacity)
}

pub(crate) fn item_outcomes(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: ListItemInteraction,
) -> (Color, Color, Color, f32) {
    let defaults = MaterialContentDefaults::on_surface(theme);

    let (label_key, icon_key, state_layer_key, opacity_key) = match (selected, interaction) {
        (true, ListItemInteraction::Pressed) => (
            "md.comp.list.list-item.selected.pressed.label-text.color",
            "md.comp.list.list-item.selected.pressed.leading-icon.color",
            "md.comp.list.list-item.selected.pressed.state-layer.color",
            "md.comp.list.list-item.selected.pressed.state-layer.opacity",
        ),
        (true, ListItemInteraction::Focused) => (
            "md.comp.list.list-item.selected.focus.label-text.color",
            "md.comp.list.list-item.selected.leading-icon.color",
            "md.comp.list.list-item.selected.focus.state-layer.color",
            "md.comp.list.list-item.selected.focus.state-layer.opacity",
        ),
        (true, ListItemInteraction::Hovered) => (
            "md.comp.list.list-item.selected.hover.label-text.color",
            "md.comp.list.list-item.selected.leading-icon.color",
            "md.comp.list.list-item.selected.hover.state-layer.color",
            "md.comp.list.list-item.selected.hover.state-layer.opacity",
        ),
        (true, ListItemInteraction::Default) => (
            "md.comp.list.list-item.selected.label-text.color",
            "md.comp.list.list-item.selected.leading-icon.color",
            "md.comp.list.list-item.selected.hover.state-layer.color",
            "md.comp.list.list-item.selected.hover.state-layer.opacity",
        ),
        (false, ListItemInteraction::Pressed) => (
            "md.comp.list.list-item.pressed.label-text.color",
            "md.comp.list.list-item.pressed.leading-icon.icon.color",
            "md.comp.list.list-item.pressed.state-layer.color",
            "md.comp.list.list-item.pressed.state-layer.opacity",
        ),
        (false, ListItemInteraction::Focused) => (
            "md.comp.list.list-item.focus.label-text.color",
            "md.comp.list.list-item.leading-icon.color",
            "md.comp.list.list-item.focus.state-layer.color",
            "md.comp.list.list-item.focus.state-layer.opacity",
        ),
        (false, ListItemInteraction::Hovered) => (
            "md.comp.list.list-item.hover.label-text.color",
            "md.comp.list.list-item.leading-icon.color",
            "md.comp.list.list-item.hover.state-layer.color",
            "md.comp.list.list-item.hover.state-layer.opacity",
        ),
        (false, ListItemInteraction::Default) => (
            "md.comp.list.list-item.label-text.color",
            "md.comp.list.list-item.leading-icon.color",
            "md.comp.list.list-item.hover.state-layer.color",
            "md.comp.list.list-item.hover.state-layer.opacity",
        ),
    };

    let tokens = MaterialTokenResolver::new(theme);
    let mut label = tokens.color_comp_or_fallback(label_key, defaults.content_color);
    let mut icon = tokens.color_comp_or_sys(icon_key, "md.sys.color.on-surface-variant");
    let state_layer = tokens.color_comp_or_fallback(state_layer_key, defaults.content_color);
    let mut opacity = list_state_layer_interaction(interaction)
        .map(|interaction| tokens.state_layer_opacity(opacity_key, interaction))
        .unwrap_or(0.0);

    if !enabled {
        let (
            disabled_label_key,
            disabled_label_opacity_key,
            disabled_icon_key,
            disabled_icon_opacity_key,
        ) = if selected {
            (
                "md.comp.list.list-item.selected.disabled.label-text.color",
                "md.comp.list.list-item.selected.disabled.label-text.opacity",
                "md.comp.list.list-item.selected.disabled.leading-icon.color",
                "md.comp.list.list-item.selected.disabled.leading-icon.opacity",
            )
        } else {
            (
                "md.comp.list.list-item.disabled.label-text.color",
                "md.comp.list.list-item.disabled.label-text.opacity",
                "md.comp.list.list-item.disabled.leading-icon.color",
                "md.comp.list.list-item.disabled.leading-icon.opacity",
            )
        };

        label = tokens.color_comp_or_fallback(disabled_label_key, defaults.content_color);
        icon = tokens.color_comp_or_sys(disabled_icon_key, "md.sys.color.on-surface-variant");

        let label_opacity =
            tokens.number_optional(Some(disabled_label_opacity_key), defaults.disabled_opacity);
        let icon_opacity =
            tokens.number_optional(Some(disabled_icon_opacity_key), defaults.disabled_opacity);
        label = alpha_mul(label, label_opacity);
        icon = alpha_mul(icon, icon_opacity);
        opacity = 0.0;
    }

    (label, icon, state_layer, opacity)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, selected: bool) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        if selected {
            "md.comp.list.list-item.selected.pressed.state-layer.opacity"
        } else {
            "md.comp.list.list-item.pressed.state-layer.opacity"
        },
        MaterialStateLayerInteraction::Pressed,
    )
}

fn list_state_layer_interaction(
    interaction: ListItemInteraction,
) -> Option<MaterialStateLayerInteraction> {
    match interaction {
        ListItemInteraction::Hovered => Some(MaterialStateLayerInteraction::Hovered),
        ListItemInteraction::Focused => Some(MaterialStateLayerInteraction::Focused),
        ListItemInteraction::Pressed => Some(MaterialStateLayerInteraction::Pressed),
        ListItemInteraction::Default => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_ui::Theme;

    use crate::tokens::v30::{TypographyOptions, theme_config};

    #[test]
    fn expressive_list_shapes_vary_by_interaction() {
        let cfg = theme_config(TypographyOptions::default());
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&cfg);
        });
        let theme = Theme::global(&app);

        let default = item_container_shape_for_interaction(
            theme,
            false,
            true,
            ListItemInteraction::Default,
            true,
        );
        assert_eq!(default, Corners::all(Px(4.0)));

        let hovered = item_container_shape_for_interaction(
            theme,
            false,
            true,
            ListItemInteraction::Hovered,
            true,
        );
        assert_eq!(hovered, Corners::all(Px(12.0)));

        let pressed = item_container_shape_for_interaction(
            theme,
            false,
            true,
            ListItemInteraction::Pressed,
            true,
        );
        assert_eq!(pressed, Corners::all(Px(16.0)));

        let selected_default = item_container_shape_for_interaction(
            theme,
            true,
            true,
            ListItemInteraction::Default,
            true,
        );
        assert_eq!(selected_default, Corners::all(Px(16.0)));

        let disabled = item_container_shape_for_interaction(
            theme,
            false,
            false,
            ListItemInteraction::Default,
            true,
        );
        assert_eq!(disabled, Corners::all(Px(4.0)));
    }
}

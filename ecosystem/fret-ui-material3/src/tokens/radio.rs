//! Typed token access for Material 3 radio buttons.
//!
//! This module centralizes token key mapping and fallback chains so radio visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RadioInteraction {
    None,
    Hovered,
    Focused,
    Pressed,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RadioSizeTokens {
    pub(crate) icon: Px,
    pub(crate) state_layer: Px,
}

fn radio_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

pub(crate) fn size_tokens(theme: &Theme) -> RadioSizeTokens {
    let icon = radio_metric(theme, "md.comp.radio-button.icon.size", Px(20.0));
    let state_layer = radio_metric(theme, "md.comp.radio-button.state-layer.size", Px(40.0));
    RadioSizeTokens { icon, state_layer }
}

pub(crate) fn state_layer_shape(theme: &Theme) -> Corners {
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[
            "md.comp.radio-button.state-layer.shape",
            "md.sys.shape.corner.full",
        ],
        Corners::all(Px(9999.0)),
    )
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    checked: bool,
    enabled: bool,
    interaction: RadioInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let Some(material_interaction) = material_state_layer_interaction(interaction) else {
        return 0.0;
    };

    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(checked, interaction),
        material_interaction,
    )
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, checked: bool) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(checked, RadioInteraction::Pressed),
        MaterialStateLayerInteraction::Pressed,
    )
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    checked: bool,
    interaction: RadioInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        state_layer_color_key(checked, interaction),
        "md.sys.color.primary",
    )
}

pub(crate) fn icon_color(
    theme: &Theme,
    checked: bool,
    enabled: bool,
    interaction: RadioInteraction,
) -> Color {
    if !enabled {
        let (color_key, opacity_key) = if checked {
            (
                "md.comp.radio-button.disabled.selected.icon.color",
                "md.comp.radio-button.disabled.selected.icon.opacity",
            )
        } else {
            (
                "md.comp.radio-button.disabled.unselected.icon.color",
                "md.comp.radio-button.disabled.unselected.icon.opacity",
            )
        };

        let tokens = MaterialTokenResolver::new(theme);
        let base = tokens.color_comp_or_sys(color_key, "md.sys.color.on-surface");
        let opacity = tokens.number_optional(Some(opacity_key), 0.38);
        return alpha_mul(base, opacity);
    }

    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(icon_color_key(checked, interaction), "md.sys.color.primary")
}

fn material_state_layer_interaction(
    interaction: RadioInteraction,
) -> Option<MaterialStateLayerInteraction> {
    match interaction {
        RadioInteraction::Pressed => Some(MaterialStateLayerInteraction::Pressed),
        RadioInteraction::Focused => Some(MaterialStateLayerInteraction::Focused),
        RadioInteraction::Hovered => Some(MaterialStateLayerInteraction::Hovered),
        RadioInteraction::None => None,
    }
}

fn state_layer_opacity_key(checked: bool, interaction: RadioInteraction) -> &'static str {
    match (checked, interaction) {
        (true, RadioInteraction::Pressed) => {
            "md.comp.radio-button.selected.pressed.state-layer.opacity"
        }
        (true, RadioInteraction::Focused) => {
            "md.comp.radio-button.selected.focus.state-layer.opacity"
        }
        (true, RadioInteraction::Hovered) => {
            "md.comp.radio-button.selected.hover.state-layer.opacity"
        }
        (false, RadioInteraction::Pressed) => {
            "md.comp.radio-button.unselected.pressed.state-layer.opacity"
        }
        (false, RadioInteraction::Focused) => {
            "md.comp.radio-button.unselected.focus.state-layer.opacity"
        }
        (false, RadioInteraction::Hovered) => {
            "md.comp.radio-button.unselected.hover.state-layer.opacity"
        }
        (_, RadioInteraction::None) => "md.comp.radio-button.unselected.hover.state-layer.opacity",
    }
}

fn state_layer_color_key(checked: bool, interaction: RadioInteraction) -> &'static str {
    match (checked, interaction) {
        (true, RadioInteraction::Pressed) => {
            "md.comp.radio-button.selected.pressed.state-layer.color"
        }
        (true, RadioInteraction::Focused) => {
            "md.comp.radio-button.selected.focus.state-layer.color"
        }
        (true, RadioInteraction::Hovered) => {
            "md.comp.radio-button.selected.hover.state-layer.color"
        }
        (true, RadioInteraction::None) => "md.comp.radio-button.selected.hover.state-layer.color",
        (false, RadioInteraction::Pressed) => {
            "md.comp.radio-button.unselected.pressed.state-layer.color"
        }
        (false, RadioInteraction::Focused) => {
            "md.comp.radio-button.unselected.focus.state-layer.color"
        }
        (false, RadioInteraction::Hovered) => {
            "md.comp.radio-button.unselected.hover.state-layer.color"
        }
        (false, RadioInteraction::None) => {
            "md.comp.radio-button.unselected.hover.state-layer.color"
        }
    }
}

fn icon_color_key(checked: bool, interaction: RadioInteraction) -> &'static str {
    match (checked, interaction) {
        (true, RadioInteraction::None) => "md.comp.radio-button.selected.icon.color",
        (true, RadioInteraction::Hovered) => "md.comp.radio-button.selected.hover.icon.color",
        (true, RadioInteraction::Focused) => "md.comp.radio-button.selected.focus.icon.color",
        (true, RadioInteraction::Pressed) => "md.comp.radio-button.selected.pressed.icon.color",
        (false, RadioInteraction::None) => "md.comp.radio-button.unselected.icon.color",
        (false, RadioInteraction::Hovered) => "md.comp.radio-button.unselected.hover.icon.color",
        (false, RadioInteraction::Focused) => "md.comp.radio-button.unselected.focus.icon.color",
        (false, RadioInteraction::Pressed) => "md.comp.radio-button.unselected.pressed.icon.color",
    }
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
    fn radio_size_defaults_match_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);
        let size = size_tokens(theme);

        assert_eq!(size.icon, Px(20.0));
        assert_eq!(size.state_layer, Px(40.0));
    }

    #[test]
    fn radio_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.radio-button.icon.size".to_string(), 22.0);
        patch
            .metrics
            .insert("md.comp.radio-button.state-layer.size".to_string(), 44.0);
        let (_app, theme) = theme_with_patch(patch);
        let size = size_tokens(&theme);

        assert_eq!(size.icon, Px(22.0));
        assert_eq!(size.state_layer, Px(44.0));
    }

    #[test]
    fn radio_state_layer_shape_prefers_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.radio-button.state-layer.shape".to_string(), 40.0);
        patch.corners.insert(
            "md.comp.radio-button.state-layer.shape".to_string(),
            Corners {
                top_left: Px(5.0),
                top_right: Px(6.0),
                bottom_right: Px(7.0),
                bottom_left: Px(8.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            state_layer_shape(&theme),
            Corners {
                top_left: Px(5.0),
                top_right: Px(6.0),
                bottom_right: Px(7.0),
                bottom_left: Px(8.0),
            }
        );
    }
}

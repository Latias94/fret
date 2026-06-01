//! Typed token access for Material 3 primary and secondary navigation tabs.
//!
//! This module centralizes token key mapping and fallback chains so tab visuals remain stable and
//! drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{MaterialStateLayerInteraction, MaterialTokenResolver};
use crate::tokens::typography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NavigationTabKind {
    Primary,
    Secondary,
}

pub(crate) fn component_prefix(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => "md.comp.primary-navigation-tab",
        NavigationTabKind::Secondary => "md.comp.secondary-navigation-tab",
    }
}

fn tab_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn tab_metric_chain(theme: &Theme, keys: &[&'static str], fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(keys, fallback)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TabInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn container_height_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    tab_metric(theme, container_height_key(kind), Px(48.0))
}

pub(crate) fn stacked_container_height_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    tab_metric(theme, stacked_container_height_key(kind), Px(72.0))
}

pub(crate) fn container_background_for(theme: &Theme, kind: NavigationTabKind) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(container_color_key(kind), "md.sys.color.surface")
}

pub(crate) fn active_indicator_height(theme: &Theme) -> Px {
    tab_metric(
        theme,
        "md.comp.primary-navigation-tab.active-indicator.height",
        Px(3.0),
    )
}

pub(crate) fn active_indicator_min_width(theme: &Theme) -> Px {
    tab_metric(
        theme,
        "md.comp.primary-navigation-tab.active-indicator.min-width",
        Px(24.0),
    )
}

pub(crate) fn divider_height_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    tab_metric_chain(
        theme,
        &[divider_height_key(kind), "md.comp.divider.thickness"],
        Px(1.0),
    )
}

pub(crate) fn divider_color_for(theme: &Theme, kind: NavigationTabKind) -> Color {
    MaterialTokenResolver::new(theme).color_comp_chain_or_sys(
        &[divider_color_key(kind), "md.comp.divider.color"],
        "md.sys.color.outline-variant",
    )
}

pub(crate) fn horizontal_text_padding() -> fret_core::Edges {
    fret_core::Edges {
        left: Px(16.0),
        right: Px(16.0),
        top: Px(0.0),
        bottom: Px(0.0),
    }
}

pub(crate) fn leading_icon_label_gap() -> Px {
    Px(8.0)
}

pub(crate) fn stacked_icon_label_gap() -> Px {
    Px(8.0)
}

pub(crate) fn scrollable_edge_padding_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    tab_metric(theme, scrollable_edge_padding_key(kind), Px(52.0))
}

pub(crate) fn scrollable_min_tab_width_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    tab_metric(theme, scrollable_min_tab_width_key(kind), Px(90.0))
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.primary-navigation-tab.active-indicator.color",
        "md.sys.color.primary",
    )
}

pub(crate) fn icon_size_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    tab_metric(theme, icon_size_key(kind), Px(24.0))
}

pub(crate) fn icon_color_for(
    theme: &Theme,
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        icon_color_key(kind, active, interaction),
        content_color_sys_key(kind, active),
    )
}

pub(crate) fn active_indicator_shape_for(theme: &Theme, kind: NavigationTabKind) -> Corners {
    if matches!(kind, NavigationTabKind::Secondary) {
        return Corners::all(Px(0.0));
    }

    MaterialTokenResolver::new(theme)
        .corners_value("md.comp.primary-navigation-tab.active-indicator.shape")
        .unwrap_or(Corners {
            top_left: Px(3.0),
            top_right: Px(3.0),
            bottom_right: Px(0.0),
            bottom_left: Px(0.0),
        })
}

pub(crate) fn indicator_matches_content(kind: NavigationTabKind) -> bool {
    matches!(kind, NavigationTabKind::Primary)
}

pub(crate) fn label_color_for(
    theme: &Theme,
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        label_color_key(kind, active, interaction),
        content_color_sys_key(kind, active),
    )
}

pub(crate) fn label_text_style_for(theme: &Theme, kind: NavigationTabKind) -> TextStyle {
    typography::text_style_with_weight_fallback(
        theme,
        Some(label_text_style_key(kind)),
        "md.sys.typescale.title-small",
        label_text_weight_key(kind),
        500.0,
        TextIntent::Control,
    )
}

pub(crate) fn state_layer_color_for(
    theme: &Theme,
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        state_layer_color_key(kind, active, interaction),
        state_layer_color_sys_key(kind, active),
    )
}

pub(crate) fn state_layer_opacity_for(
    theme: &Theme,
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> f32 {
    match interaction {
        TabInteraction::Default => 0.0,
        TabInteraction::Pressed | TabInteraction::Focused | TabInteraction::Hovered => {
            MaterialTokenResolver::new(theme).state_layer_opacity(
                state_layer_opacity_key(kind, active, interaction),
                material_state_layer_interaction(interaction),
            )
        }
    }
}

pub(crate) fn pressed_state_layer_opacity_for(
    theme: &Theme,
    kind: NavigationTabKind,
    active: bool,
) -> f32 {
    state_layer_opacity_for(theme, kind, active, TabInteraction::Pressed)
}

fn container_height_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => "md.comp.primary-navigation-tab.container.height",
        NavigationTabKind::Secondary => "md.comp.secondary-navigation-tab.container.height",
    }
}

fn content_color_sys_key(kind: NavigationTabKind, active: bool) -> &'static str {
    match (kind, active) {
        (NavigationTabKind::Primary, true) => "md.sys.color.primary",
        (NavigationTabKind::Primary, false) => "md.sys.color.on-surface-variant",
        (NavigationTabKind::Secondary, true) => "md.sys.color.on-surface",
        (NavigationTabKind::Secondary, false) => "md.sys.color.on-surface-variant",
    }
}

fn state_layer_color_sys_key(kind: NavigationTabKind, active: bool) -> &'static str {
    match (kind, active) {
        (NavigationTabKind::Primary, true) => "md.sys.color.primary",
        _ => "md.sys.color.on-surface",
    }
}

fn material_state_layer_interaction(interaction: TabInteraction) -> MaterialStateLayerInteraction {
    match interaction {
        TabInteraction::Hovered => MaterialStateLayerInteraction::Hovered,
        TabInteraction::Focused => MaterialStateLayerInteraction::Focused,
        TabInteraction::Pressed => MaterialStateLayerInteraction::Pressed,
        TabInteraction::Default => MaterialStateLayerInteraction::Hovered,
    }
}

fn divider_height_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => "md.comp.primary-navigation-tab.divider.height",
        NavigationTabKind::Secondary => "md.comp.secondary-navigation-tab.divider.height",
    }
}

fn divider_color_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => "md.comp.primary-navigation-tab.divider.color",
        NavigationTabKind::Secondary => "md.comp.secondary-navigation-tab.divider.color",
    }
}

fn stacked_container_height_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => {
            "md.comp.primary-navigation-tab.with-stacked-icon-and-label-text.container.height"
        }
        NavigationTabKind::Secondary => {
            "md.comp.secondary-navigation-tab.with-stacked-icon-and-label-text.container.height"
        }
    }
}

fn container_color_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => "md.comp.primary-navigation-tab.container.color",
        NavigationTabKind::Secondary => "md.comp.secondary-navigation-tab.container.color",
    }
}

fn scrollable_edge_padding_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => "md.comp.primary-navigation-tab.scrollable.edge-padding",
        NavigationTabKind::Secondary => "md.comp.secondary-navigation-tab.scrollable.edge-padding",
    }
}

fn scrollable_min_tab_width_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => "md.comp.primary-navigation-tab.scrollable.min-tab-width",
        NavigationTabKind::Secondary => "md.comp.secondary-navigation-tab.scrollable.min-tab-width",
    }
}

fn icon_size_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => "md.comp.primary-navigation-tab.with-icon.icon.size",
        NavigationTabKind::Secondary => "md.comp.secondary-navigation-tab.with-icon.icon.size",
    }
}

fn label_text_style_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => "md.comp.primary-navigation-tab.with-label-text.label-text",
        NavigationTabKind::Secondary => {
            "md.comp.secondary-navigation-tab.with-label-text.label-text"
        }
    }
}

fn label_text_weight_key(kind: NavigationTabKind) -> &'static str {
    match kind {
        NavigationTabKind::Primary => {
            "md.comp.primary-navigation-tab.with-label-text.label-text.weight"
        }
        NavigationTabKind::Secondary => {
            "md.comp.secondary-navigation-tab.with-label-text.label-text.weight"
        }
    }
}

fn icon_color_key(
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> &'static str {
    match (kind, active, interaction) {
        (NavigationTabKind::Primary, true, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.with-icon.active.focus.icon.color"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.with-icon.active.hover.icon.color"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.with-icon.active.pressed.icon.color"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.with-icon.active.icon.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.with-icon.inactive.focus.icon.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.with-icon.inactive.hover.icon.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.with-icon.inactive.pressed.icon.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.with-icon.inactive.icon.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Focused) => {
            "md.comp.secondary-navigation-tab.with-icon.active.focus.icon.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Hovered) => {
            "md.comp.secondary-navigation-tab.with-icon.active.hover.icon.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Pressed) => {
            "md.comp.secondary-navigation-tab.with-icon.active.pressed.icon.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Default) => {
            "md.comp.secondary-navigation-tab.with-icon.active.icon.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Focused) => {
            "md.comp.secondary-navigation-tab.with-icon.inactive.focus.icon.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Hovered) => {
            "md.comp.secondary-navigation-tab.with-icon.inactive.hover.icon.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Pressed) => {
            "md.comp.secondary-navigation-tab.with-icon.inactive.pressed.icon.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Default) => {
            "md.comp.secondary-navigation-tab.with-icon.inactive.icon.color"
        }
    }
}

fn label_color_key(
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> &'static str {
    match (kind, active, interaction) {
        (NavigationTabKind::Primary, true, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.with-label-text.active.focus.label-text.color"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.with-label-text.active.hover.label-text.color"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.with-label-text.active.pressed.label-text.color"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.with-label-text.active.label-text.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.with-label-text.inactive.focus.label-text.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.with-label-text.inactive.hover.label-text.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.with-label-text.inactive.pressed.label-text.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.with-label-text.inactive.label-text.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Focused) => {
            "md.comp.secondary-navigation-tab.with-label-text.active.focus.label-text.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Hovered) => {
            "md.comp.secondary-navigation-tab.with-label-text.active.hover.label-text.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Pressed) => {
            "md.comp.secondary-navigation-tab.with-label-text.active.pressed.label-text.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Default) => {
            "md.comp.secondary-navigation-tab.with-label-text.active.label-text.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Focused) => {
            "md.comp.secondary-navigation-tab.with-label-text.inactive.focus.label-text.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Hovered) => {
            "md.comp.secondary-navigation-tab.with-label-text.inactive.hover.label-text.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Pressed) => {
            "md.comp.secondary-navigation-tab.with-label-text.inactive.pressed.label-text.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Default) => {
            "md.comp.secondary-navigation-tab.with-label-text.inactive.label-text.color"
        }
    }
}

fn state_layer_color_key(
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> &'static str {
    match (kind, active, interaction) {
        (NavigationTabKind::Primary, true, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.active.focus.state-layer.color"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.active.hover.state-layer.color"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.active.pressed.state-layer.color"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.active.hover.state-layer.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.inactive.focus.state-layer.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.inactive.hover.state-layer.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.inactive.pressed.state-layer.color"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.inactive.hover.state-layer.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Focused) => {
            "md.comp.secondary-navigation-tab.active.focus.state-layer.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Hovered) => {
            "md.comp.secondary-navigation-tab.active.hover.state-layer.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Pressed) => {
            "md.comp.secondary-navigation-tab.active.pressed.state-layer.color"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Default) => {
            "md.comp.secondary-navigation-tab.active.hover.state-layer.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Focused) => {
            "md.comp.secondary-navigation-tab.inactive.focus.state-layer.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Hovered) => {
            "md.comp.secondary-navigation-tab.inactive.hover.state-layer.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Pressed) => {
            "md.comp.secondary-navigation-tab.inactive.pressed.state-layer.color"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Default) => {
            "md.comp.secondary-navigation-tab.inactive.hover.state-layer.color"
        }
    }
}

fn state_layer_opacity_key(
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> &'static str {
    match (kind, active, interaction) {
        (NavigationTabKind::Primary, true, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.active.pressed.state-layer.opacity"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.active.focus.state-layer.opacity"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.active.hover.state-layer.opacity"
        }
        (NavigationTabKind::Primary, true, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.active.hover.state-layer.opacity"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.inactive.pressed.state-layer.opacity"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.inactive.focus.state-layer.opacity"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.inactive.hover.state-layer.opacity"
        }
        (NavigationTabKind::Primary, false, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.inactive.hover.state-layer.opacity"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Pressed) => {
            "md.comp.secondary-navigation-tab.active.pressed.state-layer.opacity"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Focused) => {
            "md.comp.secondary-navigation-tab.active.focus.state-layer.opacity"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Hovered) => {
            "md.comp.secondary-navigation-tab.active.hover.state-layer.opacity"
        }
        (NavigationTabKind::Secondary, true, TabInteraction::Default) => {
            "md.comp.secondary-navigation-tab.active.hover.state-layer.opacity"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Pressed) => {
            "md.comp.secondary-navigation-tab.inactive.pressed.state-layer.opacity"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Focused) => {
            "md.comp.secondary-navigation-tab.inactive.focus.state-layer.opacity"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Hovered) => {
            "md.comp.secondary-navigation-tab.inactive.hover.state-layer.opacity"
        }
        (NavigationTabKind::Secondary, false, TabInteraction::Default) => {
            "md.comp.secondary-navigation-tab.inactive.hover.state-layer.opacity"
        }
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
    fn tab_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);
        let kind = NavigationTabKind::Primary;

        assert_eq!(container_height_for(theme, kind), Px(48.0));
        assert_eq!(stacked_container_height_for(theme, kind), Px(72.0));
        assert_eq!(active_indicator_height(theme), Px(3.0));
        assert_eq!(active_indicator_min_width(theme), Px(24.0));
        assert_eq!(divider_height_for(theme, kind), Px(1.0));
        assert_eq!(scrollable_edge_padding_for(theme, kind), Px(52.0));
        assert_eq!(scrollable_min_tab_width_for(theme, kind), Px(90.0));
        assert_eq!(icon_size_for(theme, kind), Px(24.0));
    }

    #[test]
    fn tab_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.primary-navigation-tab.container.height".to_string(),
            50.0,
        );
        patch.metrics.insert(
            "md.comp.primary-navigation-tab.with-stacked-icon-and-label-text.container.height"
                .to_string(),
            74.0,
        );
        patch.metrics.insert(
            "md.comp.primary-navigation-tab.active-indicator.height".to_string(),
            4.0,
        );
        patch.metrics.insert(
            "md.comp.primary-navigation-tab.active-indicator.min-width".to_string(),
            28.0,
        );
        patch.metrics.insert(
            "md.comp.primary-navigation-tab.divider.height".to_string(),
            2.0,
        );
        patch.metrics.insert(
            "md.comp.primary-navigation-tab.scrollable.edge-padding".to_string(),
            56.0,
        );
        patch.metrics.insert(
            "md.comp.primary-navigation-tab.scrollable.min-tab-width".to_string(),
            96.0,
        );
        patch.metrics.insert(
            "md.comp.primary-navigation-tab.with-icon.icon.size".to_string(),
            26.0,
        );
        let (_app, theme) = theme_with_patch(patch);
        let kind = NavigationTabKind::Primary;

        assert_eq!(container_height_for(&theme, kind), Px(50.0));
        assert_eq!(stacked_container_height_for(&theme, kind), Px(74.0));
        assert_eq!(active_indicator_height(&theme), Px(4.0));
        assert_eq!(active_indicator_min_width(&theme), Px(28.0));
        assert_eq!(divider_height_for(&theme, kind), Px(2.0));
        assert_eq!(scrollable_edge_padding_for(&theme, kind), Px(56.0));
        assert_eq!(scrollable_min_tab_width_for(&theme, kind), Px(96.0));
        assert_eq!(icon_size_for(&theme, kind), Px(26.0));
    }

    #[test]
    fn tab_divider_height_uses_divider_fallback() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.divider.thickness".to_string(), 1.5);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            divider_height_for(&theme, NavigationTabKind::Secondary),
            Px(1.5)
        );
    }

    #[test]
    fn primary_tab_active_indicator_shape_prefers_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.primary-navigation-tab.active-indicator.shape".to_string(),
            6.0,
        );
        patch.corners.insert(
            "md.comp.primary-navigation-tab.active-indicator.shape".to_string(),
            Corners {
                top_left: Px(1.0),
                top_right: Px(2.0),
                bottom_right: Px(3.0),
                bottom_left: Px(4.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            active_indicator_shape_for(&theme, NavigationTabKind::Primary),
            Corners {
                top_left: Px(1.0),
                top_right: Px(2.0),
                bottom_right: Px(3.0),
                bottom_left: Px(4.0),
            }
        );
        assert_eq!(
            active_indicator_shape_for(&theme, NavigationTabKind::Secondary),
            Corners::all(Px(0.0))
        );
    }
}

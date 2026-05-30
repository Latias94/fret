//! Typed token access for Material 3 primary and secondary navigation tabs.
//!
//! This module centralizes token key mapping and fallback chains so tab visuals remain stable and
//! drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TabInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn container_height_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    theme
        .metric_by_key(container_height_key(kind))
        .unwrap_or(Px(48.0))
}

pub(crate) fn stacked_container_height_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    theme
        .metric_by_key(stacked_container_height_key(kind))
        .unwrap_or(Px(72.0))
}

pub(crate) fn container_background_for(theme: &Theme, kind: NavigationTabKind) -> Color {
    theme
        .color_by_key(container_color_key(kind))
        .or_else(|| theme.color_by_key("md.sys.color.surface"))
        .unwrap_or_else(|| theme.color_token("md.sys.color.surface"))
}

pub(crate) fn active_indicator_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.primary-navigation-tab.active-indicator.height")
        .unwrap_or(Px(3.0))
}

pub(crate) fn active_indicator_min_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.primary-navigation-tab.active-indicator.min-width")
        .unwrap_or(Px(24.0))
}

pub(crate) fn divider_height_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    theme
        .metric_by_key(divider_height_key(kind))
        .or_else(|| theme.metric_by_key("md.comp.divider.thickness"))
        .unwrap_or(Px(1.0))
}

pub(crate) fn divider_color_for(theme: &Theme, kind: NavigationTabKind) -> Color {
    theme
        .color_by_key(divider_color_key(kind))
        .or_else(|| theme.color_by_key("md.comp.divider.color"))
        .or_else(|| theme.color_by_key("md.sys.color.outline-variant"))
        .unwrap_or_else(|| theme.color_token("md.sys.color.outline-variant"))
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
    theme
        .metric_by_key(scrollable_edge_padding_key(kind))
        .unwrap_or(Px(52.0))
}

pub(crate) fn scrollable_min_tab_width_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    theme
        .metric_by_key(scrollable_min_tab_width_key(kind))
        .unwrap_or(Px(90.0))
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.primary-navigation-tab.active-indicator.color")
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_token("md.sys.color.primary"))
}

pub(crate) fn icon_size_for(theme: &Theme, kind: NavigationTabKind) -> Px {
    theme.metric_by_key(icon_size_key(kind)).unwrap_or(Px(24.0))
}

pub(crate) fn icon_color_for(
    theme: &Theme,
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> Color {
    theme
        .color_by_key(icon_color_key(kind, active, interaction))
        .or_else(|| match (kind, active) {
            (NavigationTabKind::Primary, true) => theme.color_by_key("md.sys.color.primary"),
            (NavigationTabKind::Primary, false) => {
                theme.color_by_key("md.sys.color.on-surface-variant")
            }
            (NavigationTabKind::Secondary, true) => theme.color_by_key("md.sys.color.on-surface"),
            (NavigationTabKind::Secondary, false) => {
                theme.color_by_key("md.sys.color.on-surface-variant")
            }
        })
        .unwrap_or_else(|| match (kind, active) {
            (NavigationTabKind::Primary, true) => theme.color_token("md.sys.color.primary"),
            (NavigationTabKind::Primary, false) => {
                theme.color_token("md.sys.color.on-surface-variant")
            }
            (NavigationTabKind::Secondary, true) => theme.color_token("md.sys.color.on-surface"),
            (NavigationTabKind::Secondary, false) => {
                theme.color_token("md.sys.color.on-surface-variant")
            }
        })
}

pub(crate) fn active_indicator_shape_for(theme: &Theme, kind: NavigationTabKind) -> Corners {
    if matches!(kind, NavigationTabKind::Secondary) {
        return Corners::all(Px(0.0));
    }

    theme
        .corners_by_key("md.comp.primary-navigation-tab.active-indicator.shape")
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
    theme
        .color_by_key(label_color_key(kind, active, interaction))
        .or_else(|| match (kind, active) {
            (NavigationTabKind::Primary, true) => theme.color_by_key("md.sys.color.primary"),
            (NavigationTabKind::Primary, false) => {
                theme.color_by_key("md.sys.color.on-surface-variant")
            }
            (NavigationTabKind::Secondary, true) => theme.color_by_key("md.sys.color.on-surface"),
            (NavigationTabKind::Secondary, false) => {
                theme.color_by_key("md.sys.color.on-surface-variant")
            }
        })
        .unwrap_or_else(|| match (kind, active) {
            (NavigationTabKind::Primary, true) => theme.color_token("md.sys.color.primary"),
            (NavigationTabKind::Primary, false) => {
                theme.color_token("md.sys.color.on-surface-variant")
            }
            (NavigationTabKind::Secondary, true) => theme.color_token("md.sys.color.on-surface"),
            (NavigationTabKind::Secondary, false) => {
                theme.color_token("md.sys.color.on-surface-variant")
            }
        })
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
    theme
        .color_by_key(state_layer_color_key(kind, active, interaction))
        .or_else(|| match kind {
            NavigationTabKind::Primary if active => theme.color_by_key("md.sys.color.primary"),
            _ => theme.color_by_key("md.sys.color.on-surface"),
        })
        .unwrap_or_else(|| match kind {
            NavigationTabKind::Primary if active => theme.color_token("md.sys.color.primary"),
            _ => theme.color_token("md.sys.color.on-surface"),
        })
}

pub(crate) fn state_layer_opacity_for(
    theme: &Theme,
    kind: NavigationTabKind,
    active: bool,
    interaction: TabInteraction,
) -> f32 {
    match interaction {
        TabInteraction::Default => 0.0,
        TabInteraction::Pressed => theme
            .number_by_key(state_layer_opacity_key(
                kind,
                active,
                TabInteraction::Pressed,
            ))
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        TabInteraction::Focused => theme
            .number_by_key(state_layer_opacity_key(
                kind,
                active,
                TabInteraction::Focused,
            ))
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        TabInteraction::Hovered => theme
            .number_by_key(state_layer_opacity_key(
                kind,
                active,
                TabInteraction::Hovered,
            ))
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
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

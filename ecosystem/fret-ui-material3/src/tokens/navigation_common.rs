//! Shared token fallback helpers for Material 3 navigation surfaces.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{MaterialStateLayerInteraction, MaterialTokenResolver};
use crate::navigation_drawer::NavigationDrawerVariant;
use crate::tokens::{shape, typography};

const DEFAULT_BAR_CONTAINER_HEIGHT: Px = Px(80.0);
const DEFAULT_BAR_CONTAINER_ELEVATION: Px = Px(0.0);
const DEFAULT_BAR_ACTIVE_INDICATOR_WIDTH: Px = Px(64.0);
const DEFAULT_BAR_ACTIVE_INDICATOR_HEIGHT: Px = Px(32.0);
const DEFAULT_BAR_ACTIVE_INDICATOR_TOP_OFFSET: Px = Px(12.0);
const DEFAULT_BAR_ACTIVE_INDICATOR_RADIUS: Px = Px(9999.0);
const DEFAULT_BAR_ICON_SIZE: Px = Px(24.0);
const DEFAULT_BAR_ITEM_GAP: Px = Px(8.0);
const DEFAULT_BAR_CONTAINER_RADIUS: Px = Px(0.0);

const DEFAULT_RAIL_CONTAINER_WIDTH: Px = Px(80.0);
const DEFAULT_RAIL_ITEM_HEIGHT: Px = Px(56.0);
const DEFAULT_RAIL_VERTICAL_PADDING: Px = Px(4.0);
const DEFAULT_RAIL_ACTIVE_INDICATOR_WIDTH: Px = Px(56.0);
const DEFAULT_RAIL_ACTIVE_INDICATOR_HEIGHT: Px = Px(32.0);
const DEFAULT_RAIL_NO_LABEL_ACTIVE_INDICATOR_HEIGHT: Px = Px(56.0);
const DEFAULT_RAIL_ACTIVE_INDICATOR_RADIUS: Px = Px(9999.0);
const DEFAULT_RAIL_ICON_SIZE: Px = Px(24.0);
const DEFAULT_RAIL_CONTAINER_RADIUS: Px = Px(0.0);

const DEFAULT_DRAWER_CONTAINER_WIDTH: Px = Px(360.0);
const DEFAULT_DRAWER_ACTIVE_INDICATOR_WIDTH: Px = Px(336.0);
const DEFAULT_DRAWER_CONTAINER_RADIUS: Px = Px(0.0);
const DEFAULT_DRAWER_ACTIVE_INDICATOR_HEIGHT: Px = Px(56.0);
const DEFAULT_DRAWER_ACTIVE_INDICATOR_RADIUS: Px = Px(9999.0);
const DEFAULT_DRAWER_ICON_SIZE: Px = Px(24.0);
const DEFAULT_DRAWER_SCRIM_OPACITY: f32 = 0.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NavigationItemInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

fn nav_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn nav_metric_chain(theme: &Theme, keys: &[&'static str], fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(keys, fallback)
}

pub(crate) fn bar_container_height(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-bar.container.height",
        DEFAULT_BAR_CONTAINER_HEIGHT,
    )
}

pub(crate) fn bar_container_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-bar.container.color",
        "md.sys.color.surface-container",
    )
}

pub(crate) fn bar_container_elevation(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-bar.container.elevation",
        DEFAULT_BAR_CONTAINER_ELEVATION,
    )
}

pub(crate) fn bar_container_shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-bar.container.shadow-color",
        "md.sys.color.shadow",
    )
}

pub(crate) fn bar_container_shape(theme: &Theme) -> Corners {
    shape::corners_or_metric(theme, "md.comp.navigation-bar.container.shape")
        .or_else(|| shape::corners_or_metric(theme, "md.sys.shape.corner.none"))
        .unwrap_or(Corners::all(DEFAULT_BAR_CONTAINER_RADIUS))
}

pub(crate) fn bar_active_indicator_width(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-bar.active-indicator.width",
        DEFAULT_BAR_ACTIVE_INDICATOR_WIDTH,
    )
}

pub(crate) fn bar_active_indicator_height(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-bar.active-indicator.height",
        DEFAULT_BAR_ACTIVE_INDICATOR_HEIGHT,
    )
}

pub(crate) fn bar_active_indicator_top_offset(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-bar.active-indicator.top-offset",
        DEFAULT_BAR_ACTIVE_INDICATOR_TOP_OFFSET,
    )
}

pub(crate) fn bar_active_indicator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-bar.active-indicator.color",
        "md.sys.color.secondary-container",
    )
}

pub(crate) fn bar_active_indicator_radius(theme: &Theme) -> Px {
    nav_metric_chain(
        theme,
        &[
            "md.comp.navigation-bar.active-indicator.shape",
            "md.sys.shape.corner.full",
        ],
        DEFAULT_BAR_ACTIVE_INDICATOR_RADIUS,
    )
}

pub(crate) fn bar_active_indicator_shape(theme: &Theme) -> Corners {
    Corners::all(bar_active_indicator_radius(theme))
}

pub(crate) fn bar_state_layer_opacity(
    theme: &Theme,
    interaction: NavigationItemInteraction,
) -> f32 {
    let Some((key, interaction)) = state_layer_opacity_token("md.comp.navigation-bar", interaction)
    else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme).state_layer_opacity(&key, interaction)
}

pub(crate) fn bar_state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationItemInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }
    bar_state_layer_opacity(theme, interaction)
}

pub(crate) fn bar_state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &navigation_state_layer_color_key(
            "md.comp.navigation-bar",
            active,
            interaction,
            NavigationDefaultState::InactiveHover,
        ),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn bar_icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &navigation_content_color_key("md.comp.navigation-bar", active, interaction, "icon"),
        if active {
            "md.sys.color.on-secondary-container"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn bar_label_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &navigation_content_color_key("md.comp.navigation-bar", active, interaction, "label-text"),
        if active {
            "md.sys.color.on-surface"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn bar_label_text_style(theme: &Theme, active: bool) -> TextStyle {
    let weight_key = if active {
        "md.comp.navigation-bar.active.label-text.weight"
    } else {
        "md.comp.navigation-bar.label-text.weight"
    };
    typography::text_style_with_weight_fallback(
        theme,
        None,
        "md.sys.typescale.label-medium",
        weight_key,
        if active { 700.0 } else { 500.0 },
        TextIntent::Control,
    )
}

pub(crate) fn bar_icon_size(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-bar.icon.size",
        DEFAULT_BAR_ICON_SIZE,
    )
}

pub(crate) fn bar_item_gap(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-bar.item.gap",
        DEFAULT_BAR_ITEM_GAP,
    )
}

pub(crate) fn rail_container_width(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-rail.container.width",
        DEFAULT_RAIL_CONTAINER_WIDTH,
    )
}

pub(crate) fn rail_item_width(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-rail.item.width",
        rail_container_width(theme),
    )
}

pub(crate) fn rail_item_height(theme: &Theme) -> Px {
    nav_metric_chain(
        theme,
        &[
            "md.comp.navigation-rail.item.height",
            "md.comp.navigation-rail.active-indicator.width",
        ],
        DEFAULT_RAIL_ITEM_HEIGHT,
    )
}

pub(crate) fn rail_vertical_padding(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-rail.vertical-padding",
        DEFAULT_RAIL_VERTICAL_PADDING,
    )
}

pub(crate) fn rail_container_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-rail.container.color",
        "md.sys.color.surface",
    )
}

pub(crate) fn rail_container_shape(theme: &Theme) -> Corners {
    shape::corners_or_metric(theme, "md.comp.navigation-rail.container.shape")
        .or_else(|| shape::corners_or_metric(theme, "md.sys.shape.corner.none"))
        .unwrap_or(Corners::all(DEFAULT_RAIL_CONTAINER_RADIUS))
}

pub(crate) fn rail_active_indicator_width(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-rail.active-indicator.width",
        DEFAULT_RAIL_ACTIVE_INDICATOR_WIDTH,
    )
}

pub(crate) fn rail_active_indicator_height(theme: &Theme, has_label: bool) -> Px {
    if has_label {
        nav_metric(
            theme,
            "md.comp.navigation-rail.active-indicator.height",
            DEFAULT_RAIL_ACTIVE_INDICATOR_HEIGHT,
        )
    } else {
        nav_metric(
            theme,
            "md.comp.navigation-rail.no-label.active-indicator.height",
            DEFAULT_RAIL_NO_LABEL_ACTIVE_INDICATOR_HEIGHT,
        )
    }
}

pub(crate) fn rail_active_indicator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-rail.active-indicator.color",
        "md.sys.color.secondary-container",
    )
}

pub(crate) fn rail_active_indicator_radius(theme: &Theme) -> Px {
    nav_metric_chain(
        theme,
        &[
            "md.comp.navigation-rail.active-indicator.shape",
            "md.sys.shape.corner.full",
        ],
        DEFAULT_RAIL_ACTIVE_INDICATOR_RADIUS,
    )
}

pub(crate) fn rail_active_indicator_shape(theme: &Theme) -> Corners {
    Corners::all(rail_active_indicator_radius(theme))
}

pub(crate) fn rail_state_layer_opacity(
    theme: &Theme,
    interaction: NavigationItemInteraction,
) -> f32 {
    let Some((key, interaction)) =
        state_layer_opacity_token("md.comp.navigation-rail", interaction)
    else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme).state_layer_opacity(&key, interaction)
}

pub(crate) fn rail_state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationItemInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }
    rail_state_layer_opacity(theme, interaction)
}

pub(crate) fn rail_state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &navigation_state_layer_color_key(
            "md.comp.navigation-rail",
            active,
            interaction,
            NavigationDefaultState::InactiveHover,
        ),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn rail_icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &navigation_content_color_key("md.comp.navigation-rail", active, interaction, "icon"),
        if active {
            "md.sys.color.on-secondary-container"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn rail_label_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &navigation_content_color_key("md.comp.navigation-rail", active, interaction, "label-text"),
        if active {
            "md.sys.color.on-surface"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn rail_label_text_style(theme: &Theme, active: bool) -> TextStyle {
    let weight_key = if active {
        "md.comp.navigation-rail.active.label-text.weight"
    } else {
        "md.comp.navigation-rail.label-text.weight"
    };
    typography::text_style_with_weight_fallback(
        theme,
        None,
        "md.sys.typescale.label-medium",
        weight_key,
        if active { 700.0 } else { 500.0 },
        TextIntent::Control,
    )
}

pub(crate) fn rail_icon_size(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-rail.icon.size",
        DEFAULT_RAIL_ICON_SIZE,
    )
}

pub(crate) fn drawer_container_width(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-drawer.container.width",
        DEFAULT_DRAWER_CONTAINER_WIDTH,
    )
}

pub(crate) fn drawer_active_indicator_width(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-drawer.active-indicator.width",
        DEFAULT_DRAWER_ACTIVE_INDICATOR_WIDTH,
    )
}

pub(crate) fn drawer_item_horizontal_padding(theme: &Theme) -> Px {
    let container_w = drawer_container_width(theme);
    let active_w = drawer_active_indicator_width(theme);
    Px(((container_w.0 - active_w.0) / 2.0).max(0.0))
}

pub(crate) fn drawer_container_shape(theme: &Theme) -> Corners {
    shape::corners_or_metric(theme, "md.comp.navigation-drawer.container.shape")
        .or_else(|| shape::corners_or_metric(theme, "md.sys.shape.corner.extra-large"))
        .unwrap_or_else(|| Corners::all(DEFAULT_DRAWER_CONTAINER_RADIUS))
}

pub(crate) fn drawer_container_background(
    theme: &Theme,
    variant: NavigationDrawerVariant,
) -> Color {
    let (key, fallback) = match variant {
        NavigationDrawerVariant::Standard => (
            "md.comp.navigation-drawer.standard.container.color",
            "md.sys.color.surface",
        ),
        NavigationDrawerVariant::Modal => (
            "md.comp.navigation-drawer.modal.container.color",
            "md.sys.color.surface-container-low",
        ),
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(key, fallback)
}

pub(crate) fn drawer_container_elevation(theme: &Theme, variant: NavigationDrawerVariant) -> Px {
    match variant {
        NavigationDrawerVariant::Standard => nav_metric(
            theme,
            "md.comp.navigation-drawer.standard.container.elevation",
            Px(0.0),
        ),
        NavigationDrawerVariant::Modal => nav_metric(
            theme,
            "md.comp.navigation-drawer.modal.container.elevation",
            Px(1.0),
        ),
    }
}

pub(crate) fn drawer_active_indicator_height(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-drawer.active-indicator.height",
        DEFAULT_DRAWER_ACTIVE_INDICATOR_HEIGHT,
    )
}

pub(crate) fn drawer_active_indicator_radius(theme: &Theme) -> Px {
    nav_metric_chain(
        theme,
        &[
            "md.comp.navigation-drawer.active-indicator.shape",
            "md.sys.shape.corner.full",
        ],
        DEFAULT_DRAWER_ACTIVE_INDICATOR_RADIUS,
    )
}

pub(crate) fn drawer_active_indicator_shape(theme: &Theme) -> Corners {
    Corners::all(drawer_active_indicator_radius(theme))
}

pub(crate) fn drawer_active_indicator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-drawer.active-indicator.color",
        "md.sys.color.secondary-container",
    )
}

pub(crate) fn drawer_scrim_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-drawer.scrim.color",
        "md.sys.color.scrim",
    )
}

pub(crate) fn drawer_scrim_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_optional(
        Some("md.comp.navigation-drawer.scrim.opacity"),
        DEFAULT_DRAWER_SCRIM_OPACITY,
    )
}

pub(crate) fn drawer_pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        "md.comp.navigation-drawer.pressed.state-layer.opacity",
        MaterialStateLayerInteraction::Pressed,
    )
}

pub(crate) fn drawer_state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationItemInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let Some((key, interaction)) =
        state_layer_opacity_token("md.comp.navigation-drawer", interaction)
    else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme).state_layer_opacity(&key, interaction)
}

pub(crate) fn drawer_state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &navigation_state_layer_color_key(
            "md.comp.navigation-drawer",
            active,
            interaction,
            NavigationDefaultState::ActiveFocusWhenActive,
        ),
        if active {
            "md.sys.color.on-secondary-container"
        } else {
            "md.sys.color.on-surface"
        },
    )
}

pub(crate) fn drawer_label_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &navigation_content_color_key(
            "md.comp.navigation-drawer",
            active,
            interaction,
            "label-text",
        ),
        if active {
            "md.sys.color.on-secondary-container"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn drawer_icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &navigation_content_color_key("md.comp.navigation-drawer", active, interaction, "icon"),
        if active {
            "md.sys.color.on-secondary-container"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn drawer_label_text_style(theme: &Theme, active: bool) -> TextStyle {
    let weight_key = if active {
        "md.comp.navigation-drawer.active.label-text.weight"
    } else {
        "md.comp.navigation-drawer.label-text.weight"
    };
    typography::text_style_with_weight_fallback(
        theme,
        None,
        "md.sys.typescale.label-large",
        weight_key,
        if active { 700.0 } else { 500.0 },
        TextIntent::Control,
    )
}

pub(crate) fn drawer_large_badge_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight_fallback(
        theme,
        None,
        "md.sys.typescale.label-small",
        "md.comp.navigation-drawer.large-badge-label.weight",
        500.0,
        TextIntent::Control,
    )
}

pub(crate) fn drawer_large_badge_label_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-drawer.large-badge-label.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn drawer_icon_size(theme: &Theme) -> Px {
    nav_metric(
        theme,
        "md.comp.navigation-drawer.icon.size",
        DEFAULT_DRAWER_ICON_SIZE,
    )
}

fn state_layer_opacity_token(
    prefix: &'static str,
    interaction: NavigationItemInteraction,
) -> Option<(String, MaterialStateLayerInteraction)> {
    match interaction {
        NavigationItemInteraction::Default => None,
        NavigationItemInteraction::Pressed => Some((
            format!("{prefix}.pressed.state-layer.opacity"),
            MaterialStateLayerInteraction::Pressed,
        )),
        NavigationItemInteraction::Focused => Some((
            format!("{prefix}.focus.state-layer.opacity"),
            MaterialStateLayerInteraction::Focused,
        )),
        NavigationItemInteraction::Hovered => Some((
            format!("{prefix}.hover.state-layer.opacity"),
            MaterialStateLayerInteraction::Hovered,
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavigationDefaultState {
    InactiveHover,
    ActiveFocusWhenActive,
}

fn navigation_content_color_key(
    prefix: &'static str,
    active: bool,
    interaction: NavigationItemInteraction,
    role_suffix: &'static str,
) -> String {
    let active_segment = if active { "active" } else { "inactive" };
    match interaction {
        NavigationItemInteraction::Default => {
            format!("{prefix}.{active_segment}.{role_suffix}.color")
        }
        NavigationItemInteraction::Focused => {
            format!("{prefix}.{active_segment}.focus.{role_suffix}.color")
        }
        NavigationItemInteraction::Hovered => {
            format!("{prefix}.{active_segment}.hover.{role_suffix}.color")
        }
        NavigationItemInteraction::Pressed => {
            format!("{prefix}.{active_segment}.pressed.{role_suffix}.color")
        }
    }
}

fn navigation_state_layer_color_key(
    prefix: &'static str,
    active: bool,
    interaction: NavigationItemInteraction,
    default_state: NavigationDefaultState,
) -> String {
    let state = match interaction {
        NavigationItemInteraction::Focused => "focus",
        NavigationItemInteraction::Hovered => "hover",
        NavigationItemInteraction::Pressed => "pressed",
        NavigationItemInteraction::Default => match default_state {
            NavigationDefaultState::InactiveHover => "hover",
            NavigationDefaultState::ActiveFocusWhenActive => {
                if active {
                    "focus"
                } else {
                    "hover"
                }
            }
        },
    };

    if active {
        format!("{prefix}.active.{state}.state-layer.color")
    } else {
        format!("{prefix}.inactive.{state}.state-layer.color")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn empty_theme() -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config_patch(&ThemeConfig::default());
        });
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn navigation_bar_defaults_match_material_matrix() {
        let (_app, theme) = empty_theme();
        assert_eq!(bar_container_height(&theme), Px(80.0));
        assert_eq!(bar_container_elevation(&theme), Px(0.0));
        assert_eq!(bar_active_indicator_width(&theme), Px(64.0));
        assert_eq!(bar_active_indicator_height(&theme), Px(32.0));
        assert_eq!(bar_active_indicator_top_offset(&theme), Px(12.0));
        assert_eq!(bar_icon_size(&theme), Px(24.0));
        assert_eq!(bar_item_gap(&theme), Px(8.0));
        assert_eq!(bar_container_shape(&theme), Corners::all(Px(0.0)));
        assert_eq!(bar_active_indicator_shape(&theme), Corners::all(Px(9999.0)));
    }

    #[test]
    fn navigation_rail_defaults_match_material_matrix() {
        let (_app, theme) = empty_theme();
        assert_eq!(rail_container_width(&theme), Px(80.0));
        assert_eq!(rail_item_width(&theme), Px(80.0));
        assert_eq!(rail_item_height(&theme), Px(56.0));
        assert_eq!(rail_vertical_padding(&theme), Px(4.0));
        assert_eq!(rail_active_indicator_width(&theme), Px(56.0));
        assert_eq!(rail_active_indicator_height(&theme, true), Px(32.0));
        assert_eq!(rail_active_indicator_height(&theme, false), Px(56.0));
        assert_eq!(rail_icon_size(&theme), Px(24.0));
        assert_eq!(rail_container_shape(&theme), Corners::all(Px(0.0)));
        assert_eq!(
            rail_active_indicator_shape(&theme),
            Corners::all(Px(9999.0))
        );
    }

    #[test]
    fn navigation_drawer_defaults_match_material_matrix() {
        let (_app, theme) = empty_theme();
        assert_eq!(drawer_container_width(&theme), Px(360.0));
        assert_eq!(drawer_active_indicator_width(&theme), Px(336.0));
        assert_eq!(drawer_item_horizontal_padding(&theme), Px(12.0));
        assert_eq!(drawer_active_indicator_height(&theme), Px(56.0));
        assert_eq!(drawer_icon_size(&theme), Px(24.0));
        assert_eq!(drawer_container_shape(&theme), Corners::all(Px(0.0)));
        assert_eq!(
            drawer_active_indicator_shape(&theme),
            Corners::all(Px(9999.0))
        );
        assert_eq!(drawer_scrim_opacity(&theme), 0.4);
    }

    #[test]
    fn navigation_metric_chains_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.shape.corner.full".to_string(), 44.0);
        patch.metrics.insert(
            "md.comp.navigation-rail.active-indicator.width".to_string(),
            72.0,
        );
        patch.metrics.insert(
            "md.comp.navigation-drawer.modal.container.elevation".to_string(),
            7.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(bar_active_indicator_radius(&theme), Px(44.0));
        assert_eq!(rail_item_height(&theme), Px(72.0));
        assert_eq!(
            drawer_container_elevation(&theme, NavigationDrawerVariant::Modal),
            Px(7.0)
        );
    }

    #[test]
    fn navigation_state_layer_opacity_prefers_component_tokens() {
        let mut patch = ThemeConfig::default();
        patch.numbers.insert(
            "md.comp.navigation-bar.pressed.state-layer.opacity".to_string(),
            0.31,
        );
        patch.numbers.insert(
            "md.comp.navigation-rail.hover.state-layer.opacity".to_string(),
            0.17,
        );
        patch.numbers.insert(
            "md.comp.navigation-drawer.focus.state-layer.opacity".to_string(),
            0.22,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            bar_state_layer_target_opacity(&theme, true, NavigationItemInteraction::Pressed),
            0.31
        );
        assert_eq!(
            rail_state_layer_target_opacity(&theme, true, NavigationItemInteraction::Hovered),
            0.17
        );
        assert_eq!(
            drawer_state_layer_target_opacity(&theme, true, NavigationItemInteraction::Focused),
            0.22
        );
    }

    #[test]
    fn navigation_state_matrices_use_family_color_defaults() {
        let mut patch = ThemeConfig::default();
        patch.colors.insert(
            "md.comp.navigation-bar.active.focus.icon.color".to_string(),
            "#112233".to_string(),
        );
        patch.colors.insert(
            "md.comp.navigation-bar.active.icon.color".to_string(),
            "#223344".to_string(),
        );
        patch.colors.insert(
            "md.comp.navigation-rail.inactive.hover.label-text.color".to_string(),
            "#334455".to_string(),
        );
        patch.colors.insert(
            "md.comp.navigation-drawer.active.focus.state-layer.color".to_string(),
            "#556677".to_string(),
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            bar_icon_color(&theme, true, NavigationItemInteraction::Focused),
            theme
                .color_by_key("md.comp.navigation-bar.active.focus.icon.color")
                .expect("patched navigation bar icon color")
        );
        assert_eq!(
            bar_icon_color(&theme, true, NavigationItemInteraction::Default),
            theme
                .color_by_key("md.comp.navigation-bar.active.icon.color")
                .expect("patched navigation bar default icon color")
        );
        assert_eq!(
            rail_label_color(&theme, false, NavigationItemInteraction::Hovered),
            theme
                .color_by_key("md.comp.navigation-rail.inactive.hover.label-text.color")
                .expect("patched navigation rail label color")
        );
        assert_eq!(
            drawer_state_layer_color(&theme, true, NavigationItemInteraction::Focused),
            theme
                .color_by_key("md.comp.navigation-drawer.active.focus.state-layer.color")
                .expect("patched navigation drawer state-layer color")
        );
    }
}

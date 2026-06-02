//! Typed token access for Material 3 switches.
//!
//! This module centralizes token key mapping and fallback chains so switch visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SwitchInteraction {
    None,
    Hovered,
    Focused,
    Pressed,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SwitchChrome {
    pub(crate) track_color: Color,
    pub(crate) outline_color: Option<Color>,
    pub(crate) handle_color: Color,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SwitchSizeTokens {
    pub(crate) state_layer: Px,
    pub(crate) track_width: Px,
    pub(crate) track_height: Px,
    pub(crate) track_outline_width: Px,
    pub(crate) selected_handle_width: Px,
    pub(crate) selected_handle_height: Px,
    pub(crate) unselected_handle_width: Px,
    pub(crate) unselected_handle_height: Px,
    pub(crate) pressed_handle_width: Px,
    pub(crate) pressed_handle_height: Px,
    pub(crate) with_icon_handle_width: Px,
    pub(crate) with_icon_handle_height: Px,
    pub(crate) track_y_offset: Px,
}

fn switch_metric(theme: &Theme, key: &str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

pub(crate) fn size_tokens(theme: &Theme) -> SwitchSizeTokens {
    let state_layer = switch_metric(theme, "md.comp.switch.state-layer.size", Px(40.0));
    let track_width = switch_metric(theme, "md.comp.switch.track.width", Px(52.0));
    let track_height = switch_metric(theme, "md.comp.switch.track.height", Px(32.0));
    let track_outline_width = switch_metric(theme, "md.comp.switch.track.outline.width", Px(2.0));

    let selected_handle_width =
        switch_metric(theme, "md.comp.switch.selected.handle.width", Px(24.0));
    let selected_handle_height =
        switch_metric(theme, "md.comp.switch.selected.handle.height", Px(24.0));
    let unselected_handle_width =
        switch_metric(theme, "md.comp.switch.unselected.handle.width", Px(16.0));
    let unselected_handle_height =
        switch_metric(theme, "md.comp.switch.unselected.handle.height", Px(16.0));
    let pressed_handle_width =
        switch_metric(theme, "md.comp.switch.pressed.handle.width", Px(28.0));
    let pressed_handle_height =
        switch_metric(theme, "md.comp.switch.pressed.handle.height", Px(28.0));

    let with_icon_handle_width = switch_metric(
        theme,
        "md.comp.switch.with-icon.handle.width",
        selected_handle_width,
    );
    let with_icon_handle_height = switch_metric(
        theme,
        "md.comp.switch.with-icon.handle.height",
        selected_handle_height,
    );

    let track_y_offset = Px(((state_layer.0 - track_height.0) * 0.5).max(0.0));

    SwitchSizeTokens {
        state_layer,
        track_width,
        track_height,
        track_outline_width,
        selected_handle_width,
        selected_handle_height,
        unselected_handle_width,
        unselected_handle_height,
        pressed_handle_width,
        pressed_handle_height,
        with_icon_handle_width,
        with_icon_handle_height,
        track_y_offset,
    }
}

pub(crate) fn icon_size(theme: &Theme, selected: bool) -> Px {
    let key = if selected {
        "md.comp.switch.selected.icon.size"
    } else {
        "md.comp.switch.unselected.icon.size"
    };
    switch_metric(theme, key, Px(16.0))
}

pub(crate) fn icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: SwitchInteraction,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    if !enabled {
        let base =
            tokens.color_comp_or_sys(disabled_icon_color_key(selected), "md.sys.color.on-surface");
        let opacity = tokens.number_optional(Some(disabled_icon_opacity_key(selected)), 0.38);
        return alpha_mul(base, opacity);
    }

    let sys_key = if selected {
        "md.sys.color.on-primary"
    } else {
        "md.sys.color.on-surface-variant"
    };
    tokens.color_comp_or_sys(icon_color_key(selected, interaction), sys_key)
}

fn shape_or_full(theme: &Theme, key: &str) -> Corners {
    MaterialTokenResolver::new(theme)
        .corners_chain(&[key, "md.sys.shape.corner.full"])
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}

pub(crate) fn track_shape(theme: &Theme) -> Corners {
    shape_or_full(theme, "md.comp.switch.track.shape")
}

pub(crate) fn handle_shape(theme: &Theme) -> Corners {
    shape_or_full(theme, "md.comp.switch.handle.shape")
}

pub(crate) fn state_layer_shape(theme: &Theme) -> Corners {
    shape_or_full(theme, "md.comp.switch.state-layer.shape")
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: SwitchInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let Some(material_interaction) = material_state_layer_interaction(interaction) else {
        return 0.0;
    };

    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(selected, interaction),
        material_interaction,
    )
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, selected: bool) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(selected, SwitchInteraction::Pressed),
        MaterialStateLayerInteraction::Pressed,
    )
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: SwitchInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        state_layer_color_key(selected, interaction),
        "md.sys.color.primary",
    )
}

pub(crate) fn chrome(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: SwitchInteraction,
) -> SwitchChrome {
    if !enabled {
        return disabled_chrome(theme, selected);
    }

    let track_key = track_color_key(selected, interaction);
    let handle_key = handle_color_key(selected, interaction);

    let track_sys_key = if selected {
        "md.sys.color.primary"
    } else {
        "md.sys.color.surface-container-highest"
    };
    let track_color = MaterialTokenResolver::new(theme).color_comp_or_sys(track_key, track_sys_key);

    let handle_sys_key = if selected {
        "md.sys.color.on-primary"
    } else {
        "md.sys.color.outline"
    };
    let handle_color =
        MaterialTokenResolver::new(theme).color_comp_or_sys(handle_key, handle_sys_key);

    let outline_color = if selected {
        None
    } else {
        Some(
            MaterialTokenResolver::new(theme)
                .color_comp_or_sys(track_outline_color_key(interaction), "md.sys.color.outline"),
        )
    };

    SwitchChrome {
        track_color,
        outline_color,
        handle_color,
    }
}

fn disabled_chrome(theme: &Theme, selected: bool) -> SwitchChrome {
    let tokens = MaterialTokenResolver::new(theme);
    let track_base = if selected {
        tokens.color_comp_or_sys(
            "md.comp.switch.disabled.selected.track.color",
            "md.sys.color.on-surface",
        )
    } else {
        tokens.color_comp_or_sys(
            "md.comp.switch.disabled.unselected.track.color",
            "md.sys.color.on-surface",
        )
    };

    let track_opacity = tokens.number_optional(Some("md.comp.switch.disabled.track.opacity"), 0.12);
    let track_color = alpha_mul(track_base, track_opacity);

    let handle_base = if selected {
        tokens.color_comp_or_sys(
            "md.comp.switch.disabled.selected.handle.color",
            "md.sys.color.surface",
        )
    } else {
        tokens.color_comp_or_sys(
            "md.comp.switch.disabled.unselected.handle.color",
            "md.sys.color.on-surface",
        )
    };

    let handle_opacity = if selected {
        tokens.number_optional(
            Some("md.comp.switch.disabled.selected.handle.opacity"),
            0.38,
        )
    } else {
        tokens.number_optional(
            Some("md.comp.switch.disabled.unselected.handle.opacity"),
            0.38,
        )
    };
    let handle_color = alpha_mul(handle_base, handle_opacity);

    let outline_color = if selected {
        None
    } else {
        Some(alpha_mul(
            tokens.color_comp_or_sys(
                "md.comp.switch.disabled.unselected.track.outline.color",
                "md.sys.color.on-surface",
            ),
            handle_opacity,
        ))
    };

    SwitchChrome {
        track_color,
        outline_color,
        handle_color,
    }
}

fn material_state_layer_interaction(
    interaction: SwitchInteraction,
) -> Option<MaterialStateLayerInteraction> {
    match interaction {
        SwitchInteraction::Pressed => Some(MaterialStateLayerInteraction::Pressed),
        SwitchInteraction::Focused => Some(MaterialStateLayerInteraction::Focused),
        SwitchInteraction::Hovered => Some(MaterialStateLayerInteraction::Hovered),
        SwitchInteraction::None => None,
    }
}

fn icon_color_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.icon.color",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.icon.color",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.icon.color",
        (true, SwitchInteraction::None) => "md.comp.switch.selected.icon.color",
        (false, SwitchInteraction::Pressed) => "md.comp.switch.unselected.pressed.icon.color",
        (false, SwitchInteraction::Focused) => "md.comp.switch.unselected.focus.icon.color",
        (false, SwitchInteraction::Hovered) => "md.comp.switch.unselected.hover.icon.color",
        (false, SwitchInteraction::None) => "md.comp.switch.unselected.icon.color",
    }
}

fn disabled_icon_color_key(selected: bool) -> &'static str {
    if selected {
        "md.comp.switch.disabled.selected.icon.color"
    } else {
        "md.comp.switch.disabled.unselected.icon.color"
    }
}

fn disabled_icon_opacity_key(selected: bool) -> &'static str {
    if selected {
        "md.comp.switch.disabled.selected.icon.opacity"
    } else {
        "md.comp.switch.disabled.unselected.icon.opacity"
    }
}

fn state_layer_opacity_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.state-layer.opacity",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.state-layer.opacity",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.state-layer.opacity",
        (false, SwitchInteraction::Pressed) => {
            "md.comp.switch.unselected.pressed.state-layer.opacity"
        }
        (false, SwitchInteraction::Focused) => {
            "md.comp.switch.unselected.focus.state-layer.opacity"
        }
        (false, SwitchInteraction::Hovered) => {
            "md.comp.switch.unselected.hover.state-layer.opacity"
        }
        (_, SwitchInteraction::None) => "md.comp.switch.unselected.hover.state-layer.opacity",
    }
}

fn state_layer_color_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.state-layer.color",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.state-layer.color",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.state-layer.color",
        (true, SwitchInteraction::None) => "md.comp.switch.selected.hover.state-layer.color",
        (false, SwitchInteraction::Pressed) => {
            "md.comp.switch.unselected.pressed.state-layer.color"
        }
        (false, SwitchInteraction::Focused) => "md.comp.switch.unselected.focus.state-layer.color",
        (false, SwitchInteraction::Hovered) => "md.comp.switch.unselected.hover.state-layer.color",
        (false, SwitchInteraction::None) => "md.comp.switch.unselected.hover.state-layer.color",
    }
}

fn track_color_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::None) => "md.comp.switch.selected.track.color",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.track.color",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.track.color",
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.track.color",
        (false, SwitchInteraction::None) => "md.comp.switch.unselected.track.color",
        (false, SwitchInteraction::Hovered) => "md.comp.switch.unselected.hover.track.color",
        (false, SwitchInteraction::Focused) => "md.comp.switch.unselected.focus.track.color",
        (false, SwitchInteraction::Pressed) => "md.comp.switch.unselected.pressed.track.color",
    }
}

fn handle_color_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::None) => "md.comp.switch.selected.handle.color",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.handle.color",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.handle.color",
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.handle.color",
        (false, SwitchInteraction::None) => "md.comp.switch.unselected.handle.color",
        (false, SwitchInteraction::Hovered) => "md.comp.switch.unselected.hover.handle.color",
        (false, SwitchInteraction::Focused) => "md.comp.switch.unselected.focus.handle.color",
        (false, SwitchInteraction::Pressed) => "md.comp.switch.unselected.pressed.handle.color",
    }
}

fn track_outline_color_key(interaction: SwitchInteraction) -> &'static str {
    match interaction {
        SwitchInteraction::None => "md.comp.switch.unselected.track.outline.color",
        SwitchInteraction::Hovered => "md.comp.switch.unselected.hover.track.outline.color",
        SwitchInteraction::Focused => "md.comp.switch.unselected.focus.track.outline.color",
        SwitchInteraction::Pressed => "md.comp.switch.unselected.pressed.track.outline.color",
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
    fn switch_size_defaults_match_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);
        let size = size_tokens(theme);

        assert_eq!(size.state_layer, Px(40.0));
        assert_eq!(size.track_width, Px(52.0));
        assert_eq!(size.track_height, Px(32.0));
        assert_eq!(size.track_outline_width, Px(2.0));
        assert_eq!(size.selected_handle_width, Px(24.0));
        assert_eq!(size.selected_handle_height, Px(24.0));
        assert_eq!(size.unselected_handle_width, Px(16.0));
        assert_eq!(size.unselected_handle_height, Px(16.0));
        assert_eq!(size.pressed_handle_width, Px(28.0));
        assert_eq!(size.pressed_handle_height, Px(28.0));
        assert_eq!(size.with_icon_handle_width, Px(24.0));
        assert_eq!(size.with_icon_handle_height, Px(24.0));
        assert_eq!(size.track_y_offset, Px(4.0));
    }

    #[test]
    fn switch_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.switch.state-layer.size".to_string(), 44.0);
        patch
            .metrics
            .insert("md.comp.switch.track.width".to_string(), 60.0);
        patch
            .metrics
            .insert("md.comp.switch.track.height".to_string(), 34.0);
        patch
            .metrics
            .insert("md.comp.switch.selected.handle.height".to_string(), 26.0);
        patch
            .metrics
            .insert("md.comp.switch.with-icon.handle.width".to_string(), 30.0);
        patch
            .metrics
            .insert("md.comp.switch.selected.icon.size".to_string(), 18.0);
        let (_app, theme) = theme_with_patch(patch);
        let size = size_tokens(&theme);

        assert_eq!(size.state_layer, Px(44.0));
        assert_eq!(size.track_width, Px(60.0));
        assert_eq!(size.track_height, Px(34.0));
        assert_eq!(size.track_y_offset, Px(5.0));
        assert_eq!(size.with_icon_handle_width, Px(30.0));
        assert_eq!(size.with_icon_handle_height, Px(26.0));
        assert_eq!(icon_size(&theme, true), Px(18.0));
    }
}

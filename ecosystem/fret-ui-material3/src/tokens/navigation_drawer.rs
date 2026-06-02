//! Typed token access for Material 3 navigation drawers.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::{Theme, theme::CubicBezier};

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::navigation_drawer::NavigationDrawerVariant;
use crate::tokens::navigation_common;

pub(crate) use navigation_common::NavigationItemInteraction as NavigationDrawerItemInteraction;

const MODAL_MOTION_DURATION_KEY: &str = "md.sys.motion.duration.medium2";
const MODAL_MOTION_EASING_KEY: &str = "md.sys.motion.easing.emphasized";

pub(crate) fn container_width(theme: &Theme) -> Px {
    navigation_common::drawer_container_width(theme)
}

#[allow(dead_code)]
pub(crate) fn active_indicator_width(theme: &Theme) -> Px {
    navigation_common::drawer_active_indicator_width(theme)
}

pub(crate) fn item_horizontal_padding(theme: &Theme) -> Px {
    navigation_common::drawer_item_horizontal_padding(theme)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    navigation_common::drawer_container_shape(theme)
}

pub(crate) fn container_background(theme: &Theme, variant: NavigationDrawerVariant) -> Color {
    navigation_common::drawer_container_background(theme, variant)
}

pub(crate) fn container_elevation(theme: &Theme, variant: NavigationDrawerVariant) -> Px {
    navigation_common::drawer_container_elevation(theme, variant)
}

pub(crate) fn active_indicator_height(theme: &Theme) -> Px {
    navigation_common::drawer_active_indicator_height(theme)
}

pub(crate) fn active_indicator_shape(theme: &Theme) -> Corners {
    navigation_common::drawer_active_indicator_shape(theme)
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    navigation_common::drawer_active_indicator_color(theme)
}

pub(crate) fn scrim_color(theme: &Theme) -> Color {
    navigation_common::drawer_scrim_color(theme)
}

pub(crate) fn scrim_opacity(theme: &Theme) -> f32 {
    navigation_common::drawer_scrim_opacity(theme)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    navigation_common::drawer_pressed_state_layer_opacity(theme)
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationDrawerItemInteraction,
) -> f32 {
    navigation_common::drawer_state_layer_target_opacity(theme, enabled, interaction)
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> Color {
    navigation_common::drawer_state_layer_color(theme, active, interaction)
}

pub(crate) fn label_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> Color {
    navigation_common::drawer_label_color(theme, active, interaction)
}

pub(crate) fn icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> Color {
    navigation_common::drawer_icon_color(theme, active, interaction)
}

pub(crate) fn label_text_style(theme: &Theme, active: bool) -> TextStyle {
    navigation_common::drawer_label_text_style(theme, active)
}

pub(crate) fn large_badge_label_text_style(theme: &Theme) -> TextStyle {
    navigation_common::drawer_large_badge_label_text_style(theme)
}

pub(crate) fn large_badge_label_color(theme: &Theme) -> Color {
    navigation_common::drawer_large_badge_label_color(theme)
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    navigation_common::drawer_icon_size(theme)
}

pub(crate) fn modal_open_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys(MODAL_MOTION_DURATION_KEY, 300)
}

pub(crate) fn modal_close_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys(MODAL_MOTION_DURATION_KEY, 300)
}

pub(crate) fn modal_easing(theme: &Theme, easing_key: Option<&str>) -> CubicBezier {
    MaterialTokenResolver::new(theme)
        .easing_optional_or_linear(Some(easing_key.unwrap_or(MODAL_MOTION_EASING_KEY)))
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
    fn modal_navigation_drawer_motion_defaults_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(modal_open_duration_ms(theme), 300);
        assert_eq!(modal_close_duration_ms(theme), 300);
        assert_eq!(
            modal_easing(theme, None),
            theme.easing_required(MODAL_MOTION_EASING_KEY)
        );
    }

    #[test]
    fn modal_navigation_drawer_motion_prefers_theme_overrides() {
        let mut patch = ThemeConfig::default();
        patch
            .durations_ms
            .insert(MODAL_MOTION_DURATION_KEY.to_string(), 180);
        patch.easings.insert(
            "md.sys.motion.easing.test-modal-drawer".to_string(),
            CubicBezier {
                x1: 0.1,
                y1: 0.2,
                x2: 0.3,
                y2: 0.4,
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(modal_open_duration_ms(&theme), 180);
        assert_eq!(modal_close_duration_ms(&theme), 180);
        assert_eq!(
            modal_easing(&theme, Some("md.sys.motion.easing.test-modal-drawer")),
            CubicBezier {
                x1: 0.1,
                y1: 0.2,
                x2: 0.3,
                y2: 0.4,
            }
        );
    }
}

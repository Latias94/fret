//! Shared Material selectable menu item token outcomes.
//!
//! Select, Autocomplete, and ExposedDropdown all render Material selectable menu rows. Component
//! token modules keep their variant-specific normal colors, while this module owns the shared
//! selected/disabled content outcomes and density constants.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::list as list_tokens;

const OUTER_HORIZONTAL_PADDING: &str =
    "md.sys.fret.material.selectable-menu-item.outer-horizontal-padding";
const OUTER_VERTICAL_PADDING: &str =
    "md.sys.fret.material.selectable-menu-item.outer-vertical-padding";
const WITH_SECONDARY_OUTER_VERTICAL_PADDING: &str =
    "md.sys.fret.material.selectable-menu-item.with-secondary.outer-vertical-padding";
const CONTENT_HORIZONTAL_PADDING: &str =
    "md.sys.fret.material.selectable-menu-item.content-horizontal-padding";
const ICON_TEXT_GAP: &str = "md.sys.fret.material.selectable-menu-item.icon-text-gap";

pub(crate) fn outer_horizontal_padding(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(OUTER_HORIZONTAL_PADDING), Px(4.0))
}

pub(crate) fn outer_vertical_padding(theme: &Theme, has_secondary_text: bool) -> Px {
    let key = if has_secondary_text {
        WITH_SECONDARY_OUTER_VERTICAL_PADDING
    } else {
        OUTER_VERTICAL_PADDING
    };
    MaterialTokenResolver::new(theme)
        .metric_optional(Some(key), Px(if has_secondary_text { 2.0 } else { 0.0 }))
}

pub(crate) fn content_horizontal_padding(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(CONTENT_HORIZONTAL_PADDING), Px(12.0))
}

pub(crate) fn icon_text_gap(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(ICON_TEXT_GAP), Px(8.0))
}

pub(crate) fn container_shape(theme: &Theme, selected: bool) -> Corners {
    let tokens = MaterialTokenResolver::new(theme);
    if selected {
        tokens.corners_chain_or(
            &[
                "md.comp.menu.list-item.selected.container.shape",
                "md.sys.shape.corner.medium",
            ],
            Corners::all(Px(12.0)),
        )
    } else {
        tokens.corners_chain_or(
            &[
                "md.comp.menu.list-item.container.shape",
                "md.sys.shape.corner.extra-small",
            ],
            Corners::all(Px(4.0)),
        )
    }
}

pub(crate) fn selected_or_disabled_label_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
) -> Option<Color> {
    (selected || !enabled).then(|| {
        let (label, _, _, _) = list_tokens::item_outcomes(
            theme,
            selected,
            enabled,
            list_tokens::ListItemInteraction::Default,
        );
        label
    })
}

pub(crate) fn selected_or_disabled_icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
) -> Option<Color> {
    (selected || !enabled).then(|| {
        let (_, icon, _, _) = list_tokens::item_outcomes(
            theme,
            selected,
            enabled,
            list_tokens::ListItemInteraction::Default,
        );
        icon
    })
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
    fn selectable_menu_item_layout_defaults_to_material_policy() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(outer_horizontal_padding(theme), Px(4.0));
        assert_eq!(outer_vertical_padding(theme, false), Px(0.0));
        assert_eq!(outer_vertical_padding(theme, true), Px(2.0));
        assert_eq!(content_horizontal_padding(theme), Px(12.0));
        assert_eq!(icon_text_gap(theme), Px(8.0));
    }

    #[test]
    fn selectable_menu_item_layout_prefers_policy_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert(OUTER_HORIZONTAL_PADDING.to_string(), 6.0);
        patch
            .metrics
            .insert(OUTER_VERTICAL_PADDING.to_string(), 1.0);
        patch
            .metrics
            .insert(WITH_SECONDARY_OUTER_VERTICAL_PADDING.to_string(), 3.0);
        patch
            .metrics
            .insert(CONTENT_HORIZONTAL_PADDING.to_string(), 14.0);
        patch.metrics.insert(ICON_TEXT_GAP.to_string(), 10.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(outer_horizontal_padding(&theme), Px(6.0));
        assert_eq!(outer_vertical_padding(&theme, false), Px(1.0));
        assert_eq!(outer_vertical_padding(&theme, true), Px(3.0));
        assert_eq!(content_horizontal_padding(&theme), Px(14.0));
        assert_eq!(icon_text_gap(&theme), Px(10.0));
    }
}

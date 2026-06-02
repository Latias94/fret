//! Shared token fallback helpers for Material 3 field-anchored menus.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::{
    field_common::FieldVariant, selectable_menu_item as selectable_item_tokens,
    typography as material_typography,
};

const DEFAULT_CONTAINER_ELEVATION: Px = Px(3.0);
const DEFAULT_CONTAINER_SHAPE: Px = Px(4.0);
const DEFAULT_LIST_ITEM_HEIGHT: Px = Px(48.0);
const DEFAULT_LIST_ITEM_ICON_SIZE: Px = Px(24.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FieldMenuTokenSet {
    outlined: &'static str,
    filled: &'static str,
}

impl FieldMenuTokenSet {
    pub(crate) const fn new(outlined: &'static str, filled: &'static str) -> Self {
        Self { outlined, filled }
    }

    fn prefix(self, variant: FieldVariant) -> &'static str {
        match variant {
            FieldVariant::Outlined => self.outlined,
            FieldVariant::Filled => self.filled,
        }
    }

    fn key(self, variant: FieldVariant, suffix: &str) -> String {
        format!("{}.{}", self.prefix(variant), suffix)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuItemIconRole {
    Leading,
    Trailing,
}

impl MenuItemIconRole {
    fn size_suffix(self) -> &'static str {
        match self {
            Self::Leading => "list-item.with-leading-icon.leading-icon.size",
            Self::Trailing => "list-item.with-trailing-icon.trailing-icon.size",
        }
    }

    fn color_suffix(self) -> &'static str {
        match self {
            Self::Leading => "list-item.with-leading-icon.leading-icon.color",
            Self::Trailing => "list-item.with-trailing-icon.trailing-icon.color",
        }
    }
}

fn field_menu_metric(theme: &Theme, key: impl AsRef<str>, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key.as_ref()), fallback)
}

pub(crate) fn container_background(
    theme: &Theme,
    tokens: FieldMenuTokenSet,
    variant: FieldVariant,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &tokens.key(variant, "container.color"),
        "md.sys.color.surface-container",
    )
}

pub(crate) fn container_elevation(
    theme: &Theme,
    tokens: FieldMenuTokenSet,
    variant: FieldVariant,
) -> Px {
    field_menu_metric(
        theme,
        tokens.key(variant, "container.elevation"),
        DEFAULT_CONTAINER_ELEVATION,
    )
}

pub(crate) fn container_shadow_color(
    theme: &Theme,
    tokens: FieldMenuTokenSet,
    variant: FieldVariant,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &tokens.key(variant, "container.shadow-color"),
        "md.sys.color.shadow",
    )
}

pub(crate) fn container_shape(
    theme: &Theme,
    tokens: FieldMenuTokenSet,
    variant: FieldVariant,
) -> Corners {
    let key = tokens.key(variant, "container.shape");
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[key.as_str(), "md.sys.shape.corner.extra-small"],
        Corners::all(DEFAULT_CONTAINER_SHAPE),
    )
}

pub(crate) fn list_item_height(
    theme: &Theme,
    tokens: FieldMenuTokenSet,
    variant: FieldVariant,
) -> Px {
    field_menu_metric(
        theme,
        tokens.key(variant, "list-item.container.height"),
        DEFAULT_LIST_ITEM_HEIGHT,
    )
}

pub(crate) fn list_item_label_text_style(theme: &Theme) -> Option<TextStyle> {
    material_typography::text_style_value(
        theme,
        "md.sys.typescale.label-large",
        TextIntent::Control,
    )
}

pub(crate) fn list_item_label_text_color(
    theme: &Theme,
    tokens: FieldMenuTokenSet,
    variant: FieldVariant,
    enabled: bool,
    selected: bool,
) -> Color {
    if let Some(label) =
        selectable_item_tokens::selected_or_disabled_label_color(theme, selected, enabled)
    {
        return label;
    }

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &tokens.key(variant, "list-item.label-text.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn list_item_icon_size(
    theme: &Theme,
    tokens: FieldMenuTokenSet,
    variant: FieldVariant,
    role: MenuItemIconRole,
) -> Px {
    field_menu_metric(
        theme,
        tokens.key(variant, role.size_suffix()),
        DEFAULT_LIST_ITEM_ICON_SIZE,
    )
}

pub(crate) fn list_item_icon_color(
    theme: &Theme,
    tokens: FieldMenuTokenSet,
    variant: FieldVariant,
    role: MenuItemIconRole,
    enabled: bool,
    selected: bool,
) -> Color {
    if let Some(icon) =
        selectable_item_tokens::selected_or_disabled_icon_color(theme, selected, enabled)
    {
        return icon;
    }

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &tokens.key(variant, role.color_suffix()),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn list_item_selected_container_color(
    theme: &Theme,
    tokens: FieldMenuTokenSet,
    variant: FieldVariant,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &tokens.key(variant, "list-item.selected.container.color"),
        "md.sys.color.surface-container-highest",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    const TEST_TOKENS: FieldMenuTokenSet = FieldMenuTokenSet::new(
        "md.comp.outlined-test-field.menu",
        "md.comp.filled-test-field.menu",
    );

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn field_menu_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(
            container_elevation(theme, TEST_TOKENS, FieldVariant::Outlined),
            Px(3.0)
        );
        assert_eq!(
            list_item_height(theme, TEST_TOKENS, FieldVariant::Filled),
            Px(48.0)
        );
        assert_eq!(
            list_item_icon_size(
                theme,
                TEST_TOKENS,
                FieldVariant::Outlined,
                MenuItemIconRole::Leading,
            ),
            Px(24.0)
        );
    }

    #[test]
    fn field_menu_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.outlined-test-field.menu.container.elevation".to_string(),
            5.0,
        );
        patch.metrics.insert(
            "md.comp.filled-test-field.menu.list-item.container.height".to_string(),
            52.0,
        );
        patch.metrics.insert(
            "md.comp.outlined-test-field.menu.list-item.with-leading-icon.leading-icon.size"
                .to_string(),
            26.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_elevation(&theme, TEST_TOKENS, FieldVariant::Outlined),
            Px(5.0)
        );
        assert_eq!(
            list_item_height(&theme, TEST_TOKENS, FieldVariant::Filled),
            Px(52.0)
        );
        assert_eq!(
            list_item_icon_size(
                &theme,
                TEST_TOKENS,
                FieldVariant::Outlined,
                MenuItemIconRole::Leading,
            ),
            Px(26.0)
        );
    }
}

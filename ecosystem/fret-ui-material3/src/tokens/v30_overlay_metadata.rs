//! Metadata for curated Material Web v30 overlay tokens.
//!
//! This Module describes why a token may exist even when it is not emitted by the generated
//! Material Web v30 sassvars baseline. Runtime injection lives in `v30_overlay`; maintainer tools
//! use this metadata to keep non-Material-Web keys intentional and reviewable.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaterialOverlayTokenOrigin {
    /// Fret runtime policy that is not owned by Material.
    FretRuntimePolicy,
    /// Fret Material policy or an application-facing override token.
    FretMaterialPolicy,
    /// A concrete Material default that Material Web v30 does not emit as a sassvar key.
    MaterialDefaultBackfill,
    /// A token sourced from Compose Material3 when Material Web v30 has no equivalent key.
    ComposeMaterial3Backfill,
    /// A variant-shaped alias that keeps recipe token access uniform.
    VariantShapeAlias,
}

impl MaterialOverlayTokenOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FretRuntimePolicy => "fret_runtime_policy",
            Self::FretMaterialPolicy => "fret_material_policy",
            Self::MaterialDefaultBackfill => "material_default_backfill",
            Self::ComposeMaterial3Backfill => "compose_material3_backfill",
            Self::VariantShapeAlias => "variant_shape_alias",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialOverlayTokenMetadata {
    pub key: &'static str,
    pub origin: MaterialOverlayTokenOrigin,
    pub source: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialOverlayTokenPrefixMetadata {
    pub prefix: &'static str,
    pub origin: MaterialOverlayTokenOrigin,
    pub source: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialOverlayTokenMatch {
    Exact(&'static MaterialOverlayTokenMetadata),
    Prefix(&'static MaterialOverlayTokenPrefixMetadata),
}

impl MaterialOverlayTokenMatch {
    pub fn origin(self) -> MaterialOverlayTokenOrigin {
        match self {
            Self::Exact(meta) => meta.origin,
            Self::Prefix(meta) => meta.origin,
        }
    }

    pub fn source(self) -> &'static str {
        match self {
            Self::Exact(meta) => meta.source,
            Self::Prefix(meta) => meta.source,
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            Self::Exact(meta) => meta.reason,
            Self::Prefix(meta) => meta.reason,
        }
    }
}

use MaterialOverlayTokenOrigin as Origin;

const fn meta(
    key: &'static str,
    origin: MaterialOverlayTokenOrigin,
    source: &'static str,
    reason: &'static str,
) -> MaterialOverlayTokenMetadata {
    MaterialOverlayTokenMetadata {
        key,
        origin,
        source,
        reason,
    }
}

const fn prefix_meta(
    prefix: &'static str,
    origin: MaterialOverlayTokenOrigin,
    source: &'static str,
    reason: &'static str,
) -> MaterialOverlayTokenPrefixMetadata {
    MaterialOverlayTokenPrefixMetadata {
        prefix,
        origin,
        source,
        reason,
    }
}

const FRET_RUNTIME_SOURCE: &str = "Fret runtime policy";
const FRET_MATERIAL_SOURCE: &str = "Fret Material3 overlay";
const BUTTON_SOURCE: &str = "Material button defaults and Fret variant-shaped access";
const MENU_SOURCE: &str = "Material menu defaults and Fret variant-shaped access";
const NAVIGATION_SOURCE: &str = "Material navigation layout defaults";
const PRIMARY_TAB_SOURCE: &str = "Compose Material3 PrimaryNavigationTabTokens and TabRowDefaults";
const SECONDARY_TAB_SOURCE: &str =
    "Compose Material3 SecondaryNavigationTabTokens and TabRowDefaults";

const FRET_RUNTIME_REASON: &str = "Fret owns this runtime-level theme policy token.";
const FRET_MATERIAL_REASON: &str =
    "Fret owns this Material3 extension token so recipes can read one stable theme key.";
const VARIANT_ALIAS_REASON: &str =
    "Keeps variant-shaped recipe token access uniform when Material Web omits the explicit key.";
const MATERIAL_DEFAULT_REASON: &str =
    "Backfills a concrete Material default that Material Web v30 does not emit as a sassvar key.";
const COMPOSE_BACKFILL_REASON: &str =
    "Backfills a Material3 value from Compose where Material Web v30 has no sassvar key.";

pub const EXACT_TOKEN_METADATA: &[MaterialOverlayTokenMetadata] = &[
    meta(
        "md.sys.layout.minimum-touch-target.size",
        Origin::FretRuntimePolicy,
        FRET_RUNTIME_SOURCE,
        FRET_RUNTIME_REASON,
    ),
    meta(
        "md.sys.fret.layout.is-rtl",
        Origin::FretRuntimePolicy,
        FRET_RUNTIME_SOURCE,
        FRET_RUNTIME_REASON,
    ),
    meta(
        "md.sys.fret.material.is-expressive",
        Origin::FretMaterialPolicy,
        FRET_MATERIAL_SOURCE,
        FRET_MATERIAL_REASON,
    ),
    meta(
        "md.comp.dialog.container.shadow-color",
        Origin::VariantShapeAlias,
        FRET_MATERIAL_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.outlined.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.outlined.container.shadow-color",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.outlined.disabled.container.color",
        Origin::MaterialDefaultBackfill,
        BUTTON_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.button.outlined.disabled.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.outlined.focused.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.outlined.hovered.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.outlined.pressed.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.text.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.text.container.shadow-color",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.text.disabled.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.text.focused.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.text.hovered.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.button.text.pressed.container.elevation",
        Origin::VariantShapeAlias,
        BUTTON_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.container.max-height",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.container.vertical-padding",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.container.max-width",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.container.min-width",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.container.shape",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.content.gap",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.content.horizontal-padding",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.disabled.leading-icon.opacity",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.disabled.supporting-text.opacity",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.disabled.trailing-text.opacity",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.icon.color",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.icon.size",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.leading-icon.color",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.leading-icon.size",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.leading-icon.trailing-space",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.selected.container.shape",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.shortcut.color",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.supporting-text",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.supporting-text.color",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.supporting-text.container.height",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.list-item.supporting-text.weight",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.trailing-icon.color",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.trailing-text",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.trailing-text.color",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.trailing-text.weight",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.list-item.two-line-container.height",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.section-label.container.height",
        Origin::MaterialDefaultBackfill,
        MENU_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.menu.section-label.label-text",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.section-label.label-text.color",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.menu.section-label.label-text.weight",
        Origin::VariantShapeAlias,
        MENU_SOURCE,
        VARIANT_ALIAS_REASON,
    ),
    meta(
        "md.comp.navigation-bar.active-indicator.top-offset",
        Origin::MaterialDefaultBackfill,
        NAVIGATION_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.navigation-bar.item.gap",
        Origin::MaterialDefaultBackfill,
        NAVIGATION_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.navigation-rail.item.height",
        Origin::MaterialDefaultBackfill,
        NAVIGATION_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.navigation-rail.item.width",
        Origin::MaterialDefaultBackfill,
        NAVIGATION_SOURCE,
        MATERIAL_DEFAULT_REASON,
    ),
    meta(
        "md.comp.primary-navigation-tab.active-indicator.min-width",
        Origin::ComposeMaterial3Backfill,
        PRIMARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.primary-navigation-tab.scrollable.edge-padding",
        Origin::ComposeMaterial3Backfill,
        PRIMARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.primary-navigation-tab.scrollable.min-tab-width",
        Origin::ComposeMaterial3Backfill,
        PRIMARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.primary-navigation-tab.with-stacked-icon-and-label-text.container.height",
        Origin::ComposeMaterial3Backfill,
        PRIMARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.active.focus.state-layer.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.active.focus.state-layer.opacity",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.active.hover.state-layer.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.active.hover.state-layer.opacity",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.active.pressed.state-layer.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.active.pressed.state-layer.opacity",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.inactive.focus.state-layer.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.inactive.focus.state-layer.opacity",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.inactive.hover.state-layer.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.inactive.hover.state-layer.opacity",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.inactive.pressed.state-layer.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.inactive.pressed.state-layer.opacity",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.scrollable.edge-padding",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.scrollable.min-tab-width",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-icon.active.focus.icon.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-icon.active.hover.icon.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-icon.active.pressed.icon.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-icon.inactive.focus.icon.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-icon.inactive.hover.icon.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-icon.inactive.pressed.icon.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.active.focus.label-text.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.active.hover.label-text.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.active.label-text.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.active.pressed.label-text.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.inactive.focus.label-text.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.inactive.hover.label-text.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.inactive.label-text.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.inactive.pressed.label-text.color",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.label-text",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-label-text.label-text.weight",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
    meta(
        "md.comp.secondary-navigation-tab.with-stacked-icon-and-label-text.container.height",
        Origin::ComposeMaterial3Backfill,
        SECONDARY_TAB_SOURCE,
        COMPOSE_BACKFILL_REASON,
    ),
];

pub const PREFIX_TOKEN_METADATA: &[MaterialOverlayTokenPrefixMetadata] = &[prefix_meta(
    "md.sys.fret.material.",
    Origin::FretMaterialPolicy,
    FRET_MATERIAL_SOURCE,
    FRET_MATERIAL_REASON,
)];

pub fn metadata_for_key(key: &str) -> Option<MaterialOverlayTokenMatch> {
    if let Some(meta) = EXACT_TOKEN_METADATA.iter().find(|meta| meta.key == key) {
        return Some(MaterialOverlayTokenMatch::Exact(meta));
    }

    PREFIX_TOKEN_METADATA
        .iter()
        .find(|meta| key.starts_with(meta.prefix))
        .map(MaterialOverlayTokenMatch::Prefix)
}

pub fn is_known_non_material_web_token(key: &str) -> bool {
    metadata_for_key(key).is_some()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn exact_overlay_metadata_keys_are_unique() {
        let mut seen = BTreeSet::new();
        for meta in EXACT_TOKEN_METADATA {
            assert!(
                seen.insert(meta.key),
                "duplicate overlay token: {}",
                meta.key
            );
        }
    }

    #[test]
    fn prefix_overlay_metadata_keys_are_unique() {
        let mut seen = BTreeSet::new();
        for meta in PREFIX_TOKEN_METADATA {
            assert!(
                seen.insert(meta.prefix),
                "duplicate overlay token prefix: {}",
                meta.prefix
            );
        }
    }

    #[test]
    fn exact_metadata_wins_over_prefix_metadata() {
        let Some(MaterialOverlayTokenMatch::Exact(meta)) =
            metadata_for_key("md.sys.fret.material.is-expressive")
        else {
            panic!("expected exact overlay metadata");
        };
        assert_eq!(meta.origin, Origin::FretMaterialPolicy);
    }
}

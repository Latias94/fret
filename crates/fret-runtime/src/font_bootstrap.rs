use fret_core::TextFontFamilyConfig;

use crate::{FontCatalog, FontCatalogCache, GlobalsHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFamilyDefaultsPolicy {
    None,
    FillIfEmpty,
    /// If any UI family list is empty, seed it from the head of the current font catalog.
    ///
    /// This is primarily intended for Web/WASM bootstrap, where system font discovery is not
    /// available and we need a deterministic, minimal fallback without exploding settings to
    /// "all fonts".
    FillIfEmptyFromCatalogPrefix {
        max: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontCatalogUpdate {
    pub revision: u64,
    pub families: Vec<String>,
    pub cache: FontCatalogCache,
    pub config: TextFontFamilyConfig,
    pub config_changed: bool,
}

pub fn apply_font_catalog_update(
    app: &mut impl GlobalsHost,
    families: Vec<String>,
    policy: FontFamilyDefaultsPolicy,
) -> FontCatalogUpdate {
    let prev_rev = app.global::<FontCatalog>().map(|c| c.revision).unwrap_or(0);
    let revision = prev_rev.saturating_add(1);

    let cache = FontCatalogCache::from_families(revision, &families);
    app.set_global::<FontCatalog>(FontCatalog {
        families: families.clone(),
        revision,
    });
    app.set_global::<FontCatalogCache>(cache.clone());

    let prev_config = app
        .global::<TextFontFamilyConfig>()
        .cloned()
        .unwrap_or_default();
    let mut config = prev_config.clone();

    match policy {
        FontFamilyDefaultsPolicy::None => {}
        FontFamilyDefaultsPolicy::FillIfEmpty => {
            if config.ui_sans.is_empty() {
                config.ui_sans = families.clone();
            }
            if config.ui_serif.is_empty() {
                config.ui_serif = families.clone();
            }
            if config.ui_mono.is_empty() {
                config.ui_mono = families.clone();
            }
        }
        FontFamilyDefaultsPolicy::FillIfEmptyFromCatalogPrefix { max } => {
            let max = max.max(1);
            let seed: Vec<String> = families.iter().take(max).cloned().collect();
            if config.ui_sans.is_empty() {
                config.ui_sans = seed.clone();
            }
            if config.ui_serif.is_empty() {
                config.ui_serif = seed.clone();
            }
            if config.ui_mono.is_empty() {
                config.ui_mono = seed;
            }
        }
    }

    let config_changed = config != prev_config;
    // Always re-set the config global so renderers can react even if the value is unchanged.
    app.set_global::<TextFontFamilyConfig>(config.clone());

    FontCatalogUpdate {
        revision,
        families,
        cache,
        config,
        config_changed,
    }
}

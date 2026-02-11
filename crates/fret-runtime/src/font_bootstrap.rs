use fret_core::TextFontFamilyConfig;

use crate::{FontCatalog, FontCatalogCache, FontCatalogEntry, FontCatalogMetadata, GlobalsHost};

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
    /// If any UI family list is empty, seed it with a small curated list of common UI families.
    ///
    /// This is primarily intended for Web/WASM bootstrap.
    FillIfEmptyWithCuratedCandidates,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontCatalogUpdate {
    pub revision: u64,
    pub families: Vec<String>,
    pub cache: FontCatalogCache,
    pub config: TextFontFamilyConfig,
    pub config_changed: bool,
}

fn apply_family_defaults_policy(
    mut config: TextFontFamilyConfig,
    families: &[String],
    policy: FontFamilyDefaultsPolicy,
) -> TextFontFamilyConfig {
    match policy {
        FontFamilyDefaultsPolicy::None => {}
        FontFamilyDefaultsPolicy::FillIfEmpty => {
            if config.ui_sans.is_empty() {
                config.ui_sans = families.to_vec();
            }
            if config.ui_serif.is_empty() {
                config.ui_serif = families.to_vec();
            }
            if config.ui_mono.is_empty() {
                config.ui_mono = families.to_vec();
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
        FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates => {
            if config.ui_sans.is_empty() {
                config.ui_sans = vec![
                    "Inter".to_string(),
                    "Segoe UI".to_string(),
                    "Helvetica".to_string(),
                    "Arial".to_string(),
                    "Ubuntu".to_string(),
                    "Adwaita Sans".to_string(),
                    "Cantarell".to_string(),
                    "Noto Sans".to_string(),
                    "DejaVu Sans".to_string(),
                ];
            }
            if config.ui_serif.is_empty() {
                config.ui_serif = vec![
                    "Noto Serif".to_string(),
                    "Times New Roman".to_string(),
                    "Georgia".to_string(),
                    "DejaVu Serif".to_string(),
                ];
            }
            if config.ui_mono.is_empty() {
                config.ui_mono = vec![
                    "JetBrains Mono".to_string(),
                    "Fira Mono".to_string(),
                    "Consolas".to_string(),
                    "Menlo".to_string(),
                    "DejaVu Sans Mono".to_string(),
                    "Noto Sans Mono".to_string(),
                ];
            }
            if config.common_fallback.is_empty() {
                config.common_fallback = vec![
                    // CJK
                    "Noto Sans CJK SC".to_string(),
                    "Noto Sans CJK JP".to_string(),
                    "Noto Sans CJK TC".to_string(),
                    "Microsoft YaHei UI".to_string(),
                    "Microsoft YaHei".to_string(),
                    "PingFang SC".to_string(),
                    "Hiragino Sans".to_string(),
                    // Emoji
                    "Apple Color Emoji".to_string(),
                    "Segoe UI Emoji".to_string(),
                    "Segoe UI Symbol".to_string(),
                    "Noto Color Emoji".to_string(),
                ];
            }
        }
    }

    config
}

pub fn apply_font_catalog_update(
    app: &mut impl GlobalsHost,
    families: Vec<String>,
    policy: FontFamilyDefaultsPolicy,
) -> FontCatalogUpdate {
    let prev_rev = app.global::<FontCatalog>().map(|c| c.revision).unwrap_or(0);
    let catalog_changed = app
        .global::<FontCatalog>()
        .map(|c| c.families.as_slice() != families.as_slice())
        .unwrap_or(true);
    let revision = if catalog_changed {
        prev_rev.saturating_add(1)
    } else {
        prev_rev
    };

    let cache = if catalog_changed {
        let cache = FontCatalogCache::from_families(revision, &families);
        app.set_global::<FontCatalog>(FontCatalog {
            families: families.clone(),
            revision,
        });
        app.set_global::<FontCatalogCache>(cache.clone());
        cache
    } else {
        app.global::<FontCatalogCache>()
            .cloned()
            .unwrap_or_else(|| FontCatalogCache::from_families(revision, &families))
    };

    let prev_config = app
        .global::<TextFontFamilyConfig>()
        .cloned()
        .unwrap_or_default();
    let config = apply_family_defaults_policy(prev_config.clone(), &families, policy);

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

pub fn apply_font_catalog_update_with_metadata(
    app: &mut impl GlobalsHost,
    entries: Vec<FontCatalogEntry>,
    policy: FontFamilyDefaultsPolicy,
) -> FontCatalogUpdate {
    let families = entries.iter().map(|e| e.family.clone()).collect::<Vec<_>>();

    let prev_rev = app.global::<FontCatalog>().map(|c| c.revision).unwrap_or(0);
    let catalog_changed = app
        .global::<FontCatalog>()
        .map(|c| c.families.as_slice() != families.as_slice())
        .unwrap_or(true);
    let metadata_changed = app
        .global::<FontCatalogMetadata>()
        .map(|m| m.entries.as_slice() != entries.as_slice())
        .unwrap_or(true);

    let revision = if catalog_changed || metadata_changed {
        prev_rev.saturating_add(1)
    } else {
        prev_rev
    };

    let prev_config = app
        .global::<TextFontFamilyConfig>()
        .cloned()
        .unwrap_or_default();
    let config = apply_family_defaults_policy(prev_config.clone(), &families, policy);
    let config_changed = config != prev_config;
    app.set_global::<TextFontFamilyConfig>(config.clone());

    let cache = if catalog_changed || metadata_changed {
        let cache = FontCatalogCache::from_families(revision, &families);
        app.set_global::<FontCatalog>(FontCatalog {
            families: families.clone(),
            revision,
        });
        app.set_global::<FontCatalogCache>(cache.clone());
        app.set_global::<FontCatalogMetadata>(FontCatalogMetadata { entries, revision });
        cache
    } else {
        app.global::<FontCatalogCache>()
            .cloned()
            .unwrap_or_else(|| FontCatalogCache::from_families(revision, &families))
    };

    FontCatalogUpdate {
        revision,
        families,
        cache,
        config,
        config_changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestApp {
        globals: HashMap<TypeId, Box<dyn Any>>,
    }

    impl GlobalsHost for TestApp {
        fn global<T: 'static>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
        }

        fn set_global<T: 'static>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn with_global_mut<T: 'static, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();

            let mut value: T = self
                .globals
                .remove(&type_id)
                .and_then(|v| v.downcast::<T>().ok())
                .map(|v| *v)
                .unwrap_or_else(init);

            let out = f(&mut value, self);

            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    #[test]
    fn curated_defaults_append_known_emoji_families() {
        let mut app = TestApp::default();
        let update = apply_font_catalog_update(
            &mut app,
            vec!["Inter".to_string(), "JetBrains Mono".to_string()],
            FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
        );

        assert!(
            update
                .config
                .common_fallback
                .iter()
                .any(|v| v == "Apple Color Emoji")
        );
        assert!(
            update
                .config
                .common_fallback
                .iter()
                .any(|v| v == "Segoe UI Emoji")
        );
        assert!(
            update
                .config
                .common_fallback
                .iter()
                .any(|v| v == "Noto Color Emoji")
        );
    }

    #[test]
    fn apply_update_does_not_bump_revision_when_families_unchanged() {
        let mut app = TestApp::default();

        let update0 = apply_font_catalog_update(
            &mut app,
            vec!["Inter".to_string(), "JetBrains Mono".to_string()],
            FontFamilyDefaultsPolicy::None,
        );
        let update1 = apply_font_catalog_update(
            &mut app,
            vec!["Inter".to_string(), "JetBrains Mono".to_string()],
            FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
        );

        assert_eq!(update0.revision, update1.revision);
        let catalog = app.global::<FontCatalog>().expect("font catalog");
        assert_eq!(catalog.revision, update0.revision);
        assert_eq!(
            catalog.families,
            vec!["Inter".to_string(), "JetBrains Mono".to_string()]
        );
    }

    #[test]
    fn apply_update_with_metadata_sets_metadata_global() {
        let mut app = TestApp::default();
        let entries = vec![
            FontCatalogEntry {
                family: "Inter".to_string(),
                has_variable_axes: false,
                known_variable_axes: vec![],
                variable_axes: vec![],
                is_monospace_candidate: false,
            },
            FontCatalogEntry {
                family: "Roboto Flex".to_string(),
                has_variable_axes: true,
                known_variable_axes: vec!["wght".to_string(), "wdth".to_string()],
                variable_axes: vec![],
                is_monospace_candidate: false,
            },
        ];

        let update = apply_font_catalog_update_with_metadata(
            &mut app,
            entries.clone(),
            FontFamilyDefaultsPolicy::None,
        );

        let catalog = app.global::<FontCatalog>().expect("font catalog");
        assert_eq!(catalog.revision, update.revision);
        assert_eq!(
            catalog.families,
            vec!["Inter".to_string(), "Roboto Flex".to_string()]
        );

        let meta = app
            .global::<FontCatalogMetadata>()
            .expect("font catalog metadata");
        assert_eq!(meta.revision, update.revision);
        assert_eq!(meta.entries, entries);
    }

    #[test]
    fn apply_update_with_metadata_does_not_bump_revision_when_entries_unchanged() {
        let mut app = TestApp::default();
        let entries = vec![
            FontCatalogEntry {
                family: "Inter".to_string(),
                has_variable_axes: false,
                known_variable_axes: vec![],
                variable_axes: vec![],
                is_monospace_candidate: false,
            },
            FontCatalogEntry {
                family: "Roboto Flex".to_string(),
                has_variable_axes: true,
                known_variable_axes: vec!["wght".to_string(), "wdth".to_string()],
                variable_axes: vec![],
                is_monospace_candidate: false,
            },
        ];

        let update0 = apply_font_catalog_update_with_metadata(
            &mut app,
            entries.clone(),
            FontFamilyDefaultsPolicy::None,
        );
        let update1 = apply_font_catalog_update_with_metadata(
            &mut app,
            entries.clone(),
            FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
        );

        assert_eq!(update0.revision, update1.revision);
        let catalog = app.global::<FontCatalog>().expect("font catalog");
        assert_eq!(catalog.revision, update0.revision);
        let meta = app
            .global::<FontCatalogMetadata>()
            .expect("font catalog metadata");
        assert_eq!(meta.revision, update0.revision);
        assert_eq!(meta.entries, entries);
    }
}

use std::sync::Arc;

use crate::FontCatalog;

/// Cached font family name list for efficient UI rendering.
///
/// This is intended to be stored as a global and refreshed by runners when the renderer's font
/// backend changes (e.g. dynamic font injection on web or user-installed fonts on desktop).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontCatalogCache {
    pub revision: u64,
    families: Arc<[Arc<str>]>,
}

impl Default for FontCatalogCache {
    fn default() -> Self {
        Self {
            revision: 0,
            families: Arc::<[Arc<str>]>::from([]),
        }
    }
}

impl FontCatalogCache {
    pub fn families(&self) -> &[Arc<str>] {
        &self.families
    }

    pub fn families_arc(&self) -> Arc<[Arc<str>]> {
        self.families.clone()
    }

    pub fn from_catalog(catalog: &FontCatalog) -> Self {
        Self::from_families(catalog.revision, &catalog.families)
    }

    pub fn from_families(revision: u64, families: &[String]) -> Self {
        let families: Vec<Arc<str>> = families.iter().map(|s| Arc::from(s.as_str())).collect();
        Self {
            revision,
            families: Arc::from(families),
        }
    }
}

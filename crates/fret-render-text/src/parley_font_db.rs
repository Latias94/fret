use fret_core::{TextSlant, TextStyle};
use parley::FontContext;
use parley::fontique::{FamilyId, GenericFamily};
use read_fonts::{FontRef, TableProvider as _};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash as _, Hasher as _};

use crate::FontCatalogEntryMetadata;
use crate::FontVariableAxisMetadata;

#[derive(Debug, Default, Clone, Copy)]
pub struct ParleyShaperFontDbDiagnosticsSnapshot {
    registered_font_blobs_count: u64,
    registered_font_blobs_total_bytes: u64,
    family_id_cache_entries: u64,
    baseline_metrics_cache_entries: u64,
    catalog_entries_build_count: u64,
    all_font_names_cache_present: bool,
    all_font_catalog_entries_cache_present: bool,
}

impl ParleyShaperFontDbDiagnosticsSnapshot {
    pub fn registered_font_blobs_count(&self) -> u64 {
        self.registered_font_blobs_count
    }

    pub fn registered_font_blobs_total_bytes(&self) -> u64 {
        self.registered_font_blobs_total_bytes
    }

    pub fn family_id_cache_entries(&self) -> u64 {
        self.family_id_cache_entries
    }

    pub fn baseline_metrics_cache_entries(&self) -> u64 {
        self.baseline_metrics_cache_entries
    }

    pub fn catalog_entries_build_count(&self) -> u64 {
        self.catalog_entries_build_count
    }

    pub fn all_font_names_cache_present(&self) -> bool {
        self.all_font_names_cache_present
    }

    pub fn all_font_catalog_entries_cache_present(&self) -> bool {
        self.all_font_catalog_entries_cache_present
    }
}

pub(crate) struct ParleyFontDbState {
    system_fonts_enabled: bool,
    registered_font_blobs: VecDeque<RegisteredFontBlob>,
    registered_font_blobs_total_bytes: usize,
    family_id_cache_lower: HashMap<String, FamilyId>,
    all_font_names_cache: Option<Vec<String>>,
    all_font_catalog_entries_cache: Option<Vec<FontCatalogEntryMetadata>>,
    base_line_metrics_cache: HashMap<u64, (f32, f32)>,
    catalog_entries_build_count: u64,
}

#[derive(Debug, Clone)]
struct RegisteredFontBlob {
    hash: u64,
    len: usize,
    blob: parley::fontique::Blob<u8>,
}

impl Default for ParleyFontDbState {
    fn default() -> Self {
        Self {
            system_fonts_enabled: true,
            registered_font_blobs: VecDeque::new(),
            registered_font_blobs_total_bytes: 0,
            family_id_cache_lower: HashMap::new(),
            all_font_names_cache: None,
            all_font_catalog_entries_cache: None,
            base_line_metrics_cache: HashMap::new(),
            catalog_entries_build_count: 0,
        }
    }
}

fn env_disables_font_catalog_monospace_probe() -> bool {
    let Ok(raw) = std::env::var("FRET_TEXT_FONT_CATALOG_MONOSPACE_PROBE") else {
        return false;
    };
    let v = raw.trim().to_ascii_lowercase();
    matches!(v.as_str(), "0" | "false" | "no" | "off")
}

fn registered_font_blobs_max_count() -> usize {
    // Keep enough space for apps that load multiple font families (UI, mono, icons, etc) while
    // preventing unbounded growth when hot-reloading or repeatedly injecting fonts.
    std::env::var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(256)
        .min(4096)
}

fn registered_font_blobs_max_bytes() -> usize {
    // Safety valve for memory-backed font injection. This is a soft cap: we evict the oldest
    // entries until we are within budget.
    std::env::var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(256 * 1024 * 1024)
        .min(2 * 1024 * 1024 * 1024)
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(bytes);
    hasher.finish()
}

pub(crate) fn font_environment_fingerprint(
    all_font_names: &[String],
    all_font_catalog_entries: &[FontCatalogEntryMetadata],
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "fret.text.font_environment.v1".hash(&mut hasher);
    all_font_names.hash(&mut hasher);
    all_font_catalog_entries.hash(&mut hasher);
    hasher.finish()
}

impl ParleyFontDbState {
    pub(crate) fn diagnostics_snapshot(&self) -> ParleyShaperFontDbDiagnosticsSnapshot {
        ParleyShaperFontDbDiagnosticsSnapshot {
            registered_font_blobs_count: self.registered_font_blobs.len() as u64,
            registered_font_blobs_total_bytes: self.registered_font_blobs_total_bytes as u64,
            family_id_cache_entries: self.family_id_cache_lower.len() as u64,
            baseline_metrics_cache_entries: self.base_line_metrics_cache.len() as u64,
            catalog_entries_build_count: self.catalog_entries_build_count,
            all_font_names_cache_present: self.all_font_names_cache.is_some(),
            all_font_catalog_entries_cache_present: self.all_font_catalog_entries_cache.is_some(),
        }
    }

    pub(crate) fn system_fonts_enabled(&self) -> bool {
        self.system_fonts_enabled
    }

    pub(crate) fn disable_system_fonts(&mut self) {
        self.system_fonts_enabled = false;
        self.invalidate_catalog_caches();
    }

    #[cfg(test)]
    pub(crate) fn record_registered_font_blob_bytes_for_tests(&mut self, bytes: Vec<u8>) {
        let blob = parley::fontique::Blob::<u8>::from(bytes);
        self.record_registered_font_blob(blob);
    }

    #[cfg(test)]
    pub(crate) fn registered_font_blob_lengths_for_tests(&self) -> Vec<usize> {
        self.registered_font_blobs.iter().map(|b| b.len).collect()
    }

    #[cfg(test)]
    pub(crate) fn registered_font_blob_total_bytes_for_tests(&self) -> usize {
        self.registered_font_blobs_total_bytes
    }

    fn invalidate_catalog_caches(&mut self) {
        self.family_id_cache_lower.clear();
        self.all_font_names_cache = None;
        self.all_font_catalog_entries_cache = None;
    }

    fn record_registered_font_blob(&mut self, blob: parley::fontique::Blob<u8>) {
        let bytes = blob.as_ref();
        let len = bytes.len();
        let hash = hash_bytes(bytes);

        if let Some(ix) = self
            .registered_font_blobs
            .iter()
            .position(|v| v.hash == hash && v.len == len && v.blob.as_ref() == bytes)
        {
            // LRU: keep the most recently injected fonts near the back.
            let entry = self.registered_font_blobs.remove(ix);
            if let Some(entry) = entry {
                self.registered_font_blobs.push_back(entry);
            }
            return;
        }

        self.registered_font_blobs_total_bytes =
            self.registered_font_blobs_total_bytes.saturating_add(len);
        self.registered_font_blobs
            .push_back(RegisteredFontBlob { hash, len, blob });

        let max_count = registered_font_blobs_max_count();
        let max_bytes = registered_font_blobs_max_bytes();
        while self.registered_font_blobs.len() > max_count
            || self.registered_font_blobs_total_bytes > max_bytes
        {
            let Some(evicted) = self.registered_font_blobs.pop_front() else {
                break;
            };
            self.registered_font_blobs_total_bytes = self
                .registered_font_blobs_total_bytes
                .saturating_sub(evicted.len);
        }
    }

    pub(crate) fn all_font_names(&mut self, fcx: &mut FontContext) -> Vec<String> {
        if let Some(cache) = self.all_font_names_cache.as_ref() {
            return cache.clone();
        }

        let mut by_lower: HashMap<String, String> = HashMap::new();
        for name in fcx.collection.family_names() {
            let key = name.to_ascii_lowercase();
            by_lower.entry(key).or_insert_with(|| name.to_string());
        }

        let mut names: Vec<String> = by_lower.into_values().collect();
        names.sort_unstable_by(|a, b| {
            a.to_ascii_lowercase()
                .cmp(&b.to_ascii_lowercase())
                .then(a.cmp(b))
        });

        self.all_font_names_cache = Some(names.clone());
        names
    }

    pub(crate) fn all_font_catalog_entries(
        &mut self,
        fcx: &mut FontContext,
    ) -> Vec<FontCatalogEntryMetadata> {
        if let Some(cache) = self.all_font_catalog_entries_cache.as_ref() {
            return cache.clone();
        }
        self.catalog_entries_build_count = self.catalog_entries_build_count.saturating_add(1);

        fn axis_tag_string(tag_be_bytes: [u8; 4]) -> String {
            String::from_utf8_lossy(&tag_be_bytes).to_string()
        }

        let mut by_lower: HashMap<String, String> = HashMap::new();
        for name in fcx.collection.family_names() {
            let key = name.to_ascii_lowercase();
            by_lower.entry(key).or_insert_with(|| name.to_string());
        }

        let mut names: Vec<String> = by_lower.into_values().collect();
        names.sort_unstable_by(|a, b| {
            a.to_ascii_lowercase()
                .cmp(&b.to_ascii_lowercase())
                .then(a.cmp(b))
        });

        let mut out: Vec<FontCatalogEntryMetadata> = Vec::with_capacity(names.len());
        for family in names {
            let Some(id) = fcx.collection.family_id(&family) else {
                continue;
            };
            let Some(info) = fcx.collection.family(id) else {
                continue;
            };

            let mut has_variable_axes = false;
            let mut has_wght = false;
            let mut has_wdth = false;
            let mut has_slnt = false;
            let mut has_ital = false;
            let mut has_opsz = false;

            for font in info.fonts() {
                has_variable_axes |= !font.axes().is_empty();
                has_wght |= font.has_weight_axis();
                has_wdth |= font.has_width_axis();
                has_slnt |= font.has_slant_axis();
                has_ital |= font.has_italic_axis();
                has_opsz |= font.has_optical_size_axis();
            }

            let mut known_variable_axes: Vec<String> = Vec::new();
            if has_wght {
                known_variable_axes.push("wght".to_string());
            }
            if has_wdth {
                known_variable_axes.push("wdth".to_string());
            }
            if has_slnt {
                known_variable_axes.push("slnt".to_string());
            }
            if has_ital {
                known_variable_axes.push("ital".to_string());
            }
            if has_opsz {
                known_variable_axes.push("opsz".to_string());
            }

            let variable_axes = info
                .default_font()
                .map(|font| {
                    font.axes()
                        .iter()
                        .take(64)
                        .map(|axis| FontVariableAxisMetadata {
                            tag: axis_tag_string(axis.tag.to_be_bytes()),
                            min_bits: axis.min.to_bits(),
                            max_bits: axis.max.to_bits(),
                            default_bits: axis.default.to_bits(),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let is_monospace_candidate = if env_disables_font_catalog_monospace_probe() {
                false
            } else {
                info.default_font()
                    .and_then(|font| {
                        let blob = font.load(Some(&mut fcx.source_cache))?;
                        let face = FontRef::from_index(blob.as_ref(), font.index()).ok()?;
                        let post = face.post().ok()?;
                        Some(post.is_fixed_pitch() != 0)
                    })
                    .unwrap_or(false)
            };

            out.push(FontCatalogEntryMetadata {
                family,
                has_variable_axes,
                known_variable_axes,
                variable_axes,
                is_monospace_candidate,
            });
        }

        self.all_font_catalog_entries_cache = Some(out.clone());
        out
    }

    pub(crate) fn resolve_family_id(
        &mut self,
        fcx: &mut FontContext,
        name: &str,
    ) -> Option<FamilyId> {
        let name = name.trim();
        if name.is_empty() {
            return None;
        }

        if let Some(id) = fcx.collection.family_id(name) {
            return Some(id);
        }

        let target = name.to_ascii_lowercase();
        if let Some(id) = self.family_id_cache_lower.get(&target).copied() {
            return Some(id);
        }

        let mut resolved_name: Option<String> = None;
        for candidate in fcx.collection.family_names() {
            if candidate.to_ascii_lowercase() != target {
                continue;
            }
            resolved_name = Some(candidate.to_string());
            break;
        }

        let resolved = resolved_name
            .as_deref()
            .and_then(|name| fcx.collection.family_id(name));

        if let Some(id) = resolved {
            self.family_id_cache_lower.insert(target, id);
        }
        resolved
    }

    pub(crate) fn generic_family_ids(
        &self,
        fcx: &mut FontContext,
        generic: GenericFamily,
    ) -> Vec<FamilyId> {
        fcx.collection.generic_families(generic).collect()
    }

    pub(crate) fn set_generic_family_ids(
        &mut self,
        fcx: &mut FontContext,
        generic: GenericFamily,
        ids: &[FamilyId],
    ) -> bool {
        let before = self.generic_family_ids(fcx, generic);
        if before == ids {
            return false;
        }
        fcx.collection
            .set_generic_families(generic, ids.iter().copied());
        true
    }

    pub(crate) fn add_fonts(
        &mut self,
        fcx: &mut FontContext,
        fonts: impl IntoIterator<Item = Vec<u8>>,
    ) -> usize {
        let mut added = 0usize;
        for data in fonts {
            let blob = parley::fontique::Blob::<u8>::from(data);
            self.record_registered_font_blob(blob.clone());
            let families = fcx.collection.register_fonts(blob, None);
            added = added.saturating_add(families.iter().map(|(_, fonts)| fonts.len()).sum());
        }
        if added > 0 {
            self.invalidate_catalog_caches();
        }
        added
    }

    pub(crate) fn system_font_rescan_seed(&self) -> Option<crate::SystemFontRescanSeed> {
        if !self.system_fonts_enabled {
            return None;
        }

        Some(crate::SystemFontRescanSeed {
            registered_font_blobs: self
                .registered_font_blobs
                .iter()
                .map(|b| b.blob.clone())
                .collect(),
        })
    }

    pub(crate) fn apply_system_font_rescan_result(
        &mut self,
        fcx: &mut FontContext,
        result: crate::SystemFontRescanResult,
    ) -> bool {
        if !self.system_fonts_enabled {
            return false;
        }

        let crate::SystemFontRescanResult {
            collection,
            all_font_names,
            all_font_catalog_entries,
            environment_fingerprint,
        } = result;

        if self.current_font_environment_fingerprint(fcx) == environment_fingerprint {
            return false;
        }

        fcx.collection = collection;
        self.invalidate_catalog_caches();
        self.all_font_names_cache = Some(all_font_names);
        self.all_font_catalog_entries_cache = Some(all_font_catalog_entries);
        true
    }

    pub(crate) fn current_font_environment_fingerprint(&mut self, fcx: &mut FontContext) -> u64 {
        let all_font_names = self.all_font_names(fcx);
        let all_font_catalog_entries = self.all_font_catalog_entries(fcx);
        font_environment_fingerprint(&all_font_names, &all_font_catalog_entries)
    }

    pub(crate) fn base_line_metrics_cache_key(
        &self,
        default_locale: Option<&str>,
        common_fallback_stack_suffix: &str,
        style: &TextStyle,
        scale: f32,
    ) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "fret.text.base_line_metrics.v1".hash(&mut hasher);
        style.font.hash(&mut hasher);
        style.size.0.to_bits().hash(&mut hasher);
        style.weight.0.hash(&mut hasher);
        match style.slant {
            TextSlant::Normal => 0u8,
            TextSlant::Italic => 1u8,
            TextSlant::Oblique => 2u8,
        }
        .hash(&mut hasher);
        style
            .letter_spacing_em
            .map(|v| v.to_bits())
            .unwrap_or(0)
            .hash(&mut hasher);
        default_locale.hash(&mut hasher);
        common_fallback_stack_suffix.hash(&mut hasher);
        scale.to_bits().hash(&mut hasher);
        hasher.finish()
    }

    pub(crate) fn base_line_metrics(&self, key: u64) -> Option<(f32, f32)> {
        self.base_line_metrics_cache.get(&key).copied()
    }

    pub(crate) fn insert_base_line_metrics(&mut self, key: u64, metrics: (f32, f32)) {
        self.base_line_metrics_cache.insert(key, metrics);
    }
}

pub(crate) fn run_system_font_rescan(
    seed: crate::SystemFontRescanSeed,
) -> crate::SystemFontRescanResult {
    let mut fcx = FontContext::default();
    fcx.collection = parley::fontique::Collection::new(parley::fontique::CollectionOptions {
        shared: false,
        system_fonts: true,
    });
    fcx.source_cache = parley::fontique::SourceCache::default();

    for blob in seed.registered_font_blobs {
        let _ = fcx.collection.register_fonts(blob, None);
    }

    let mut font_db = ParleyFontDbState::default();
    let all_font_names = font_db.all_font_names(&mut fcx);
    let all_font_catalog_entries = font_db.all_font_catalog_entries(&mut fcx);
    let environment_fingerprint =
        font_environment_fingerprint(&all_font_names, &all_font_catalog_entries);
    crate::SystemFontRescanResult {
        collection: fcx.collection,
        all_font_names,
        all_font_catalog_entries,
        environment_fingerprint,
    }
}

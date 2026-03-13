pub mod cache_keys;
pub mod cache_tuning;
pub mod decorations;
pub mod fallback_policy;
pub mod font_instance_key;
pub mod font_names;
pub mod font_stack;
pub mod font_trace;
pub mod geometry;
pub mod line_layout;
pub mod measure;
pub mod parley_shaper;
pub mod prepare_layout;
pub mod spans;
pub mod wrapper;

#[inline]
pub fn effective_text_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontCatalogEntryMetadata {
    pub family: String,
    pub has_variable_axes: bool,
    pub known_variable_axes: Vec<String>,
    pub variable_axes: Vec<FontVariableAxisMetadata>,
    pub is_monospace_candidate: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FontVariableAxisMetadata {
    pub tag: String,
    pub min_bits: u32,
    pub max_bits: u32,
    pub default_bits: u32,
}

#[derive(Debug, Clone)]
pub struct SystemFontRescanSeed {
    pub(crate) registered_font_blobs: Vec<parley::fontique::Blob<u8>>,
}

pub struct SystemFontRescanResult {
    pub(crate) collection: parley::fontique::Collection,
    pub(crate) all_font_names: Vec<String>,
    pub(crate) all_font_catalog_entries: Vec<FontCatalogEntryMetadata>,
    pub(crate) environment_fingerprint: u64,
}

const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    let _ = assert_send::<SystemFontRescanSeed> as fn();
    let _ = assert_send::<SystemFontRescanResult> as fn();
    let _ = assert_sync::<SystemFontRescanSeed> as fn();
    let _ = assert_sync::<SystemFontRescanResult> as fn();
};

impl std::fmt::Debug for SystemFontRescanResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemFontRescanResult")
            .field("all_font_names_len", &self.all_font_names.len())
            .field(
                "all_font_catalog_entries_len",
                &self.all_font_catalog_entries.len(),
            )
            .field("environment_fingerprint", &self.environment_fingerprint)
            .finish_non_exhaustive()
    }
}

impl SystemFontRescanSeed {
    pub fn run(self) -> SystemFontRescanResult {
        parley_shaper::run_system_font_rescan(self)
    }
}

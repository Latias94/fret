mod cache_keys;
mod cache_tuning;
mod decorations;
mod fallback_policy;
mod font_instance_key;
mod font_names;
mod font_stack;
mod font_trace;
mod geometry;
mod line_layout;
mod measure;
mod parley_font_db;
mod parley_shaper;
mod prepare_layout;
mod spans;
mod wrapper;
mod wrapper_balance;
mod wrapper_boundaries;
mod wrapper_paragraphs;
mod wrapper_ranges;
mod wrapper_slices;

pub use cache_keys::*;
pub use cache_tuning::*;
pub use decorations::*;
pub use fallback_policy::*;
pub use font_instance_key::*;
pub use font_names::*;
pub use font_stack::*;
pub use font_trace::*;
pub use geometry::*;
pub use line_layout::*;
pub use measure::*;
pub use parley_shaper::{
    FontSynthesis, GlyphFontData, ParleyGlyph, ParleyShaper, ParleyShaperFontDbDiagnosticsSnapshot,
    ShapedCluster, ShapedLineLayout, run_system_font_rescan,
};
pub use prepare_layout::*;
pub use spans::*;
pub use wrapper::*;

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

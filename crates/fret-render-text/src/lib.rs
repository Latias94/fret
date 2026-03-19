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

pub use cache_keys::{
    TextBlobKey, TextMeasureKey, TextShapeKey, spans_paint_fingerprint, spans_shaping_fingerprint,
};
pub use cache_tuning::{
    measure_shaping_cache_entries, measure_shaping_cache_min_text_len_bytes,
    released_blob_cache_entries,
};
pub use decorations::{
    TextDecoration, TextDecorationKind, TextDecorationMetricsPx,
    decoration_metrics_px_for_font_bytes, decorations_for_lines,
};
pub use fallback_policy::{CommonFallbackMode, TextFallbackPolicyV1, common_fallback_stack_suffix};
pub use font_instance_key::{FontFaceKey, variation_key_from_normalized_coords};
pub use font_names::best_family_name_from_font_bytes;
pub use font_stack::{GenericFamilyInjectionState, apply_font_families_inner};
pub use font_trace::{FontTraceFamilyResolved, FontTraceState};
pub use geometry::{
    TextLineCluster, TextLineDecorationGeometry, TextLineGeometry, caret_rect_from_lines,
    caret_stops_for_slice, caret_x_from_stops, hit_test_point_from_lines, hit_test_x_from_stops,
    selection_rects_from_lines, selection_rects_from_lines_clipped,
};
pub use line_layout::TextLineLayout;
pub use measure::TextMeasureCaches;
pub use parley_shaper::{
    FontSynthesis, GlyphFontData, ParleyGlyph, ParleyShaper, ParleyShaperFontDbDiagnosticsSnapshot,
    ShapedCluster, ShapedLineLayout, run_system_font_rescan,
};
pub use prepare_layout::{PreparedLayout, PreparedLine, prepare_layout_from_wrapped};
pub use spans::{
    ResolvedDecoration, ResolvedSpan, paint_span_for_text_range, resolve_spans_for_text,
    sanitize_spans_for_text,
};
pub use wrapper::{WrappedLayout, wrap_with_constraints, wrap_with_constraints_measure_only};

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

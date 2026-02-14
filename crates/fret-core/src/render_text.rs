use crate::FrameId;
use crate::{FontId, Px, TextOverflow, TextWrap};

/// Per-frame renderer-owned text cache counters.
///
/// This is intended for diagnostics bundles and on-screen debug panels. The runner can update it
/// once per rendered frame to make renderer-level churn (text blobs, glyph atlas pressure) visible
/// in a single artifact.
#[derive(Debug, Default, Clone, Copy)]
pub struct RendererTextPerfSnapshot {
    pub frame_id: FrameId,

    pub font_stack_key: u64,
    pub font_db_revision: u64,
    /// Fingerprint of the effective font fallback policy (locale + family config + injection).
    ///
    /// This is intended for diagnostics only.
    pub fallback_policy_key: u64,

    /// Total count of missing/tofu glyphs observed across text prepared this frame.
    ///
    /// Implementation note: this is currently approximated as the number of shaped glyphs whose
    /// glyph id is `0` (the `.notdef` glyph) across prepared text runs.
    pub frame_missing_glyphs: u64,
    /// Count of prepared text blobs that contained at least one missing/tofu glyph.
    pub frame_texts_with_missing_glyphs: u64,

    pub blobs_live: u64,
    pub blob_cache_entries: u64,
    pub shape_cache_entries: u64,
    pub measure_cache_buckets: u64,

    pub unwrapped_layout_cache_entries: u64,
    pub frame_unwrapped_layout_cache_hits: u64,
    pub frame_unwrapped_layout_cache_misses: u64,
    pub frame_unwrapped_layouts_created: u64,

    pub frame_cache_resets: u64,
    pub frame_blob_cache_hits: u64,
    pub frame_blob_cache_misses: u64,
    pub frame_blobs_created: u64,
    pub frame_shape_cache_hits: u64,
    pub frame_shape_cache_misses: u64,
    pub frame_shapes_created: u64,

    pub mask_atlas: RendererGlyphAtlasPerfSnapshot,
    pub color_atlas: RendererGlyphAtlasPerfSnapshot,
    pub subpixel_atlas: RendererGlyphAtlasPerfSnapshot,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RendererGlyphAtlasPerfSnapshot {
    pub width: u32,
    pub height: u32,
    pub pages: u32,
    pub entries: u64,

    pub used_px: u64,
    pub capacity_px: u64,

    pub frame_hits: u64,
    pub frame_misses: u64,
    pub frame_inserts: u64,
    pub frame_evict_glyphs: u64,
    pub frame_evict_pages: u64,
    pub frame_out_of_space: u64,
    pub frame_too_large: u64,

    pub frame_pending_uploads: u64,
    pub frame_pending_upload_bytes: u64,
    pub frame_upload_bytes: u64,
}

/// Per-frame renderer-owned font selection trace, intended for diagnostics bundles.
///
/// This is **not** a stable contract for apps; it exists to make font fallback issues auditable and
/// to help keep refactors fearless (especially around variable fonts and fallback chain semantics).
#[derive(Debug, Default, Clone)]
pub struct RendererTextFontTraceSnapshot {
    pub frame_id: FrameId,
    /// Bounded list of trace entries for the frame.
    ///
    /// Renderers are expected to keep this list small. Typical policies:
    ///
    /// - record entries only when missing/tofu glyphs are observed
    /// - record a small ring of the most recent prepared blobs when explicitly enabled
    pub entries: Vec<RendererTextFontTraceEntry>,
}

#[derive(Debug, Clone)]
pub struct RendererTextFontTraceEntry {
    /// Human-readable preview of the text (may be truncated).
    pub text_preview: String,
    pub text_len_bytes: u32,

    pub font: FontId,
    pub font_size: Px,
    pub scale_factor: f32,

    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub max_width: Option<Px>,

    pub locale_bcp47: Option<String>,

    pub missing_glyphs: u32,

    /// Families used by shaping/rasterization for this blob (best-effort).
    pub families: Vec<RendererTextFontTraceFamilyUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererTextFontTraceFamilyUsage {
    pub family: String,
    pub glyphs: u32,
    pub missing_glyphs: u32,
    pub class: RendererTextFontTraceFamilyClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererTextFontTraceFamilyClass {
    Requested,
    CommonFallback,
    SystemFallback,
    Unknown,
}

/// Snapshot of the effective renderer font fallback policy, intended for diagnostics bundles.
#[derive(Debug, Default, Clone)]
pub struct RendererTextFallbackPolicySnapshot {
    pub frame_id: FrameId,
    pub font_stack_key: u64,
    pub font_db_revision: u64,
    pub fallback_policy_key: u64,

    pub system_fonts_enabled: bool,
    pub locale_bcp47: Option<String>,

    pub common_fallback_injection: crate::TextCommonFallbackInjection,
    pub prefer_common_fallback: bool,

    /// The effective suffix appended to named-family stacks when common fallback is preferred.
    pub common_fallback_stack_suffix: String,
    /// The effective candidate list used to build `common_fallback_stack_suffix` (trimmed + deduped, preserving order).
    pub common_fallback_candidates: Vec<String>,
}

use crate::FrameId;

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

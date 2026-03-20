use super::{
    DebugGlyphAtlasLookup, GlyphInstance, GlyphQuadKind, TextDecoration, TextFontFaceUsage,
    TextLine, TextShape, TextSystem,
};
use std::{collections::HashSet, sync::Arc};

fn estimate_text_shape_heap_bytes(shape: &TextShape) -> u64 {
    let mul = |len: usize, item_size: usize| -> u64 {
        ((len as u128) * (item_size as u128)).min(u64::MAX as u128) as u64
    };

    let mut bytes: u64 = (std::mem::size_of::<TextShape>() as u128).min(u64::MAX as u128) as u64;

    bytes = bytes.saturating_add(mul(
        shape.glyphs().len(),
        std::mem::size_of::<GlyphInstance>(),
    ));
    bytes = bytes.saturating_add(mul(shape.lines().len(), std::mem::size_of::<TextLine>()));
    bytes = bytes.saturating_add(mul(
        shape.caret_stops().len(),
        std::mem::size_of::<(usize, fret_core::geometry::Px)>(),
    ));
    bytes = bytes.saturating_add(mul(
        shape.font_faces().len(),
        std::mem::size_of::<TextFontFaceUsage>(),
    ));

    for line in shape.lines() {
        bytes = bytes.saturating_add(mul(
            line.caret_stops_capacity(),
            std::mem::size_of::<(usize, fret_core::geometry::Px)>(),
        ));
        bytes = bytes.saturating_add(mul(
            line.clusters().len(),
            std::mem::size_of::<fret_render_text::TextLineCluster>(),
        ));
    }

    bytes
}

impl TextSystem {
    pub fn begin_frame_diagnostics(&mut self) {
        self.font_runtime.font_trace.begin_frame();

        self.frame_perf.clear();
        self.atlas_runtime.begin_frame_diagnostics();
    }

    pub fn font_trace_snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextFontTraceSnapshot {
        self.font_runtime.font_trace.snapshot(frame_id)
    }

    pub fn diagnostics_snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextPerfSnapshot {
        let font_db = self.parley_shaper.font_db_diagnostics_snapshot();
        let mut shape_cache_bytes_estimate_total: u64 = 0;
        let mut seen_shapes: HashSet<usize> = HashSet::new();

        let mut add_shape = |shape: &Arc<TextShape>| {
            let ptr = Arc::as_ptr(shape) as usize;
            if !seen_shapes.insert(ptr) {
                return;
            }
            shape_cache_bytes_estimate_total = shape_cache_bytes_estimate_total
                .saturating_add(estimate_text_shape_heap_bytes(shape.as_ref()));
        };

        for shape in self.layout_cache.shape_cache.values() {
            add_shape(shape);
        }
        for blob in self.blob_state.blobs.values() {
            add_shape(&blob.shape);
        }

        let mut blob_paint_palette_bytes_estimate_total: u64 = 0;
        let mut blob_decorations_bytes_estimate_total: u64 = 0;
        let mut seen_palettes: HashSet<usize> = HashSet::new();
        let mut seen_decorations: HashSet<usize> = HashSet::new();

        for blob in self.blob_state.blobs.values() {
            if let Some(palette) = blob.paint_palette.as_ref() {
                let ptr = palette.as_ptr() as usize;
                if seen_palettes.insert(ptr) {
                    blob_paint_palette_bytes_estimate_total =
                        blob_paint_palette_bytes_estimate_total.saturating_add(
                            ((palette.len() as u128)
                                * (std::mem::size_of::<Option<fret_core::Color>>() as u128))
                                .min(u64::MAX as u128) as u64,
                        );
                }
            }
            let ptr = blob.decorations.as_ptr() as usize;
            if seen_decorations.insert(ptr) {
                blob_decorations_bytes_estimate_total = blob_decorations_bytes_estimate_total
                    .saturating_add(
                        ((blob.decorations.len() as u128)
                            * (std::mem::size_of::<TextDecoration>() as u128))
                            .min(u64::MAX as u128) as u64,
                    );
            }
        }

        fret_core::RendererTextPerfSnapshot {
            frame_id,
            font_stack_key: self.font_runtime.font_stack_key,
            font_db_revision: self.font_runtime.font_db_revision,
            fallback_policy_key: self.font_runtime.fallback_policy.fallback_policy_key(),
            frame_missing_glyphs: self.frame_perf.missing_glyphs,
            frame_texts_with_missing_glyphs: self.frame_perf.texts_with_missing_glyphs,
            blobs_live: self.blob_state.blobs.len() as u64,
            blob_cache_entries: self.blob_state.blob_cache.len() as u64,
            shape_cache_entries: self.layout_cache.shape_cache.len() as u64,
            measure_cache_buckets: self.layout_cache.measure.buckets_len() as u64,
            shape_cache_bytes_estimate_total,
            blob_paint_palette_bytes_estimate_total,
            blob_decorations_bytes_estimate_total,
            unwrapped_layout_cache_entries: 0,
            frame_unwrapped_layout_cache_hits: 0,
            frame_unwrapped_layout_cache_misses: 0,
            frame_unwrapped_layouts_created: 0,
            frame_cache_resets: self.frame_perf.cache_resets,
            frame_blob_cache_hits: self.frame_perf.blob_cache_hits,
            frame_blob_cache_misses: self.frame_perf.blob_cache_misses,
            frame_blobs_created: self.frame_perf.blobs_created,
            frame_shape_cache_hits: self.frame_perf.shape_cache_hits,
            frame_shape_cache_misses: self.frame_perf.shape_cache_misses,
            frame_shapes_created: self.frame_perf.shapes_created,
            mask_atlas: self.atlas_runtime.mask_atlas.diagnostics_snapshot(),
            color_atlas: self.atlas_runtime.color_atlas.diagnostics_snapshot(),
            subpixel_atlas: self.atlas_runtime.subpixel_atlas.diagnostics_snapshot(),
            registered_font_blobs_count: font_db.registered_font_blobs_count(),
            registered_font_blobs_total_bytes: font_db.registered_font_blobs_total_bytes(),
            family_id_cache_entries: font_db.family_id_cache_entries(),
            baseline_metrics_cache_entries: font_db.baseline_metrics_cache_entries(),
        }
    }

    pub fn fallback_policy_snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextFallbackPolicySnapshot {
        self.font_runtime.fallback_policy.diagnostics_snapshot(
            frame_id,
            self.font_runtime.font_stack_key,
            self.font_runtime.font_db_revision,
            &self.parley_shaper,
        )
    }

    pub(crate) fn take_atlas_perf_snapshot(&mut self) -> super::TextAtlasPerfSnapshot {
        let mask = self.atlas_runtime.mask_atlas.take_perf_snapshot();
        let color = self.atlas_runtime.color_atlas.take_perf_snapshot();
        let subpixel = self.atlas_runtime.subpixel_atlas.take_perf_snapshot();

        super::TextAtlasPerfSnapshot {
            uploads: mask.uploads + color.uploads + subpixel.uploads,
            upload_bytes: mask.upload_bytes + color.upload_bytes + subpixel.upload_bytes,
            evicted_glyphs: mask.evicted_glyphs + color.evicted_glyphs + subpixel.evicted_glyphs,
            evicted_pages: mask.evicted_pages + color.evicted_pages + subpixel.evicted_pages,
            evicted_page_glyphs: mask.evicted_page_glyphs
                + color.evicted_page_glyphs
                + subpixel.evicted_page_glyphs,
            resets: mask.resets + color.resets + subpixel.resets,
        }
    }

    pub(crate) fn atlas_revision(&self) -> u64 {
        self.atlas_runtime
            .mask_atlas
            .revision()
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ self.atlas_runtime.color_atlas.revision().rotate_left(1)
            ^ self.atlas_runtime.subpixel_atlas.revision().rotate_left(2)
    }

    pub(crate) fn glyph_uv_for_instance(&self, glyph: &GlyphInstance) -> Option<(u16, [f32; 4])> {
        let atlas = self.atlas_runtime.atlas(glyph.kind());

        let entry = atlas.entry(glyph.key)?;
        let (w, h) = atlas.dimensions();
        let w = w as f32;
        let h = h as f32;
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let u0 = entry.x as f32 / w;
        let v0 = entry.y as f32 / h;
        let u1 = (entry.x.saturating_add(entry.w) as f32) / w;
        let v1 = (entry.y.saturating_add(entry.h) as f32) / h;
        Some((entry.page, [u0, v0, u1, v1]))
    }

    pub(crate) fn debug_atlas_dims(&self, kind: GlyphQuadKind) -> (u32, u32) {
        self.atlas_runtime.atlas(kind).dimensions()
    }

    pub(crate) fn debug_lookup_glyph_atlas_entry(
        &self,
        kind: GlyphQuadKind,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        let atlas = self.atlas_runtime.atlas(kind);

        let k = atlas.find_key_for_bounds(page, x, y, w, h)?;

        Some(DebugGlyphAtlasLookup {
            font_data_id: k.font.font_data_id(),
            face_index: k.font.face_index(),
            variation_key: k.font.variation_key(),
            synthesis_embolden: k.font.synthesis_embolden(),
            synthesis_skew_degrees: k.font.synthesis_skew_degrees(),
            glyph_id: k.glyph_id,
            size_bits: k.size_bits,
            x_bin: k.x_bin,
            y_bin: k.y_bin,
            kind: match k.kind {
                GlyphQuadKind::Mask => "mask",
                GlyphQuadKind::Color => "color",
                GlyphQuadKind::Subpixel => "subpixel",
            },
        })
    }
}

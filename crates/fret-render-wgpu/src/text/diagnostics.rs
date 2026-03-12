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
        shape.glyphs.len(),
        std::mem::size_of::<GlyphInstance>(),
    ));
    bytes = bytes.saturating_add(mul(shape.lines.len(), std::mem::size_of::<TextLine>()));
    bytes = bytes.saturating_add(mul(
        shape.caret_stops.len(),
        std::mem::size_of::<(usize, fret_core::geometry::Px)>(),
    ));
    bytes = bytes.saturating_add(mul(
        shape.font_faces.len(),
        std::mem::size_of::<TextFontFaceUsage>(),
    ));

    for line in shape.lines.iter() {
        bytes = bytes.saturating_add(mul(
            line.caret_stops.capacity(),
            std::mem::size_of::<(usize, fret_core::geometry::Px)>(),
        ));
        bytes = bytes.saturating_add(mul(
            line.clusters().len(),
            std::mem::size_of::<fret_render_text::geometry::TextLineCluster>(),
        ));
    }

    bytes
}

impl TextSystem {
    pub fn begin_frame_diagnostics(&mut self) {
        self.font_trace.begin_frame();

        self.perf_frame_cache_resets = 0;
        self.perf_frame_blob_cache_hits = 0;
        self.perf_frame_blob_cache_misses = 0;
        self.perf_frame_blobs_created = 0;
        self.perf_frame_shape_cache_hits = 0;
        self.perf_frame_shape_cache_misses = 0;
        self.perf_frame_shapes_created = 0;
        self.perf_frame_missing_glyphs = 0;
        self.perf_frame_texts_with_missing_glyphs = 0;
        self.mask_atlas.begin_frame_diagnostics();
        self.color_atlas.begin_frame_diagnostics();
        self.subpixel_atlas.begin_frame_diagnostics();
    }

    pub fn font_trace_snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextFontTraceSnapshot {
        self.font_trace.snapshot(frame_id)
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

        for shape in self.shape_cache.values() {
            add_shape(shape);
        }
        for blob in self.blobs.values() {
            add_shape(&blob.shape);
        }

        let mut blob_paint_palette_bytes_estimate_total: u64 = 0;
        let mut blob_decorations_bytes_estimate_total: u64 = 0;
        let mut seen_palettes: HashSet<usize> = HashSet::new();
        let mut seen_decorations: HashSet<usize> = HashSet::new();

        for blob in self.blobs.values() {
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
            font_stack_key: self.font_stack_key,
            font_db_revision: self.font_db_revision,
            fallback_policy_key: self.fallback_policy.fallback_policy_key,
            frame_missing_glyphs: self.perf_frame_missing_glyphs,
            frame_texts_with_missing_glyphs: self.perf_frame_texts_with_missing_glyphs,
            blobs_live: self.blobs.len() as u64,
            blob_cache_entries: self.blob_cache.len() as u64,
            shape_cache_entries: self.shape_cache.len() as u64,
            measure_cache_buckets: self.measure.buckets_len() as u64,
            shape_cache_bytes_estimate_total,
            blob_paint_palette_bytes_estimate_total,
            blob_decorations_bytes_estimate_total,
            unwrapped_layout_cache_entries: 0,
            frame_unwrapped_layout_cache_hits: 0,
            frame_unwrapped_layout_cache_misses: 0,
            frame_unwrapped_layouts_created: 0,
            frame_cache_resets: self.perf_frame_cache_resets,
            frame_blob_cache_hits: self.perf_frame_blob_cache_hits,
            frame_blob_cache_misses: self.perf_frame_blob_cache_misses,
            frame_blobs_created: self.perf_frame_blobs_created,
            frame_shape_cache_hits: self.perf_frame_shape_cache_hits,
            frame_shape_cache_misses: self.perf_frame_shape_cache_misses,
            frame_shapes_created: self.perf_frame_shapes_created,
            mask_atlas: self.mask_atlas.diagnostics_snapshot(),
            color_atlas: self.color_atlas.diagnostics_snapshot(),
            subpixel_atlas: self.subpixel_atlas.diagnostics_snapshot(),
            registered_font_blobs_count: font_db.registered_font_blobs_count,
            registered_font_blobs_total_bytes: font_db.registered_font_blobs_total_bytes,
            family_id_cache_entries: font_db.family_id_cache_entries,
            baseline_metrics_cache_entries: font_db.baseline_metrics_cache_entries,
        }
    }

    pub fn fallback_policy_snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextFallbackPolicySnapshot {
        fret_core::RendererTextFallbackPolicySnapshot {
            frame_id,
            font_stack_key: self.font_stack_key,
            font_db_revision: self.font_db_revision,
            fallback_policy_key: self.fallback_policy.fallback_policy_key,
            system_fonts_enabled: self.parley_shaper.system_fonts_enabled(),
            locale_bcp47: self.fallback_policy.locale_bcp47.clone(),
            common_fallback_injection: self
                .fallback_policy
                .font_family_config
                .common_fallback_injection,
            prefer_common_fallback: self.fallback_policy.prefer_common_fallback(),
            common_fallback_stack_suffix: self
                .parley_shaper
                .common_fallback_stack_suffix()
                .to_string(),
            common_fallback_candidates: self.fallback_policy.common_fallback_candidates.clone(),
        }
    }

    pub(crate) fn take_atlas_perf_snapshot(&mut self) -> super::TextAtlasPerfSnapshot {
        let mask = self.mask_atlas.take_perf_snapshot();
        let color = self.color_atlas.take_perf_snapshot();
        let subpixel = self.subpixel_atlas.take_perf_snapshot();

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
        self.mask_atlas
            .revision()
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ self.color_atlas.revision().rotate_left(1)
            ^ self.subpixel_atlas.revision().rotate_left(2)
    }

    pub(crate) fn glyph_uv_for_instance(&self, glyph: &GlyphInstance) -> Option<(u16, [f32; 4])> {
        let atlas = match glyph.kind() {
            GlyphQuadKind::Mask => &self.mask_atlas,
            GlyphQuadKind::Color => &self.color_atlas,
            GlyphQuadKind::Subpixel => &self.subpixel_atlas,
        };

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
        match kind {
            GlyphQuadKind::Mask => self.mask_atlas.dimensions(),
            GlyphQuadKind::Color => self.color_atlas.dimensions(),
            GlyphQuadKind::Subpixel => self.subpixel_atlas.dimensions(),
        }
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
        let atlas = match kind {
            GlyphQuadKind::Mask => &self.mask_atlas,
            GlyphQuadKind::Color => &self.color_atlas,
            GlyphQuadKind::Subpixel => &self.subpixel_atlas,
        };

        let k = atlas.find_key_for_bounds(page, x, y, w, h)?;

        Some(DebugGlyphAtlasLookup {
            font_data_id: k.font.font_data_id,
            face_index: k.font.face_index,
            variation_key: k.font.variation_key,
            synthesis_embolden: k.font.synthesis_embolden,
            synthesis_skew_degrees: k.font.synthesis_skew_degrees,
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

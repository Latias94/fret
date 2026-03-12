use super::atlas::{
    GlyphAtlasEntry, GlyphKey, subpixel_bin_as_float, subpixel_bin_q4, subpixel_bin_y,
};
use super::{
    GlyphInstance, GlyphQuadKind, TextBlob, TextFontFaceUsage, TextLine, TextShape, TextSystem,
};
use fret_core::{
    AttributedText, TextBlobId, TextConstraints, TextInputRef, TextMetrics, TextSpan, TextStyle,
    geometry::Px,
};
use fret_render_text::cache_keys::{TextBlobKey, TextShapeKey};
use fret_render_text::decorations::TextDecorationMetricsPx;
use fret_render_text::font_instance_key::{FontFaceKey, variation_key_from_normalized_coords};
use fret_render_text::font_trace::FontTraceFamilyResolved;
use fret_render_text::{
    parley_shaper::ParleyGlyph,
    prepare_layout::PreparedLine,
    spans::{ResolvedSpan, paint_span_for_text_range},
};
use std::{collections::HashMap, sync::Arc};

pub(super) struct PrepareShapeBuildContext {
    pub(super) wrapped: crate::text::wrapper::WrappedLayout,
    pub(super) epoch: u64,
    pub(super) glyphs: Vec<super::GlyphInstance>,
    pub(super) face_usage: HashMap<FontFaceKey, (u32, u32)>,
    pub(super) lines: Vec<TextLine>,
}

impl TextSystem {
    #[allow(dead_code)]
    pub fn prepare_input(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        match input {
            TextInputRef::Plain { text, style } => self.prepare(text, style, constraints),
            TextInputRef::Attributed { text, base, spans } => {
                let spans = fret_render_text::spans::sanitize_spans_for_text(text, spans);
                if spans.is_none() {
                    return self.prepare(text, base, constraints);
                }
                let rich = AttributedText {
                    text: Arc::<str>::from(text),
                    spans: spans.expect("non-empty spans"),
                };
                self.prepare_attributed(&rich, base, constraints)
            }
        }
    }

    pub fn prepare(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let key = fret_render_text::cache_keys::TextBlobKey::new(
            text,
            style,
            constraints,
            self.font_stack_key,
        );
        self.prepare_with_key(key, style, None, constraints)
    }

    pub fn prepare_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let spans = fret_render_text::spans::sanitize_spans_for_text(
            rich.text.as_ref(),
            rich.spans.as_ref(),
        );
        if spans.is_none() {
            return self.prepare(rich.text.as_ref(), base_style, constraints);
        }
        let rich = AttributedText {
            text: rich.text.clone(),
            spans: spans.expect("non-empty spans"),
        };
        let key = fret_render_text::cache_keys::TextBlobKey::new_attributed(
            &rich,
            base_style,
            constraints,
            self.font_stack_key,
        );
        self.prepare_with_key(key, base_style, Some(rich.spans.as_ref()), constraints)
    }

    pub(super) fn try_reuse_cached_blob(
        &mut self,
        key: &TextBlobKey,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> Option<(TextBlobId, TextMetrics)> {
        let id = self.blob_cache.get(key).copied()?;

        let mut hit: Option<(TextMetrics, u32, Arc<TextShape>, bool)> = None;
        if let Some(blob) = self.blobs.get_mut(id) {
            self.perf_frame_blob_cache_hits = self.perf_frame_blob_cache_hits.saturating_add(1);
            let was_released = blob.ref_count == 0;
            blob.ref_count = blob.ref_count.saturating_add(1);
            hit = Some((
                blob.shape.metrics,
                blob.shape.missing_glyphs,
                blob.shape.clone(),
                was_released,
            ));
        }

        if let Some((metrics, missing_glyphs, shape, was_released)) = hit {
            if was_released {
                self.remove_released_blob(id);
            }
            if missing_glyphs > 0 {
                self.perf_frame_missing_glyphs = self
                    .perf_frame_missing_glyphs
                    .saturating_add(u64::from(missing_glyphs));
                self.perf_frame_texts_with_missing_glyphs =
                    self.perf_frame_texts_with_missing_glyphs.saturating_add(1);
            }
            self.maybe_record_font_trace_entry(text, style, constraints, &shape);
            return Some((id, metrics));
        }

        // Stale cache entry (shouldn't happen, but keep it robust).
        self.blob_cache.remove(key);
        self.blob_key_by_id.remove(&id);
        None
    }

    pub(super) fn finalize_prepared_blob(
        &mut self,
        key: TextBlobKey,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
        shape: Arc<TextShape>,
        resolved_spans: Option<&[ResolvedSpan]>,
        paint_palette: Option<Arc<[Option<fret_core::Color>]>>,
        scale: f32,
        snap_vertical: bool,
    ) -> (TextBlobId, TextMetrics) {
        let decoration_metrics = self.decoration_metrics_for_shape(style, scale, &shape);
        let decorations: Vec<super::TextDecoration> = resolved_spans
            .map(|spans| {
                fret_render_text::decorations::decorations_for_lines(
                    shape.lines.as_ref(),
                    spans,
                    decoration_metrics,
                    scale,
                    snap_vertical,
                )
            })
            .unwrap_or_default();

        let metrics = shape.metrics;
        if shape.missing_glyphs > 0 {
            self.perf_frame_missing_glyphs = self
                .perf_frame_missing_glyphs
                .saturating_add(u64::from(shape.missing_glyphs));
            self.perf_frame_texts_with_missing_glyphs =
                self.perf_frame_texts_with_missing_glyphs.saturating_add(1);
        }
        self.maybe_record_font_trace_entry(text, style, constraints, &shape);
        let id = self.blobs.insert(TextBlob {
            shape,
            paint_palette,
            decorations: Arc::from(decorations),
            ref_count: 1,
        });
        self.perf_frame_blobs_created = self.perf_frame_blobs_created.saturating_add(1);
        self.blob_cache.insert(key.clone(), id);
        self.blob_key_by_id.insert(id, key);
        (id, metrics)
    }

    pub(super) fn try_reuse_cached_shape(
        &mut self,
        shape_key: &TextShapeKey,
    ) -> Option<Arc<TextShape>> {
        let shape = self.shape_cache.get(shape_key)?.clone();
        self.perf_frame_shape_cache_hits = self.perf_frame_shape_cache_hits.saturating_add(1);
        Some(shape)
    }

    pub(super) fn cache_prepared_shape(
        &mut self,
        shape_key: TextShapeKey,
        shape: Arc<TextShape>,
    ) -> Arc<TextShape> {
        self.perf_frame_shapes_created = self.perf_frame_shapes_created.saturating_add(1);
        self.shape_cache.insert(shape_key, shape.clone());
        shape
    }

    pub(super) fn begin_prepare_shape_build(
        &mut self,
        text: &str,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
    ) -> PrepareShapeBuildContext {
        let input = match spans {
            Some(spans) => TextInputRef::Attributed {
                text,
                base: style,
                spans,
            },
            None => TextInputRef::Plain { text, style },
        };
        let wrapped = self.wrap_for_prepare(input, constraints);
        let epoch = {
            let e = self.glyph_atlas_epoch;
            self.glyph_atlas_epoch = self.glyph_atlas_epoch.saturating_add(1);
            e
        };

        PrepareShapeBuildContext {
            wrapped,
            epoch,
            glyphs: Vec::new(),
            face_usage: HashMap::new(),
            lines: Vec::new(),
        }
    }

    pub(super) fn finish_prepared_shape(
        &self,
        glyphs: Vec<GlyphInstance>,
        lines: Vec<TextLine>,
        face_usage: HashMap<FontFaceKey, (u32, u32)>,
        metrics: TextMetrics,
        missing_glyphs: u32,
        first_line_caret_stops: Vec<(usize, Px)>,
    ) -> Arc<TextShape> {
        let mut face_usages: Vec<TextFontFaceUsage> = Vec::with_capacity(face_usage.len());
        for (face, (glyphs, missing)) in face_usage {
            face_usages.push(TextFontFaceUsage {
                font_data_id: face.font_data_id,
                face_index: face.face_index,
                variation_key: face.variation_key,
                synthesis_embolden: face.synthesis_embolden,
                synthesis_skew_degrees: face.synthesis_skew_degrees,
                glyphs,
                missing_glyphs: missing,
            });
        }
        face_usages.sort_by(|a, b| {
            b.glyphs
                .cmp(&a.glyphs)
                .then_with(|| a.font_data_id.cmp(&b.font_data_id))
                .then_with(|| a.face_index.cmp(&b.face_index))
                .then_with(|| a.variation_key.cmp(&b.variation_key))
                .then_with(|| a.synthesis_embolden.cmp(&b.synthesis_embolden))
                .then_with(|| a.synthesis_skew_degrees.cmp(&b.synthesis_skew_degrees))
        });

        Arc::new(TextShape {
            glyphs: Arc::from(glyphs),
            metrics,
            lines: Arc::from(lines),
            caret_stops: Arc::from(first_line_caret_stops),
            missing_glyphs,
            font_faces: Arc::from(face_usages),
        })
    }

    pub(super) fn materialize_prepared_line(
        &mut self,
        prepared_line: PreparedLine,
        resolved_spans: Option<&[ResolvedSpan]>,
        scale: f32,
        epoch: u64,
        glyphs: &mut Vec<GlyphInstance>,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
        lines: &mut Vec<TextLine>,
    ) {
        let PreparedLine {
            layout,
            glyphs: prepared_glyphs,
        } = prepared_line;
        lines.push(layout);

        for g in prepared_glyphs {
            let Ok(glyph_id) = u16::try_from(g.id) else {
                continue;
            };
            let face_key = self.register_prepared_glyph_face(&g, face_usage);

            let (x, x_bin) = subpixel_bin_q4(g.x);
            let (y, y_bin) = subpixel_bin_y(g.y);

            let paint_span = prepared_glyph_paint_span(resolved_spans, &g);

            let size_bits = g.font_size.to_bits();
            let mut atlas_hit: Option<(GlyphKey, GlyphAtlasEntry)> = None;
            let color_key = GlyphKey {
                font: face_key,
                glyph_id: g.id,
                size_bits,
                x_bin,
                y_bin,
                kind: GlyphQuadKind::Color,
            };
            if let Some(entry) = self.color_atlas.get(color_key, epoch) {
                atlas_hit = Some((color_key, entry));
            } else {
                let subpixel_key = GlyphKey {
                    font: face_key,
                    glyph_id: g.id,
                    size_bits,
                    x_bin,
                    y_bin,
                    kind: GlyphQuadKind::Subpixel,
                };
                if let Some(entry) = self.subpixel_atlas.get(subpixel_key, epoch) {
                    atlas_hit = Some((subpixel_key, entry));
                } else {
                    let mask_key = GlyphKey {
                        font: face_key,
                        glyph_id: g.id,
                        size_bits,
                        x_bin,
                        y_bin,
                        kind: GlyphQuadKind::Mask,
                    };
                    if let Some(entry) = self.mask_atlas.get(mask_key, epoch) {
                        atlas_hit = Some((mask_key, entry));
                    }
                }
            }

            let (glyph_key, x0_px, y0_px, w_px, h_px) = if let Some((glyph_key, entry)) = atlas_hit
            {
                (
                    glyph_key,
                    x as f32 + entry.placement_left as f32,
                    y as f32 - entry.placement_top as f32,
                    entry.w as f32,
                    entry.h as f32,
                )
            } else {
                let Some(font_ref) =
                    parley::swash::FontRef::from_index(g.font.data.data(), g.font.index as usize)
                else {
                    continue;
                };

                let mut scaler_builder = self
                    .parley_scale
                    .builder(font_ref)
                    .size(g.font_size.max(1.0))
                    .hint(false);
                if !g.normalized_coords.is_empty() {
                    scaler_builder = scaler_builder.normalized_coords(g.normalized_coords.iter());
                }
                let mut scaler = scaler_builder.build();

                let offset_px = parley::swash::zeno::Vector::new(
                    subpixel_bin_as_float(x_bin),
                    subpixel_bin_as_float(y_bin),
                );

                let Some(image) = parley::swash::scale::Render::new(&[
                    parley::swash::scale::Source::ColorOutline(0),
                    parley::swash::scale::Source::ColorBitmap(
                        parley::swash::scale::StrikeWith::BestFit,
                    ),
                    parley::swash::scale::Source::Outline,
                ])
                .offset(offset_px)
                .render(&mut scaler, glyph_id) else {
                    continue;
                };

                if image.placement.width == 0 || image.placement.height == 0 {
                    continue;
                }

                let placement = image.placement;
                let (kind, bytes_per_pixel) = match image.content {
                    parley::swash::scale::image::Content::Mask => (GlyphQuadKind::Mask, 1),
                    parley::swash::scale::image::Content::Color => (GlyphQuadKind::Color, 4),
                    parley::swash::scale::image::Content::SubpixelMask => {
                        (GlyphQuadKind::Subpixel, 4)
                    }
                };

                let glyph_key = GlyphKey {
                    font: face_key,
                    glyph_id: g.id,
                    size_bits,
                    x_bin,
                    y_bin,
                    kind,
                };

                let data = image.data;
                match kind {
                    GlyphQuadKind::Mask => {
                        let _ = self.mask_atlas.get_or_insert(
                            glyph_key,
                            placement.width,
                            placement.height,
                            placement.left,
                            placement.top,
                            bytes_per_pixel,
                            data,
                            epoch,
                        );
                    }
                    GlyphQuadKind::Color => {
                        let _ = self.color_atlas.get_or_insert(
                            glyph_key,
                            placement.width,
                            placement.height,
                            placement.left,
                            placement.top,
                            bytes_per_pixel,
                            data,
                            epoch,
                        );
                    }
                    GlyphQuadKind::Subpixel => {
                        let _ = self.subpixel_atlas.get_or_insert(
                            glyph_key,
                            placement.width,
                            placement.height,
                            placement.left,
                            placement.top,
                            bytes_per_pixel,
                            data,
                            epoch,
                        );
                    }
                }

                (
                    glyph_key,
                    x as f32 + placement.left as f32,
                    y as f32 - placement.top as f32,
                    placement.width as f32,
                    placement.height as f32,
                )
            };

            glyphs.push(GlyphInstance {
                rect: [x0_px / scale, y0_px / scale, w_px / scale, h_px / scale],
                paint_span,
                key: glyph_key,
            });
        }
    }

    fn register_prepared_glyph_face(
        &mut self,
        glyph: &ParleyGlyph,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
    ) -> FontFaceKey {
        let font_data_id = glyph.font.data.id();
        let face_index = glyph.font.index;
        self.font_data_by_face
            .entry((font_data_id, face_index))
            .or_insert_with(|| glyph.font.clone());

        let face_key = FontFaceKey {
            font_data_id,
            face_index,
            variation_key: variation_key_from_normalized_coords(&glyph.normalized_coords),
            synthesis_embolden: glyph.synthesis.embolden(),
            synthesis_skew_degrees: glyph
                .synthesis
                .skew()
                .unwrap_or(0.0)
                .clamp(i8::MIN as f32, i8::MAX as f32) as i8,
        };
        if !glyph.normalized_coords.is_empty() {
            self.font_instance_coords_by_face
                .entry(face_key)
                .or_insert_with(|| glyph.normalized_coords.clone());
        }

        let usage = face_usage.entry(face_key).or_insert((0, 0));
        usage.0 = usage.0.saturating_add(1);
        if glyph.id == 0 {
            usage.1 = usage.1.saturating_add(1);
        }

        face_key
    }

    pub(super) fn maybe_record_font_trace_entry(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
        shape: &Arc<TextShape>,
    ) {
        let mut families: Vec<FontTraceFamilyResolved> =
            Vec::with_capacity(shape.font_faces.len().max(1));
        for usage in shape.font_faces.iter() {
            let family = self
                .family_name_for_face(usage.font_data_id, usage.face_index)
                .unwrap_or_else(|| {
                    format!(
                        "font_data_id={} face_index={}",
                        usage.font_data_id, usage.face_index
                    )
                });
            families.push(FontTraceFamilyResolved {
                family,
                glyphs: usage.glyphs,
                missing_glyphs: usage.missing_glyphs,
            });
        }
        self.font_trace.maybe_record(
            text,
            style,
            constraints,
            &self.fallback_policy,
            shape.missing_glyphs,
            families,
        );
    }

    pub(super) fn decoration_metrics_for_shape(
        &self,
        style: &TextStyle,
        scale: f32,
        shape: &Arc<TextShape>,
    ) -> Option<TextDecorationMetricsPx> {
        let usage = shape.font_faces.first()?;

        let face_key = FontFaceKey {
            font_data_id: usage.font_data_id,
            face_index: usage.face_index,
            variation_key: usage.variation_key,
            synthesis_embolden: usage.synthesis_embolden,
            synthesis_skew_degrees: usage.synthesis_skew_degrees,
        };

        let font_data = self
            .font_data_by_face
            .get(&(usage.font_data_id, usage.face_index))?;
        let coords: &[i16] = self
            .font_instance_coords_by_face
            .get(&face_key)
            .map(|v| v.as_ref())
            .unwrap_or(&[]);

        let ppem = style.size.0 * scale;
        fret_render_text::decorations::decoration_metrics_px_for_font_bytes(
            font_data.data.data(),
            usage.face_index,
            coords,
            ppem,
        )
    }

    fn family_name_for_face(&mut self, font_data_id: u64, face_index: u32) -> Option<String> {
        if let Some(name) = self
            .font_face_family_name_cache
            .get(&(font_data_id, face_index))
            .cloned()
        {
            return Some(name);
        }

        let font_data = self.font_data_by_face.get(&(font_data_id, face_index))?;
        let name = fret_render_text::font_names::best_family_name_from_font_bytes(
            font_data.data.data(),
            face_index,
        )?;
        self.font_face_family_name_cache
            .insert((font_data_id, face_index), name.clone());
        Some(name)
    }

    pub(super) fn wrap_for_prepare(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> crate::text::wrapper::WrappedLayout {
        crate::text::wrapper::wrap_with_constraints(&mut self.parley_shaper, input, constraints)
    }
}

fn prepared_glyph_paint_span(
    resolved_spans: Option<&[ResolvedSpan]>,
    glyph: &ParleyGlyph,
) -> Option<u16> {
    resolved_spans
        .and_then(|spans| paint_span_for_text_range(spans, &glyph.text_range, glyph.is_rtl))
}

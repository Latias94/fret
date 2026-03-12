use fret_core::{
    TextBlobId, TextConstraints, TextInputRef, TextMetrics, TextSpan, TextStyle, geometry::Px,
};
use slotmap::SlotMap;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    sync::Arc,
};

use fret_render_text::cache_keys::{TextBlobKey, TextShapeKey};
use fret_render_text::decorations::decorations_for_lines;

#[cfg(test)]
use fret_render_text::cache_keys::spans_paint_fingerprint;
pub use fret_render_text::decorations::{TextDecoration, TextDecorationKind};
pub(crate) use fret_render_text::effective_text_scale_factor;
use fret_render_text::fallback_policy::TextFallbackPolicyV1;
use fret_render_text::font_instance_key::{FontFaceKey, variation_key_from_normalized_coords};
use fret_render_text::font_stack::GenericFamilyInjectionState;
use fret_render_text::font_trace::FontTraceState;
use fret_render_text::measure::TextMeasureCaches;
use fret_render_text::spans::{paint_span_for_text_range, resolve_spans_for_text};
pub use fret_render_text::{
    FontCatalogEntryMetadata, SystemFontRescanResult, SystemFontRescanSeed,
};

mod atlas;
mod blobs;
mod diagnostics;
mod fonts;
mod measure;
mod prepare;
mod quality;
mod queries;

use self::atlas::{
    GlyphAtlas, GlyphAtlasEntry, GlyphKey, TEXT_ATLAS_MAX_PAGES, subpixel_bin_as_float,
    subpixel_bin_q4, subpixel_bin_y,
};
pub use self::quality::TextQualitySettings;
use self::quality::TextQualityState;

pub(crate) mod parley_shaper {
    pub use fret_render_text::parley_shaper::*;
}

pub(crate) mod wrapper {
    pub use fret_render_text::wrapper::*;
}

#[derive(Debug, Clone)]
pub struct GlyphInstance {
    /// Logical-space rect relative to the text baseline origin.
    pub rect: [f32; 4],
    pub paint_span: Option<u16>,
    key: GlyphKey,
}

impl GlyphInstance {
    pub fn kind(&self) -> GlyphQuadKind {
        self.key.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphQuadKind {
    Mask,
    Color,
    Subpixel,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub(crate) struct DebugGlyphAtlasLookup {
    pub(crate) font_data_id: u64,
    pub(crate) face_index: u32,
    pub(crate) variation_key: u64,
    pub(crate) synthesis_embolden: bool,
    pub(crate) synthesis_skew_degrees: i8,
    pub(crate) glyph_id: u32,
    pub(crate) size_bits: u32,
    pub(crate) x_bin: u8,
    pub(crate) y_bin: u8,
    pub(crate) kind: &'static str,
}

#[derive(Debug, Clone)]
pub struct TextBlob {
    pub shape: Arc<TextShape>,
    pub paint_palette: Option<Arc<[Option<fret_core::Color>]>>,
    pub decorations: Arc<[TextDecoration]>,
    ref_count: u32,
}

#[derive(Debug, Clone)]
pub struct TextShape {
    pub glyphs: Arc<[GlyphInstance]>,
    pub metrics: TextMetrics,
    pub lines: Arc<[TextLine]>,
    pub caret_stops: Arc<[(usize, Px)]>,
    pub missing_glyphs: u32,
    pub(crate) font_faces: Arc<[TextFontFaceUsage]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TextFontFaceUsage {
    pub(crate) font_data_id: u64,
    pub(crate) face_index: u32,
    pub(crate) variation_key: u64,
    pub(crate) synthesis_embolden: bool,
    /// Faux italic/oblique skew in degrees (fontique synthesis), applied at rasterization time.
    pub(crate) synthesis_skew_degrees: i8,
    pub(crate) glyphs: u32,
    pub(crate) missing_glyphs: u32,
}

pub use fret_render_text::line_layout::TextLineLayout as TextLine;

#[allow(dead_code)]
fn subpixel_mask_to_alpha(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 4);
    for rgba in data.chunks_exact(4) {
        out.push(rgba[0].max(rgba[1]).max(rgba[2]));
    }
    out
}

pub struct TextSystem {
    parley_shaper: crate::text::parley_shaper::ParleyShaper,
    parley_scale: parley::swash::scale::ScaleContext,
    font_stack_key: u64,
    font_db_revision: u64,
    fallback_policy: TextFallbackPolicyV1,
    quality: TextQualityState,
    generic_injections: GenericFamilyInjectionState,

    blobs: SlotMap<TextBlobId, TextBlob>,
    blob_cache: HashMap<TextBlobKey, TextBlobId>,
    blob_key_by_id: HashMap<TextBlobId, TextBlobKey>,
    released_blob_lru: VecDeque<TextBlobId>,
    released_blob_set: HashSet<TextBlobId>,
    shape_cache: HashMap<TextShapeKey, Arc<TextShape>>,
    measure: TextMeasureCaches,

    mask_atlas: GlyphAtlas,
    color_atlas: GlyphAtlas,
    subpixel_atlas: GlyphAtlas,
    atlas_bind_group_layout: wgpu::BindGroupLayout,

    text_pin_mask: Vec<Vec<GlyphKey>>,
    text_pin_color: Vec<Vec<GlyphKey>>,
    text_pin_subpixel: Vec<Vec<GlyphKey>>,
    font_data_by_face: HashMap<(u64, u32), parley::FontData>,
    font_instance_coords_by_face: HashMap<FontFaceKey, Arc<[i16]>>,
    font_face_family_name_cache: HashMap<(u64, u32), String>,

    perf_frame_cache_resets: u64,
    perf_frame_blob_cache_hits: u64,
    perf_frame_blob_cache_misses: u64,
    perf_frame_blobs_created: u64,
    perf_frame_shape_cache_hits: u64,
    perf_frame_shape_cache_misses: u64,
    perf_frame_shapes_created: u64,
    perf_frame_missing_glyphs: u64,
    perf_frame_texts_with_missing_glyphs: u64,

    glyph_atlas_epoch: u64,

    font_trace: FontTraceState,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TextAtlasPerfSnapshot {
    pub(crate) uploads: u64,
    pub(crate) upload_bytes: u64,
    pub(crate) evicted_glyphs: u64,
    pub(crate) evicted_pages: u64,
    pub(crate) evicted_page_glyphs: u64,
    pub(crate) resets: u64,
}

pub type TextFontFamilyConfig = fret_core::TextFontFamilyConfig;

impl TextSystem {
    pub fn new(device: &wgpu::Device) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret glyph atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret glyph atlas bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
            });

        let mask_atlas = GlyphAtlas::new(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph mask atlas",
            atlas_width,
            atlas_height,
            wgpu::TextureFormat::R8Unorm,
            0,
            TEXT_ATLAS_MAX_PAGES,
        );
        let color_atlas = GlyphAtlas::new(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph color atlas",
            atlas_width,
            atlas_height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            0,
            TEXT_ATLAS_MAX_PAGES,
        );
        let subpixel_atlas = GlyphAtlas::new(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph subpixel atlas",
            atlas_width,
            atlas_height,
            wgpu::TextureFormat::Rgba8Unorm,
            0,
            TEXT_ATLAS_MAX_PAGES,
        );

        let parley_shaper = crate::text::parley_shaper::ParleyShaper::new();

        let fallback_policy = TextFallbackPolicyV1::new(&parley_shaper);

        let mut out = Self {
            parley_shaper,
            parley_scale: parley::swash::scale::ScaleContext::new(),
            // Non-zero by default so callers can treat `0` as "unknown/uninitialized" if desired.
            font_stack_key: 1,
            font_db_revision: 1,
            fallback_policy,
            quality: TextQualityState::new(TextQualitySettings::default()),
            generic_injections: GenericFamilyInjectionState::default(),

            blobs: SlotMap::with_key(),
            blob_cache: HashMap::new(),
            blob_key_by_id: HashMap::new(),
            released_blob_lru: VecDeque::new(),
            released_blob_set: HashSet::new(),
            shape_cache: HashMap::new(),
            measure: TextMeasureCaches::new(),

            mask_atlas,
            color_atlas,
            subpixel_atlas,
            atlas_bind_group_layout,

            text_pin_mask: vec![Vec::new(); 3],
            text_pin_color: vec![Vec::new(); 3],
            text_pin_subpixel: vec![Vec::new(); 3],
            font_data_by_face: HashMap::new(),
            font_instance_coords_by_face: HashMap::new(),
            font_face_family_name_cache: HashMap::new(),

            perf_frame_cache_resets: 0,
            perf_frame_blob_cache_hits: 0,
            perf_frame_blob_cache_misses: 0,
            perf_frame_blobs_created: 0,
            perf_frame_shape_cache_hits: 0,
            perf_frame_shape_cache_misses: 0,
            perf_frame_shapes_created: 0,
            perf_frame_missing_glyphs: 0,
            perf_frame_texts_with_missing_glyphs: 0,

            glyph_atlas_epoch: 1,

            font_trace: FontTraceState::default(),
        };

        let _ = out.apply_font_families_inner(&out.fallback_policy.font_family_config.clone());
        out.fallback_policy.recompute_key(&out.parley_shaper);
        out.recompute_font_stack_key();
        out
    }

    fn prepare_with_key(
        &mut self,
        mut key: TextBlobKey,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let text = key.text.clone();
        key.backend = 1;

        let scale = effective_text_scale_factor(constraints.scale_factor);
        let snap_vertical = scale.fract().abs() > 1e-4;

        if let Some(id) = self.blob_cache.get(&key).copied() {
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
                self.maybe_record_font_trace_entry(text.as_ref(), style, constraints, &shape);
                return (id, metrics);
            }

            // Stale cache entry (shouldn't happen, but keep it robust).
            self.blob_cache.remove(&key);
            self.blob_key_by_id.remove(&id);
        }
        self.perf_frame_blob_cache_misses = self.perf_frame_blob_cache_misses.saturating_add(1);

        let resolved_spans = spans.and_then(|spans| resolve_spans_for_text(text.as_ref(), spans));
        let paint_palette = resolved_spans.as_ref().map(|spans| {
            let mut palette: Vec<Option<fret_core::Color>> = Vec::with_capacity(spans.len());
            palette.extend(spans.iter().map(|s| s.fg));
            Arc::<[Option<fret_core::Color>]>::from(palette)
        });

        let shape_key = TextShapeKey::from_blob_key(&key);
        let shape = if let Some(shape) = self.shape_cache.get(&shape_key) {
            self.perf_frame_shape_cache_hits = self.perf_frame_shape_cache_hits.saturating_add(1);
            shape.clone()
        } else {
            self.perf_frame_shape_cache_misses =
                self.perf_frame_shape_cache_misses.saturating_add(1);
            let shape = {
                let input = match spans {
                    Some(spans) => TextInputRef::Attributed {
                        text: text.as_ref(),
                        base: style,
                        spans,
                    },
                    None => TextInputRef::Plain {
                        text: text.as_ref(),
                        style,
                    },
                };
                let wrapped = self.wrap_for_prepare(input, constraints);
                let epoch = {
                    let e = self.glyph_atlas_epoch;
                    self.glyph_atlas_epoch = self.glyph_atlas_epoch.saturating_add(1);
                    e
                };

                let mut glyphs: Vec<GlyphInstance> = Vec::new();
                let mut face_usage: HashMap<FontFaceKey, (u32, u32)> = HashMap::new();
                let mut lines: Vec<TextLine> = Vec::new();

                let (metrics, missing_glyphs, first_line_caret_stops) = {
                    let prepared = fret_render_text::prepare_layout::prepare_layout_from_wrapped(
                        text.as_ref(),
                        wrapped,
                        constraints,
                        scale,
                        snap_vertical,
                    );

                    let metrics = prepared.metrics;
                    let missing_glyphs = prepared.missing_glyphs;
                    let first_line_caret_stops = prepared.first_line_caret_stops;

                    lines.reserve(prepared.lines.len().max(1));
                    for prepared_line in prepared.lines {
                        lines.push(prepared_line.layout);

                        for g in prepared_line.glyphs {
                            let Ok(glyph_id) = u16::try_from(g.id) else {
                                continue;
                            };
                            let font_data_id = g.font.data.id();
                            let face_index = g.font.index;
                            self.font_data_by_face
                                .entry((font_data_id, face_index))
                                .or_insert_with(|| g.font.clone());

                            let variation_key =
                                variation_key_from_normalized_coords(&g.normalized_coords);
                            let synthesis_embolden = g.synthesis.embolden();
                            let synthesis_skew_degrees = g
                                .synthesis
                                .skew()
                                .unwrap_or(0.0)
                                .clamp(i8::MIN as f32, i8::MAX as f32)
                                as i8;

                            let face_key = FontFaceKey {
                                font_data_id,
                                face_index,
                                variation_key,
                                synthesis_embolden,
                                synthesis_skew_degrees,
                            };
                            if !g.normalized_coords.is_empty() {
                                self.font_instance_coords_by_face
                                    .entry(face_key)
                                    .or_insert_with(|| g.normalized_coords.clone());
                            }

                            let usage = face_usage.entry(face_key).or_insert((0, 0));
                            usage.0 = usage.0.saturating_add(1);
                            if g.id == 0 {
                                usage.1 = usage.1.saturating_add(1);
                            }

                            let (x, x_bin) = subpixel_bin_q4(g.x);
                            let (y, y_bin) = subpixel_bin_y(g.y);

                            let paint_span = resolved_spans.as_deref().and_then(|spans| {
                                paint_span_for_text_range(spans, &g.text_range, g.is_rtl)
                            });

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

                            let (glyph_key, x0_px, y0_px, w_px, h_px) =
                                if let Some((glyph_key, entry)) = atlas_hit {
                                    (
                                        glyph_key,
                                        x as f32 + entry.placement_left as f32,
                                        y as f32 - entry.placement_top as f32,
                                        entry.w as f32,
                                        entry.h as f32,
                                    )
                                } else {
                                    let Some(font_ref) = parley::swash::FontRef::from_index(
                                        g.font.data.data(),
                                        g.font.index as usize,
                                    ) else {
                                        continue;
                                    };

                                    let mut scaler_builder = self
                                        .parley_scale
                                        .builder(font_ref)
                                        .size(g.font_size.max(1.0))
                                        .hint(false);
                                    if !g.normalized_coords.is_empty() {
                                        scaler_builder = scaler_builder
                                            .normalized_coords(g.normalized_coords.iter());
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
                                        parley::swash::scale::image::Content::Mask => {
                                            (GlyphQuadKind::Mask, 1)
                                        }
                                        parley::swash::scale::image::Content::Color => {
                                            (GlyphQuadKind::Color, 4)
                                        }
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

                    (metrics, missing_glyphs, first_line_caret_stops)
                };

                #[cfg(any())]
                let _metrics_legacy = {
                    let kept_end = wrapped.kept_end;

                    let first_baseline_px = wrapped
                        .lines
                        .first()
                        .map(|l| l.baseline.max(0.0))
                        .unwrap_or(0.0);
                    let first_baseline_px = if snap_vertical
                        && let Some(first) = wrapped.lines.first()
                    {
                        let top_px = 0.0_f32;
                        let bottom_px = (top_px + first.line_height.max(0.0)).round().max(top_px);
                        let height_px = (bottom_px - top_px).max(0.0);
                        (top_px + first.baseline.max(0.0))
                            .round()
                            .clamp(top_px, top_px + height_px)
                    } else {
                        first_baseline_px
                    };

                    let metrics = metrics_from_wrapped_lines(&wrapped.lines, scale);
                    lines.reserve(wrapped.lines.len().max(1));

                    for (i, (range, line)) in wrapped
                        .line_ranges
                        .iter()
                        .cloned()
                        .zip(wrapped.lines.into_iter())
                        .enumerate()
                    {
                        if snap_vertical {
                            line_top_px = line_top_px.round();
                        }

                        let line_height_px_raw = line.line_height.max(0.0);
                        let line_baseline_px_raw = line.baseline.max(0.0);

                        let (line_height_px, baseline_pos_px) = if snap_vertical {
                            let bottom_px =
                                (line_top_px + line_height_px_raw).round().max(line_top_px);
                            let height_px = (bottom_px - line_top_px).max(0.0);
                            let baseline_pos_px = (line_top_px + line_baseline_px_raw)
                                .round()
                                .clamp(line_top_px, line_top_px + height_px);
                            (height_px, baseline_pos_px)
                        } else {
                            (line_height_px_raw, line_top_px + line_baseline_px_raw)
                        };

                        let line_offset_px = baseline_pos_px - first_baseline_px;

                        let slice = &text[range.clone()];
                        let (line_min_x_px, line_max_x_px) = shaped_line_visual_x_bounds_px(&line);
                        let line_visual_width_px = (line_max_x_px - line_min_x_px).max(0.0);
                        let line_align_offset_px =
                            align_offset_px_for_line(line_min_x_px, line_visual_width_px);
                        let line_align_offset = Px(line_align_offset_px / scale);

                        let clusters: Arc<[fret_render_text::geometry::TextLineCluster]> =
                            if line.clusters.is_empty() {
                                Arc::from([])
                            } else {
                                let mut out: Vec<fret_render_text::geometry::TextLineCluster> =
                                    Vec::with_capacity(line.clusters.len());
                                for c in &line.clusters {
                                    let start = (range.start + c.text_range.start).min(kept_end);
                                    let end = (range.start + c.text_range.end).min(kept_end);
                                    if start >= end {
                                        continue;
                                    }

                                    let x0 = ((c.x0 + line_align_offset_px) / scale).max(0.0);
                                    let x1 = ((c.x1 + line_align_offset_px) / scale).max(0.0);
                                    let x0 = if x0.is_finite() { Px(x0) } else { Px(0.0) };
                                    let x1 = if x1.is_finite() { Px(x1) } else { Px(0.0) };

                                    out.push(fret_render_text::geometry::TextLineCluster {
                                        text_range: start..end,
                                        x0,
                                        x1,
                                        is_rtl: c.is_rtl,
                                    });
                                }
                                Arc::from(out)
                            };

                        let mut caret_stops = caret_stops_for_slice(
                            slice,
                            range.start,
                            &line.clusters,
                            line_visual_width_px.max(0.0),
                            scale,
                            kept_end,
                        );
                        if line_align_offset.0 != 0.0 {
                            for (_, x) in caret_stops.iter_mut() {
                                *x = Px(x.0 + line_align_offset.0);
                            }
                        }
                        if i == 0 {
                            first_line_caret_stops = caret_stops.clone();
                        }

                        lines.push(TextLine::new(
                            range.start,
                            range.end.min(kept_end),
                            Px((line_visual_width_px / scale).max(0.0)),
                            Px((line_top_px / scale).max(0.0)),
                            Px((baseline_pos_px / scale).max(0.0)),
                            Px(((line_height_px / scale).max(0.0)).max(1.0)),
                            Px((line.ascent.abs().max(0.0) / scale).max(0.0)),
                            Px((line.descent.abs().max(0.0) / scale).max(0.0)),
                            caret_stops,
                            clusters,
                        ));

                        for g in line.glyphs {
                            let Ok(glyph_id) = u16::try_from(g.id) else {
                                continue;
                            };
                            let font_data_id = g.font.data.id();
                            let face_index = g.font.index;
                            self.font_data_by_face
                                .entry((font_data_id, face_index))
                                .or_insert_with(|| g.font.clone());
                            let variation_key =
                                variation_key_from_normalized_coords(&g.normalized_coords);
                            let synthesis_embolden = g.synthesis.embolden();
                            let synthesis_skew_degrees = g
                                .synthesis
                                .skew()
                                .unwrap_or(0.0)
                                .clamp(i8::MIN as f32, i8::MAX as f32)
                                as i8;
                            let face_key = FontFaceKey {
                                font_data_id,
                                face_index,
                                variation_key,
                                synthesis_embolden,
                                synthesis_skew_degrees,
                            };
                            if !g.normalized_coords.is_empty() {
                                self.font_instance_coords_by_face
                                    .entry(face_key)
                                    .or_insert_with(|| g.normalized_coords.clone());
                            }

                            let usage = face_usage.entry(face_key).or_insert((0, 0));
                            usage.0 = usage.0.saturating_add(1);
                            if g.id == 0 {
                                missing_glyphs = missing_glyphs.saturating_add(1);
                                usage.1 = usage.1.saturating_add(1);
                            }

                            let pos_y = g.y + line_offset_px;
                            let (x, x_bin) = subpixel_bin_q4(g.x + line_align_offset_px);
                            let (y, y_bin) = subpixel_bin_y(pos_y);

                            let text_range = (range.start + g.text_range.start)
                                ..(range.start + g.text_range.end);
                            let paint_span = resolved_spans.as_deref().and_then(|spans| {
                                paint_span_for_text_range(spans, &text_range, g.is_rtl)
                            });

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

                            let (glyph_key, x0_px, y0_px, w_px, h_px) =
                                if let Some((glyph_key, entry)) = atlas_hit {
                                    (
                                        glyph_key,
                                        x as f32 + entry.placement_left as f32,
                                        y as f32 - entry.placement_top as f32,
                                        entry.w as f32,
                                        entry.h as f32,
                                    )
                                } else {
                                    let Some(font_ref) = parley::swash::FontRef::from_index(
                                        g.font.data.data(),
                                        g.font.index as usize,
                                    ) else {
                                        continue;
                                    };

                                    let mut scaler_builder = self
                                        .parley_scale
                                        .builder(font_ref)
                                        .size(g.font_size.max(1.0))
                                        .hint(false);
                                    if !g.normalized_coords.is_empty() {
                                        scaler_builder = scaler_builder
                                            .normalized_coords(g.normalized_coords.iter());
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
                                        parley::swash::scale::image::Content::Mask => {
                                            (GlyphQuadKind::Mask, 1)
                                        }
                                        parley::swash::scale::image::Content::Color => {
                                            (GlyphQuadKind::Color, 4)
                                        }
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

                        line_top_px += line_height_px;
                    }

                    metrics
                    /* legacy: removed UnwrappedWordLtr fast-path
                    WrappedForPrepare::UnwrappedWordLtr {
                        kept_end,
                        unwrapped,
                        lines: slices,
                        ..
                    } => {
                        let first_baseline_px = unwrapped.baseline.max(0.0);
                        let first_baseline_px = if snap_vertical {
                            let top_px = 0.0_f32;
                            let bottom_px = (top_px + unwrapped.line_height.max(0.0))
                                .round()
                                .max(top_px);
                            let height_px = (bottom_px - top_px).max(0.0);
                            (top_px + unwrapped.baseline.max(0.0))
                                .round()
                                .clamp(top_px, top_px + height_px)
                        } else {
                            first_baseline_px
                        };

                        let mut max_w_px = 0.0_f32;
                        for s in &slices {
                            max_w_px = max_w_px.max(s.width_px.max(0.0));
                        }
                        let metrics = metrics_for_uniform_lines(
                            max_w_px,
                            slices.len().max(1),
                            unwrapped.baseline.max(0.0),
                            unwrapped.line_height.max(0.0),
                            scale,
                        );

                        lines.reserve(slices.len().max(1));

                        for (i, s) in slices.into_iter().enumerate() {
                            if snap_vertical {
                                line_top_px = line_top_px.round();
                            }

                            let line_height_px_raw = unwrapped.line_height.max(0.0);
                            let line_baseline_px_raw = unwrapped.baseline.max(0.0);

                            let (line_height_px, baseline_pos_px) = if snap_vertical {
                                let bottom_px =
                                    (line_top_px + line_height_px_raw).round().max(line_top_px);
                                let height_px = (bottom_px - line_top_px).max(0.0);
                                let baseline_pos_px = (line_top_px + line_baseline_px_raw)
                                    .round()
                                    .clamp(line_top_px, line_top_px + height_px);
                                (height_px, baseline_pos_px)
                            } else {
                                (line_height_px_raw, line_top_px + line_baseline_px_raw)
                            };

                            let line_offset_px = baseline_pos_px - first_baseline_px;

                            let slice = &text[s.range.clone()];
                            let line_align_offset_px =
                                align_offset_px_for_line(0.0, s.width_px.max(0.0));
                            let line_align_offset = Px(line_align_offset_px / scale);

                            let mut caret_stops = caret_stops_for_slice_from_unwrapped_ltr(
                                slice,
                                s.range.start,
                                &unwrapped.clusters,
                                s.cluster_range.clone(),
                                s.line_start_x,
                                s.width_px.max(0.0),
                                scale,
                                kept_end,
                            );
                            if line_align_offset.0 != 0.0 {
                                for (_, x) in caret_stops.iter_mut() {
                                    *x = Px(x.0 + line_align_offset.0);
                                }
                            }
                            if i == 0 {
                                first_line_caret_stops = caret_stops.clone();
                            }

                            lines.push(TextLine {
                                start: s.range.start,
                                end: s.range.end.min(kept_end),
                                width: Px((s.width_px / scale).max(0.0)),
                                y_top: Px((line_top_px / scale).max(0.0)),
                                y_baseline: Px((baseline_pos_px / scale).max(0.0)),
                                height: Px(((line_height_px / scale).max(0.0)).max(1.0)),
                                ascent: Px((unwrapped.ascent.abs().max(0.0) / scale).max(0.0)),
                                descent: Px((unwrapped.descent.abs().max(0.0) / scale).max(0.0)),
                                caret_stops,
                            });

                            for g in unwrapped.glyphs[s.glyph_range.clone()].iter() {
                                let Ok(glyph_id) = u16::try_from(g.id) else {
                                    continue;
                                };
                                let font_data_id = g.font.data.id();
                                let face_index = g.font.index;
                                self.font_data_by_face
                                    .entry((font_data_id, face_index))
                                    .or_insert_with(|| g.font.clone());
                                let variation_key =
                                    variation_key_from_normalized_coords(&g.normalized_coords);
                                let synthesis_embolden = g.synthesis.embolden();
                                let synthesis_skew_degrees = g
                                    .synthesis
                                    .skew()
                                    .unwrap_or(0.0)
                                    .clamp(i8::MIN as f32, i8::MAX as f32)
                                    as i8;
                                let face_key = FontFaceKey {
                                    font_data_id,
                                    face_index,
                                    variation_key,
                                    synthesis_embolden,
                                    synthesis_skew_degrees,
                                };
                                if !g.normalized_coords.is_empty() {
                                    self.font_instance_coords_by_face
                                        .entry(face_key)
                                        .or_insert_with(|| g.normalized_coords.clone());
                                }

                                let usage = face_usage.entry(face_key).or_insert((0, 0));
                                usage.0 = usage.0.saturating_add(1);
                                if g.id == 0 {
                                    missing_glyphs = missing_glyphs.saturating_add(1);
                                    usage.1 = usage.1.saturating_add(1);
                                }

                                let pos_y = g.y + line_offset_px;
                                let x = g.x - s.line_start_x + line_align_offset_px;
                                let (x, x_bin) = subpixel_bin_q4(x);
                                let (y, y_bin) = subpixel_bin_y(pos_y);

                                let text_range = g.text_range.clone();
                                let paint_span = resolved_spans.as_deref().and_then(|spans| {
                                    paint_span_for_text_range(spans, &text_range, g.is_rtl)
                                });

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
                                    if let Some(entry) =
                                        self.subpixel_atlas.get(subpixel_key, epoch)
                                    {
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

                                let (glyph_key, x0_px, y0_px, w_px, h_px) =
                                    if let Some((glyph_key, entry)) = atlas_hit {
                                        (
                                            glyph_key,
                                            x as f32 + entry.placement_left as f32,
                                            y as f32 - entry.placement_top as f32,
                                            entry.w as f32,
                                            entry.h as f32,
                                        )
                                    } else {
                                        let Some(font_ref) = parley::swash::FontRef::from_index(
                                            g.font.data.data(),
                                            g.font.index as usize,
                                        ) else {
                                            continue;
                                        };

                                        let mut scaler_builder = self
                                            .parley_scale
                                            .builder(font_ref)
                                            .size(g.font_size.max(1.0))
                                            .hint(false);
                                        if !g.normalized_coords.is_empty() {
                                            scaler_builder = scaler_builder
                                                .normalized_coords(g.normalized_coords.iter());
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

                                        if image.placement.width == 0 || image.placement.height == 0
                                        {
                                            continue;
                                        }

                                        let placement = image.placement;
                                        let (kind, bytes_per_pixel) = match image.content {
                                            parley::swash::scale::image::Content::Mask => {
                                                (GlyphQuadKind::Mask, 1)
                                            }
                                            parley::swash::scale::image::Content::Color => {
                                                (GlyphQuadKind::Color, 4)
                                            }
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
                                    rect: [
                                        x0_px / scale,
                                        y0_px / scale,
                                        w_px / scale,
                                        h_px / scale,
                                    ],
                                    paint_span,
                                    key: glyph_key,
                                });
                            }

                            line_top_px += line_height_px;
                        }

                        metrics
                    */
                };

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
            };
            self.perf_frame_shapes_created = self.perf_frame_shapes_created.saturating_add(1);
            self.shape_cache.insert(shape_key.clone(), shape.clone());
            shape
        };

        let decoration_metrics = self.decoration_metrics_for_shape(style, scale, &shape);
        let decorations: Vec<TextDecoration> = resolved_spans
            .as_deref()
            .map(|spans| {
                decorations_for_lines(
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
        self.maybe_record_font_trace_entry(text.as_ref(), style, constraints, &shape);
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
}

#[cfg(test)]
mod tests;

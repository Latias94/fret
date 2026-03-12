use fret_core::scene::{Scene, SceneOp};
use fret_core::{
    AttributedText, CaretAffinity, HitTestResult, Point, Rect, TextBlobId, TextConstraints,
    TextInputRef, TextMetrics, TextSpan, TextStyle, geometry::Px,
};
use slotmap::SlotMap;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    sync::Arc,
};

use fret_render_text::cache_keys::{TextBlobKey, TextShapeKey};
use fret_render_text::decorations::{TextDecorationMetricsPx, decorations_for_lines};

#[cfg(test)]
use fret_render_text::cache_keys::spans_paint_fingerprint;
pub use fret_render_text::decorations::{TextDecoration, TextDecorationKind};
pub(crate) use fret_render_text::effective_text_scale_factor;
use fret_render_text::fallback_policy::TextFallbackPolicyV1;
use fret_render_text::font_instance_key::{FontFaceKey, variation_key_from_normalized_coords};
use fret_render_text::font_stack::GenericFamilyInjectionState;
use fret_render_text::font_trace::{FontTraceFamilyResolved, FontTraceState};
use fret_render_text::measure::TextMeasureCaches;

use fret_render_text::spans::{
    paint_span_for_text_range, resolve_spans_for_text, sanitize_spans_for_text,
};
pub use fret_render_text::{
    FontCatalogEntryMetadata, SystemFontRescanResult, SystemFontRescanSeed,
};

mod atlas;
mod diagnostics;
mod fonts;
mod quality;

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

    pub fn atlas_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.atlas_bind_group_layout
    }

    pub fn mask_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.mask_atlas.bind_group(page)
    }

    pub fn color_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.color_atlas.bind_group(page)
    }

    pub fn subpixel_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.subpixel_atlas.bind_group(page)
    }

    pub fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        self.mask_atlas.flush_uploads(queue);
        self.color_atlas.flush_uploads(queue);
        self.subpixel_atlas.flush_uploads(queue);
    }

    pub fn prepare_for_scene(&mut self, scene: &Scene, frame_index: u64) {
        let ring_len = self
            .text_pin_mask
            .len()
            .min(self.text_pin_color.len())
            .min(self.text_pin_subpixel.len());
        if ring_len == 0 {
            return;
        }
        let bucket = (frame_index as usize) % ring_len;

        let old_mask = std::mem::take(&mut self.text_pin_mask[bucket]);
        let old_color = std::mem::take(&mut self.text_pin_color[bucket]);
        let old_subpixel = std::mem::take(&mut self.text_pin_subpixel[bucket]);
        self.mask_atlas.dec_live_refs(&old_mask);
        self.color_atlas.dec_live_refs(&old_color);
        self.subpixel_atlas.dec_live_refs(&old_subpixel);

        let mut mask_keys: HashSet<GlyphKey> = HashSet::new();
        let mut color_keys: HashSet<GlyphKey> = HashSet::new();
        let mut subpixel_keys: HashSet<GlyphKey> = HashSet::new();

        for op in scene.ops() {
            let SceneOp::Text { text, .. } = *op else {
                continue;
            };
            let Some(blob) = self.blobs.get(text) else {
                continue;
            };
            for glyph in blob.shape.glyphs.as_ref() {
                match glyph.kind() {
                    GlyphQuadKind::Mask => {
                        mask_keys.insert(glyph.key);
                    }
                    GlyphQuadKind::Color => {
                        color_keys.insert(glyph.key);
                    }
                    GlyphQuadKind::Subpixel => {
                        subpixel_keys.insert(glyph.key);
                    }
                }
            }
        }

        let epoch = frame_index;
        let mut new_mask: Vec<GlyphKey> = mask_keys.into_iter().collect();
        let mut new_color: Vec<GlyphKey> = color_keys.into_iter().collect();
        let mut new_subpixel: Vec<GlyphKey> = subpixel_keys.into_iter().collect();

        for &key in &new_mask {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_color {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_subpixel {
            self.ensure_glyph_in_atlas(key, epoch);
        }

        self.mask_atlas.inc_live_refs(&new_mask);
        self.color_atlas.inc_live_refs(&new_color);
        self.subpixel_atlas.inc_live_refs(&new_subpixel);

        self.text_pin_mask[bucket].append(&mut new_mask);
        self.text_pin_color[bucket].append(&mut new_color);
        self.text_pin_subpixel[bucket].append(&mut new_subpixel);
    }

    fn ensure_glyph_in_atlas(&mut self, key: GlyphKey, epoch: u64) {
        let already_present = match key.kind {
            GlyphQuadKind::Mask => self.mask_atlas.get(key, epoch).is_some(),
            GlyphQuadKind::Color => self.color_atlas.get(key, epoch).is_some(),
            GlyphQuadKind::Subpixel => self.subpixel_atlas.get(key, epoch).is_some(),
        };
        if already_present {
            return;
        }

        self.ensure_parley_glyph(key, epoch);
    }

    fn ensure_parley_glyph(&mut self, key: GlyphKey, epoch: u64) {
        let Some(font_data) = self
            .font_data_by_face
            .get(&(key.font.font_data_id, key.font.face_index))
        else {
            return;
        };

        let Some(font_ref) =
            parley::swash::FontRef::from_index(font_data.data.data(), key.font.face_index as usize)
        else {
            return;
        };
        let Ok(glyph_id) = u16::try_from(key.glyph_id) else {
            return;
        };

        let font_size = f32::from_bits(key.size_bits).max(1.0);
        let mut scaler_builder = self
            .parley_scale
            .builder(font_ref)
            .size(font_size)
            .hint(false);
        if let Some(coords) = self.font_instance_coords_by_face.get(&key.font) {
            scaler_builder = scaler_builder.normalized_coords(coords.iter());
        }
        let mut scaler = scaler_builder.build();

        let offset_px = parley::swash::zeno::Vector::new(
            subpixel_bin_as_float(key.x_bin),
            subpixel_bin_as_float(key.y_bin),
        );
        let mut render = parley::swash::scale::Render::new(&[
            parley::swash::scale::Source::ColorOutline(0),
            parley::swash::scale::Source::ColorBitmap(parley::swash::scale::StrikeWith::BestFit),
            parley::swash::scale::Source::Outline,
        ]);
        render.offset(offset_px);

        if key.font.synthesis_embolden {
            // `fontique::Synthesis::embolden` is boolean; pick a conservative strength in px.
            // This is renderer-internal and should only affect raster output + cache identity.
            let strength = (font_size / 48.0).clamp(0.25, 1.0);
            render.embolden(strength);
        }

        if key.font.synthesis_skew_degrees != 0 {
            let angle =
                parley::swash::zeno::Angle::from_degrees(key.font.synthesis_skew_degrees as f32);
            let t = parley::swash::zeno::Transform::skew(angle, parley::swash::zeno::Angle::ZERO);
            render.transform(Some(t));
        }

        let Some(image) = render.render(&mut scaler, glyph_id) else {
            return;
        };
        if image.placement.width == 0 || image.placement.height == 0 {
            return;
        }

        let (image_kind, bytes_per_pixel) = match image.content {
            parley::swash::scale::image::Content::Mask => (GlyphQuadKind::Mask, 1),
            parley::swash::scale::image::Content::Color => (GlyphQuadKind::Color, 4),
            parley::swash::scale::image::Content::SubpixelMask => (GlyphQuadKind::Subpixel, 4),
        };
        if image_kind != key.kind {
            return;
        }

        let data = image.data;

        match key.kind {
            GlyphQuadKind::Mask => {
                let _ = self.mask_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
            GlyphQuadKind::Color => {
                let _ = self.color_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
            GlyphQuadKind::Subpixel => {
                let _ = self.subpixel_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
        }
    }

    pub fn blob(&self, id: TextBlobId) -> Option<&TextBlob> {
        self.blobs.get(id)
    }

    #[allow(dead_code)]
    pub fn prepare_input(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        match input {
            TextInputRef::Plain { text, style } => self.prepare(text, style, constraints),
            TextInputRef::Attributed { text, base, spans } => {
                let spans = sanitize_spans_for_text(text, spans);
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
        let key = TextBlobKey::new(text, style, constraints, self.font_stack_key);
        self.prepare_with_key(key, style, None, constraints)
    }

    pub fn prepare_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let spans = sanitize_spans_for_text(rich.text.as_ref(), rich.spans.as_ref());
        if spans.is_none() {
            return self.prepare(rich.text.as_ref(), base_style, constraints);
        }
        let rich = AttributedText {
            text: rich.text.clone(),
            spans: spans.expect("non-empty spans"),
        };
        let key = TextBlobKey::new_attributed(&rich, base_style, constraints, self.font_stack_key);
        self.prepare_with_key(key, base_style, Some(rich.spans.as_ref()), constraints)
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

    fn maybe_record_font_trace_entry(
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

    fn decoration_metrics_for_shape(
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

    pub fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        return self.measure.measure_plain(
            &mut self.parley_shaper,
            text,
            style,
            constraints,
            self.font_stack_key,
        );

        #[cfg(any())]
        {
            const MEASURE_CACHE_PER_BUCKET_LIMIT: usize = 256;
            const MEASURE_CACHE_PER_BUCKET_LIMIT_WRAP_NONE: usize = 2048;

            let mut normalized_constraints = constraints;
            if normalized_constraints.wrap == TextWrap::None {
                normalized_constraints.max_width = None;
            }

            let key = TextMeasureKey::new(style, normalized_constraints, self.font_stack_key);
            let text_hash = hash_text(text);
            if let Some(bucket) = self.measure_cache.get_mut(&key)
                && let Some(idx) = bucket.iter().position(|e| {
                    e.text_hash == text_hash && e.spans_hash == 0 && e.text.as_ref() == text
                })
                && let Some(hit) = bucket.remove(idx)
            {
                let mut metrics = hit.metrics;
                bucket.push_back(hit);
                if constraints.wrap == TextWrap::None
                    && constraints.overflow == TextOverflow::Ellipsis
                    && let Some(max_width) = constraints.max_width
                {
                    metrics.size.width = max_width;
                }
                return metrics;
            }

            let scale = effective_text_scale_factor(constraints.scale_factor);
            let allow_fast_wrap_measure = constraints.scale_factor.is_finite()
                && constraints.scale_factor.fract().abs() <= 1e-4;
            let max_width_for_fast = match constraints {
                TextConstraints {
                    max_width: Some(max_width),
                    wrap: TextWrap::Word | TextWrap::WordBreak | TextWrap::Grapheme,
                    overflow: TextOverflow::Clip,
                    ..
                } if allow_fast_wrap_measure && !text.contains('\n') => Some(max_width),
                _ => None,
            };

            let metrics = if let Some(max_width) = max_width_for_fast {
                let allow_shaping_cache = text.len()
                    >= fret_render_text::cache_tuning::measure_shaping_cache_min_text_len_bytes();

                let shaping_key = TextMeasureShapingKey {
                    text_hash,
                    text_len: text.len(),
                    spans_shaping_key: 0,
                    font: style.font.clone(),
                    font_stack_key: self.font_stack_key,
                    size_bits: style.size.0.to_bits(),
                    weight: style.weight.0,
                    slant: match style.slant {
                        TextSlant::Normal => 0,
                        TextSlant::Italic => 1,
                        TextSlant::Oblique => 2,
                    },
                    line_height_bits: style.line_height.map(|px| px.0.to_bits()),
                    letter_spacing_bits: style.letter_spacing_em.map(|v| v.to_bits()),
                    scale_bits: constraints.scale_factor.to_bits(),
                };

                let max_width_px = max_width.0 * scale;

                if allow_shaping_cache {
                    let (width_px, baseline_px, line_height_px, _clusters) = if let Some(hit) =
                        self.measure_shaping_cache.get(&shaping_key)
                        && hit.text.as_ref() == text
                        && hit.spans.is_none()
                    {
                        (
                            hit.width_px,
                            hit.baseline_px,
                            hit.line_height_px,
                            hit.clusters.clone(),
                        )
                    } else {
                        let line = self
                            .parley_shaper
                            .shape_single_line_metrics(TextInputRef::plain(text, style), scale);
                        let clusters: Arc<[parley_shaper::ShapedCluster]> =
                            Arc::from(line.clusters);

                        let existed = self
                            .measure_shaping_cache
                            .insert(
                                shaping_key.clone(),
                                TextMeasureShapingEntry {
                                    text: Arc::<str>::from(text),
                                    spans: None,
                                    width_px: line.width,
                                    baseline_px: line.baseline,
                                    line_height_px: line.line_height,
                                    clusters: clusters.clone(),
                                },
                            )
                            .is_some();
                        if !existed {
                            self.measure_shaping_fifo.push_back(shaping_key.clone());
                            let limit =
                                fret_render_text::cache_tuning::measure_shaping_cache_entries();
                            while self.measure_shaping_fifo.len() > limit {
                                let Some(evict) = self.measure_shaping_fifo.pop_front() else {
                                    break;
                                };
                                self.measure_shaping_cache.remove(&evict);
                            }
                        }

                        (line.width, line.baseline, line.line_height, clusters)
                    };

                    if width_px <= max_width_px + 0.5 {
                        metrics_for_uniform_lines(
                            width_px.max(0.0),
                            1,
                            baseline_px,
                            line_height_px,
                            scale,
                        )
                    } else {
                        let wrapped = crate::text::wrapper::wrap_with_constraints_measure_only(
                            &mut self.parley_shaper,
                            TextInputRef::plain(text, style),
                            normalized_constraints,
                        );
                        metrics_from_wrapped_lines(&wrapped.lines, scale)
                    }
                } else {
                    let line = self
                        .parley_shaper
                        .shape_single_line_metrics(TextInputRef::plain(text, style), scale);
                    let width_px = line.width;
                    let baseline_px = line.baseline;
                    let line_height_px = line.line_height;
                    let _clusters = line.clusters;

                    if width_px <= max_width_px + 0.5 {
                        metrics_for_uniform_lines(
                            width_px.max(0.0),
                            1,
                            baseline_px,
                            line_height_px,
                            scale,
                        )
                    } else {
                        let wrapped = crate::text::wrapper::wrap_with_constraints_measure_only(
                            &mut self.parley_shaper,
                            TextInputRef::plain(text, style),
                            normalized_constraints,
                        );
                        metrics_from_wrapped_lines(&wrapped.lines, scale)
                    }
                }
            } else {
                // Keep measurement aligned with prepare/paint under fractional scale factors while
                // avoiding per-glyph work in layout. The metrics-only wrapper shares the same Parley
                // shaping + line breaking, but does not materialize glyph runs.
                let wrapped = crate::text::wrapper::wrap_with_constraints_measure_only(
                    &mut self.parley_shaper,
                    TextInputRef::plain(text, style),
                    normalized_constraints,
                );
                metrics_from_wrapped_lines(&wrapped.lines, scale)
            };

            let bucket = self.measure_cache.entry(key).or_default();
            bucket.push_back(TextMeasureEntry {
                text_hash,
                spans_hash: 0,
                text: Arc::<str>::from(text),
                spans: None,
                metrics,
            });
            let limit = match normalized_constraints.wrap {
                TextWrap::None => MEASURE_CACHE_PER_BUCKET_LIMIT_WRAP_NONE,
                TextWrap::Word | TextWrap::WordBreak | TextWrap::Grapheme => {
                    MEASURE_CACHE_PER_BUCKET_LIMIT
                }
            };
            while bucket.len() > limit {
                bucket.pop_front();
            }

            let mut metrics = metrics;
            if constraints.wrap == TextWrap::None
                && constraints.overflow == TextOverflow::Ellipsis
                && let Some(max_width) = constraints.max_width
            {
                metrics.size.width = max_width;
            }
            metrics
        }
    }

    pub fn measure_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        return self.measure.measure_attributed(
            &mut self.parley_shaper,
            rich,
            base_style,
            constraints,
            self.font_stack_key,
        );

        #[cfg(any())]
        {
            const MEASURE_CACHE_PER_BUCKET_LIMIT: usize = 256;
            const MEASURE_CACHE_PER_BUCKET_LIMIT_WRAP_NONE: usize = 2048;

            let mut normalized_constraints = constraints;
            if normalized_constraints.wrap == TextWrap::None {
                normalized_constraints.max_width = None;
            }

            let key = TextMeasureKey::new(base_style, normalized_constraints, self.font_stack_key);
            let text_hash = hash_text(rich.text.as_ref());
            let spans_hash = spans_shaping_fingerprint(rich.spans.as_ref());

            if let Some(bucket) = self.measure_cache.get_mut(&key)
                && let Some(idx) = bucket.iter().position(|e| {
                    e.text_hash == text_hash
                        && e.spans_hash == spans_hash
                        && e.text.as_ref() == rich.text.as_ref()
                        && e.spans.as_ref().is_some_and(|s| {
                            Arc::ptr_eq(s, &rich.spans) || s.as_ref() == rich.spans.as_ref()
                        })
                })
                && let Some(hit) = bucket.remove(idx)
            {
                let mut metrics = hit.metrics;
                bucket.push_back(hit);
                if constraints.wrap == TextWrap::None
                    && constraints.overflow == TextOverflow::Ellipsis
                    && let Some(max_width) = constraints.max_width
                {
                    metrics.size.width = max_width;
                }
                return metrics;
            }

            let scale = effective_text_scale_factor(constraints.scale_factor);
            let allow_fast_wrap_measure = constraints.scale_factor.is_finite()
                && constraints.scale_factor.fract().abs() <= 1e-4;
            let max_width_for_fast = match constraints {
                TextConstraints {
                    max_width: Some(max_width),
                    wrap: TextWrap::Word | TextWrap::WordBreak | TextWrap::Grapheme,
                    overflow: TextOverflow::Clip,
                    ..
                } if allow_fast_wrap_measure && !rich.text.as_ref().contains('\n') => {
                    Some(max_width)
                }
                _ => None,
            };

            let metrics = if let Some(max_width) = max_width_for_fast {
                let allow_shaping_cache = rich.text.len()
                    >= fret_render_text::cache_tuning::measure_shaping_cache_min_text_len_bytes();

                let shaping_key = TextMeasureShapingKey {
                    text_hash,
                    text_len: rich.text.len(),
                    spans_shaping_key: spans_hash,
                    font: base_style.font.clone(),
                    font_stack_key: self.font_stack_key,
                    size_bits: base_style.size.0.to_bits(),
                    weight: base_style.weight.0,
                    slant: match base_style.slant {
                        TextSlant::Normal => 0,
                        TextSlant::Italic => 1,
                        TextSlant::Oblique => 2,
                    },
                    line_height_bits: base_style.line_height.map(|px| px.0.to_bits()),
                    letter_spacing_bits: base_style.letter_spacing_em.map(|v| v.to_bits()),
                    scale_bits: constraints.scale_factor.to_bits(),
                };

                let max_width_px = max_width.0 * scale;
                let text = rich.text.as_ref();

                if allow_shaping_cache {
                    let (width_px, baseline_px, line_height_px, _clusters) = if let Some(hit) =
                        self.measure_shaping_cache.get(&shaping_key)
                        && hit.text.as_ref() == rich.text.as_ref()
                        && hit.spans.as_ref().is_some_and(|s| {
                            Arc::ptr_eq(s, &rich.spans) || s.as_ref() == rich.spans.as_ref()
                        }) {
                        (
                            hit.width_px,
                            hit.baseline_px,
                            hit.line_height_px,
                            hit.clusters.clone(),
                        )
                    } else {
                        let line = self.parley_shaper.shape_single_line_metrics(
                            TextInputRef::Attributed {
                                text: rich.text.as_ref(),
                                base: base_style,
                                spans: rich.spans.as_ref(),
                            },
                            scale,
                        );
                        let clusters: Arc<[parley_shaper::ShapedCluster]> =
                            Arc::from(line.clusters);

                        let existed = self
                            .measure_shaping_cache
                            .insert(
                                shaping_key.clone(),
                                TextMeasureShapingEntry {
                                    text: rich.text.clone(),
                                    spans: Some(rich.spans.clone()),
                                    width_px: line.width,
                                    baseline_px: line.baseline,
                                    line_height_px: line.line_height,
                                    clusters: clusters.clone(),
                                },
                            )
                            .is_some();
                        if !existed {
                            self.measure_shaping_fifo.push_back(shaping_key.clone());
                            let limit =
                                fret_render_text::cache_tuning::measure_shaping_cache_entries();
                            while self.measure_shaping_fifo.len() > limit {
                                let Some(evict) = self.measure_shaping_fifo.pop_front() else {
                                    break;
                                };
                                self.measure_shaping_cache.remove(&evict);
                            }
                        }

                        (line.width, line.baseline, line.line_height, clusters)
                    };

                    if width_px <= max_width_px + 0.5 {
                        metrics_for_uniform_lines(
                            width_px.max(0.0),
                            1,
                            baseline_px,
                            line_height_px,
                            scale,
                        )
                    } else {
                        let wrapped = crate::text::wrapper::wrap_with_constraints_measure_only(
                            &mut self.parley_shaper,
                            TextInputRef::Attributed {
                                text,
                                base: base_style,
                                spans: rich.spans.as_ref(),
                            },
                            normalized_constraints,
                        );
                        metrics_from_wrapped_lines(&wrapped.lines, scale)
                    }
                } else {
                    let line = self.parley_shaper.shape_single_line_metrics(
                        TextInputRef::Attributed {
                            text: rich.text.as_ref(),
                            base: base_style,
                            spans: rich.spans.as_ref(),
                        },
                        scale,
                    );
                    let width_px = line.width;
                    let baseline_px = line.baseline;
                    let line_height_px = line.line_height;
                    let _clusters = line.clusters;

                    if width_px <= max_width_px + 0.5 {
                        metrics_for_uniform_lines(
                            width_px.max(0.0),
                            1,
                            baseline_px,
                            line_height_px,
                            scale,
                        )
                    } else {
                        let wrapped = crate::text::wrapper::wrap_with_constraints_measure_only(
                            &mut self.parley_shaper,
                            TextInputRef::Attributed {
                                text,
                                base: base_style,
                                spans: rich.spans.as_ref(),
                            },
                            normalized_constraints,
                        );
                        metrics_from_wrapped_lines(&wrapped.lines, scale)
                    }
                }
            } else {
                let text = rich.text.as_ref();
                let wrapped = crate::text::wrapper::wrap_with_constraints_measure_only(
                    &mut self.parley_shaper,
                    TextInputRef::Attributed {
                        text,
                        base: base_style,
                        spans: rich.spans.as_ref(),
                    },
                    normalized_constraints,
                );
                metrics_from_wrapped_lines(&wrapped.lines, scale)
            };

            let bucket = self.measure_cache.entry(key).or_default();
            bucket.push_back(TextMeasureEntry {
                text_hash,
                spans_hash,
                text: rich.text.clone(),
                spans: Some(rich.spans.clone()),
                metrics,
            });
            let limit = match normalized_constraints.wrap {
                TextWrap::None => MEASURE_CACHE_PER_BUCKET_LIMIT_WRAP_NONE,
                TextWrap::Word | TextWrap::WordBreak | TextWrap::Grapheme => {
                    MEASURE_CACHE_PER_BUCKET_LIMIT
                }
            };
            while bucket.len() > limit {
                bucket.pop_front();
            }

            let mut metrics = metrics;
            if constraints.wrap == TextWrap::None
                && constraints.overflow == TextOverflow::Ellipsis
                && let Some(max_width) = constraints.max_width
            {
                metrics.size.width = max_width;
            }
            metrics
        }
    }

    pub fn caret_x(&self, blob: TextBlobId, index: usize) -> Option<Px> {
        let blob_id = blob;
        let blob = self.blobs.get(blob_id)?;
        if blob.shape.lines.len() > 1 {
            return Some(
                self.caret_rect(blob_id, index, CaretAffinity::Downstream)?
                    .origin
                    .x,
            );
        }
        let stops = blob.shape.caret_stops.as_ref();
        Some(fret_render_text::geometry::caret_x_from_stops(stops, index))
    }

    pub fn hit_test_x(&self, blob: TextBlobId, x: Px) -> Option<usize> {
        let blob_id = blob;
        let blob = self.blobs.get(blob_id)?;
        if blob.shape.lines.len() > 1 {
            return Some(self.hit_test_point(blob_id, Point::new(x, Px(0.0)))?.index);
        }
        let stops = blob.shape.caret_stops.as_ref();
        Some(fret_render_text::geometry::hit_test_x_from_stops(stops, x))
    }

    pub fn caret_stops(&self, blob: TextBlobId) -> Option<&[(usize, Px)]> {
        Some(self.blobs.get(blob)?.shape.caret_stops.as_ref())
    }

    pub fn caret_rect(
        &self,
        blob: TextBlobId,
        index: usize,
        affinity: CaretAffinity,
    ) -> Option<Rect> {
        let blob = self.blobs.get(blob)?;
        fret_render_text::geometry::caret_rect_from_lines(
            blob.shape.lines.as_ref(),
            index,
            affinity,
        )
    }

    pub fn hit_test_point(&self, blob: TextBlobId, point: Point) -> Option<HitTestResult> {
        let blob = self.blobs.get(blob)?;
        fret_render_text::geometry::hit_test_point_from_lines(blob.shape.lines.as_ref(), point)
    }

    pub fn selection_rects(
        &self,
        blob: TextBlobId,
        range: (usize, usize),
        out: &mut Vec<Rect>,
    ) -> Option<()> {
        let blob = self.blobs.get(blob)?;
        fret_render_text::geometry::selection_rects_from_lines(
            blob.shape.lines.as_ref(),
            range,
            out,
        );
        Some(())
    }

    pub fn selection_rects_clipped(
        &self,
        blob: TextBlobId,
        range: (usize, usize),
        clip: Rect,
        out: &mut Vec<Rect>,
    ) -> Option<()> {
        let blob = self.blobs.get(blob)?;
        fret_render_text::geometry::selection_rects_from_lines_clipped(
            blob.shape.lines.as_ref(),
            range,
            clip,
            out,
        );
        Some(())
    }

    pub fn first_line_metrics(&self, blob: TextBlobId) -> Option<fret_core::TextLineMetrics> {
        let blob = self.blobs.get(blob)?;
        let line = blob.shape.lines.first()?;
        Some(fret_core::TextLineMetrics {
            ascent: line.ascent,
            descent: line.descent,
            line_height: line.height,
        })
    }

    pub fn first_line_ink_metrics(&self, blob: TextBlobId) -> Option<fret_core::TextInkMetrics> {
        let blob = self.blobs.get(blob)?;
        let line = blob.shape.lines.first()?;
        Some(fret_core::TextInkMetrics {
            ascent: line.ink_ascent,
            descent: line.ink_descent,
        })
    }

    pub fn last_line_metrics(&self, blob: TextBlobId) -> Option<fret_core::TextLineMetrics> {
        let blob = self.blobs.get(blob)?;
        let line = blob.shape.lines.last()?;
        Some(fret_core::TextLineMetrics {
            ascent: line.ascent,
            descent: line.descent,
            line_height: line.height,
        })
    }

    pub fn last_line_ink_metrics(&self, blob: TextBlobId) -> Option<fret_core::TextInkMetrics> {
        let blob = self.blobs.get(blob)?;
        let line = blob.shape.lines.last()?;
        Some(fret_core::TextInkMetrics {
            ascent: line.ink_ascent,
            descent: line.ink_descent,
        })
    }

    pub fn release(&mut self, blob: TextBlobId) {
        let entries = fret_render_text::cache_tuning::released_blob_cache_entries();

        let Some(b) = self.blobs.get_mut(blob) else {
            return;
        };

        if b.ref_count > 1 {
            b.ref_count = b.ref_count.saturating_sub(1);
            return;
        }

        if b.ref_count == 0 {
            return;
        }

        if entries > 0 {
            b.ref_count = 0;
            self.insert_released_blob(blob, entries);
            return;
        }

        self.evict_blob(blob);
    }

    fn remove_released_blob(&mut self, id: TextBlobId) {
        if !self.released_blob_set.remove(&id) {
            return;
        }
        if let Some(pos) = self.released_blob_lru.iter().position(|v| *v == id) {
            self.released_blob_lru.remove(pos);
        }
    }

    fn insert_released_blob(&mut self, id: TextBlobId, entries: usize) {
        if entries == 0 {
            return;
        }

        if !self.released_blob_set.insert(id)
            && let Some(pos) = self.released_blob_lru.iter().position(|v| *v == id)
        {
            self.released_blob_lru.remove(pos);
        }
        self.released_blob_lru.push_back(id);

        while self.released_blob_lru.len() > entries {
            let Some(evict) = self.released_blob_lru.pop_front() else {
                break;
            };
            self.released_blob_set.remove(&evict);
            if self.blobs.get(evict).is_some_and(|b| b.ref_count > 0) {
                continue;
            }
            self.evict_blob(evict);
        }
    }

    fn clear_released_blob_cache(&mut self) {
        self.released_blob_lru.clear();
        self.released_blob_set.clear();
    }

    fn evict_blob(&mut self, blob: TextBlobId) {
        self.remove_released_blob(blob);

        let remove_shape = self
            .blobs
            .get(blob)
            .is_some_and(|b| Arc::strong_count(&b.shape) == 2);

        if let Some(key) = self.blob_key_by_id.remove(&blob) {
            self.blob_cache.remove(&key);
            if remove_shape {
                let shape_key = TextShapeKey::from_blob_key(&key);
                self.shape_cache.remove(&shape_key);
            }
        }
        let _ = self.blobs.remove(blob);
    }

    fn wrap_for_prepare(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> crate::text::wrapper::WrappedLayout {
        crate::text::wrapper::wrap_with_constraints(&mut self.parley_shaper, input, constraints)
    }
}

#[cfg(test)]
mod tests;

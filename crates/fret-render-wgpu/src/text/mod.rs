use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextSpan, TextStyle};
use slotmap::SlotMap;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use fret_render_text::cache_keys::{TextBlobKey, TextShapeKey};

#[cfg(test)]
use fret_render_text::cache_keys::spans_paint_fingerprint;
pub use fret_render_text::decorations::{TextDecoration, TextDecorationKind};
pub(crate) use fret_render_text::effective_text_scale_factor;
use fret_render_text::fallback_policy::TextFallbackPolicyV1;
use fret_render_text::font_instance_key::FontFaceKey;
use fret_render_text::font_stack::GenericFamilyInjectionState;
use fret_render_text::font_trace::FontTraceState;
use fret_render_text::measure::TextMeasureCaches;
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
mod types;

use self::atlas::{GlyphAtlas, GlyphKey, TEXT_ATLAS_MAX_PAGES};
pub use self::quality::TextQualitySettings;
use self::quality::TextQualityState;
#[cfg(test)]
pub(crate) use self::types::subpixel_mask_to_alpha;
pub(crate) use self::types::{DebugGlyphAtlasLookup, TextAtlasPerfSnapshot, TextFontFaceUsage};
pub use self::types::{
    GlyphInstance, GlyphQuadKind, TextBlob, TextFontFamilyConfig, TextLine, TextShape,
};

pub(crate) mod parley_shaper {
    pub use fret_render_text::parley_shaper::*;
}

pub(crate) mod wrapper {
    pub use fret_render_text::wrapper::*;
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
        key: TextBlobKey,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        self.prepare_with_key_driver(key, style, spans, constraints)
    }
}

#[cfg(test)]
mod tests;

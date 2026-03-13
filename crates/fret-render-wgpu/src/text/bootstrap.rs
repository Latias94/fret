use super::TextSystem;
use super::atlas::{GlyphAtlas, TEXT_ATLAS_MAX_PAGES};
use super::quality::{TextQualitySettings, TextQualityState};
use fret_render_text::fallback_policy::TextFallbackPolicyV1;
use fret_render_text::font_stack::GenericFamilyInjectionState;
use fret_render_text::font_trace::FontTraceState;
use fret_render_text::measure::TextMeasureCaches;
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet, VecDeque};

const TEXT_ATLAS_WIDTH: u32 = 2048;
const TEXT_ATLAS_HEIGHT: u32 = 2048;

pub(super) struct TextBootstrapResources {
    pub(super) parley_shaper: crate::text::parley_shaper::ParleyShaper,
    pub(super) fallback_policy: TextFallbackPolicyV1,
    pub(super) atlas_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) mask_atlas: GlyphAtlas,
    pub(super) color_atlas: GlyphAtlas,
    pub(super) subpixel_atlas: GlyphAtlas,
}

pub(super) fn bootstrap_text_resources(device: &wgpu::Device) -> TextBootstrapResources {
    let atlas_sampler = create_atlas_sampler(device);
    let atlas_bind_group_layout = create_atlas_bind_group_layout(device);

    let mask_atlas = create_text_atlas(
        device,
        &atlas_bind_group_layout,
        &atlas_sampler,
        "fret glyph mask atlas",
        wgpu::TextureFormat::R8Unorm,
    );
    let color_atlas = create_text_atlas(
        device,
        &atlas_bind_group_layout,
        &atlas_sampler,
        "fret glyph color atlas",
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );
    let subpixel_atlas = create_text_atlas(
        device,
        &atlas_bind_group_layout,
        &atlas_sampler,
        "fret glyph subpixel atlas",
        wgpu::TextureFormat::Rgba8Unorm,
    );

    let parley_shaper = crate::text::parley_shaper::ParleyShaper::new();
    let fallback_policy = TextFallbackPolicyV1::new(&parley_shaper);

    TextBootstrapResources {
        parley_shaper,
        fallback_policy,
        atlas_bind_group_layout,
        mask_atlas,
        color_atlas,
        subpixel_atlas,
    }
}

pub(super) fn build_text_system(device: &wgpu::Device) -> TextSystem {
    let bootstrap = bootstrap_text_resources(device);

    let mut out = TextSystem {
        parley_shaper: bootstrap.parley_shaper,
        parley_scale: parley::swash::scale::ScaleContext::new(),
        // Non-zero by default so callers can treat `0` as "unknown/uninitialized" if desired.
        font_stack_key: 1,
        font_db_revision: 1,
        fallback_policy: bootstrap.fallback_policy,
        quality: TextQualityState::new(TextQualitySettings::default()),
        generic_injections: GenericFamilyInjectionState::default(),

        blobs: SlotMap::with_key(),
        blob_cache: HashMap::new(),
        blob_key_by_id: HashMap::new(),
        released_blob_lru: VecDeque::new(),
        released_blob_set: HashSet::new(),
        shape_cache: HashMap::new(),
        measure: TextMeasureCaches::new(),

        mask_atlas: bootstrap.mask_atlas,
        color_atlas: bootstrap.color_atlas,
        subpixel_atlas: bootstrap.subpixel_atlas,
        atlas_bind_group_layout: bootstrap.atlas_bind_group_layout,

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

    out.finish_initial_font_bootstrap();
    out
}

fn create_atlas_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fret glyph atlas sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

fn create_atlas_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
    })
}

fn create_text_atlas(
    device: &wgpu::Device,
    atlas_bind_group_layout: &wgpu::BindGroupLayout,
    atlas_sampler: &wgpu::Sampler,
    label_prefix: &str,
    format: wgpu::TextureFormat,
) -> GlyphAtlas {
    GlyphAtlas::new(
        device,
        atlas_bind_group_layout,
        atlas_sampler,
        label_prefix,
        TEXT_ATLAS_WIDTH,
        TEXT_ATLAS_HEIGHT,
        format,
        0,
        TEXT_ATLAS_MAX_PAGES,
    )
}

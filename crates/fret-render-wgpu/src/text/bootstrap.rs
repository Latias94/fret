use super::TextSystem;
use super::atlas::{GlyphAtlas, TEXT_ATLAS_MAX_PAGES};
use super::atlas_epoch::TextAtlasEpochState;
use super::atlas_runtime_state::TextAtlasRuntimeState;
use super::blob_state::TextBlobState;
use super::face_cache::TextFaceCacheState;
use super::font_runtime_state::TextFontRuntimeState;
use super::frame_perf::TextFramePerfState;
use super::layout_cache_state::TextLayoutCacheState;
use super::pin_state::TextPinState;
use super::quality::{TextQualitySettings, TextQualityState};
use fret_render_text::TextFallbackPolicyV1;

const TEXT_ATLAS_WIDTH: u32 = 2048;
const TEXT_ATLAS_HEIGHT: u32 = 2048;

pub(super) struct TextBootstrapResources {
    pub(super) parley_shaper: fret_render_text::ParleyShaper,
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

    let parley_shaper = fret_render_text::ParleyShaper::new();
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
        font_runtime: TextFontRuntimeState::new(bootstrap.fallback_policy),
        quality: TextQualityState::new(TextQualitySettings::default()),

        blob_state: TextBlobState::new(),
        layout_cache: TextLayoutCacheState::new(),

        atlas_runtime: TextAtlasRuntimeState::new(
            bootstrap.mask_atlas,
            bootstrap.color_atlas,
            bootstrap.subpixel_atlas,
            bootstrap.atlas_bind_group_layout,
        ),

        pin_state: TextPinState::with_ring_len(3),
        face_cache: TextFaceCacheState::default(),

        frame_perf: TextFramePerfState::default(),

        atlas_epoch: TextAtlasEpochState::new(1),
    };

    out.finish_initial_font_bootstrap();
    out
}

impl TextSystem {
    pub fn new(device: &wgpu::Device) -> Self {
        build_text_system(device)
    }
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

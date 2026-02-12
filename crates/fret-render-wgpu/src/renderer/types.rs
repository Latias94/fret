use bytemuck::{Pod, Zeroable};
use fret_core::scene::MAX_STOPS;
use fret_core::scene::UvRect;
use std::sync::Arc;
use std::time::Duration;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ClipRRectUniform {
    pub(super) rect: [f32; 4],
    pub(super) corner_radii: [f32; 4],
    pub(super) inv0: [f32; 4],
    pub(super) inv1: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct MaskGradientUniform {
    /// Bounds in local pixel coordinates (x, y, w, h). Outside bounds, the mask is treated as 1.0.
    pub(super) bounds: [f32; 4],
    /// 1 = LinearGradient, 2 = RadialGradient.
    pub(super) kind: u32,
    pub(super) tile_mode: u32,
    pub(super) stop_count: u32,
    pub(super) _pad0: u32,
    /// Linear: start.xy end.xy. Radial: center.xy radius.xy.
    pub(super) params0: [f32; 4],
    pub(super) inv0: [f32; 4],
    pub(super) inv1: [f32; 4],
    pub(super) stop_alphas0: [f32; 4],
    pub(super) stop_alphas1: [f32; 4],
    pub(super) stop_offsets0: [f32; 4],
    pub(super) stop_offsets1: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ViewportUniform {
    pub(super) viewport_size: [f32; 2],
    pub(super) clip_head: u32,
    pub(super) clip_count: u32,
    pub(super) mask_head: u32,
    pub(super) mask_count: u32,
    /// Masks active at this scope boundary are excluded from draw shaders (applied later by
    /// composites). See ADR 0239.
    pub(super) mask_scope_head: u32,
    pub(super) mask_scope_count: u32,
    pub(super) output_is_srgb: u32,
    pub(super) _pad: u32,
    /// The viewport-space rect that mask textures are scoped to (top-left origin in pixels).
    ///
    /// For non-effect draws this is the full viewport. For effect scopes this is the effect
    /// scissor rect so viewport-scoped mask targets can be generated and sampled correctly.
    pub(super) mask_viewport_origin: [f32; 2],
    /// The viewport-space size of the rect that mask textures are scoped to (in pixels).
    pub(super) mask_viewport_size: [f32; 2],
    /// Padding to match WGSL uniform layout rules: `vec4<f32>` requires 16-byte alignment.
    pub(super) _pad_text_gamma: [u32; 2],

    /// Text gamma correction ratios (GPUI-aligned). Applied to grayscale coverage masks and
    /// subpixel RGB coverage in the text sampling shaders.
    pub(super) text_gamma_ratios: [f32; 4],
    /// Enhanced contrast factor for grayscale text (mask glyphs).
    pub(super) text_grayscale_enhanced_contrast: f32,
    /// Enhanced contrast factor for subpixel text (RGB coverage glyphs).
    pub(super) text_subpixel_enhanced_contrast: f32,
    pub(super) _pad_text_quality: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ScaleParamsUniform {
    pub(super) scale: u32,
    pub(super) _pad0: u32,
    pub(super) src_origin: [u32; 2],
    pub(super) dst_origin: [u32; 2],
    pub(super) _pad1: u32,
    pub(super) _pad2: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct PaintGpu {
    pub(super) kind: u32,
    pub(super) tile_mode: u32,
    pub(super) color_space: u32,
    pub(super) stop_count: u32,
    pub(super) params0: [f32; 4],
    pub(super) params1: [f32; 4],
    pub(super) params2: [f32; 4],
    pub(super) params3: [f32; 4],
    pub(super) stop_colors: [[f32; 4]; MAX_STOPS],
    pub(super) stop_offsets0: [f32; 4],
    pub(super) stop_offsets1: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct QuadInstance {
    pub(super) rect: [f32; 4],
    pub(super) transform0: [f32; 4],
    pub(super) transform1: [f32; 4],
    pub(super) fill_paint: PaintGpu,
    pub(super) border_paint: PaintGpu,
    pub(super) corner_radii: [f32; 4],
    pub(super) border: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ViewportVertex {
    pub(super) pos_px: [f32; 2],
    pub(super) uv: [f32; 2],
    pub(super) opacity: f32,
    pub(super) _pad: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct TextVertex {
    pub(super) pos_px: [f32; 2],
    pub(super) uv: [f32; 2],
    pub(super) color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct PathVertex {
    pub(super) pos_px: [f32; 2],
    pub(super) color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ScissorRect {
    pub(super) x: u32,
    pub(super) y: u32,
    pub(super) w: u32,
    pub(super) h: u32,
}

impl ScissorRect {
    pub(super) fn full(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            w: width,
            h: height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum SvgRasterKind {
    AlphaMask,
    Rgba,
}

pub(super) const SVG_MASK_ATLAS_PAGE_SIZE_PX: u32 = 1024;
pub(super) const SVG_MASK_ATLAS_PADDING_PX: u32 = 1;

#[derive(Debug, Default, Clone, Copy)]
pub struct RenderPerfSnapshot {
    pub frames: u64,

    pub encode_scene_us: u64,
    pub prepare_svg_us: u64,
    pub prepare_text_us: u64,

    // Non-text upload churn (best-effort). These counters attempt to make CPU->GPU texture uploads
    // visible in diagnostics, beyond text atlas updates.
    pub svg_uploads: u64,
    pub svg_upload_bytes: u64,
    pub image_uploads: u64,
    pub image_upload_bytes: u64,

    // SVG raster cache (best-effort). These are intended to distinguish one-time warmup from
    // steady-state thrash (e.g. budget-driven eviction + repeated re-upload).
    pub svg_raster_budget_bytes: u64,
    pub svg_rasters_live: u64,
    pub svg_standalone_bytes_live: u64,
    pub svg_mask_atlas_pages_live: u64,
    pub svg_mask_atlas_bytes_live: u64,
    pub svg_mask_atlas_used_px: u64,
    pub svg_mask_atlas_capacity_px: u64,
    pub svg_raster_cache_hits: u64,
    pub svg_raster_cache_misses: u64,
    pub svg_raster_budget_evictions: u64,
    pub svg_mask_atlas_page_evictions: u64,
    pub svg_mask_atlas_entries_evicted: u64,

    // Text atlas churn (best-effort). These numbers are per-frame signals and should be treated as
    // diagnostic hints rather than strict correctness metrics.
    pub text_atlas_revision: u64,
    pub text_atlas_uploads: u64,
    pub text_atlas_upload_bytes: u64,
    pub text_atlas_evicted_glyphs: u64,
    pub text_atlas_evicted_pages: u64,
    pub text_atlas_evicted_page_glyphs: u64,
    pub text_atlas_resets: u64,

    // Intermediate pool churn (best-effort; used for blur/effect pipelines).
    pub intermediate_budget_bytes: u64,
    pub intermediate_in_use_bytes: u64,
    pub intermediate_peak_in_use_bytes: u64,
    pub intermediate_release_targets: u64,
    pub intermediate_pool_allocations: u64,
    pub intermediate_pool_reuses: u64,
    pub intermediate_pool_releases: u64,
    pub intermediate_pool_evictions: u64,
    pub intermediate_pool_free_bytes: u64,
    pub intermediate_pool_free_textures: u64,

    pub draw_calls: u64,
    pub quad_draw_calls: u64,
    pub viewport_draw_calls: u64,
    pub image_draw_calls: u64,
    pub text_draw_calls: u64,
    pub path_draw_calls: u64,
    pub mask_draw_calls: u64,
    pub fullscreen_draw_calls: u64,
    pub clip_mask_draw_calls: u64,

    pub pipeline_switches: u64,
    pub pipeline_switches_quad: u64,
    pub pipeline_switches_viewport: u64,
    pub pipeline_switches_mask: u64,
    pub pipeline_switches_text_mask: u64,
    pub pipeline_switches_text_color: u64,
    pub pipeline_switches_text_subpixel: u64,
    pub pipeline_switches_path: u64,
    pub pipeline_switches_path_msaa: u64,
    pub pipeline_switches_composite: u64,
    pub pipeline_switches_fullscreen: u64,
    pub pipeline_switches_clip_mask: u64,
    pub bind_group_switches: u64,
    pub uniform_bind_group_switches: u64,
    pub texture_bind_group_switches: u64,
    pub scissor_sets: u64,

    pub uniform_bytes: u64,
    pub instance_bytes: u64,
    pub vertex_bytes: u64,

    pub scene_encoding_cache_hits: u64,
    pub scene_encoding_cache_misses: u64,

    // Tier B materials (ADR 0235) observability (best-effort).
    pub material_quad_ops: u64,
    pub material_sampled_quad_ops: u64,
    pub material_distinct: u64,
    pub material_unknown_ids: u64,
    pub material_degraded_due_to_budget: u64,
}

#[derive(Debug, Default)]
pub(super) struct RenderPerfStats {
    pub(super) frames: u64,

    pub(super) encode_scene: Duration,
    pub(super) prepare_svg: Duration,
    pub(super) prepare_text: Duration,

    pub(super) svg_uploads: u64,
    pub(super) svg_upload_bytes: u64,
    pub(super) image_uploads: u64,
    pub(super) image_upload_bytes: u64,

    pub(super) svg_raster_budget_bytes: u64,
    pub(super) svg_rasters_live: u64,
    pub(super) svg_standalone_bytes_live: u64,
    pub(super) svg_mask_atlas_pages_live: u64,
    pub(super) svg_mask_atlas_bytes_live: u64,
    pub(super) svg_mask_atlas_used_px: u64,
    pub(super) svg_mask_atlas_capacity_px: u64,
    pub(super) svg_raster_cache_hits: u64,
    pub(super) svg_raster_cache_misses: u64,
    pub(super) svg_raster_budget_evictions: u64,
    pub(super) svg_mask_atlas_page_evictions: u64,
    pub(super) svg_mask_atlas_entries_evicted: u64,

    pub(super) text_atlas_revision: u64,
    pub(super) text_atlas_uploads: u64,
    pub(super) text_atlas_upload_bytes: u64,
    pub(super) text_atlas_evicted_glyphs: u64,
    pub(super) text_atlas_evicted_pages: u64,
    pub(super) text_atlas_evicted_page_glyphs: u64,
    pub(super) text_atlas_resets: u64,

    pub(super) intermediate_budget_bytes: u64,
    pub(super) intermediate_in_use_bytes: u64,
    pub(super) intermediate_peak_in_use_bytes: u64,
    pub(super) intermediate_release_targets: u64,
    pub(super) intermediate_pool_allocations: u64,
    pub(super) intermediate_pool_reuses: u64,
    pub(super) intermediate_pool_releases: u64,
    pub(super) intermediate_pool_evictions: u64,
    pub(super) intermediate_pool_free_bytes: u64,
    pub(super) intermediate_pool_free_textures: u64,

    pub(super) draw_calls: u64,
    pub(super) quad_draw_calls: u64,
    pub(super) viewport_draw_calls: u64,
    pub(super) image_draw_calls: u64,
    pub(super) text_draw_calls: u64,
    pub(super) path_draw_calls: u64,
    pub(super) mask_draw_calls: u64,
    pub(super) fullscreen_draw_calls: u64,
    pub(super) clip_mask_draw_calls: u64,

    pub(super) pipeline_switches: u64,
    pub(super) pipeline_switches_quad: u64,
    pub(super) pipeline_switches_viewport: u64,
    pub(super) pipeline_switches_mask: u64,
    pub(super) pipeline_switches_text_mask: u64,
    pub(super) pipeline_switches_text_color: u64,
    pub(super) pipeline_switches_text_subpixel: u64,
    pub(super) pipeline_switches_path: u64,
    pub(super) pipeline_switches_path_msaa: u64,
    pub(super) pipeline_switches_composite: u64,
    pub(super) pipeline_switches_fullscreen: u64,
    pub(super) pipeline_switches_clip_mask: u64,
    pub(super) bind_group_switches: u64,
    pub(super) uniform_bind_group_switches: u64,
    pub(super) texture_bind_group_switches: u64,
    pub(super) scissor_sets: u64,

    pub(super) uniform_bytes: u64,
    pub(super) instance_bytes: u64,
    pub(super) vertex_bytes: u64,

    pub(super) scene_encoding_cache_hits: u64,
    pub(super) scene_encoding_cache_misses: u64,

    pub(super) material_quad_ops: u64,
    pub(super) material_sampled_quad_ops: u64,
    pub(super) material_distinct: u64,
    pub(super) material_unknown_ids: u64,
    pub(super) material_degraded_due_to_budget: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SvgPerfSnapshot {
    pub frames: u64,
    pub prepare_svg_ops_us: u64,

    pub cache_hits: u64,
    pub cache_misses: u64,

    pub alpha_raster_count: u64,
    pub alpha_raster_us: u64,
    pub rgba_raster_count: u64,
    pub rgba_raster_us: u64,

    pub alpha_atlas_inserts: u64,
    pub alpha_atlas_write_us: u64,
    pub alpha_standalone_uploads: u64,
    pub alpha_standalone_upload_us: u64,
    pub rgba_uploads: u64,
    pub rgba_upload_us: u64,

    pub atlas_pages_live: usize,
    pub svg_rasters_live: usize,
    pub svg_standalone_bytes_live: u64,
    pub svg_mask_atlas_bytes_live: u64,
    pub svg_mask_atlas_used_px: u64,
    pub svg_mask_atlas_capacity_px: u64,
}

#[derive(Debug, Default)]
pub(super) struct SvgPerfStats {
    pub(super) frames: u64,
    pub(super) prepare_svg_ops: Duration,

    pub(super) cache_hits: u64,
    pub(super) cache_misses: u64,

    pub(super) alpha_raster_count: u64,
    pub(super) alpha_raster: Duration,
    pub(super) rgba_raster_count: u64,
    pub(super) rgba_raster: Duration,

    pub(super) alpha_atlas_inserts: u64,
    pub(super) alpha_atlas_write: Duration,
    pub(super) alpha_standalone_uploads: u64,
    pub(super) alpha_standalone_upload: Duration,
    pub(super) rgba_uploads: u64,
    pub(super) rgba_upload: Duration,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct IntermediatePerfSnapshot {
    pub frames: u64,
    pub budget_bytes: u64,

    pub last_frame_in_use_bytes: u64,
    pub last_frame_peak_in_use_bytes: u64,
    pub last_frame_release_targets: u64,
    pub blur_degraded_to_quarter: u64,
    pub blur_disabled_due_to_budget: u64,
    pub pool_free_bytes: u64,
    pub pool_free_textures: u64,

    pub pool_allocations: u64,
    pub pool_reuses: u64,
    pub pool_releases: u64,
    pub pool_evictions: u64,
}

#[derive(Debug, Default)]
pub(super) struct IntermediatePerfStats {
    pub(super) frames: u64,
    pub(super) last_frame_in_use_bytes: u64,
    pub(super) last_frame_peak_in_use_bytes: u64,
    pub(super) last_frame_release_targets: u64,
    pub(super) blur_degraded_to_quarter: u64,
    pub(super) blur_disabled_due_to_budget: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SvgRasterKey {
    pub(super) svg: fret_core::SvgId,
    pub(super) target_w: u32,
    pub(super) target_h: u32,
    pub(super) smooth_scale_bits: u32,
    pub(super) kind: SvgRasterKind,
    pub(super) fit: fret_core::SvgFit,
}

pub(super) enum SvgRasterStorage {
    Standalone {
        _texture: wgpu::Texture,
    },
    MaskAtlas {
        page_index: usize,
        alloc_id: etagere::AllocId,
    },
}

pub(super) struct SvgMaskAtlasPage {
    pub(super) image: fret_core::ImageId,
    pub(super) size_px: (u32, u32),
    pub(super) allocator: etagere::BucketedAtlasAllocator,
    pub(super) entries: usize,
    pub(super) last_used_epoch: u64,
    pub(super) _texture: wgpu::Texture,
}

pub(super) struct SvgRasterEntry {
    pub(super) image: fret_core::ImageId,
    pub(super) uv: UvRect,
    pub(super) size_px: (u32, u32),
    pub(super) approx_bytes: u64,
    pub(super) last_used_epoch: u64,
    pub(super) storage: SvgRasterStorage,
}

#[derive(Debug, Clone)]
pub(super) struct SvgEntry {
    pub(super) bytes: Arc<[u8]>,
    pub(super) refs: u32,
}

impl SvgMaskAtlasPage {
    pub(super) fn bytes(&self) -> u64 {
        u64::from(self.size_px.0).saturating_mul(u64::from(self.size_px.1))
    }
}

pub(super) struct DrawCall {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_instance: u32,
    pub(super) instance_count: u32,
}

pub(super) struct ViewportDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) target: fret_core::RenderTargetId,
}

#[derive(Clone, Copy)]
pub(super) struct ImageDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) image: fret_core::ImageId,
}

#[derive(Clone, Copy)]
pub(super) struct MaskDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) image: fret_core::ImageId,
}

pub(super) struct TextDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) kind: TextDrawKind,
    pub(super) atlas_page: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TextDrawKind {
    Mask,
    Color,
    Subpixel,
}

#[derive(Clone, Copy)]
pub(super) struct PathDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
}

pub(super) struct PathIntermediate {
    pub(super) size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) sample_count: u32,
    pub(super) _msaa_texture: Option<wgpu::Texture>,
    pub(super) msaa_view: Option<wgpu::TextureView>,
    pub(super) _resolved_texture: wgpu::Texture,
    pub(super) resolved_view: wgpu::TextureView,
    pub(super) bind_group: wgpu::BindGroup,
}

pub(super) enum OrderedDraw {
    Quad(DrawCall),
    Viewport(ViewportDraw),
    Image(ImageDraw),
    Mask(MaskDraw),
    Text(TextDraw),
    Path(PathDraw),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum EffectMarkerKind {
    Push {
        scissor: ScissorRect,
        uniform_index: u32,
        mode: fret_core::EffectMode,
        chain: fret_core::EffectChain,
        quality: fret_core::EffectQuality,
    },
    Pop,
    CompositeGroupPush {
        scissor: ScissorRect,
        uniform_index: u32,
        mode: fret_core::BlendMode,
        quality: fret_core::EffectQuality,
    },
    CompositeGroupPop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct EffectMarker {
    pub(super) draw_ix: usize,
    pub(super) kind: EffectMarkerKind,
}

#[derive(Default)]
pub(super) struct SceneEncoding {
    pub(super) instances: Vec<QuadInstance>,
    pub(super) viewport_vertices: Vec<ViewportVertex>,
    pub(super) text_vertices: Vec<TextVertex>,
    pub(super) path_vertices: Vec<PathVertex>,
    pub(super) clips: Vec<ClipRRectUniform>,
    pub(super) masks: Vec<MaskGradientUniform>,
    pub(super) uniforms: Vec<ViewportUniform>,
    pub(super) ordered_draws: Vec<OrderedDraw>,
    pub(super) effect_markers: Vec<EffectMarker>,

    pub(super) material_quad_ops: u64,
    pub(super) material_sampled_quad_ops: u64,
    pub(super) material_distinct: u64,
    pub(super) material_unknown_ids: u64,
    pub(super) material_degraded_due_to_budget: u64,
}

impl SceneEncoding {
    pub(super) fn clear(&mut self) {
        self.instances.clear();
        self.viewport_vertices.clear();
        self.text_vertices.clear();
        self.path_vertices.clear();
        self.clips.clear();
        self.masks.clear();
        self.uniforms.clear();
        self.ordered_draws.clear();
        self.effect_markers.clear();
        self.material_quad_ops = 0;
        self.material_sampled_quad_ops = 0;
        self.material_distinct = 0;
        self.material_unknown_ids = 0;
        self.material_degraded_due_to_budget = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SceneEncodingCacheKey {
    pub(super) format: wgpu::TextureFormat,
    pub(super) viewport_size: (u32, u32),
    pub(super) scale_factor_bits: u32,
    pub(super) scene_fingerprint: u64,
    pub(super) scene_ops_len: usize,
    pub(super) render_targets_generation: u64,
    pub(super) images_generation: u64,
    pub(super) text_atlas_revision: u64,
    pub(super) text_quality_key: u64,
}

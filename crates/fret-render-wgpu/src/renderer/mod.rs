use crate::images::ImageRegistry;
use crate::svg::SvgRenderer;
use crate::targets::RenderTargetRegistry;
use crate::text::TextSystem;
pub(super) use fret_core::{
    geometry::{Point, Px, Rect, Size, Transform2D},
    scene::{Color, Scene, SceneOp, UvRect},
};
use fret_render_core::RenderTargetIngestStrategy;
use slotmap::SlotMap;
use std::collections::HashMap;
use std::sync::Arc;

// Split from the original single-file renderer for maintainability.
mod bind_group_builders;
mod bind_group_caches;
mod clip_path_mask_cache;
mod gpu_globals;
mod gpu_pipelines;
mod gpu_textures;
mod path;
mod revisioned_cache;
mod types;
mod uniform_resources;
mod util;

mod buffers;
mod config;
mod frame_targets;
mod fullscreen;
mod intermediate_pool;
mod pipelines;
mod render_plan;
mod render_plan_compiler;
mod render_plan_dump;
mod render_plan_effects;
mod render_scene;
mod resources;
mod services;
mod shaders;
mod svg;
#[cfg(test)]
mod tests;

use bind_group_caches::BindGroupCaches;
use clip_path_mask_cache::*;
use fullscreen::*;
use gpu_globals::GpuGlobals;
use gpu_pipelines::GpuPipelines;
use gpu_textures::GpuTextures;
use intermediate_pool::*;
use path::*;
use render_plan::*;
use types::*;
pub use types::{IntermediatePerfSnapshot, RenderPerfSnapshot, SvgPerfSnapshot};
use uniform_resources::UniformResources;
use util::*;

#[derive(Debug, Clone, Copy)]
pub struct ClearColor(pub wgpu::Color);

impl Default for ClearColor {
    fn default() -> Self {
        Self(wgpu::Color {
            r: 0.08,
            g: 0.09,
            b: 0.10,
            a: 1.0,
        })
    }
}

pub struct Renderer {
    adapter: wgpu::Adapter,
    uniform_bind_group: wgpu::BindGroup,
    uniforms: UniformResources,
    globals: GpuGlobals,
    textures: GpuTextures,
    pipelines: GpuPipelines,

    quad_instances: buffers::StorageRingBuffer<QuadInstance>,

    path_paints: buffers::StorageRingBuffer<PaintGpu>,

    text_paints: buffers::StorageRingBuffer<PaintGpu>,

    viewport_vertices: buffers::RingBuffer<ViewportVertex>,

    text_vertices: buffers::RingBuffer<TextVertex>,

    clip_mask_param_buffer: wgpu::Buffer,
    clip_mask_param_bind_group: wgpu::BindGroup,
    clip_mask_param_bind_group_layout: wgpu::BindGroupLayout,

    scale_pipeline_format: Option<wgpu::TextureFormat>,
    downsample_pipeline: Option<wgpu::RenderPipeline>,
    upscale_pipeline: Option<wgpu::RenderPipeline>,
    upscale_masked_pipeline: Option<wgpu::RenderPipeline>,
    upscale_mask_pipeline: Option<wgpu::RenderPipeline>,
    scale_bind_group_layout: Option<wgpu::BindGroupLayout>,
    scale_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,
    scale_param_buffer: wgpu::Buffer,
    scale_param_stride: u64,
    scale_param_capacity: usize,

    backdrop_warp_pipeline_format: Option<wgpu::TextureFormat>,
    backdrop_warp_pipeline: Option<wgpu::RenderPipeline>,
    backdrop_warp_masked_pipeline: Option<wgpu::RenderPipeline>,
    backdrop_warp_mask_pipeline: Option<wgpu::RenderPipeline>,
    backdrop_warp_bind_group_layout: Option<wgpu::BindGroupLayout>,
    backdrop_warp_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    backdrop_warp_image_pipeline: Option<wgpu::RenderPipeline>,
    backdrop_warp_image_masked_pipeline: Option<wgpu::RenderPipeline>,
    backdrop_warp_image_mask_pipeline: Option<wgpu::RenderPipeline>,
    backdrop_warp_image_bind_group_layout: Option<wgpu::BindGroupLayout>,
    backdrop_warp_image_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,
    backdrop_warp_param_buffer: wgpu::Buffer,

    color_adjust_pipeline_format: Option<wgpu::TextureFormat>,
    color_adjust_pipeline: Option<wgpu::RenderPipeline>,
    color_adjust_masked_pipeline: Option<wgpu::RenderPipeline>,
    color_adjust_mask_pipeline: Option<wgpu::RenderPipeline>,
    color_adjust_bind_group_layout: Option<wgpu::BindGroupLayout>,
    color_adjust_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,
    color_adjust_param_buffer: wgpu::Buffer,

    color_matrix_pipeline_format: Option<wgpu::TextureFormat>,
    color_matrix_pipeline: Option<wgpu::RenderPipeline>,
    color_matrix_masked_pipeline: Option<wgpu::RenderPipeline>,
    color_matrix_mask_pipeline: Option<wgpu::RenderPipeline>,
    color_matrix_bind_group_layout: Option<wgpu::BindGroupLayout>,
    color_matrix_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,
    color_matrix_param_buffer: wgpu::Buffer,

    alpha_threshold_pipeline_format: Option<wgpu::TextureFormat>,
    alpha_threshold_pipeline: Option<wgpu::RenderPipeline>,
    alpha_threshold_masked_pipeline: Option<wgpu::RenderPipeline>,
    alpha_threshold_mask_pipeline: Option<wgpu::RenderPipeline>,
    alpha_threshold_bind_group_layout: Option<wgpu::BindGroupLayout>,
    alpha_threshold_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,
    alpha_threshold_param_buffer: wgpu::Buffer,

    drop_shadow_pipeline_format: Option<wgpu::TextureFormat>,
    drop_shadow_pipeline: Option<wgpu::RenderPipeline>,
    drop_shadow_masked_pipeline: Option<wgpu::RenderPipeline>,
    drop_shadow_mask_pipeline: Option<wgpu::RenderPipeline>,
    drop_shadow_bind_group_layout: Option<wgpu::BindGroupLayout>,
    drop_shadow_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,
    drop_shadow_param_buffer: wgpu::Buffer,

    path_vertices: buffers::RingBuffer<PathVertex>,

    path_intermediate: Option<PathIntermediate>,
    path_composite_vertices: wgpu::Buffer,
    path_composite_vertex_capacity: usize,

    text_system: TextSystem,

    paths: SlotMap<fret_core::PathId, PreparedPath>,
    path_cache: HashMap<PathCacheKey, CachedPathEntry>,
    path_cache_capacity: usize,
    path_cache_epoch: u64,

    svg_renderer: SvgRenderer,
    svgs: SlotMap<fret_core::SvgId, SvgEntry>,
    svg_hash_index: HashMap<u64, Vec<fret_core::SvgId>>,
    svg_rasters: HashMap<SvgRasterKey, SvgRasterEntry>,
    svg_mask_atlas_pages: Vec<Option<SvgMaskAtlasPage>>,
    svg_mask_atlas_free: Vec<usize>,
    // Bytes used by standalone SVG rasters (not atlas-backed).
    svg_raster_bytes: u64,
    // Bytes reserved by SVG alpha-mask atlas pages.
    svg_mask_atlas_bytes: u64,
    svg_raster_budget_bytes: u64,
    svg_raster_epoch: u64,
    svg_perf_enabled: bool,
    svg_perf: SvgPerfStats,

    clip_path_mask_cache: ClipPathMaskCache,

    perf_enabled: bool,
    // Per-frame SVG cache stats (best-effort; populated only when `perf_enabled` is true).
    perf_svg_raster_cache_hits: u64,
    perf_svg_raster_cache_misses: u64,
    perf_svg_raster_budget_evictions: u64,
    perf_svg_mask_atlas_page_evictions: u64,
    perf_svg_mask_atlas_entries_evicted: u64,
    perf_pending_render_target_updates_requested_by_ingest:
        [u64; RenderTargetIngestStrategy::COUNT],
    perf_pending_render_target_updates_by_ingest: [u64; RenderTargetIngestStrategy::COUNT],
    perf_pending_render_target_updates_ingest_fallbacks: u64,
    perf_pending_render_target_metadata_degradations_color_encoding_dropped: u64,
    perf: RenderPerfStats,
    last_frame_perf: Option<RenderPerfSnapshot>,
    last_render_plan_segment_report: Option<Vec<RenderPlanSegmentReport>>,
    render_scene_frame_index: u64,

    path_msaa_samples: u32,
    debug_offscreen_blit_enabled: bool,
    debug_pixelate_scale: u32,
    debug_blur_radius: u32,
    debug_blur_scissor: Option<ScissorRect>,
    intermediate_budget_bytes: u64,
    intermediate_perf_enabled: bool,
    intermediate_perf: IntermediatePerfStats,
    intermediate_pool: IntermediatePool,

    render_targets: RenderTargetRegistry,
    images: ImageRegistry,

    bind_group_caches: BindGroupCaches,
    render_target_revisions: HashMap<fret_core::RenderTargetId, u64>,
    render_targets_generation: u64,

    image_revisions: HashMap<fret_core::ImageId, u64>,
    images_generation: u64,

    scene_encoding_cache_key: Option<SceneEncodingCacheKey>,
    scene_encoding_cache: SceneEncoding,
    scene_encoding_scratch: SceneEncoding,

    materials: SlotMap<fret_core::MaterialId, MaterialEntry>,
    materials_by_desc: HashMap<fret_core::MaterialDescriptor, fret_core::MaterialId>,
    material_paint_budget_per_frame: u64,
    material_distinct_budget_per_frame: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RenderPlanSegmentReport {
    draw_range: (usize, usize),
    start_uniform_fingerprint: u64,
    flags_mask: u8,
    scene_draw_range_passes: u32,
    path_msaa_batch_passes: u32,
}

#[derive(Clone, Copy, Debug)]
struct MaterialEntry {
    desc: fret_core::MaterialDescriptor,
    refs: u32,
}

pub struct RenderSceneParams<'a> {
    pub format: wgpu::TextureFormat,
    pub target_view: &'a wgpu::TextureView,
    pub scene: &'a Scene,
    pub clear: ClearColor,
    pub scale_factor: f32,
    pub viewport_size: (u32, u32),
}

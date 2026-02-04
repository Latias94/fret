use crate::images::ImageRegistry;
use crate::svg::SvgRenderer;
use crate::targets::RenderTargetRegistry;
use crate::text::TextSystem;
pub(super) use fret_core::{
    geometry::{Point, Px, Rect, Size, Transform2D},
    scene::{Color, Scene, SceneOp, UvRect},
};
use slotmap::SlotMap;
use std::collections::HashMap;
use std::sync::Arc;

// Split from the original single-file renderer for maintainability.
mod path;
mod types;
mod util;

mod buffers;
mod config;
mod frame_targets;
mod fullscreen;
mod intermediate_pool;
mod pipelines;
mod render_plan;
mod render_plan_dump;
mod render_plan_effects;
mod render_scene;
mod resources;
mod services;
mod shaders;
mod svg;
#[cfg(test)]
mod tests;

use fullscreen::*;
use intermediate_pool::*;
use path::*;
use render_plan::*;
use types::*;
pub use types::{IntermediatePerfSnapshot, RenderPerfSnapshot, SvgPerfSnapshot};
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
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_stride: u64,
    uniform_capacity: usize,
    clip_buffer: wgpu::Buffer,
    clip_capacity: usize,

    quad_pipeline_format: Option<wgpu::TextureFormat>,
    quad_pipeline: Option<wgpu::RenderPipeline>,

    viewport_pipeline_format: Option<wgpu::TextureFormat>,
    viewport_pipeline: Option<wgpu::RenderPipeline>,
    viewport_bind_group_layout: wgpu::BindGroupLayout,
    viewport_sampler: wgpu::Sampler,

    instance_buffers: Vec<wgpu::Buffer>,
    instance_buffer_index: usize,
    instance_capacity: usize,

    viewport_vertex_buffers: Vec<wgpu::Buffer>,
    viewport_vertex_buffer_index: usize,
    viewport_vertex_capacity: usize,

    text_pipeline_format: Option<wgpu::TextureFormat>,
    text_pipeline: Option<wgpu::RenderPipeline>,

    text_color_pipeline_format: Option<wgpu::TextureFormat>,
    text_color_pipeline: Option<wgpu::RenderPipeline>,

    text_subpixel_pipeline_format: Option<wgpu::TextureFormat>,
    text_subpixel_pipeline: Option<wgpu::RenderPipeline>,

    mask_pipeline_format: Option<wgpu::TextureFormat>,
    mask_pipeline: Option<wgpu::RenderPipeline>,

    text_vertex_buffers: Vec<wgpu::Buffer>,
    text_vertex_buffer_index: usize,
    text_vertex_capacity: usize,

    path_pipeline_format: Option<wgpu::TextureFormat>,
    path_pipeline: Option<wgpu::RenderPipeline>,

    path_msaa_pipeline_format: Option<wgpu::TextureFormat>,
    path_msaa_pipeline: Option<wgpu::RenderPipeline>,
    path_msaa_pipeline_sample_count: Option<u32>,

    composite_pipeline_format: Option<wgpu::TextureFormat>,
    composite_pipeline: Option<wgpu::RenderPipeline>,
    composite_mask_pipeline: Option<wgpu::RenderPipeline>,
    composite_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    clip_mask_pipeline: Option<wgpu::RenderPipeline>,
    clip_mask_param_buffer: wgpu::Buffer,
    clip_mask_param_bind_group: wgpu::BindGroup,
    clip_mask_param_bind_group_layout: wgpu::BindGroupLayout,

    blit_pipeline_format: Option<wgpu::TextureFormat>,
    blit_pipeline: Option<wgpu::RenderPipeline>,
    blit_bind_group_layout: Option<wgpu::BindGroupLayout>,
    blit_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    blur_pipeline_format: Option<wgpu::TextureFormat>,
    blur_h_pipeline: Option<wgpu::RenderPipeline>,
    blur_v_pipeline: Option<wgpu::RenderPipeline>,
    blur_h_masked_pipeline: Option<wgpu::RenderPipeline>,
    blur_v_masked_pipeline: Option<wgpu::RenderPipeline>,
    blur_h_mask_pipeline: Option<wgpu::RenderPipeline>,
    blur_v_mask_pipeline: Option<wgpu::RenderPipeline>,

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

    color_adjust_pipeline_format: Option<wgpu::TextureFormat>,
    color_adjust_pipeline: Option<wgpu::RenderPipeline>,
    color_adjust_masked_pipeline: Option<wgpu::RenderPipeline>,
    color_adjust_mask_pipeline: Option<wgpu::RenderPipeline>,
    color_adjust_bind_group_layout: Option<wgpu::BindGroupLayout>,
    color_adjust_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,
    color_adjust_param_buffer: wgpu::Buffer,

    path_vertex_buffers: Vec<wgpu::Buffer>,
    path_vertex_buffer_index: usize,
    path_vertex_capacity: usize,

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

    perf_enabled: bool,
    perf: RenderPerfStats,
    last_frame_perf: Option<RenderPerfSnapshot>,
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

    viewport_bind_groups: HashMap<fret_core::RenderTargetId, (u64, wgpu::BindGroup)>,
    render_target_revisions: HashMap<fret_core::RenderTargetId, u64>,
    render_targets_generation: u64,

    image_bind_groups: HashMap<fret_core::ImageId, (u64, wgpu::BindGroup)>,
    image_revisions: HashMap<fret_core::ImageId, u64>,
    images_generation: u64,

    scene_encoding_cache_key: Option<SceneEncodingCacheKey>,
    scene_encoding_cache: SceneEncoding,
    scene_encoding_scratch: SceneEncoding,
}

pub struct RenderSceneParams<'a> {
    pub format: wgpu::TextureFormat,
    pub target_view: &'a wgpu::TextureView,
    pub scene: &'a Scene,
    pub clear: ClearColor,
    pub scale_factor: f32,
    pub viewport_size: (u32, u32),
}

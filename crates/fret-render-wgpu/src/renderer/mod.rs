use crate::svg::SvgRenderer;
use crate::text::TextSystem;
pub(super) use fret_core::{
    geometry::{Point, Px, Rect, Size, Transform2D},
    scene::{Color, Scene, SceneOp, UvRect},
};
use slotmap::SlotMap;
use std::collections::HashMap;
use std::sync::Arc;

// Split from the original single-file renderer for maintainability.
mod bind_group_builders;
mod bind_group_caches;
mod blur_primitive;
mod clip_path_mask_cache;
mod gpu_effect_params;
mod gpu_globals;
mod gpu_pipelines;
mod gpu_registries;
mod gpu_resources;
mod gpu_textures;
mod path;
mod revisioned_cache;
mod types;
mod uniform_resources;
mod util;

mod buffers;
mod config;
mod diagnostics;
mod frame_targets;
mod fullscreen;
mod intermediate_pool;
mod material_effects;
mod pipelines;
mod render_plan;
mod render_plan_compiler;
mod render_plan_dump;
mod render_plan_effects;
mod render_scene;
mod render_text_dump;
mod resources;
mod scene_encoding_cache;
mod services;
mod shaders;
mod svg;
#[cfg(test)]
mod tests;

mod v3_pyramid;

use clip_path_mask_cache::*;
use diagnostics::*;
use fullscreen::*;
use gpu_effect_params::GpuEffectParams;
use gpu_globals::GpuGlobals;
use gpu_pipelines::GpuPipelines;
use gpu_resources::GpuResources;
use gpu_textures::GpuTextures;
use intermediate_pool::*;
use material_effects::*;
use path::*;
use render_plan::*;
use scene_encoding_cache::SceneEncodingCache;
use types::*;
pub use types::{BlurQualityCounters, BlurQualitySnapshot};
pub use types::{
    CustomEffectV3SourceDegradationCounters, EffectDegradationCounters, EffectDegradationSnapshot,
};
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
    viewport_uniform_bytes_scratch: Vec<u8>,
    render_space_bytes_scratch: Vec<u8>,
    plan_quad_vertices_scratch: Vec<ViewportVertex>,
    plan_quad_vertex_bases_scratch: Vec<Option<u32>>,
    render_plan_scene_draw_range_passes_scratch: Vec<u32>,
    render_plan_path_msaa_batch_passes_scratch: Vec<u32>,
    render_plan_segment_report_scratch: Vec<RenderPlanSegmentReport>,
    render_plan_dump_scratch: render_plan_dump::RenderPlanJsonDumpScratch,
    render_plan_strict_output_clear: bool,
    globals: GpuGlobals,
    textures: GpuTextures,
    effect_params: GpuEffectParams,
    pipelines: GpuPipelines,

    custom_effect_v3_pyramid: v3_pyramid::CustomEffectV3PyramidState,

    quad_instances: buffers::StorageRingBuffer<QuadInstance>,

    path_paints: buffers::StorageRingBuffer<PaintGpu>,

    text_paints: buffers::StorageRingBuffer<PaintGpu>,

    viewport_vertices: buffers::RingBuffer<ViewportVertex>,

    text_vertices: buffers::RingBuffer<TextVertex>,

    path_vertices: buffers::RingBuffer<PathVertex>,

    path_intermediate: Option<PathIntermediate>,
    path_composite_vertices: wgpu::Buffer,
    path_composite_vertex_capacity: usize,

    text_system: TextSystem,

    path_state: PathState,

    svg_registry_state: svg::SvgRegistryState,
    svg_raster_state: svg::SvgRasterState,

    clip_path_mask_cache: ClipPathMaskCache,

    diagnostics_state: DiagnosticsState,

    path_msaa_samples: u32,
    debug_offscreen_blit_enabled: bool,
    debug_pixelate_scale: u32,
    debug_blur_radius: u32,
    debug_blur_scissor: Option<ScissorRect>,
    intermediate_state: IntermediateState,

    gpu_resources: GpuResources,

    scene_encoding_cache: SceneEncodingCache,

    material_effect_state: MaterialEffectState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RenderPlanSegmentReport {
    draw_range: (usize, usize),
    start_uniform_fingerprint: u64,
    flags_mask: u8,
    scene_draw_range_passes: u32,
    path_msaa_batch_passes: u32,
}

pub struct RenderSceneParams<'a> {
    pub format: wgpu::TextureFormat,
    pub target_view: &'a wgpu::TextureView,
    pub scene: &'a Scene,
    pub clear: ClearColor,
    pub scale_factor: f32,
    pub viewport_size: (u32, u32),
}

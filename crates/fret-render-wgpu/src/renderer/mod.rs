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
mod frame_scratch;
mod frame_targets;
mod fullscreen;
mod intermediate_pool;
mod material_effects;
mod pipelines;
mod render_plan;
mod render_plan_compiler;
mod render_plan_dump;
mod render_plan_effects;
mod render_plan_reporting;
mod render_scene;
mod render_scene_config;
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
use frame_scratch::*;
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
use render_plan_reporting::*;
use render_scene_config::*;
use scene_encoding_cache::SceneEncodingState;
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
    frame_scratch_state: FrameScratchState,
    render_plan_reporting_state: RenderPlanReportingState,
    render_scene_config_state: RenderSceneConfigState,
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

    text_system: TextSystem,

    path_state: PathState,

    render_text_dump_state: render_text_dump::RenderTextDumpState,

    svg_registry_state: svg::SvgRegistryState,
    svg_raster_state: svg::SvgRasterState,

    clip_path_mask_cache: ClipPathMaskCache,

    diagnostics_state: DiagnosticsState,

    intermediate_state: IntermediateState,

    gpu_resources: GpuResources,

    scene_encoding_state: SceneEncodingState,

    material_effect_state: MaterialEffectState,
}
pub struct RenderSceneParams<'a> {
    pub format: wgpu::TextureFormat,
    pub target_view: &'a wgpu::TextureView,
    pub scene: &'a Scene,
    pub clear: ClearColor,
    pub scale_factor: f32,
    pub viewport_size: (u32, u32),
}

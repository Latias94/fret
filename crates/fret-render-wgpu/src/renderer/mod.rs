use crate::svg::SvgRenderer;
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
mod scene_encoding_cache;
mod services;
mod shaders;
mod svg;
#[cfg(test)]
mod tests;

mod v3_pyramid;

use clip_path_mask_cache::*;
use fullscreen::*;
use gpu_effect_params::GpuEffectParams;
use gpu_globals::GpuGlobals;
use gpu_pipelines::GpuPipelines;
use gpu_resources::GpuResources;
use gpu_textures::GpuTextures;
use intermediate_pool::*;
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

    custom_effect_v3_pyramid_scratch: Option<v3_pyramid::CustomEffectV3PyramidScratch>,
    custom_effect_v3_pyramid_cache: Option<CustomEffectV3PyramidCache>,
    plan_target_write_epochs: [u32; 8],

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

    gpu_resources: GpuResources,

    scene_encoding_cache: SceneEncodingCache,

    materials: SlotMap<fret_core::MaterialId, MaterialEntry>,
    materials_by_desc: HashMap<fret_core::MaterialDescriptor, fret_core::MaterialId>,
    materials_generation: u64,
    material_paint_budget_per_frame: u64,
    material_distinct_budget_per_frame: usize,

    custom_effects: SlotMap<fret_core::EffectId, CustomEffectEntry>,
    custom_effect_hash_index: HashMap<u64, Vec<fret_core::EffectId>>,
    custom_effects_generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CustomEffectV3PyramidCache {
    src_raw: PlanTarget,
    src_size: (u32, u32),
    format: wgpu::TextureFormat,
    levels: u32,
    src_raw_epoch: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RenderPlanSegmentReport {
    draw_range: (usize, usize),
    start_uniform_fingerprint: u64,
    flags_mask: u8,
    scene_draw_range_passes: u32,
    path_msaa_batch_passes: u32,
}

impl Renderer {
    pub(in crate::renderer) fn reset_frame_local_custom_effect_v3_caches(&mut self) {
        self.custom_effect_v3_pyramid_cache = None;
        self.plan_target_write_epochs = [0; 8];
    }

    pub(in crate::renderer) fn bump_plan_target_write_epoch(&mut self, target: PlanTarget) {
        let ix = plan_target_epoch_slot(target);
        self.plan_target_write_epochs[ix] = self.plan_target_write_epochs[ix].saturating_add(1);
        if self
            .custom_effect_v3_pyramid_cache
            .is_some_and(|c| c.src_raw == target)
        {
            self.custom_effect_v3_pyramid_cache = None;
        }
    }

    pub(in crate::renderer) fn can_reuse_custom_effect_v3_pyramid(
        &self,
        src_raw: PlanTarget,
        src_size: (u32, u32),
        format: wgpu::TextureFormat,
        levels: u32,
    ) -> bool {
        let Some(cache) = self.custom_effect_v3_pyramid_cache else {
            return false;
        };
        if cache.src_raw != src_raw
            || cache.src_size != src_size
            || cache.format != format
            || cache.levels != levels
        {
            return false;
        }
        let ix = plan_target_epoch_slot(src_raw);
        cache.src_raw_epoch == self.plan_target_write_epochs[ix]
    }

    pub(in crate::renderer) fn set_custom_effect_v3_pyramid_cache(
        &mut self,
        src_raw: PlanTarget,
        src_size: (u32, u32),
        format: wgpu::TextureFormat,
        levels: u32,
    ) {
        let ix = plan_target_epoch_slot(src_raw);
        self.custom_effect_v3_pyramid_cache = Some(CustomEffectV3PyramidCache {
            src_raw,
            src_size,
            format,
            levels,
            src_raw_epoch: self.plan_target_write_epochs[ix],
        });
    }
}

fn plan_target_epoch_slot(target: PlanTarget) -> usize {
    match target {
        PlanTarget::Output => 0,
        PlanTarget::Intermediate0 => 1,
        PlanTarget::Intermediate1 => 2,
        PlanTarget::Intermediate2 => 3,
        PlanTarget::Intermediate3 => 4,
        PlanTarget::Mask0 => 5,
        PlanTarget::Mask1 => 6,
        PlanTarget::Mask2 => 7,
    }
}

#[derive(Clone, Copy, Debug)]
struct MaterialEntry {
    desc: fret_core::MaterialDescriptor,
    refs: u32,
}

#[derive(Clone, Debug)]
struct CustomEffectEntry {
    abi: CustomEffectAbi,
    raw_source: Arc<str>,
    wgsl_unmasked: Arc<str>,
    wgsl_masked: Arc<str>,
    wgsl_mask: Arc<str>,
    refs: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CustomEffectAbi {
    V1,
    V2,
    V3,
}

pub struct RenderSceneParams<'a> {
    pub format: wgpu::TextureFormat,
    pub target_view: &'a wgpu::TextureView,
    pub scene: &'a Scene,
    pub clear: ClearColor,
    pub scale_factor: f32,
    pub viewport_size: (u32, u32),
}

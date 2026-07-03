use crate::svg::SvgRenderer;
use crate::text::TextSystem;
pub(super) use fret_core::{
    geometry::{Point, Px, Rect, Size, Transform2D},
    scene::{Color, Scene, SceneOp, UvPoint, UvRect},
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
#[cfg(not(target_arch = "wasm32"))]
mod debug_dump_gate;
mod diagnostics;
mod frame_binding_state;
mod frame_scratch;
mod frame_targets;
mod fullscreen;
mod geometry_upload;
mod intermediate_pool;
mod material_effects;
mod pipelines;
mod render_plan;
mod render_plan_compiler;
mod render_plan_dump;
#[cfg(not(target_arch = "wasm32"))]
mod render_plan_dump_assemble;
#[cfg(target_arch = "wasm32")]
#[path = "render_plan_dump_assemble_wasm.rs"]
mod render_plan_dump_assemble;
#[cfg(not(target_arch = "wasm32"))]
mod render_plan_dump_emit;
#[cfg(not(target_arch = "wasm32"))]
mod render_plan_dump_encode;
#[cfg(not(target_arch = "wasm32"))]
mod render_plan_dump_summary;
mod render_plan_effects;
mod render_plan_reporting;
mod render_plan_reporting_perf;
mod render_scene;
mod render_scene_config;
#[cfg(not(target_arch = "wasm32"))]
mod render_text_dump;
#[cfg(target_arch = "wasm32")]
#[path = "render_text_dump_wasm.rs"]
mod render_text_dump;
mod resources;
mod scene_chunk_encoding_cache;
mod scene_encoding_cache;
mod scene_encoding_cache_diagnostics;
mod services;
mod services_assets;
mod services_custom_effects;
mod shaders;
mod svg;
#[cfg(test)]
mod tests;

mod v3_pyramid;

use clip_path_mask_cache::*;
use diagnostics::*;
use frame_binding_state::*;
use frame_scratch::*;
use fullscreen::*;
use geometry_upload::*;
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
use render_scene::frame_assembler::FrameAssembler;
use render_scene_config::*;
use scene_chunk_encoding_cache::*;
use scene_encoding_cache::SceneEncodingState;
use types::*;
pub use types::{BlurQualityCounters, BlurQualitySnapshot};
pub use types::{
    CustomEffectV3SourceDegradationCounters, EffectDegradationCounters, EffectDegradationSnapshot,
};
pub use types::{
    GeometryUploadPerfSnapshot, IntermediatePerfSnapshot, RenderPerfSnapshot,
    SceneEncodingCacheMissHistogramSnapshot, SvgPerfSnapshot,
};
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
    frame_binding_state: FrameBindingState,
    frame_scratch_state: FrameScratchState,
    render_plan_reporting_state: RenderPlanReportingState,
    render_scene_config_state: RenderSceneConfigState,
    globals: GpuGlobals,
    textures: GpuTextures,
    effect_params: GpuEffectParams,
    pipelines: GpuPipelines,
    geometry_upload_state: GeometryUploadState,

    custom_effect_v3_pyramid: v3_pyramid::CustomEffectV3PyramidState,

    text_system: TextSystem,
    text_scene_resource_key_state: TextSceneResourceKeyState,

    path_state: PathState,

    #[cfg(not(target_arch = "wasm32"))]
    render_text_dump_state: render_text_dump::RenderTextDumpState,

    svg_registry_state: svg::SvgRegistryState,
    svg_raster_state: svg::SvgRasterState,

    clip_path_mask_cache: ClipPathMaskCache,

    diagnostics_state: DiagnosticsState,

    intermediate_state: IntermediateState,

    gpu_resources: GpuResources,

    frame_assembler: FrameAssembler,

    scene_encoding_state: SceneEncodingState,

    material_effect_state: MaterialEffectState,
}
pub struct RenderSceneParams<'a> {
    pub format: wgpu::TextureFormat,
    pub target_view: &'a wgpu::TextureView,
    pub source: RenderSceneSourceSelection<'a>,
    pub clear: ClearColor,
    pub scale_factor: f32,
    pub viewport_size: (u32, u32),
}

#[derive(Debug, Clone, Copy)]
pub enum RenderSceneSource<'a> {
    FlatCompat {
        scene: &'a Scene,
    },
    ChunkManifest {
        manifest: &'a fret_core::SceneChunkManifest,
        debug_scene: Option<&'a Scene>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSceneChunkAuthorityPolicy {
    FlatCompat,
    ChunkManifestWhenSupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSceneDebugFlatOraclePolicy {
    Disabled,
    Requested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSceneSourcePolicy {
    pub chunk_authority: RenderSceneChunkAuthorityPolicy,
    pub debug_flat_oracle: RenderSceneDebugFlatOraclePolicy,
}

impl RenderSceneSourcePolicy {
    pub const fn flat_compat() -> Self {
        Self {
            chunk_authority: RenderSceneChunkAuthorityPolicy::FlatCompat,
            debug_flat_oracle: RenderSceneDebugFlatOraclePolicy::Disabled,
        }
    }

    pub const fn chunk_manifest_when_supported() -> Self {
        Self {
            chunk_authority: RenderSceneChunkAuthorityPolicy::ChunkManifestWhenSupported,
            debug_flat_oracle: RenderSceneDebugFlatOraclePolicy::Disabled,
        }
    }

    pub const fn with_debug_flat_oracle(mut self) -> Self {
        self.debug_flat_oracle = RenderSceneDebugFlatOraclePolicy::Requested;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkLaunchStreamClass {
    ResourceFreeQuad,
    ResourceFreeVertexColor,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkLaunchUnsupportedReason {
    NoManifest,
    EmptyManifest,
    ManifestUnsupported(fret_core::SceneChunkManifestUnsupportedReason),
    MixedStreams,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkLaunchSupport {
    Supported {
        stream_class: ChunkLaunchStreamClass,
    },
    Unsupported {
        stream_class: Option<ChunkLaunchStreamClass>,
        reason: ChunkLaunchUnsupportedReason,
    },
}

impl ChunkLaunchSupport {
    pub const fn is_supported(self) -> bool {
        matches!(self, Self::Supported { .. })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderSceneSourceSelection<'a> {
    source: RenderSceneSource<'a>,
    chunk_manifest: Option<&'a fret_core::SceneChunkManifest>,
    chunk_support: ChunkLaunchSupport,
    debug_flat_oracle_requested: bool,
}

impl<'a> RenderSceneSourceSelection<'a> {
    pub fn flat_compat(scene: &'a Scene) -> Self {
        Self {
            source: RenderSceneSource::FlatCompat { scene },
            chunk_manifest: None,
            chunk_support: ChunkLaunchSupport::Unsupported {
                stream_class: None,
                reason: ChunkLaunchUnsupportedReason::NoManifest,
            },
            debug_flat_oracle_requested: false,
        }
    }

    pub fn chunk_manifest(
        manifest: &'a fret_core::SceneChunkManifest,
        debug_scene: Option<&'a Scene>,
    ) -> Self {
        Self {
            source: RenderSceneSource::ChunkManifest {
                manifest,
                debug_scene,
            },
            chunk_manifest: Some(manifest),
            chunk_support: ChunkLaunchSupportMatrix::evaluate(manifest),
            debug_flat_oracle_requested: debug_scene.is_some(),
        }
    }

    pub fn source(self) -> RenderSceneSource<'a> {
        self.source
    }

    pub fn assembly_manifest(self) -> Option<&'a fret_core::SceneChunkManifest> {
        self.chunk_manifest
    }

    pub fn chunk_support(self) -> ChunkLaunchSupport {
        self.chunk_support
    }

    pub fn debug_flat_oracle_requested(self) -> bool {
        self.debug_flat_oracle_requested
    }
}

pub struct ChunkLaunchSupportMatrix;

impl ChunkLaunchSupportMatrix {
    pub fn evaluate(manifest: &fret_core::SceneChunkManifest) -> ChunkLaunchSupport {
        let stream_class = manifest_stream_class(manifest);

        match stream_class {
            None => ChunkLaunchSupport::Unsupported {
                stream_class: None,
                reason: ChunkLaunchUnsupportedReason::EmptyManifest,
            },
            Some(stream_class) => {
                if let Some(reason) = manifest.assembly_unsupported_reasons().first().copied() {
                    return ChunkLaunchSupport::Unsupported {
                        stream_class: Some(stream_class),
                        reason: ChunkLaunchUnsupportedReason::ManifestUnsupported(reason),
                    };
                }

                match stream_class {
                    ChunkLaunchStreamClass::ResourceFreeQuad => ChunkLaunchSupport::Supported {
                        stream_class: ChunkLaunchStreamClass::ResourceFreeQuad,
                    },
                    ChunkLaunchStreamClass::ResourceFreeVertexColor => {
                        ChunkLaunchSupport::Supported {
                            stream_class: ChunkLaunchStreamClass::ResourceFreeVertexColor,
                        }
                    }
                    ChunkLaunchStreamClass::Mixed => ChunkLaunchSupport::Unsupported {
                        stream_class: Some(ChunkLaunchStreamClass::Mixed),
                        reason: ChunkLaunchUnsupportedReason::MixedStreams,
                    },
                }
            }
        }
    }
}

fn manifest_stream_class(
    manifest: &fret_core::SceneChunkManifest,
) -> Option<ChunkLaunchStreamClass> {
    let mut stream_class = None;
    for entry in manifest.entries() {
        let streams = entry.chunk().closure().draw_streams();
        let entry_class = if streams.is_quad_only() {
            ChunkLaunchStreamClass::ResourceFreeQuad
        } else if streams.is_vertex_color_only() {
            ChunkLaunchStreamClass::ResourceFreeVertexColor
        } else {
            ChunkLaunchStreamClass::Mixed
        };

        stream_class = match (stream_class, entry_class) {
            (None, class) => Some(class),
            (Some(existing), class) if existing == class => Some(existing),
            _ => Some(ChunkLaunchStreamClass::Mixed),
        };
    }

    stream_class
}
pub fn select_render_scene_source<'a>(
    scene: &'a Scene,
    manifest: &'a fret_core::SceneChunkManifest,
    policy: RenderSceneSourcePolicy,
) -> RenderSceneSourceSelection<'a> {
    let chunk_support = ChunkLaunchSupportMatrix::evaluate(manifest);
    let chunk_manifest_is_authoritative = policy.chunk_authority
        == RenderSceneChunkAuthorityPolicy::ChunkManifestWhenSupported
        && chunk_support.is_supported();
    let debug_flat_oracle_requested =
        policy.debug_flat_oracle == RenderSceneDebugFlatOraclePolicy::Requested;

    let source = if chunk_manifest_is_authoritative {
        RenderSceneSource::ChunkManifest {
            manifest,
            debug_scene: debug_flat_oracle_requested.then_some(scene),
        }
    } else {
        RenderSceneSource::FlatCompat { scene }
    };

    RenderSceneSourceSelection {
        source,
        chunk_manifest: Some(manifest),
        chunk_support,
        debug_flat_oracle_requested,
    }
}

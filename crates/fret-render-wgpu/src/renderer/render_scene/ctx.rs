use super::super::frame_targets::FrameTargets;
use super::super::{RenderPerfStats, SceneEncoding};

pub(super) struct ExecuteCtx<'a> {
    pub(super) device: &'a wgpu::Device,
    pub(super) queue: &'a wgpu::Queue,
    pub(super) frame_index: u64,
    pub(super) format: wgpu::TextureFormat,
    pub(super) target_view: &'a wgpu::TextureView,
    pub(super) viewport_size: (u32, u32),
    pub(super) usage: wgpu::TextureUsages,
    pub(super) encoder: &'a mut wgpu::CommandEncoder,
    pub(super) frame_targets: &'a mut FrameTargets,
    pub(super) encoding: &'a SceneEncoding,
    pub(super) render_space_offset_u32: u32,
    pub(super) scale_param_size: u64,
    pub(super) scale_param_cursor: &'a mut u32,
    pub(super) quad_vertex_size: u64,
    pub(super) quad_vertex_bases: &'a [Option<u32>],
    pub(super) perf_enabled: bool,
    pub(super) frame_perf: &'a mut RenderPerfStats,
}

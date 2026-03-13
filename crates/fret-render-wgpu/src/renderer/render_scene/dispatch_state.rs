use super::super::frame_targets::FrameTargets;
use super::super::*;
use super::executor::{RecordPassCtx, RecordPassResources, RenderSceneExecutor};
use super::helpers::{render_plan_pass_trace_kind, render_plan_pass_trace_meta};

pub(super) struct RenderSceneDispatchState {
    encoder: wgpu::CommandEncoder,
    frame_targets: FrameTargets,
    scale_param_cursor: u32,
    scale_param_size: u64,
    quad_vertex_size: u64,
}

impl RenderSceneDispatchState {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        Self {
            encoder: device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("fret renderer encoder"),
            }),
            frame_targets: FrameTargets::default(),
            scale_param_cursor: 0,
            scale_param_size: std::mem::size_of::<ScaleParamsUniform>() as u64,
            quad_vertex_size: std::mem::size_of::<ViewportVertex>() as u64,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_passes(
        &mut self,
        renderer: &mut Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame_index: u64,
        format: wgpu::TextureFormat,
        target_view: &wgpu::TextureView,
        viewport_size: (u32, u32),
        plan: &RenderPlan,
        encoding: &SceneEncoding,
        quad_vertex_bases: &[Option<u32>],
        resources: &RecordPassResources<'_>,
        perf_enabled: bool,
        trace_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) {
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST;
        let render_space_capacity = renderer.render_space_capacity();
        let render_space_stride = renderer.render_space_stride();

        let (_, record_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.record_passes", frame_index),
            || {
                let mut executor = RenderSceneExecutor::new(
                    renderer,
                    device,
                    queue,
                    frame_index,
                    format,
                    target_view,
                    viewport_size,
                    usage,
                    &mut self.encoder,
                    &mut self.frame_targets,
                    encoding,
                    self.scale_param_size,
                    &mut self.scale_param_cursor,
                    self.quad_vertex_size,
                    quad_vertex_bases,
                    perf_enabled,
                    frame_perf,
                );

                for (pass_index, planned_pass) in plan.passes.iter().enumerate() {
                    debug_assert!(
                        pass_index < render_space_capacity,
                        "render_space_capacity too small for RenderPlan passes"
                    );
                    let pass_span = if trace_enabled {
                        let kind = render_plan_pass_trace_kind(planned_pass);
                        let meta = render_plan_pass_trace_meta(planned_pass);
                        tracing::trace_span!(
                            "fret.renderer.pass",
                            frame_index,
                            pass_index = pass_index as u32,
                            kind,
                            src = ?meta.src,
                            dst = ?meta.dst,
                            load = meta.load.unwrap_or(""),
                            scissor = ?meta.scissor,
                            scissor_space = meta
                                .scissor_space
                                .map(|s| s.as_str())
                                .unwrap_or(""),
                            render_origin = ?meta.render_origin,
                            render_size = ?meta.render_size
                        )
                    } else {
                        tracing::Span::none()
                    };
                    let _pass_guard = pass_span.enter();
                    let render_space_offset =
                        (pass_index as u64).saturating_mul(render_space_stride);
                    let render_space_offset_u32 = render_space_offset as u32;
                    let ctx = RecordPassCtx {
                        plan,
                        pass_index,
                        render_space_offset_u32,
                    };

                    executor.record_pass(planned_pass, &ctx, resources);
                }
            },
        );
        if let Some(record_elapsed) = record_elapsed {
            frame_perf.record_passes += record_elapsed;
        }
    }

    pub(super) fn finish(
        self,
        renderer: &mut Renderer,
        frame_index: u64,
        plan: &RenderPlan,
        perf_enabled: bool,
        trace_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) -> wgpu::CommandBuffer {
        let Self {
            encoder,
            mut frame_targets,
            ..
        } = self;

        let (cmd, finish_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.encoder.finish", frame_index),
            || encoder.finish(),
        );
        if let Some(finish_elapsed) = finish_elapsed {
            frame_perf.encoder_finish += finish_elapsed;
        }

        renderer.intermediate_state.record_in_use_bytes(
            frame_targets.in_use_bytes(),
            frame_targets.peak_in_use_bytes(),
        );
        if perf_enabled {
            frame_perf.intermediate_in_use_bytes = frame_targets.in_use_bytes();
            frame_perf.intermediate_peak_in_use_bytes = frame_targets.peak_in_use_bytes();
            frame_perf.intermediate_release_targets = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::ReleaseTarget(_)))
                .count() as u64;
        }
        let release_targets = plan
            .passes
            .iter()
            .filter(|p| matches!(p, RenderPlanPass::ReleaseTarget(_)))
            .count() as u64;
        renderer
            .intermediate_state
            .record_release_targets(release_targets);
        frame_targets.release_all(
            &mut renderer.intermediate_state.pool,
            renderer.intermediate_state.budget_bytes,
        );

        cmd
    }
}

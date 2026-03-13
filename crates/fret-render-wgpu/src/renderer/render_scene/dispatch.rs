use super::super::frame_targets::FrameTargets;
use super::super::*;
use super::executor::{RecordPassCtx, RecordPassResources, RenderSceneExecutor};
use super::helpers::{render_plan_pass_trace_kind, render_plan_pass_trace_meta};

impl Renderer {
    pub(super) fn dispatch_render_plan(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame_index: u64,
        format: wgpu::TextureFormat,
        target_view: &wgpu::TextureView,
        viewport_size: (u32, u32),
        plan: &RenderPlan,
        encoding: &SceneEncoding,
        quad_vertex_bases: &[Option<u32>],
        viewport_vertex_buffer: &wgpu::Buffer,
        text_vertex_buffer: &wgpu::Buffer,
        path_vertex_buffer: &wgpu::Buffer,
        quad_instance_bind_group: &wgpu::BindGroup,
        text_paint_bind_group: &wgpu::BindGroup,
        path_paint_bind_group: &wgpu::BindGroup,
        perf_enabled: bool,
        trace_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) -> wgpu::CommandBuffer {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fret renderer encoder"),
        });

        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST;
        let mut frame_targets = FrameTargets::default();
        let scale_param_size = std::mem::size_of::<ScaleParamsUniform>() as u64;
        let mut scale_param_cursor: u32 = 0;
        let quad_vertex_size = std::mem::size_of::<ViewportVertex>() as u64;

        let (_, record_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.record_passes", frame_index),
            || {
                let render_space_capacity = self.render_space_capacity();
                let render_space_stride = self.render_space_stride();
                let mut executor = RenderSceneExecutor::new(
                    self,
                    device,
                    queue,
                    frame_index,
                    format,
                    target_view,
                    viewport_size,
                    usage,
                    &mut encoder,
                    &mut frame_targets,
                    encoding,
                    scale_param_size,
                    &mut scale_param_cursor,
                    quad_vertex_size,
                    quad_vertex_bases,
                    perf_enabled,
                    frame_perf,
                );
                let resources = RecordPassResources {
                    viewport_vertex_buffer,
                    text_vertex_buffer,
                    path_vertex_buffer,
                    quad_instance_bind_group,
                    text_paint_bind_group,
                    path_paint_bind_group,
                };
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

                    executor.record_pass(planned_pass, &ctx, &resources);
                }
            },
        );
        if let Some(record_elapsed) = record_elapsed {
            frame_perf.record_passes += record_elapsed;
        }

        let (cmd, finish_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.encoder.finish", frame_index),
            || encoder.finish(),
        );
        if let Some(finish_elapsed) = finish_elapsed {
            frame_perf.encoder_finish += finish_elapsed;
        }

        self.intermediate_state.record_in_use_bytes(
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
        self.intermediate_state
            .record_release_targets(release_targets);
        frame_targets.release_all(
            &mut self.intermediate_state.pool,
            self.intermediate_state.budget_bytes,
        );

        cmd
    }
}

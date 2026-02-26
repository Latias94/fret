use super::super::*;

impl Renderer {
    pub(super) fn compile_render_plan_for_scene(
        &mut self,
        frame_index: u64,
        perf_enabled: bool,
        trace_enabled: bool,
        encoding: &SceneEncoding,
        scale_factor: f32,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        clear: wgpu::Color,
        path_samples: u32,
        postprocess: DebugPostprocess,
        frame_perf: &mut RenderPerfStats,
    ) -> RenderPlan {
        let (plan, plan_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.plan.compile", frame_index),
            || {
                RenderPlan::compile_for_scene(
                    encoding,
                    scale_factor,
                    viewport_size,
                    format,
                    clear,
                    path_samples,
                    postprocess,
                    self.intermediate_budget_bytes,
                )
            },
        );
        if let Some(plan_elapsed) = plan_elapsed {
            frame_perf.plan_compile += plan_elapsed;
        }
        plan
    }
}

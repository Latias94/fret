use super::super::*;

impl Renderer {
    pub(super) fn record_render_plan_diagnostics_for_frame(
        &mut self,
        perf_enabled: bool,
        plan: &RenderPlan,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        frame_index: u64,
        postprocess: DebugPostprocess,
        ordered_draws_len: usize,
        effect_markers: &[EffectMarker],
        frame_perf: &mut RenderPerfStats,
    ) {
        self.render_plan_reporting_state
            .record_render_plan_diagnostics_for_frame(
                &mut self.diagnostics_state,
                perf_enabled,
                plan,
                viewport_size,
                format,
                frame_index,
                postprocess,
                ordered_draws_len,
                effect_markers,
                frame_perf,
            );
    }
}

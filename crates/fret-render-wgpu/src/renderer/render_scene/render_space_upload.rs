use super::super::*;
impl Renderer {
    pub(super) fn upload_render_space_uniforms_for_plan(
        &mut self,
        queue: &wgpu::Queue,
        plan: &RenderPlan,
    ) {
        self.frame_binding_state
            .upload_render_space_uniforms_for_plan(queue, &mut self.frame_scratch_state, plan);
    }
}

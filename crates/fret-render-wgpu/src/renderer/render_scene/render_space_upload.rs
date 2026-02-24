use super::super::*;
use super::helpers::render_plan_pass_render_space;

impl Renderer {
    pub(super) fn upload_render_space_uniforms_for_plan(
        &mut self,
        queue: &wgpu::Queue,
        plan: &RenderPlan,
    ) {
        debug_assert!(
            (std::mem::size_of::<RenderSpaceUniform>() as u64) <= self.uniforms.render_space_stride,
            "render_space_stride must fit RenderSpaceUniform"
        );

        let render_space_uniform_size = std::mem::size_of::<RenderSpaceUniform>();
        let render_space_stride = self.uniforms.render_space_stride as usize;
        let render_space_bytes_len = render_space_stride.saturating_mul(plan.passes.len());
        self.render_space_bytes_scratch.clear();
        self.render_space_bytes_scratch
            .resize(render_space_bytes_len, 0u8);
        for (pass_index, planned_pass) in plan.passes.iter().enumerate() {
            let Some((origin, size)) = render_plan_pass_render_space(planned_pass) else {
                continue;
            };
            let offset = render_space_stride.saturating_mul(pass_index);
            self.render_space_bytes_scratch[offset..offset + render_space_uniform_size]
                .copy_from_slice(bytemuck::bytes_of(&RenderSpaceUniform {
                    origin_px: [origin.0 as f32, origin.1 as f32],
                    size_px: [size.0.max(1) as f32, size.1.max(1) as f32],
                }));
        }
        if !self.render_space_bytes_scratch.is_empty() {
            let _ = self
                .uniforms
                .write_render_space_bytes(queue, &self.render_space_bytes_scratch);
        }
    }
}

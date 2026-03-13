use super::super::*;

fn render_plan_uses_path_intermediate(passes: &[RenderPlanPass]) -> bool {
    passes
        .iter()
        .any(|pass| matches!(pass, RenderPlanPass::PathMsaaBatch(_)))
}

impl PathIntermediate {
    pub(in crate::renderer) fn estimated_msaa_bytes(&self) -> u64 {
        if self.sample_count > 1 {
            estimate_texture_bytes(self.size, self.format, self.sample_count)
        } else {
            0
        }
    }

    pub(in crate::renderer) fn estimated_resolved_bytes(&self) -> u64 {
        estimate_texture_bytes(self.size, self.format, 1)
    }

    pub(in crate::renderer) fn estimated_bytes(&self) -> u64 {
        self.estimated_msaa_bytes()
            .saturating_add(self.estimated_resolved_bytes())
    }
}

impl Renderer {
    pub(in crate::renderer) fn sync_path_intermediate_for_plan(
        &mut self,
        device: &wgpu::Device,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        sample_count: u32,
        passes: &[RenderPlanPass],
    ) {
        if sample_count <= 1 || !render_plan_uses_path_intermediate(passes) {
            self.path_state.clear_intermediate();
            return;
        }

        self.ensure_path_intermediate(device, viewport_size, format, sample_count);
    }

    pub(in crate::renderer) fn ensure_path_composite_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        required_vertices: usize,
    ) {
        self.path_state
            .ensure_path_composite_vertex_buffer(device, required_vertices);
    }

    pub(in crate::renderer) fn path_intermediate_ref(&self) -> Option<&PathIntermediate> {
        self.path_state.intermediate()
    }

    pub(in crate::renderer) fn path_composite_vertices_ref(&self) -> &wgpu::Buffer {
        self.path_state.composite_vertices()
    }

    pub(in crate::renderer) fn ensure_path_intermediate(
        &mut self,
        device: &wgpu::Device,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) {
        self.path_state.ensure_path_intermediate(
            device,
            &self.globals,
            viewport_size,
            format,
            sample_count,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_path_msaa_batch_pass() -> RenderPlanPass {
        RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass {
            segment: SceneSegmentId(0),
            target: PlanTarget::Output,
            target_origin: (0, 0),
            target_size: (1, 1),
            draw_range: 0..1,
            union_scissor: AbsoluteScissorRect(ScissorRect::full(1, 1)),
            batch_uniform_index: 0,
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        })
    }

    #[test]
    fn render_plan_usage_detection_only_counts_path_msaa_batches() {
        let non_path_passes = [RenderPlanPass::ReleaseTarget(PlanTarget::Output)];
        let path_msaa_passes = [sample_path_msaa_batch_pass()];

        assert!(!render_plan_uses_path_intermediate(&[]));
        assert!(!render_plan_uses_path_intermediate(&non_path_passes));
        assert!(render_plan_uses_path_intermediate(&path_msaa_passes));
    }
}

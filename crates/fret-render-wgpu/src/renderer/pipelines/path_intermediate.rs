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
            self.path_intermediate = None;
            return;
        }

        self.ensure_path_intermediate(device, viewport_size, format, sample_count);
    }

    pub(in crate::renderer) fn ensure_path_composite_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        required_vertices: usize,
    ) {
        if required_vertices == 0 {
            return;
        }

        if self.path_composite_vertex_capacity >= required_vertices {
            return;
        }

        let next_capacity = required_vertices
            .next_power_of_two()
            .max(self.path_composite_vertex_capacity.max(64 * 6));

        self.path_composite_vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret path composite vertices"),
            size: (next_capacity * std::mem::size_of::<ViewportVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.path_composite_vertex_capacity = next_capacity;
    }

    pub(in crate::renderer) fn ensure_path_intermediate(
        &mut self,
        device: &wgpu::Device,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) {
        let needs_rebuild = match &self.path_intermediate {
            Some(cur) => {
                cur.size != viewport_size
                    || cur.format != format
                    || cur.sample_count != sample_count
            }
            None => true,
        };
        if !needs_rebuild {
            return;
        }

        let (msaa_texture, msaa_view) = if sample_count > 1 {
            let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fret path intermediate msaa"),
                size: wgpu::Extent3d {
                    width: viewport_size.0,
                    height: viewport_size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(msaa_texture), Some(msaa_view))
        } else {
            (None, None)
        };

        let resolved_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret path intermediate resolved"),
            size: wgpu::Extent3d {
                width: viewport_size.0,
                height: viewport_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let resolved_view = resolved_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret path intermediate bind group"),
            layout: &self.globals.viewport_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.globals.viewport_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&resolved_view),
                },
            ],
        });

        self.path_intermediate = Some(PathIntermediate {
            size: viewport_size,
            format,
            sample_count,
            _msaa_texture: msaa_texture,
            msaa_view,
            _resolved_texture: resolved_texture,
            resolved_view,
            bind_group,
        });
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

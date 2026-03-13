use super::bind_group_builders::{UniformBindGroupGlobals, UniformMaskImageBindGroupGlobals};
use super::*;
use crate::renderer::render_scene::helpers::render_plan_pass_render_space;

pub(super) struct FrameBindingState {
    uniform_bind_group: wgpu::BindGroup,
    uniforms: UniformResources,
}

impl FrameBindingState {
    pub(super) fn new(uniform_bind_group: wgpu::BindGroup, uniforms: UniformResources) -> Self {
        Self {
            uniform_bind_group,
            uniforms,
        }
    }

    pub(super) fn uniform_bind_group(&self) -> &wgpu::BindGroup {
        &self.uniform_bind_group
    }

    pub(super) fn uniform_stride(&self) -> u64 {
        self.uniforms.uniform_stride
    }

    pub(super) fn render_space_stride(&self) -> u64 {
        self.uniforms.render_space_stride
    }

    pub(super) fn render_space_capacity(&self) -> usize {
        self.uniforms.render_space_capacity
    }

    fn create_uniform_bind_group(
        &self,
        device: &wgpu::Device,
        label: &'static str,
        globals: &GpuGlobals,
        uniform_buffer: &wgpu::Buffer,
        clip_buffer: &wgpu::Buffer,
        mask_buffer: &wgpu::Buffer,
        render_space_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        UniformBindGroupGlobals {
            layout: &globals.uniform_bind_group_layout,
            material_catalog_view: &globals.material_catalog_view,
            material_catalog_sampler: &globals.material_catalog_sampler,
            mask_image_sampler: &globals.mask_image_sampler,
            mask_image_identity_view: &globals.mask_image_identity_view,
        }
        .create(
            device,
            label,
            uniform_buffer,
            clip_buffer,
            mask_buffer,
            render_space_buffer,
        )
    }

    fn rebuild_uniform_bind_group(
        &mut self,
        device: &wgpu::Device,
        globals: &GpuGlobals,
        label: &'static str,
    ) {
        self.uniform_bind_group = self.create_uniform_bind_group(
            device,
            label,
            globals,
            &self.uniforms.uniform_buffer,
            &self.uniforms.clip_buffer,
            &self.uniforms.mask_buffer,
            &self.uniforms.render_space_buffer,
        );
    }

    pub(super) fn ensure_uniform_capacity(
        &mut self,
        device: &wgpu::Device,
        globals: &GpuGlobals,
        gpu_resources: &mut GpuResources,
        needed: usize,
    ) {
        if !self
            .uniforms
            .ensure_viewport_uniform_capacity(device, needed)
        {
            return;
        }
        gpu_resources
            .caches_mut()
            .invalidate_uniform_mask_image_override_bind_groups();
        self.rebuild_uniform_bind_group(device, globals, "fret uniforms bind group (resized)");
    }

    pub(super) fn ensure_render_space_capacity(
        &mut self,
        device: &wgpu::Device,
        globals: &GpuGlobals,
        gpu_resources: &mut GpuResources,
        needed: usize,
    ) {
        if !self.uniforms.ensure_render_space_capacity(device, needed) {
            return;
        }
        gpu_resources
            .caches_mut()
            .invalidate_uniform_mask_image_override_bind_groups();
        self.rebuild_uniform_bind_group(
            device,
            globals,
            "fret uniforms bind group (resized render space)",
        );
    }

    pub(super) fn ensure_clip_capacity(
        &mut self,
        device: &wgpu::Device,
        globals: &GpuGlobals,
        gpu_resources: &mut GpuResources,
        needed: usize,
    ) {
        if !self.uniforms.ensure_clip_capacity(device, needed) {
            return;
        }
        gpu_resources
            .caches_mut()
            .invalidate_uniform_mask_image_override_bind_groups();
        self.rebuild_uniform_bind_group(
            device,
            globals,
            "fret uniforms bind group (resized clip buffer)",
        );
    }

    pub(super) fn ensure_mask_capacity(
        &mut self,
        device: &wgpu::Device,
        globals: &GpuGlobals,
        gpu_resources: &mut GpuResources,
        needed: usize,
    ) {
        if !self.uniforms.ensure_mask_capacity(device, needed) {
            return;
        }
        gpu_resources
            .caches_mut()
            .invalidate_uniform_mask_image_override_bind_groups();
        self.rebuild_uniform_bind_group(
            device,
            globals,
            "fret uniforms bind group (resized mask buffer)",
        );
    }

    pub(super) fn upload_frame_uniforms_and_prepare_bind_groups(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        globals: &GpuGlobals,
        gpu_resources: &mut GpuResources,
        frame_scratch_state: &mut FrameScratchState,
        uniforms: &[ViewportUniform],
        clips: &[ClipRRectUniform],
        masks: &[MaskGradientUniform],
        ordered_draws: &[OrderedDraw],
        uniform_mask_images: &[Option<UniformMaskImageSelection>],
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) {
        self.ensure_uniform_capacity(device, globals, gpu_resources, uniforms.len());
        let viewport_uniform_bytes_scratch = frame_scratch_state.viewport_uniform_bytes_mut();
        let uniform_bytes_written = self.uniforms.write_viewport_uniforms_into(
            queue,
            uniforms,
            viewport_uniform_bytes_scratch,
        ) as u64;
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf
                .uniform_bytes
                .saturating_add(uniform_bytes_written);
        }

        self.ensure_clip_capacity(device, globals, gpu_resources, clips.len().max(1));
        let clip_bytes_written = self.uniforms.write_clips(queue, clips) as u64;
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf.uniform_bytes.saturating_add(clip_bytes_written);
        }

        self.ensure_mask_capacity(device, globals, gpu_resources, masks.len().max(1));
        let mask_bytes_written = self.uniforms.write_masks(queue, masks) as u64;
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf.uniform_bytes.saturating_add(mask_bytes_written);
        }

        for item in ordered_draws {
            let OrderedDraw::Viewport(draw) = item else {
                continue;
            };

            gpu_resources.ensure_viewport_sampler_texture_bind_group_for_target(
                device,
                &globals.viewport_bind_group_layout,
                &globals.viewport_sampler,
                draw.target,
            );
        }

        for item in ordered_draws {
            let image = match item {
                OrderedDraw::Image(draw) => draw.image,
                OrderedDraw::Mask(draw) => draw.image,
                _ => continue,
            };
            gpu_resources.ensure_image_sampler_texture_bind_groups_for_image(
                device,
                &globals.viewport_bind_group_layout,
                &globals.viewport_sampler,
                &globals.image_sampler_nearest,
                image,
            );
        }

        let bind_group_globals = UniformMaskImageBindGroupGlobals {
            layout: &globals.uniform_bind_group_layout,
            uniform_buffer: &self.uniforms.uniform_buffer,
            clip_buffer: &self.uniforms.clip_buffer,
            mask_buffer: &self.uniforms.mask_buffer,
            material_catalog_view: &globals.material_catalog_view,
            material_catalog_sampler: &globals.material_catalog_sampler,
            render_space_buffer: &self.uniforms.render_space_buffer,
        };

        for &sel in uniform_mask_images.iter().flatten() {
            gpu_resources.ensure_uniform_mask_image_override_bind_groups_for_image(
                device,
                &bind_group_globals,
                &globals.mask_image_sampler,
                &globals.mask_image_sampler_nearest,
                sel.image,
                self.uniforms.revision(),
            );
        }
    }

    pub(super) fn upload_render_space_uniforms_for_plan(
        &mut self,
        queue: &wgpu::Queue,
        frame_scratch_state: &mut FrameScratchState,
        plan: &RenderPlan,
    ) {
        debug_assert!(
            (std::mem::size_of::<RenderSpaceUniform>() as u64) <= self.uniforms.render_space_stride,
            "render_space_stride must fit RenderSpaceUniform"
        );

        let render_space_uniform_size = std::mem::size_of::<RenderSpaceUniform>();
        let render_space_stride = self.uniforms.render_space_stride as usize;
        let render_space_bytes_len = render_space_stride.saturating_mul(plan.passes.len());
        let render_space_bytes_scratch =
            frame_scratch_state.render_space_bytes_mut(render_space_bytes_len);
        for (pass_index, planned_pass) in plan.passes.iter().enumerate() {
            let Some((origin, size)) = render_plan_pass_render_space(planned_pass) else {
                continue;
            };
            let offset = render_space_stride.saturating_mul(pass_index);
            render_space_bytes_scratch[offset..offset + render_space_uniform_size].copy_from_slice(
                bytemuck::bytes_of(&RenderSpaceUniform {
                    origin_px: [origin.0 as f32, origin.1 as f32],
                    size_px: [size.0.max(1) as f32, size.1.max(1) as f32],
                }),
            );
        }
        if !render_space_bytes_scratch.is_empty() {
            let _ = self
                .uniforms
                .write_render_space_bytes(queue, render_space_bytes_scratch);
        }
    }

    pub(super) fn pick_uniform_bind_group_for_mask_image<'a>(
        &'a self,
        gpu_resources: &'a GpuResources,
        mask_image: Option<UniformMaskImageSelection>,
    ) -> &'a wgpu::BindGroup {
        let Some(sel) = mask_image else {
            return &self.uniform_bind_group;
        };
        let Some(groups) = gpu_resources
            .caches()
            .get_uniform_mask_image_bind_groups(sel.image)
        else {
            return &self.uniform_bind_group;
        };
        groups.pick(sel.sampling)
    }
}

impl Renderer {
    pub(super) fn base_uniform_bind_group(&self) -> &wgpu::BindGroup {
        self.frame_binding_state.uniform_bind_group()
    }

    pub(super) fn uniform_stride(&self) -> u64 {
        self.frame_binding_state.uniform_stride()
    }

    pub(super) fn render_space_stride(&self) -> u64 {
        self.frame_binding_state.render_space_stride()
    }

    pub(super) fn render_space_capacity(&self) -> usize {
        self.frame_binding_state.render_space_capacity()
    }

    pub(super) fn ensure_render_space_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        self.frame_binding_state.ensure_render_space_capacity(
            device,
            &self.globals,
            &mut self.gpu_resources,
            needed,
        );
    }
}

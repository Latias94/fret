pub(super) struct GpuEffectParams {
    pub(super) clip_mask_param_buffer: wgpu::Buffer,
    pub(super) clip_mask_param_bind_group: wgpu::BindGroup,
    pub(super) clip_mask_param_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) scale_param_buffer: wgpu::Buffer,
    pub(super) scale_param_stride: u64,
    pub(super) scale_param_capacity: usize,

    pub(super) backdrop_warp_param_buffer: wgpu::Buffer,

    pub(super) color_adjust_param_buffer: wgpu::Buffer,
    pub(super) color_matrix_param_buffer: wgpu::Buffer,
    pub(super) alpha_threshold_param_buffer: wgpu::Buffer,
    pub(super) noise_param_buffer: wgpu::Buffer,
    pub(super) drop_shadow_param_buffer: wgpu::Buffer,
    pub(super) custom_effect_param_buffer: wgpu::Buffer,
    pub(super) custom_effect_v2_input_meta_buffer: wgpu::Buffer,
}

impl GpuEffectParams {
    pub(super) fn ensure_scale_param_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.scale_param_capacity {
            return;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.scale_param_capacity.saturating_mul(2).max(1));
        let scale_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret scale params buffer (resized)"),
            size: self.scale_param_stride * new_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.scale_param_buffer = scale_param_buffer;
        self.scale_param_capacity = new_capacity;
    }
}

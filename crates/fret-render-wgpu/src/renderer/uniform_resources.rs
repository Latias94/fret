pub(super) struct UniformResources {
    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) uniform_stride: u64,
    pub(super) uniform_capacity: usize,

    pub(super) render_space_buffer: wgpu::Buffer,
    pub(super) render_space_stride: u64,
    pub(super) render_space_capacity: usize,

    pub(super) clip_buffer: wgpu::Buffer,
    pub(super) clip_capacity: usize,

    pub(super) mask_buffer: wgpu::Buffer,
    pub(super) mask_capacity: usize,

    revision: u64,
}

impl UniformResources {
    pub(super) fn new(
        uniform_buffer: wgpu::Buffer,
        uniform_stride: u64,
        uniform_capacity: usize,
        render_space_buffer: wgpu::Buffer,
        render_space_stride: u64,
        render_space_capacity: usize,
        clip_buffer: wgpu::Buffer,
        clip_capacity: usize,
        mask_buffer: wgpu::Buffer,
        mask_capacity: usize,
    ) -> Self {
        Self {
            uniform_buffer,
            uniform_stride,
            uniform_capacity,
            render_space_buffer,
            render_space_stride,
            render_space_capacity,
            clip_buffer,
            clip_capacity,
            mask_buffer,
            mask_capacity,
            revision: 1,
        }
    }

    pub(super) fn revision(&self) -> u64 {
        self.revision
    }

    pub(super) fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    pub(super) fn ensure_viewport_uniform_capacity(
        &mut self,
        device: &wgpu::Device,
        needed: usize,
    ) -> bool {
        if needed <= self.uniform_capacity {
            return false;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.uniform_capacity.saturating_mul(2).max(1));
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret uniforms buffer (resized)"),
            size: self.uniform_stride * new_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.uniform_buffer = uniform_buffer;
        self.uniform_capacity = new_capacity;
        self.bump_revision();
        true
    }

    pub(super) fn ensure_render_space_capacity(
        &mut self,
        device: &wgpu::Device,
        needed: usize,
    ) -> bool {
        if needed <= self.render_space_capacity {
            return false;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.render_space_capacity.saturating_mul(2).max(1));

        let render_space_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret render-space uniform buffer (resized)"),
            size: self.render_space_stride * new_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.render_space_buffer = render_space_buffer;
        self.render_space_capacity = new_capacity;
        self.bump_revision();
        true
    }

    pub(super) fn ensure_clip_capacity(&mut self, device: &wgpu::Device, needed: usize) -> bool {
        if needed <= self.clip_capacity {
            return false;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.clip_capacity.saturating_mul(2).max(1));
        let clip_entry_size = std::mem::size_of::<super::ClipRRectUniform>() as u64;
        let clip_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret clip stack buffer (resized)"),
            size: clip_entry_size.saturating_mul(new_capacity as u64).max(4),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.clip_buffer = clip_buffer;
        self.clip_capacity = new_capacity;
        self.bump_revision();
        true
    }

    pub(super) fn ensure_mask_capacity(&mut self, device: &wgpu::Device, needed: usize) -> bool {
        if needed <= self.mask_capacity {
            return false;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.mask_capacity.saturating_mul(2).max(1));
        let mask_entry_size = std::mem::size_of::<super::MaskGradientUniform>() as u64;
        let mask_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret mask stack buffer (resized)"),
            size: mask_entry_size.saturating_mul(new_capacity as u64).max(4),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.mask_buffer = mask_buffer;
        self.mask_capacity = new_capacity;
        self.bump_revision();
        true
    }

    pub(super) fn write_viewport_uniforms(
        &self,
        queue: &wgpu::Queue,
        uniforms: &[super::ViewportUniform],
    ) -> usize {
        if uniforms.is_empty() {
            return 0;
        }

        let uniform_size = std::mem::size_of::<super::ViewportUniform>() as u64;
        let mut uniform_bytes = vec![0u8; (self.uniform_stride * uniforms.len() as u64) as usize];
        for (i, u) in uniforms.iter().enumerate() {
            let offset = (self.uniform_stride * i as u64) as usize;
            uniform_bytes[offset..offset + uniform_size as usize]
                .copy_from_slice(bytemuck::bytes_of(u));
        }

        queue.write_buffer(&self.uniform_buffer, 0, &uniform_bytes);
        uniform_bytes.len()
    }

    pub(super) fn write_clips(
        &self,
        queue: &wgpu::Queue,
        clips: &[super::ClipRRectUniform],
    ) -> usize {
        if clips.is_empty() {
            return 0;
        }

        queue.write_buffer(&self.clip_buffer, 0, bytemuck::cast_slice(clips));
        std::mem::size_of::<super::ClipRRectUniform>() * clips.len()
    }

    pub(super) fn write_masks(
        &self,
        queue: &wgpu::Queue,
        masks: &[super::MaskGradientUniform],
    ) -> usize {
        if masks.is_empty() {
            return 0;
        }

        queue.write_buffer(&self.mask_buffer, 0, bytemuck::cast_slice(masks));
        std::mem::size_of::<super::MaskGradientUniform>() * masks.len()
    }

    pub(super) fn write_render_space_bytes(
        &self,
        queue: &wgpu::Queue,
        render_space_bytes: &[u8],
    ) -> usize {
        if render_space_bytes.is_empty() {
            return 0;
        }
        queue.write_buffer(&self.render_space_buffer, 0, render_space_bytes);
        render_space_bytes.len()
    }
}

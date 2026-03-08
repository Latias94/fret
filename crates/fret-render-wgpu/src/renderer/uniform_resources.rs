fn skip_redundant_uniform_uploads_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("FRET_RENDER_WGPU_SKIP_REDUNDANT_UNIFORM_UPLOADS")
            .is_some_and(|v| !v.is_empty() && v != "0")
    })
}

fn write_buffer_if_needed(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    bytes: &[u8],
    last_uploaded: &mut Vec<u8>,
) -> bool {
    if skip_redundant_uniform_uploads_enabled() && last_uploaded.as_slice() == bytes {
        return false;
    }
    queue.write_buffer(buffer, 0, bytes);
    if skip_redundant_uniform_uploads_enabled() {
        last_uploaded.clear();
        last_uploaded.extend_from_slice(bytes);
    }
    true
}

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

    last_uniform_bytes: Vec<u8>,
    last_render_space_bytes: Vec<u8>,
    last_clip_bytes: Vec<u8>,
    last_mask_bytes: Vec<u8>,
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
            last_uniform_bytes: Vec::new(),
            last_render_space_bytes: Vec::new(),
            last_clip_bytes: Vec::new(),
            last_mask_bytes: Vec::new(),
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
        self.last_uniform_bytes.clear();
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
        self.last_render_space_bytes.clear();
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
        self.last_clip_bytes.clear();
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
        self.last_mask_bytes.clear();
        self.bump_revision();
        true
    }

    pub(super) fn write_viewport_uniforms_into(
        &mut self,
        queue: &wgpu::Queue,
        uniforms: &[super::ViewportUniform],
        uniform_bytes_scratch: &mut Vec<u8>,
    ) -> usize {
        if uniforms.is_empty() {
            return 0;
        }

        let uniform_size = std::mem::size_of::<super::ViewportUniform>() as u64;
        let uniform_bytes_len = (self.uniform_stride * uniforms.len() as u64) as usize;
        uniform_bytes_scratch.clear();
        uniform_bytes_scratch.resize(uniform_bytes_len, 0u8);
        for (i, u) in uniforms.iter().enumerate() {
            let offset = (self.uniform_stride * i as u64) as usize;
            uniform_bytes_scratch[offset..offset + uniform_size as usize]
                .copy_from_slice(bytemuck::bytes_of(u));
        }

        if write_buffer_if_needed(
            queue,
            &self.uniform_buffer,
            uniform_bytes_scratch,
            &mut self.last_uniform_bytes,
        ) {
            uniform_bytes_scratch.len()
        } else {
            0
        }
    }

    pub(super) fn write_clips(
        &mut self,
        queue: &wgpu::Queue,
        clips: &[super::ClipRRectUniform],
    ) -> usize {
        if clips.is_empty() {
            return 0;
        }

        let clip_bytes = bytemuck::cast_slice(clips);
        if write_buffer_if_needed(
            queue,
            &self.clip_buffer,
            clip_bytes,
            &mut self.last_clip_bytes,
        ) {
            std::mem::size_of_val(clips)
        } else {
            0
        }
    }

    pub(super) fn write_masks(
        &mut self,
        queue: &wgpu::Queue,
        masks: &[super::MaskGradientUniform],
    ) -> usize {
        if masks.is_empty() {
            return 0;
        }

        let mask_bytes = bytemuck::cast_slice(masks);
        if write_buffer_if_needed(
            queue,
            &self.mask_buffer,
            mask_bytes,
            &mut self.last_mask_bytes,
        ) {
            std::mem::size_of_val(masks)
        } else {
            0
        }
    }

    pub(super) fn write_render_space_bytes(
        &mut self,
        queue: &wgpu::Queue,
        render_space_bytes: &[u8],
    ) -> usize {
        if render_space_bytes.is_empty() {
            return 0;
        }
        if write_buffer_if_needed(
            queue,
            &self.render_space_buffer,
            render_space_bytes,
            &mut self.last_render_space_bytes,
        ) {
            render_space_bytes.len()
        } else {
            0
        }
    }
}

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
}

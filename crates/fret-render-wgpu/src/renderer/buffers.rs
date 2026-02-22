use super::*;

pub(super) struct StorageRingBuffer<T> {
    buffers: Vec<wgpu::Buffer>,
    bind_groups: Vec<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,
    buffer_index: usize,
    capacity: usize,
    label_prefix: Arc<str>,
    usage: wgpu::BufferUsages,
    _marker: std::marker::PhantomData<T>,
}

impl<T> StorageRingBuffer<T> {
    pub(super) fn new(
        device: &wgpu::Device,
        frames_in_flight: usize,
        capacity: usize,
        bind_group_layout: wgpu::BindGroupLayout,
        label_prefix: impl Into<Arc<str>>,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let label_prefix = label_prefix.into();
        let element_size = std::mem::size_of::<T>() as u64;
        let buffers: Vec<wgpu::Buffer> = (0..frames_in_flight)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("{label_prefix} #{i}")),
                    size: (capacity as u64).saturating_mul(element_size).max(4),
                    usage,
                    mapped_at_creation: false,
                })
            })
            .collect();

        let bind_groups: Vec<wgpu::BindGroup> = buffers
            .iter()
            .enumerate()
            .map(|(i, buffer)| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("{label_prefix} bind group #{i}")),
                    layout: &bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                })
            })
            .collect();

        Self {
            buffers,
            bind_groups,
            bind_group_layout,
            buffer_index: 0,
            capacity,
            label_prefix,
            usage,
            _marker: std::marker::PhantomData,
        }
    }

    pub(super) fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub(super) fn ensure_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.capacity {
            return;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.capacity.saturating_mul(2).max(1));
        let element_size = std::mem::size_of::<T>() as u64;

        let mut new_buffers = Vec::with_capacity(self.buffers.len());
        let mut new_bind_groups = Vec::with_capacity(self.bind_groups.len());
        for i in 0..self.buffers.len() {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{} (resized) #{i}", self.label_prefix)),
                size: (new_capacity as u64).saturating_mul(element_size).max(4),
                usage: self.usage,
                mapped_at_creation: false,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{} bind group (resized) #{i}", self.label_prefix)),
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
            new_buffers.push(buffer);
            new_bind_groups.push(bind_group);
        }

        self.buffers = new_buffers;
        self.bind_groups = new_bind_groups;
        self.buffer_index = 0;
        self.capacity = new_capacity;
    }

    pub(super) fn next_pair(&mut self) -> (wgpu::Buffer, wgpu::BindGroup) {
        let idx = self.buffer_index;
        self.buffer_index = (self.buffer_index + 1) % self.buffers.len();
        (self.buffers[idx].clone(), self.bind_groups[idx].clone())
    }
}

pub(super) struct RingBuffer<T> {
    buffers: Vec<wgpu::Buffer>,
    buffer_index: usize,
    capacity: usize,
    label_prefix: Arc<str>,
    usage: wgpu::BufferUsages,
    _marker: std::marker::PhantomData<T>,
}

impl<T> RingBuffer<T> {
    pub(super) fn new(
        device: &wgpu::Device,
        frames_in_flight: usize,
        capacity: usize,
        label_prefix: impl Into<Arc<str>>,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let label_prefix = label_prefix.into();
        let element_size = std::mem::size_of::<T>() as u64;
        let buffers: Vec<wgpu::Buffer> = (0..frames_in_flight)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("{label_prefix} #{i}")),
                    size: (capacity as u64).saturating_mul(element_size).max(4),
                    usage,
                    mapped_at_creation: false,
                })
            })
            .collect();

        Self {
            buffers,
            buffer_index: 0,
            capacity,
            label_prefix,
            usage,
            _marker: std::marker::PhantomData,
        }
    }

    pub(super) fn ensure_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.capacity {
            return;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.capacity.saturating_mul(2).max(1));
        let element_size = std::mem::size_of::<T>() as u64;

        let mut new_buffers = Vec::with_capacity(self.buffers.len());
        for i in 0..self.buffers.len() {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{} (resized) #{i}", self.label_prefix)),
                size: (new_capacity as u64).saturating_mul(element_size).max(4),
                usage: self.usage,
                mapped_at_creation: false,
            });
            new_buffers.push(buffer);
        }

        self.buffers = new_buffers;
        self.buffer_index = 0;
        self.capacity = new_capacity;
    }

    pub(super) fn next_buffer(&mut self) -> wgpu::Buffer {
        let idx = self.buffer_index;
        self.buffer_index = (self.buffer_index + 1) % self.buffers.len();
        self.buffers[idx].clone()
    }
}

impl Renderer {
    fn create_uniform_bind_group(
        &self,
        device: &wgpu::Device,
        label: &'static str,
        uniform_buffer: &wgpu::Buffer,
        clip_buffer: &wgpu::Buffer,
        mask_buffer: &wgpu::Buffer,
        render_space_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        let render_space_size = std::mem::size_of::<RenderSpaceUniform>() as u64;

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &self.uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: uniform_buffer,
                        offset: 0,
                        size: Some(std::num::NonZeroU64::new(uniform_size).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: clip_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: mask_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.material_catalog_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.material_catalog_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: render_space_buffer,
                        offset: 0,
                        size: Some(std::num::NonZeroU64::new(render_space_size).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&self.mask_image_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&self.mask_image_identity_view),
                },
            ],
        })
    }

    pub(super) fn ensure_uniform_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.uniform_capacity {
            return;
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

        let uniform_bind_group = self.create_uniform_bind_group(
            device,
            "fret uniforms bind group (resized)",
            &uniform_buffer,
            &self.clip_buffer,
            &self.mask_buffer,
            &self.render_space_buffer,
        );

        self.uniform_buffer = uniform_buffer;
        self.uniform_bind_group = uniform_bind_group;
        self.bind_group_caches.clear_uniform_mask_images();
        self.uniform_capacity = new_capacity;
    }

    pub(super) fn ensure_render_space_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.render_space_capacity {
            return;
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

        // Rebuild the uniform bind group because it references `render_space_buffer`.
        let uniform_bind_group = self.create_uniform_bind_group(
            device,
            "fret uniforms bind group (resized render space)",
            &self.uniform_buffer,
            &self.clip_buffer,
            &self.mask_buffer,
            &render_space_buffer,
        );

        self.render_space_buffer = render_space_buffer;
        self.uniform_bind_group = uniform_bind_group;
        self.bind_group_caches.clear_uniform_mask_images();
        self.render_space_capacity = new_capacity;
    }

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

    pub(super) fn ensure_clip_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.clip_capacity {
            return;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.clip_capacity.saturating_mul(2).max(1));
        let clip_entry_size = std::mem::size_of::<ClipRRectUniform>() as u64;
        let clip_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret clip stack buffer (resized)"),
            size: clip_entry_size.saturating_mul(new_capacity as u64).max(4),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = self.create_uniform_bind_group(
            device,
            "fret uniforms bind group (resized clip buffer)",
            &self.uniform_buffer,
            &clip_buffer,
            &self.mask_buffer,
            &self.render_space_buffer,
        );

        self.clip_buffer = clip_buffer;
        self.uniform_bind_group = uniform_bind_group;
        self.bind_group_caches.clear_uniform_mask_images();
        self.clip_capacity = new_capacity;
    }

    pub(super) fn ensure_mask_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.mask_capacity {
            return;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.mask_capacity.saturating_mul(2).max(1));
        let mask_entry_size = std::mem::size_of::<MaskGradientUniform>() as u64;
        let mask_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret mask stack buffer (resized)"),
            size: mask_entry_size.saturating_mul(new_capacity as u64).max(4),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = self.create_uniform_bind_group(
            device,
            "fret uniforms bind group (resized mask buffer)",
            &self.uniform_buffer,
            &self.clip_buffer,
            &mask_buffer,
            &self.render_space_buffer,
        );

        self.mask_buffer = mask_buffer;
        self.uniform_bind_group = uniform_bind_group;
        self.bind_group_caches.clear_uniform_mask_images();
        self.mask_capacity = new_capacity;
    }
}

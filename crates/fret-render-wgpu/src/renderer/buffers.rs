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

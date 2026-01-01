pub(in crate::renderer) struct SvgRasterGpu<'a> {
    pub(in crate::renderer) device: &'a wgpu::Device,
    pub(in crate::renderer) queue: &'a wgpu::Queue,
}

pub(in crate::renderer) type SvgMaskAtlasInsert = (
    fret_core::ImageId,
    fret_core::UvRect,
    (u32, u32),
    usize,
    etagere::AllocId,
);

mod atlas;
mod cache;
mod prepare;
mod raster;

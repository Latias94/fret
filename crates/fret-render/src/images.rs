pub use fret_core::ImageColorSpace;
use fret_core::ImageId;
use slotmap::SlotMap;

pub struct UploadedRgba8Image {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub format: wgpu::TextureFormat,
    pub color_space: ImageColorSpace,
}

pub struct ImageDescriptor {
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub format: wgpu::TextureFormat,
    pub color_space: ImageColorSpace,
}

struct ImageEntry {
    view: wgpu::TextureView,
    #[allow(dead_code)]
    size: (u32, u32),
    #[allow(dead_code)]
    format: wgpu::TextureFormat,
    #[allow(dead_code)]
    color_space: ImageColorSpace,
}

#[derive(Default)]
pub struct ImageRegistry {
    images: SlotMap<ImageId, ImageEntry>,
}

impl ImageRegistry {
    pub fn register(&mut self, desc: ImageDescriptor) -> ImageId {
        debug_assert_eq!(
            desc.format.is_srgb(),
            desc.color_space == ImageColorSpace::Srgb,
            "ImageDescriptor.format must match ImageColorSpace"
        );
        self.images.insert(ImageEntry {
            view: desc.view,
            size: desc.size,
            format: desc.format,
            color_space: desc.color_space,
        })
    }

    pub fn update(&mut self, id: ImageId, desc: ImageDescriptor) -> bool {
        debug_assert_eq!(
            desc.format.is_srgb(),
            desc.color_space == ImageColorSpace::Srgb,
            "ImageDescriptor.format must match ImageColorSpace"
        );
        let Some(entry) = self.images.get_mut(id) else {
            return false;
        };
        entry.view = desc.view;
        entry.size = desc.size;
        entry.format = desc.format;
        entry.color_space = desc.color_space;
        true
    }

    pub fn unregister(&mut self, id: ImageId) -> bool {
        self.images.remove(id).is_some()
    }

    pub(crate) fn get(&self, id: ImageId) -> Option<&wgpu::TextureView> {
        self.images.get(id).map(|t| &t.view)
    }
}

pub fn upload_rgba8_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    size_px: (u32, u32),
    rgba: &[u8],
    color_space: ImageColorSpace,
) -> UploadedRgba8Image {
    let (w, h) = size_px;
    debug_assert_eq!(rgba.len(), (w as usize) * (h as usize) * 4);

    let format = match color_space {
        ImageColorSpace::Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        ImageColorSpace::Linear => wgpu::TextureFormat::Rgba8Unorm,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fret rgba8 image"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bytes_per_row = w.saturating_mul(4);
    let aligned_bytes_per_row = bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);

    let data = if aligned_bytes_per_row == bytes_per_row {
        rgba.to_vec()
    } else {
        let mut padded = vec![0u8; (aligned_bytes_per_row * h) as usize];
        for row in 0..h as usize {
            let src0 = row * (w as usize) * 4;
            let src1 = src0 + (w as usize) * 4;
            let dst0 = row * aligned_bytes_per_row as usize;
            let dst1 = dst0 + (w as usize) * 4;
            padded[dst0..dst1].copy_from_slice(&rgba[src0..src1]);
        }
        padded
    };

    if w > 0 && h > 0 {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(aligned_bytes_per_row),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    UploadedRgba8Image {
        texture,
        view,
        size: size_px,
        format,
        color_space,
    }
}

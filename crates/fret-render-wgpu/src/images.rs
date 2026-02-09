use fret_core::ImageId;
pub use fret_core::{AlphaMode, ImageColorSpace};
use slotmap::SlotMap;

use crate::upload_counters::record_image_upload;

pub struct UploadedRgba8Image {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub format: wgpu::TextureFormat,
    pub color_space: ImageColorSpace,
}

impl UploadedRgba8Image {
    pub fn write_region(
        &self,
        queue: &wgpu::Queue,
        origin: (u32, u32),
        size_px: (u32, u32),
        bytes_per_row: u32,
        rgba: &[u8],
    ) {
        write_rgba8_texture_region(queue, &self.texture, origin, size_px, bytes_per_row, rgba);
    }
}

pub struct ImageDescriptor {
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub format: wgpu::TextureFormat,
    pub color_space: ImageColorSpace,
    pub alpha_mode: AlphaMode,
}

struct ImageEntry {
    view: wgpu::TextureView,
    size: (u32, u32),
    #[allow(dead_code)]
    format: wgpu::TextureFormat,
    #[allow(dead_code)]
    color_space: ImageColorSpace,
    #[allow(dead_code)]
    alpha_mode: AlphaMode,
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
            alpha_mode: desc.alpha_mode,
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
        entry.alpha_mode = desc.alpha_mode;
        true
    }

    pub fn unregister(&mut self, id: ImageId) -> bool {
        self.images.remove(id).is_some()
    }

    pub(crate) fn get(&self, id: ImageId) -> Option<&wgpu::TextureView> {
        self.images.get(id).map(|t| &t.view)
    }

    pub(crate) fn alpha_mode(&self, id: ImageId) -> Option<AlphaMode> {
        self.images.get(id).map(|t| t.alpha_mode)
    }

    pub(crate) fn size_px(&self, id: ImageId) -> Option<(u32, u32)> {
        self.images.get(id).map(|t| t.size)
    }
}

pub fn create_rgba8_image_storage(
    device: &wgpu::Device,
    size_px: (u32, u32),
    color_space: ImageColorSpace,
) -> UploadedRgba8Image {
    let (w, h) = size_px;

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
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    UploadedRgba8Image {
        texture,
        view,
        size: size_px,
        format,
        color_space,
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
    let UploadedRgba8Image {
        texture,
        view,
        size,
        format,
        color_space,
    } = create_rgba8_image_storage(device, size_px, color_space);

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
        record_image_upload(data.len());
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
        size,
        format,
        color_space,
    }
}

pub fn write_rgba8_texture_region(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    origin: (u32, u32),
    size_px: (u32, u32),
    bytes_per_row: u32,
    rgba: &[u8],
) {
    let (w, h) = size_px;
    if w == 0 || h == 0 {
        return;
    }

    let row_bytes = w.saturating_mul(4);
    debug_assert!(row_bytes > 0);

    if bytes_per_row < row_bytes {
        debug_assert!(
            false,
            "write_rgba8_texture_region bytes_per_row must be >= width*4"
        );
        return;
    }

    let expected_len = (bytes_per_row as usize).saturating_mul(h as usize);
    debug_assert_eq!(rgba.len(), expected_len);
    if rgba.len() != expected_len {
        return;
    }

    let aligned_ok = bytes_per_row.is_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let mut owned: Vec<u8> = Vec::new();
    let (bytes, dst_bpr) = if aligned_ok {
        (rgba, bytes_per_row)
    } else {
        let aligned_bpr = row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let aligned_bpr = aligned_bpr.max(row_bytes);

        owned.resize((aligned_bpr as usize).saturating_mul(h as usize), 0);
        for row in 0..h as usize {
            let src0 = row.saturating_mul(bytes_per_row as usize);
            let src1 = src0.saturating_add(row_bytes as usize);
            let dst0 = row.saturating_mul(aligned_bpr as usize);
            let dst1 = dst0.saturating_add(row_bytes as usize);
            owned[dst0..dst1].copy_from_slice(&rgba[src0..src1]);
        }

        (&owned[..], aligned_bpr)
    };

    record_image_upload(bytes.len());
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: origin.0,
                y: origin.1,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(dst_bpr),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

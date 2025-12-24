use fret_core::ImageId;
use slotmap::SlotMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageColorSpace {
    Srgb,
    Linear,
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

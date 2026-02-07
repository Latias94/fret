use fret_core::RenderTargetId;
use fret_render_core::RenderTargetColorSpace;
use slotmap::SlotMap;

pub struct RenderTargetDescriptor {
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub format: wgpu::TextureFormat,
    pub color_space: RenderTargetColorSpace,
}

struct RenderTargetEntry {
    view: wgpu::TextureView,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    color_space: RenderTargetColorSpace,
}

#[derive(Default)]
pub struct RenderTargetRegistry {
    targets: SlotMap<RenderTargetId, RenderTargetEntry>,
}

impl RenderTargetRegistry {
    pub fn register(&mut self, desc: RenderTargetDescriptor) -> RenderTargetId {
        debug_assert_eq!(
            desc.format.is_srgb(),
            desc.color_space == RenderTargetColorSpace::Srgb,
            "RenderTargetDescriptor.format must match RenderTargetColorSpace"
        );
        self.targets.insert(RenderTargetEntry {
            view: desc.view,
            size: desc.size,
            format: desc.format,
            color_space: desc.color_space,
        })
    }

    pub fn update(&mut self, id: RenderTargetId, desc: RenderTargetDescriptor) -> bool {
        debug_assert_eq!(
            desc.format.is_srgb(),
            desc.color_space == RenderTargetColorSpace::Srgb,
            "RenderTargetDescriptor.format must match RenderTargetColorSpace"
        );
        let Some(entry) = self.targets.get_mut(id) else {
            return false;
        };
        entry.view = desc.view;
        entry.size = desc.size;
        entry.format = desc.format;
        entry.color_space = desc.color_space;
        true
    }

    pub fn unregister(&mut self, id: RenderTargetId) -> bool {
        self.targets.remove(id).is_some()
    }

    pub(crate) fn get(&self, id: RenderTargetId) -> Option<&wgpu::TextureView> {
        self.targets.get(id).map(|t| &t.view)
    }
}

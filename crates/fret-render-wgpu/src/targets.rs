use fret_core::RenderTargetId;
use fret_render_core::{RenderTargetColorSpace, RenderTargetMetadata};
use slotmap::SlotMap;

pub struct RenderTargetDescriptor {
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub format: wgpu::TextureFormat,
    pub color_space: RenderTargetColorSpace,
    pub metadata: RenderTargetMetadata,
}

struct RenderTargetEntry {
    view: wgpu::TextureView,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    color_space: RenderTargetColorSpace,
    metadata: RenderTargetMetadata,
}

#[derive(Default)]
pub struct RenderTargetRegistry {
    targets: SlotMap<RenderTargetId, RenderTargetEntry>,
}

fn estimate_texture_bytes(size: (u32, u32), format: wgpu::TextureFormat) -> u64 {
    let (w, h) = size;
    let (bw, bh) = format.block_dimensions();
    let block_bytes = format.block_copy_size(None).unwrap_or(16) as u64;
    let blocks_w = u64::from(w).div_ceil(u64::from(bw.max(1)));
    let blocks_h = u64::from(h).div_ceil(u64::from(bh.max(1)));
    blocks_w
        .saturating_mul(blocks_h)
        .saturating_mul(block_bytes)
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
            metadata: desc.metadata,
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
        entry.metadata = desc.metadata;
        true
    }

    pub fn unregister(&mut self, id: RenderTargetId) -> bool {
        self.targets.remove(id).is_some()
    }

    pub(crate) fn get(&self, id: RenderTargetId) -> Option<&wgpu::TextureView> {
        self.targets.get(id).map(|t| &t.view)
    }

    pub(crate) fn metadata(&self, id: RenderTargetId) -> Option<RenderTargetMetadata> {
        self.targets.get(id).map(|t| t.metadata)
    }

    pub(crate) fn diagnostics_estimated_bytes(&self) -> (u64, u64, u64) {
        let mut total: u64 = 0;
        let mut max: u64 = 0;
        for entry in self.targets.values() {
            let bytes = estimate_texture_bytes(entry.size, entry.format);
            total = total.saturating_add(bytes);
            max = max.max(bytes);
        }
        (self.targets.len() as u64, total, max)
    }
}

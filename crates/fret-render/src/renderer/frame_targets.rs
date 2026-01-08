use super::intermediate_pool::{IntermediatePool, PooledTexture};
use super::render_plan::PlanTarget;

#[derive(Default)]
pub(in crate::renderer) struct FrameTargets {
    intermediate0: Option<FrameTarget>,
    intermediate1: Option<FrameTarget>,
    intermediate2: Option<FrameTarget>,
}

struct FrameTarget {
    size: (u32, u32),
    texture: PooledTexture,
    view: wgpu::TextureView,
}

impl FrameTargets {
    pub(super) fn ensure_target(
        &mut self,
        pool: &mut IntermediatePool,
        device: &wgpu::Device,
        target: PlanTarget,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> wgpu::TextureView {
        let size = (size.0.max(1), size.1.max(1));
        let slot = match target {
            PlanTarget::Intermediate0 => &mut self.intermediate0,
            PlanTarget::Intermediate1 => &mut self.intermediate1,
            PlanTarget::Intermediate2 => &mut self.intermediate2,
            PlanTarget::Output => unreachable!("Output is not an intermediate target"),
        };

        if slot.as_ref().is_some_and(|existing| existing.size == size) {
            return slot.as_ref().unwrap().view.clone();
        }

        if let Some(existing) = slot.take() {
            pool.release(existing.texture);
        }

        let texture =
            pool.acquire_texture(device, "fret intermediate target", size, format, usage, 1);
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        slot.replace(FrameTarget {
            size,
            texture,
            view,
        });
        slot.as_ref().unwrap().view.clone()
    }

    pub(super) fn release_all(&mut self, pool: &mut IntermediatePool) {
        if let Some(t) = self.intermediate0.take() {
            pool.release(t.texture);
        }
        if let Some(t) = self.intermediate1.take() {
            pool.release(t.texture);
        }
        if let Some(t) = self.intermediate2.take() {
            pool.release(t.texture);
        }
    }
}

pub(in crate::renderer) fn downsampled_size(size: (u32, u32), scale: u32) -> (u32, u32) {
    let scale = scale.max(1);
    (size.0.max(1).div_ceil(scale), size.1.max(1).div_ceil(scale))
}

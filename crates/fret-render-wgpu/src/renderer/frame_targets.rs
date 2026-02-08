use super::intermediate_pool::{IntermediatePool, PooledTexture};
use super::render_plan::PlanTarget;

#[derive(Default)]
pub(in crate::renderer) struct FrameTargets {
    intermediate0: Option<FrameTarget>,
    intermediate1: Option<FrameTarget>,
    intermediate2: Option<FrameTarget>,
    mask0: Option<FrameTarget>,
    mask1: Option<FrameTarget>,
    mask2: Option<FrameTarget>,
    bytes_in_use: u64,
    peak_bytes_in_use: u64,
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
            PlanTarget::Mask0 => &mut self.mask0,
            PlanTarget::Mask1 => &mut self.mask1,
            PlanTarget::Mask2 => &mut self.mask2,
            PlanTarget::Output => unreachable!("Output is not an intermediate target"),
        };

        if slot.as_ref().is_some_and(|existing| existing.size == size) {
            return slot.as_ref().unwrap().view.clone();
        }

        if let Some(existing) = slot.take() {
            self.bytes_in_use = self.bytes_in_use.saturating_sub(existing.texture.bytes);
            pool.release(existing.texture);
        }

        let texture =
            pool.acquire_texture(device, "fret intermediate target", size, format, usage, 1);
        self.bytes_in_use = self.bytes_in_use.saturating_add(texture.bytes);
        self.peak_bytes_in_use = self.peak_bytes_in_use.max(self.bytes_in_use);
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

    pub(super) fn require_target(&self, target: PlanTarget, size: (u32, u32)) -> wgpu::TextureView {
        let size = (size.0.max(1), size.1.max(1));
        let slot = match target {
            PlanTarget::Intermediate0 => self.intermediate0.as_ref(),
            PlanTarget::Intermediate1 => self.intermediate1.as_ref(),
            PlanTarget::Intermediate2 => self.intermediate2.as_ref(),
            PlanTarget::Mask0 => self.mask0.as_ref(),
            PlanTarget::Mask1 => self.mask1.as_ref(),
            PlanTarget::Mask2 => self.mask2.as_ref(),
            PlanTarget::Output => unreachable!("Output is not an intermediate target"),
        };
        let existing = slot.expect("required intermediate target must exist");
        assert_eq!(
            existing.size, size,
            "required intermediate target size mismatch"
        );
        existing.view.clone()
    }

    pub(super) fn release_target(&mut self, pool: &mut IntermediatePool, target: PlanTarget) {
        let slot = match target {
            PlanTarget::Intermediate0 => &mut self.intermediate0,
            PlanTarget::Intermediate1 => &mut self.intermediate1,
            PlanTarget::Intermediate2 => &mut self.intermediate2,
            PlanTarget::Mask0 => &mut self.mask0,
            PlanTarget::Mask1 => &mut self.mask1,
            PlanTarget::Mask2 => &mut self.mask2,
            PlanTarget::Output => unreachable!("Output is not an intermediate target"),
        };
        if let Some(t) = slot.take() {
            self.bytes_in_use = self.bytes_in_use.saturating_sub(t.texture.bytes);
            pool.release(t.texture);
        }
    }

    pub(super) fn release_all(&mut self, pool: &mut IntermediatePool, budget_bytes: u64) {
        self.release_target(pool, PlanTarget::Intermediate0);
        self.release_target(pool, PlanTarget::Intermediate1);
        self.release_target(pool, PlanTarget::Intermediate2);
        self.release_target(pool, PlanTarget::Mask0);
        self.release_target(pool, PlanTarget::Mask1);
        self.release_target(pool, PlanTarget::Mask2);
        pool.enforce_budget(budget_bytes);
    }

    pub(super) fn in_use_bytes(&self) -> u64 {
        self.bytes_in_use
    }

    pub(super) fn peak_in_use_bytes(&self) -> u64 {
        self.peak_bytes_in_use
    }
}

pub(in crate::renderer) fn downsampled_size(size: (u32, u32), scale: u32) -> (u32, u32) {
    let scale = scale.max(1);
    (size.0.max(1).div_ceil(scale), size.1.max(1).div_ceil(scale))
}

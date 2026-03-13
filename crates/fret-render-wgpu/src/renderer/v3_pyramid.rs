use super::*;

#[derive(Debug)]
pub(super) struct CustomEffectV3PyramidScratch {
    pub(super) size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) levels: u32,
    pub(super) full_view: wgpu::TextureView,
    pub(super) mip_views: Vec<wgpu::TextureView>,
}

impl CustomEffectV3PyramidScratch {
    pub(super) fn mip_size(&self, level: u32) -> (u32, u32) {
        let mut w = self.size.0.max(1);
        let mut h = self.size.1.max(1);
        let mut l = 0u32;
        while l < level {
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            l += 1;
        }
        (w, h)
    }

    pub(super) fn estimated_bytes(&self) -> u64 {
        estimate_mipped_texture_bytes(self.size, self.format, 1, self.levels)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CustomEffectV3PyramidCache {
    src_raw: PlanTarget,
    src_size: (u32, u32),
    format: wgpu::TextureFormat,
    levels: u32,
    src_raw_epoch: u32,
}

#[derive(Debug, Default)]
pub(super) struct CustomEffectV3PyramidState {
    scratch: Option<CustomEffectV3PyramidScratch>,
    cache: Option<CustomEffectV3PyramidCache>,
    plan_target_write_epochs: [u32; 8],
}

impl CustomEffectV3PyramidState {
    pub(super) fn reset_frame_local_caches(&mut self) {
        self.cache = None;
        self.plan_target_write_epochs = [0; 8];
    }

    pub(super) fn bump_plan_target_write_epoch(&mut self, target: PlanTarget) {
        let ix = plan_target_epoch_slot(target);
        self.plan_target_write_epochs[ix] = self.plan_target_write_epochs[ix].saturating_add(1);
        if self.cache.is_some_and(|cache| cache.src_raw == target) {
            self.cache = None;
        }
    }

    pub(super) fn can_reuse(
        &self,
        src_raw: PlanTarget,
        src_size: (u32, u32),
        format: wgpu::TextureFormat,
        levels: u32,
    ) -> bool {
        let Some(cache) = self.cache else {
            return false;
        };
        if cache.src_raw != src_raw
            || cache.src_size != src_size
            || cache.format != format
            || cache.levels != levels
        {
            return false;
        }
        let ix = plan_target_epoch_slot(src_raw);
        cache.src_raw_epoch == self.plan_target_write_epochs[ix]
    }

    pub(super) fn set_cache(
        &mut self,
        src_raw: PlanTarget,
        src_size: (u32, u32),
        format: wgpu::TextureFormat,
        levels: u32,
    ) {
        let ix = plan_target_epoch_slot(src_raw);
        self.cache = Some(CustomEffectV3PyramidCache {
            src_raw,
            src_size,
            format,
            levels,
            src_raw_epoch: self.plan_target_write_epochs[ix],
        });
    }

    pub(super) fn scratch_bytes_estimate(&self) -> u64 {
        self.scratch
            .as_ref()
            .map_or(0, CustomEffectV3PyramidScratch::estimated_bytes)
    }

    pub(super) fn ensure_scratch(
        &mut self,
        device: &wgpu::Device,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        levels: u32,
    ) -> &CustomEffectV3PyramidScratch {
        let size = (size.0.max(1), size.1.max(1));
        let levels = levels.max(1);
        let recreate = self
            .scratch
            .as_ref()
            .is_none_or(|s| s.size != size || s.format != format || s.levels != levels);
        if recreate {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fret custom-effect v3 pyramid scratch"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            let full_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut mip_views = Vec::with_capacity(levels as usize);
            for level in 0..levels {
                mip_views.push(texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("fret custom-effect v3 pyramid mip view"),
                    format: Some(format),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    base_array_layer: 0,
                    array_layer_count: Some(1),
                    usage: None,
                }));
            }

            self.scratch = Some(CustomEffectV3PyramidScratch {
                size,
                format,
                levels,
                full_view,
                mip_views,
            });
        }

        self.scratch.as_ref().expect("pyramid scratch must exist")
    }
}

fn plan_target_epoch_slot(target: PlanTarget) -> usize {
    match target {
        PlanTarget::Output => 0,
        PlanTarget::Intermediate0 => 1,
        PlanTarget::Intermediate1 => 2,
        PlanTarget::Intermediate2 => 3,
        PlanTarget::Intermediate3 => 4,
        PlanTarget::Mask0 => 5,
        PlanTarget::Mask1 => 6,
        PlanTarget::Mask2 => 7,
    }
}

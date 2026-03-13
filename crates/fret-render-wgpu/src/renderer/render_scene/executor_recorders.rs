use super::super::*;
use super::executor::RenderSceneExecutor;
use super::helpers::{ensure_color_dst_view_owned, require_color_src_view, require_mask_view};

pub(super) struct CustomEffectV3PyramidScratchSnapshot {
    pub(super) full_view: wgpu::TextureView,
    pub(super) mip_views: Vec<wgpu::TextureView>,
    pub(super) mip_sizes: Vec<(u32, u32)>,
}

impl<'a> RenderSceneExecutor<'a> {
    pub(super) fn require_color_src_view(
        &self,
        src: PlanTarget,
        src_size: (u32, u32),
        pass_name: &'static str,
    ) -> Option<wgpu::TextureView> {
        require_color_src_view(&*self.frame_targets, src, src_size, pass_name)
    }

    pub(super) fn require_mask_view(
        &self,
        mask_target: PlanTarget,
        mask_size: (u32, u32),
        pass_name: &'static str,
    ) -> Option<wgpu::TextureView> {
        require_mask_view(&*self.frame_targets, mask_target, mask_size, pass_name)
    }

    pub(super) fn ensure_color_dst_view_owned(
        &mut self,
        dst: PlanTarget,
        dst_size: (u32, u32),
        pass_name: &'static str,
    ) -> Option<wgpu::TextureView> {
        ensure_color_dst_view_owned(
            &mut *self.frame_targets,
            &mut self.renderer.intermediate_state.pool,
            self.device,
            dst,
            dst_size,
            self.format,
            self.usage,
            pass_name,
        )
    }

    pub(super) fn custom_effect_v3_pyramid_reuse(
        &mut self,
        src_raw: PlanTarget,
        src_size: (u32, u32),
        levels: u32,
    ) -> bool {
        let reuse = self.renderer.custom_effect_v3_pyramid.can_reuse(
            src_raw,
            src_size,
            self.format,
            levels,
        );
        if self.perf_enabled {
            if reuse {
                self.frame_perf.custom_effect_v3_pyramid_cache_hits = self
                    .frame_perf
                    .custom_effect_v3_pyramid_cache_hits
                    .saturating_add(1);
            } else {
                self.frame_perf.custom_effect_v3_pyramid_cache_misses = self
                    .frame_perf
                    .custom_effect_v3_pyramid_cache_misses
                    .saturating_add(1);
            }
        }
        reuse
    }

    pub(super) fn custom_effect_v3_pyramid_scratch_snapshot(
        &mut self,
        size: (u32, u32),
        levels: u32,
    ) -> CustomEffectV3PyramidScratchSnapshot {
        let scratch = self.renderer.custom_effect_v3_pyramid.ensure_scratch(
            self.device,
            size,
            self.format,
            levels,
        );
        let mip_views = scratch.mip_views.iter().cloned().collect::<Vec<_>>();
        let mip_sizes = (0..scratch.levels)
            .map(|level| scratch.mip_size(level))
            .collect::<Vec<_>>();
        CustomEffectV3PyramidScratchSnapshot {
            full_view: scratch.full_view.clone(),
            mip_views,
            mip_sizes,
        }
    }

    pub(super) fn set_custom_effect_v3_pyramid_cache(
        &mut self,
        src_raw: PlanTarget,
        src_size: (u32, u32),
        levels: u32,
    ) {
        self.renderer
            .custom_effect_v3_pyramid
            .set_cache(src_raw, src_size, self.format, levels);
    }
}

use super::super::*;
use super::executor::RenderSceneExecutor;
use super::helpers::{
    ensure_color_dst_view_owned, ensure_mask_dst_view, require_color_src_view, require_mask_view,
};

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

    pub(super) fn ensure_clip_path_mask_target_and_try_cache_copy(
        &mut self,
        cache_key: u64,
        dst: PlanTarget,
        target_size: (u32, u32),
    ) -> Option<(wgpu::TextureView, bool)> {
        if !matches!(
            dst,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ) {
            debug_assert!(false, "PathClipMask dst must be Mask[0-2]");
            return None;
        }

        let (pass_target_texture, pass_target_view) =
            self.frame_targets.ensure_target_with_texture(
                &mut self.renderer.intermediate_state.pool,
                self.device,
                dst,
                target_size,
                wgpu::TextureFormat::R8Unorm,
                self.usage,
            );

        let cache_hit = self.renderer.clip_path_mask_cache.try_copy_into(
            &mut *self.encoder,
            cache_key,
            target_size,
            pass_target_texture,
            self.frame_index,
        );
        if cache_hit && self.perf_enabled {
            self.frame_perf.clip_path_mask_cache_hits =
                self.frame_perf.clip_path_mask_cache_hits.saturating_add(1);
        }

        Some((pass_target_view, cache_hit))
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

    pub(super) fn ensure_mask_dst_view(
        &mut self,
        dst: PlanTarget,
        dst_size: (u32, u32),
        pass_name: &'static str,
    ) -> Option<wgpu::TextureView> {
        ensure_mask_dst_view(
            &mut *self.frame_targets,
            &mut self.renderer.intermediate_state.pool,
            self.device,
            dst,
            dst_size,
            self.usage,
            pass_name,
        )
    }

    pub(super) fn store_clip_path_mask_cache_from_target(
        &mut self,
        cache_key: u64,
        dst: PlanTarget,
        target_size: (u32, u32),
    ) {
        if !matches!(
            dst,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ) {
            debug_assert!(false, "PathClipMask dst must be Mask[0-2]");
            return;
        }

        let (pass_target_texture, _) = self.frame_targets.ensure_target_with_texture(
            &mut self.renderer.intermediate_state.pool,
            self.device,
            dst,
            target_size,
            wgpu::TextureFormat::R8Unorm,
            self.usage,
        );
        self.renderer.clip_path_mask_cache.store_from(
            &mut self.renderer.intermediate_state.pool,
            self.device,
            &mut *self.encoder,
            cache_key,
            target_size,
            pass_target_texture,
            self.frame_index,
        );
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

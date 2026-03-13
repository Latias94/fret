use super::super::*;
use super::executor::RenderSceneExecutor;
use super::helpers::{
    ensure_color_dst_view_owned, ensure_mask_dst_view, require_color_src_view, require_mask_view,
};

fn downsample_scissor_2x(scissor: ScissorRect, dst_size: (u32, u32)) -> Option<ScissorRect> {
    if scissor.w == 0 || scissor.h == 0 {
        return None;
    }
    let x0 = scissor.x / 2;
    let y0 = scissor.y / 2;
    let x1 = scissor.x.saturating_add(scissor.w).saturating_add(1) / 2;
    let y1 = scissor.y.saturating_add(scissor.h).saturating_add(1) / 2;
    let x1 = x1.min(dst_size.0);
    let y1 = y1.min(dst_size.1);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(ScissorRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}

pub(super) struct CustomEffectV3PyramidScratchSnapshot {
    pub(super) full_view: wgpu::TextureView,
    pub(super) mip_views: Vec<wgpu::TextureView>,
    pub(super) mip_sizes: Vec<(u32, u32)>,
}

pub(super) struct CustomEffectV3PreparedSourceViews {
    pub(super) src_view: wgpu::TextureView,
    pub(super) src_raw_view: wgpu::TextureView,
    pub(super) src_pyramid_view: wgpu::TextureView,
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

    pub(super) fn build_custom_effect_v3_pyramid(
        &mut self,
        src_raw: PlanTarget,
        src_size: (u32, u32),
        levels: u32,
        build_scissor: Option<LocalScissorRect>,
        src_raw_view: &wgpu::TextureView,
        mip_views: &[wgpu::TextureView],
        mip_sizes: &[(u32, u32)],
    ) {
        self.renderer.ensure_blit_pipeline(self.device, self.format);
        self.renderer
            .ensure_mip_downsample_box_pipeline(self.device, self.format);

        let mut pyramid_scissor = build_scissor.map(|s| s.0);

        let blit_layout = self.renderer.blit_bind_group_layout_ref();
        let blit_bind_group = create_texture_bind_group(
            self.device,
            "fret custom-effect v3 pyramid blit bind group",
            blit_layout,
            src_raw_view,
        );
        run_fullscreen_triangle_pass(
            &mut *self.encoder,
            "fret custom-effect v3 pyramid blit",
            self.renderer.blit_pipeline_ref(),
            &mip_views[0],
            mip_sizes[0],
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            &blit_bind_group,
            &[],
            pyramid_scissor.map(LocalScissorRect),
            self.perf_enabled.then_some(&mut *self.frame_perf),
        );

        let downsample_layout = self.renderer.mip_downsample_box_bind_group_layout_ref();
        for level in 1..(mip_views.len() as u32) {
            let src_level = (level - 1) as usize;
            let bind_group = create_texture_bind_group(
                self.device,
                "fret mip downsample box bind group",
                downsample_layout,
                &mip_views[src_level],
            );
            pyramid_scissor =
                pyramid_scissor.and_then(|s| downsample_scissor_2x(s, mip_sizes[level as usize]));
            run_fullscreen_triangle_pass(
                &mut *self.encoder,
                "fret mip downsample box pass",
                self.renderer.mip_downsample_box_pipeline_ref(),
                &mip_views[level as usize],
                mip_sizes[level as usize],
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                &bind_group,
                &[],
                pyramid_scissor.map(LocalScissorRect),
                self.perf_enabled.then_some(&mut *self.frame_perf),
            );
        }

        self.set_custom_effect_v3_pyramid_cache(src_raw, src_size, levels);
    }

    pub(super) fn prepare_custom_effect_v3_source_views(
        &mut self,
        pass: &CustomEffectV3Pass,
    ) -> Option<CustomEffectV3PreparedSourceViews> {
        let common = pass.common;
        let src_view =
            self.require_color_src_view(common.src, common.src_size, "CustomEffectV3")?;
        let src_raw_view =
            self.require_color_src_view(pass.src_raw, common.src_size, "CustomEffectV3")?;

        let pyramid_override_view = if pass.pyramid_wanted && pass.pyramid_levels >= 2 {
            let reuse = self.custom_effect_v3_pyramid_reuse(
                pass.src_raw,
                common.src_size,
                pass.pyramid_levels,
            );
            let scratch = self
                .custom_effect_v3_pyramid_scratch_snapshot(common.src_size, pass.pyramid_levels);
            let mip_views = scratch.mip_views;
            let mip_sizes = scratch.mip_sizes;
            let full_view = scratch.full_view;

            if !reuse {
                self.build_custom_effect_v3_pyramid(
                    pass.src_raw,
                    common.src_size,
                    pass.pyramid_levels,
                    pass.pyramid_build_scissor,
                    &src_raw_view,
                    &mip_views,
                    &mip_sizes,
                );
            }

            Some(full_view)
        } else {
            None
        };

        let src_pyramid_view = if let Some(view) = pyramid_override_view {
            view
        } else {
            self.require_color_src_view(pass.src_pyramid, common.src_size, "CustomEffectV3")?
        };

        Some(CustomEffectV3PreparedSourceViews {
            src_view,
            src_raw_view,
            src_pyramid_view,
        })
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

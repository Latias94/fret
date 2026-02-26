use super::super::*;

impl Renderer {
    pub(super) fn ensure_effect_pipelines_for_plan(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        path_samples: u32,
        plan: &RenderPlan,
    ) {
        let needs_scale = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::ScaleNearest(_)));
        let needs_blur = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::Blur(_)));
        let needs_clip_mask = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::ClipMask(_)));
        let needs_blit = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::FullscreenBlit(_)));
        let needs_blit_srgb_encode = plan.passes.iter().any(|p| match p {
            RenderPlanPass::FullscreenBlit(pass) => pass.encode_output_srgb,
            _ => false,
        });
        let needs_composite = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::CompositePremul(_)));
        let needs_color_adjust = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::ColorAdjust(_)));
        let needs_backdrop_warp = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::BackdropWarp(_)));
        let needs_color_matrix = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::ColorMatrix(_)));
        let needs_alpha_threshold = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::AlphaThreshold(_)));
        let needs_dither = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::Dither(_)));
        let needs_noise = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::Noise(_)));
        let needs_drop_shadow = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::DropShadow(_)));

        let mut custom_effects: std::collections::HashSet<fret_core::EffectId> =
            std::collections::HashSet::new();
        let mut custom_effects_v2: std::collections::HashSet<fret_core::EffectId> =
            std::collections::HashSet::new();
        for pass in &plan.passes {
            match pass {
                RenderPlanPass::CustomEffect(pass) => {
                    custom_effects.insert(pass.effect);
                }
                RenderPlanPass::CustomEffectV2(pass) => {
                    custom_effects_v2.insert(pass.effect);
                }
                _ => {}
            }
        }
        let needs_custom_effect = !custom_effects.is_empty();
        let needs_custom_effect_v2 = !custom_effects_v2.is_empty();

        if needs_blit || needs_blur {
            self.ensure_blit_pipeline(device, format);
        }
        if needs_blit_srgb_encode {
            self.ensure_blit_srgb_encode_pipeline(device, format);
        }
        if needs_scale {
            self.ensure_scale_nearest_pipelines(device, format);
        }
        if needs_blur {
            self.ensure_blur_pipelines(device, format);
        }
        if needs_clip_mask {
            self.ensure_clip_mask_pipeline(device);
        }
        if needs_composite && path_samples <= 1 {
            self.ensure_composite_pipeline(device, format);
        }
        if needs_backdrop_warp {
            self.ensure_backdrop_warp_pipeline(device, format);
        }
        if needs_color_adjust {
            self.ensure_color_adjust_pipeline(device, format);
        }
        if needs_color_matrix {
            self.ensure_color_matrix_pipeline(device, format);
        }
        if needs_alpha_threshold {
            self.ensure_alpha_threshold_pipeline(device, format);
        }
        if needs_dither {
            self.ensure_dither_pipeline(device, format);
        }
        if needs_noise {
            self.ensure_noise_pipeline(device, format);
        }
        if needs_drop_shadow {
            self.ensure_drop_shadow_pipeline(device, format);
        }
        if needs_custom_effect {
            for id in custom_effects {
                self.ensure_custom_effect_pipelines(device, format, id);
            }
        }
        if needs_custom_effect_v2 {
            for id in custom_effects_v2 {
                self.ensure_custom_effect_v2_pipelines(device, format, id);
            }
        }

        if self.intermediate_perf_enabled {
            self.intermediate_perf.last_frame_release_targets = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::ReleaseTarget(_)))
                .count() as u64;
        }

        let scale_pass_count = plan
            .passes
            .iter()
            .filter(|p| matches!(p, RenderPlanPass::ScaleNearest(_)))
            .count();
        self.effect_params
            .ensure_scale_param_capacity(device, scale_pass_count);
        self.ensure_render_space_capacity(device, plan.passes.len());
    }
}

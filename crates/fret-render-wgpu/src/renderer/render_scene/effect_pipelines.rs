use super::super::*;

impl Renderer {
    pub(super) fn ensure_effect_pipelines_for_plan(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        path_samples: u32,
        plan: &RenderPlan,
    ) {
        let mut needs_scale = false;
        let mut needs_blur = false;
        let mut needs_clip_mask = false;
        let mut needs_blit = false;
        let mut needs_blit_srgb_encode = false;
        let mut needs_composite = false;
        let mut needs_color_adjust = false;
        let mut needs_backdrop_warp = false;
        let mut needs_color_matrix = false;
        let mut needs_alpha_threshold = false;
        let mut needs_dither = false;
        let mut needs_noise = false;
        let mut needs_drop_shadow = false;
        let mut release_targets = 0usize;
        let mut scale_pass_count = 0usize;
        let mut custom_effects: std::collections::HashSet<fret_core::EffectId> =
            std::collections::HashSet::new();
        let mut custom_effects_v2: std::collections::HashSet<fret_core::EffectId> =
            std::collections::HashSet::new();
        let mut custom_effects_v3: std::collections::HashSet<fret_core::EffectId> =
            std::collections::HashSet::new();
        for pass in &plan.passes {
            match pass {
                RenderPlanPass::ScaleNearest(_) => {
                    needs_scale = true;
                    scale_pass_count += 1;
                }
                RenderPlanPass::Blur(_) => {
                    needs_blur = true;
                }
                RenderPlanPass::ClipMask(_) => {
                    needs_clip_mask = true;
                }
                RenderPlanPass::FullscreenBlit(pass) => {
                    needs_blit = true;
                    needs_blit_srgb_encode |= pass.encode_output_srgb;
                }
                RenderPlanPass::CompositePremul(_) => {
                    needs_composite = true;
                }
                RenderPlanPass::ColorAdjust(_) => {
                    needs_color_adjust = true;
                }
                RenderPlanPass::BackdropWarp(_) => {
                    needs_backdrop_warp = true;
                }
                RenderPlanPass::ColorMatrix(_) => {
                    needs_color_matrix = true;
                }
                RenderPlanPass::AlphaThreshold(_) => {
                    needs_alpha_threshold = true;
                }
                RenderPlanPass::Dither(_) => {
                    needs_dither = true;
                }
                RenderPlanPass::Noise(_) => {
                    needs_noise = true;
                }
                RenderPlanPass::DropShadow(_) => {
                    needs_drop_shadow = true;
                }
                RenderPlanPass::CustomEffect(pass) => {
                    custom_effects.insert(pass.common.effect);
                }
                RenderPlanPass::CustomEffectV2(pass) => {
                    custom_effects_v2.insert(pass.common.effect);
                }
                RenderPlanPass::CustomEffectV3(pass) => {
                    custom_effects_v3.insert(pass.common.effect);
                }
                RenderPlanPass::ReleaseTarget(_) => {
                    release_targets += 1;
                }
                _ => {}
            }
        }
        let needs_custom_effect = !custom_effects.is_empty();
        let needs_custom_effect_v2 = !custom_effects_v2.is_empty();
        let needs_custom_effect_v3 = !custom_effects_v3.is_empty();

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
        if needs_custom_effect_v3 {
            for id in custom_effects_v3 {
                self.ensure_custom_effect_v3_pipelines(device, format, id);
            }
        }

        self.intermediate_state
            .record_release_targets(release_targets as u64);

        self.effect_params
            .ensure_scale_param_capacity(device, scale_pass_count);
        self.ensure_render_space_capacity(device, plan.passes.len());
    }
}

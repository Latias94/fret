use super::super::frame_targets::downsampled_size;
use super::super::*;

impl Renderer {
    pub(super) fn pick_debug_postprocess(
        &mut self,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
    ) -> DebugPostprocess {
        if self.debug_pixelate_scale > 0 {
            return DebugPostprocess::Pixelate {
                scale: self.debug_pixelate_scale,
            };
        }

        if self.debug_blur_radius > 0 {
            let radius = self.debug_blur_radius.max(1);
            let budget = self.intermediate_budget_bytes;
            let full = estimate_texture_bytes(viewport_size, format, 1);
            let half = estimate_texture_bytes(downsampled_size(viewport_size, 2), format, 1);
            let quarter = estimate_texture_bytes(downsampled_size(viewport_size, 4), format, 1);

            let required_half = full.saturating_add(half.saturating_mul(2));
            let required_quarter = full.saturating_add(quarter.saturating_mul(2));

            let default_downsample_scale = if radius > 4 { 4 } else { 2 };
            let mut downsample_scale = default_downsample_scale;
            if downsample_scale == 2 && required_half > budget {
                downsample_scale = 4;
                if self.intermediate_perf_enabled {
                    self.intermediate_perf.blur_degraded_to_quarter = self
                        .intermediate_perf
                        .blur_degraded_to_quarter
                        .saturating_add(1);
                }
            }

            if downsample_scale == 4 && required_quarter > budget {
                if self.intermediate_perf_enabled {
                    self.intermediate_perf.blur_disabled_due_to_budget = self
                        .intermediate_perf
                        .blur_disabled_due_to_budget
                        .saturating_add(1);
                }
                return DebugPostprocess::None;
            }

            return DebugPostprocess::Blur {
                radius,
                downsample_scale,
                scissor: self.debug_blur_scissor,
            };
        }

        if self.debug_offscreen_blit_enabled {
            return DebugPostprocess::OffscreenBlit {
                src: PlanTarget::Intermediate0,
            };
        }

        DebugPostprocess::None
    }
}

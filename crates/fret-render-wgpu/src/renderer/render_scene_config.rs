use super::*;
use std::sync::OnceLock;

pub(super) struct RenderSceneConfigState {
    render_plan_strict_output_clear: bool,
    path_msaa_samples: u32,
    debug_offscreen_blit_enabled: bool,
    debug_pixelate_scale: u32,
    debug_blur_radius: u32,
    debug_blur_scissor: Option<ScissorRect>,
}

impl RenderSceneConfigState {
    pub(super) fn new() -> Self {
        Self {
            render_plan_strict_output_clear: std::env::var("FRET_RENDER_PLAN_STRICT_OUTPUT_CLEAR")
                .is_ok_and(|v| v != "0"),
            path_msaa_samples: 4,
            debug_offscreen_blit_enabled: false,
            debug_pixelate_scale: 0,
            debug_blur_radius: 0,
            debug_blur_scissor: None,
        }
    }

    pub(super) fn render_plan_strict_output_clear(&self) -> bool {
        self.render_plan_strict_output_clear
    }

    pub(super) fn path_msaa_samples(&self) -> u32 {
        self.path_msaa_samples
    }

    pub(super) fn set_path_msaa_samples(&mut self, samples: u32) -> bool {
        let samples = Self::path_msaa_samples_override_from_env().unwrap_or(samples);
        let next_samples = normalize_path_msaa_samples(samples);
        let changed = self.path_msaa_samples != next_samples;
        self.path_msaa_samples = next_samples;
        changed
    }

    pub(super) fn effective_path_msaa_samples(
        &self,
        adapter: &wgpu::Adapter,
        format: wgpu::TextureFormat,
    ) -> u32 {
        let requested = self.path_msaa_samples.max(1);
        if requested == 1 {
            return 1;
        }

        if adapter.get_info().backend == wgpu::Backend::Vulkan
            && std::env::var_os("FRET_DISABLE_VULKAN_PATH_MSAA").is_some()
        {
            static WARNED: OnceLock<()> = OnceLock::new();
            if WARNED.set(()).is_ok() {
                let info = adapter.get_info();
                tracing::warn!(
                    backend = ?info.backend,
                    vendor = info.vendor,
                    device = info.device,
                    driver = info.driver,
                    driver_info = info.driver_info,
                    "Vulkan path MSAA is disabled via FRET_DISABLE_VULKAN_PATH_MSAA=1."
                );
            }
            return 1;
        }

        let features = adapter.get_texture_format_features(format);
        if !features
            .allowed_usages
            .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return 1;
        }

        if !features
            .allowed_usages
            .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || !features
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
        {
            return 1;
        }

        for candidate in [16u32, 8, 4, 2] {
            if candidate <= requested && features.flags.sample_count_supported(candidate) {
                return candidate;
            }
        }
        1
    }

    pub(super) fn debug_offscreen_blit_enabled(&self) -> bool {
        self.debug_offscreen_blit_enabled
    }

    pub(super) fn set_debug_offscreen_blit_enabled(&mut self, enabled: bool) {
        self.debug_offscreen_blit_enabled = enabled;
    }

    pub(super) fn debug_pixelate_scale(&self) -> u32 {
        self.debug_pixelate_scale
    }

    pub(super) fn set_debug_pixelate_scale(&mut self, scale: u32) {
        self.debug_pixelate_scale = scale.min(128);
    }

    pub(super) fn debug_blur_radius(&self) -> u32 {
        self.debug_blur_radius
    }

    pub(super) fn set_debug_blur_radius(&mut self, radius: u32) {
        self.debug_blur_radius = radius.min(64);
    }

    pub(super) fn debug_blur_scissor(&self) -> Option<ScissorRect> {
        self.debug_blur_scissor
    }

    pub(super) fn debug_blur_scissor_tuple(&self) -> Option<(u32, u32, u32, u32)> {
        self.debug_blur_scissor.map(|s| (s.x, s.y, s.w, s.h))
    }

    pub(super) fn set_debug_blur_scissor(&mut self, scissor: Option<(u32, u32, u32, u32)>) {
        self.debug_blur_scissor = scissor.and_then(|(x, y, w, h)| {
            if w == 0 || h == 0 {
                return None;
            }
            Some(ScissorRect { x, y, w, h })
        });
    }

    fn path_msaa_samples_override_from_env() -> Option<u32> {
        static OVERRIDE: OnceLock<Option<u32>> = OnceLock::new();
        *OVERRIDE.get_or_init(|| {
            let Ok(raw) = std::env::var("FRET_RENDER_WGPU_PATH_MSAA_SAMPLES") else {
                return None;
            };

            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }

            match trimmed.parse::<u32>() {
                Ok(samples) => {
                    tracing::warn!(
                        path_msaa_samples = samples,
                        "Renderer path MSAA samples overridden via FRET_RENDER_WGPU_PATH_MSAA_SAMPLES."
                    );
                    Some(samples)
                }
                Err(_) => {
                    tracing::warn!(
                        raw = trimmed,
                        "Invalid FRET_RENDER_WGPU_PATH_MSAA_SAMPLES; ignoring override."
                    );
                    None
                }
            }
        })
    }
}

impl Default for RenderSceneConfigState {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_path_msaa_samples(samples: u32) -> u32 {
    let samples = samples.max(1).min(16);
    if samples == 1 {
        1
    } else {
        let pow2_floor = 1u32 << (31 - samples.leading_zeros());
        pow2_floor.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_path_msaa_samples_to_supported_shapes() {
        assert_eq!(normalize_path_msaa_samples(0), 1);
        assert_eq!(normalize_path_msaa_samples(1), 1);
        assert_eq!(normalize_path_msaa_samples(3), 2);
        assert_eq!(normalize_path_msaa_samples(5), 4);
        assert_eq!(normalize_path_msaa_samples(17), 16);
    }

    #[test]
    fn clamps_debug_knobs_and_rejects_zero_sized_scissors() {
        let mut state = RenderSceneConfigState::new();
        state.set_debug_pixelate_scale(999);
        state.set_debug_blur_radius(999);
        state.set_debug_blur_scissor(Some((10, 20, 0, 30)));
        assert_eq!(state.debug_pixelate_scale(), 128);
        assert_eq!(state.debug_blur_radius(), 64);
        assert_eq!(state.debug_blur_scissor(), None);

        state.set_debug_blur_scissor(Some((10, 20, 30, 40)));
        assert_eq!(
            state.debug_blur_scissor(),
            Some(ScissorRect {
                x: 10,
                y: 20,
                w: 30,
                h: 40,
            })
        );
    }
}

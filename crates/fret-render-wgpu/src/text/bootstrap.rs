use super::TextSystem;
use super::atlas_epoch::TextAtlasEpochState;
use super::atlas_runtime_state::TextAtlasRuntimeState;
use super::blob_state::TextBlobState;
use super::face_cache::TextFaceCacheState;
use super::font_runtime_state::TextFontRuntimeState;
use super::frame_perf::TextFramePerfState;
use super::layout_cache_state::TextLayoutCacheState;
use super::pin_state::TextPinState;
use super::quality::{TextQualitySettings, TextQualityState};
use fret_render_text::TextFallbackPolicyV1;

fn bootstrap_text_resources(
    device: &wgpu::Device,
) -> (
    fret_render_text::ParleyShaper,
    TextFallbackPolicyV1,
    TextAtlasRuntimeState,
) {
    let parley_shaper = fret_render_text::ParleyShaper::new();
    let fallback_policy = TextFallbackPolicyV1::new(&parley_shaper);

    (
        parley_shaper,
        fallback_policy,
        TextAtlasRuntimeState::bootstrap(device),
    )
}

pub(super) fn build_text_system(device: &wgpu::Device) -> TextSystem {
    let (parley_shaper, fallback_policy, atlas_runtime) = bootstrap_text_resources(device);

    let mut out = TextSystem {
        parley_shaper,
        parley_scale: parley::swash::scale::ScaleContext::new(),
        font_runtime: TextFontRuntimeState::new(fallback_policy),
        quality: TextQualityState::new(TextQualitySettings::default()),

        blob_state: TextBlobState::new(),
        layout_cache: TextLayoutCacheState::new(),

        atlas_runtime,

        pin_state: TextPinState::with_ring_len(3),
        face_cache: TextFaceCacheState::default(),

        frame_perf: TextFramePerfState::default(),

        atlas_epoch: TextAtlasEpochState::new(1),
    };

    out.finish_initial_font_bootstrap();
    out
}

impl TextSystem {
    pub fn new(device: &wgpu::Device) -> Self {
        build_text_system(device)
    }
}

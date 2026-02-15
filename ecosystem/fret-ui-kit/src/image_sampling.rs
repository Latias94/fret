use fret_core::scene::ImageSamplingHint;
use fret_ui::element::ImageProps;

/// Ecosystem-only helpers for setting image sampling hints.
///
/// This keeps `fret-ui` as a mechanism layer while allowing policy layers to opt in to
/// nearest-neighbor sampling for pixel-art / canvas-style content.
pub trait ImageSamplingExt {
    fn sampling_hint(self, hint: ImageSamplingHint) -> Self;
    fn nearest(self) -> Self;
    fn linear(self) -> Self;
}

impl ImageSamplingExt for ImageProps {
    fn sampling_hint(self, hint: ImageSamplingHint) -> Self {
        self.sampling(hint)
    }

    fn nearest(self) -> Self {
        self.sampling(ImageSamplingHint::Nearest)
    }

    fn linear(self) -> Self {
        self.sampling(ImageSamplingHint::Linear)
    }
}

use super::TextSystem;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextQualitySettings {
    /// Gamma parameter for shader-side alpha correction.
    ///
    /// This is used to derive the `gamma_ratios` uniform (GPUI-aligned). Values are clamped to
    /// `[1.0, 2.2]`.
    pub gamma: f32,
    /// Enhanced contrast factor for grayscale (mask) glyph sampling.
    pub grayscale_enhanced_contrast: f32,
    /// Enhanced contrast factor for subpixel (RGB coverage) glyph sampling.
    pub subpixel_enhanced_contrast: f32,
}

impl Default for TextQualitySettings {
    fn default() -> Self {
        // Windows-first defaults, aligned with the Zed/GPUI baseline (see ADR 0105/0142).
        Self {
            gamma: 1.8,
            grayscale_enhanced_contrast: 1.0,
            subpixel_enhanced_contrast: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TextQualityState {
    pub(super) gamma: f32,
    pub(super) gamma_ratios: [f32; 4],
    pub(super) grayscale_enhanced_contrast: f32,
    pub(super) subpixel_enhanced_contrast: f32,
    pub(super) key: u64,
}

impl TextQualityState {
    pub(super) fn new(settings: TextQualitySettings) -> Self {
        let gamma = settings.gamma.clamp(1.0, 2.2);
        let grayscale_enhanced_contrast = settings.grayscale_enhanced_contrast.max(0.0);
        let subpixel_enhanced_contrast = settings.subpixel_enhanced_contrast.max(0.0);

        let gamma_ratios = gamma_correction_ratios(gamma);

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        gamma.to_bits().hash(&mut hasher);
        grayscale_enhanced_contrast.to_bits().hash(&mut hasher);
        subpixel_enhanced_contrast.to_bits().hash(&mut hasher);
        let key = hasher.finish();

        Self {
            gamma,
            gamma_ratios,
            grayscale_enhanced_contrast,
            subpixel_enhanced_contrast,
            key,
        }
    }
}

// Adapted from the Microsoft Terminal alpha correction tables (via Zed/GPUI).
// See ADR 0029 / ADR 0107 for the rationale and references.
fn gamma_correction_ratios(gamma: f32) -> [f32; 4] {
    const GAMMA_INCORRECT_TARGET_RATIOS: [[f32; 4]; 13] = [
        [0.0000 / 4.0, 0.0000 / 4.0, 0.0000 / 4.0, 0.0000 / 4.0], // gamma = 1.0
        [0.0166 / 4.0, -0.0807 / 4.0, 0.2227 / 4.0, -0.0751 / 4.0], // gamma = 1.1
        [0.0350 / 4.0, -0.1760 / 4.0, 0.4325 / 4.0, -0.1370 / 4.0], // gamma = 1.2
        [0.0543 / 4.0, -0.2821 / 4.0, 0.6302 / 4.0, -0.1876 / 4.0], // gamma = 1.3
        [0.0739 / 4.0, -0.3963 / 4.0, 0.8167 / 4.0, -0.2287 / 4.0], // gamma = 1.4
        [0.0933 / 4.0, -0.5161 / 4.0, 0.9926 / 4.0, -0.2616 / 4.0], // gamma = 1.5
        [0.1121 / 4.0, -0.6395 / 4.0, 1.1588 / 4.0, -0.2877 / 4.0], // gamma = 1.6
        [0.1300 / 4.0, -0.7649 / 4.0, 1.3159 / 4.0, -0.3080 / 4.0], // gamma = 1.7
        [0.1469 / 4.0, -0.8911 / 4.0, 1.4644 / 4.0, -0.3234 / 4.0], // gamma = 1.8
        [0.1627 / 4.0, -1.0170 / 4.0, 1.6051 / 4.0, -0.3347 / 4.0], // gamma = 1.9
        [0.1773 / 4.0, -1.1420 / 4.0, 1.7385 / 4.0, -0.3426 / 4.0], // gamma = 2.0
        [0.1908 / 4.0, -1.2652 / 4.0, 1.8650 / 4.0, -0.3476 / 4.0], // gamma = 2.1
        [0.2031 / 4.0, -1.3864 / 4.0, 1.9851 / 4.0, -0.3501 / 4.0], // gamma = 2.2
    ];

    const NORM13: f32 = ((0x10000 as f64) / (255.0 * 255.0) * 4.0) as f32;
    const NORM24: f32 = ((0x100 as f64) / (255.0) * 4.0) as f32;

    let index = ((gamma * 10.0).round() as usize).clamp(10, 22) - 10;
    let ratios = GAMMA_INCORRECT_TARGET_RATIOS[index];
    [
        ratios[0] * NORM13,
        ratios[1] * NORM24,
        ratios[2] * NORM13,
        ratios[3] * NORM24,
    ]
}

impl TextSystem {
    pub fn text_quality_key(&self) -> u64 {
        self.quality.key
    }

    pub fn text_quality_uniforms(&self) -> ([f32; 4], f32, f32) {
        (
            self.quality.gamma_ratios,
            self.quality.grayscale_enhanced_contrast,
            self.quality.subpixel_enhanced_contrast,
        )
    }

    pub fn set_text_quality_settings(&mut self, settings: TextQualitySettings) -> bool {
        let next = TextQualityState::new(settings);
        if next == self.quality {
            return false;
        }
        self.quality = next;
        true
    }
}

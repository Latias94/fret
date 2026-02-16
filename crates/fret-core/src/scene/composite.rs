use super::{EffectQuality, Rect};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BlendMode {
    /// Premultiplied alpha-over (the baseline compositing contract; ADR 0040).
    #[default]
    Over,
    /// Additive blending (used for glow/beam).
    Add,
    /// Multiply blending (used for grain/darken overlays).
    Multiply,
    /// Screen blending (used for light overlays).
    Screen,
    /// Channel-wise darken (`min(dst, src)` for color channels).
    ///
    /// This mode is bounded and portable because it maps to a fixed-function blend operation
    /// (no destination sampling).
    Darken,
    /// Channel-wise lighten (`max(dst, src)` for color channels).
    ///
    /// This mode is bounded and portable because it maps to a fixed-function blend operation
    /// (no destination sampling).
    Lighten,
    /// Subtract (`dst - src`, clamped to `[0, 1]`) for color channels.
    ///
    /// This mode is bounded and portable because it maps to a fixed-function blend operation
    /// (no destination sampling).
    Subtract,
}

impl BlendMode {
    pub const COUNT: usize = 7;

    pub const ALL: [BlendMode; Self::COUNT] = [
        BlendMode::Over,
        BlendMode::Add,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::Subtract,
    ];

    pub const fn pipeline_index(self) -> usize {
        match self {
            BlendMode::Over => 0,
            BlendMode::Add => 1,
            BlendMode::Multiply => 2,
            BlendMode::Screen => 3,
            BlendMode::Darken => 4,
            BlendMode::Lighten => 5,
            BlendMode::Subtract => 6,
        }
    }
}

/// Descriptor for an isolated compositing group (ADR 0247).
///
/// The group is rendered into an offscreen intermediate and then composited back onto the parent
/// target using the requested `mode`. `bounds` is a computation bound (not an implicit clip).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompositeGroupDesc {
    /// Computation bounds (not an implicit clip), see ADR 0247.
    pub bounds: Rect,
    pub mode: BlendMode,
    pub quality: EffectQuality,
    /// Group-level opacity multiplier applied when the group is composited back to its parent.
    ///
    /// This enables CSS-like isolated opacity semantics (e.g. `saveLayerAlpha`): overlapping
    /// children inside the group blend with each other normally, then the final group result is
    /// multiplied by this opacity.
    ///
    /// Default: `1.0`.
    pub opacity: f32,
}

impl CompositeGroupDesc {
    pub const fn new(bounds: Rect, mode: BlendMode, quality: EffectQuality) -> Self {
        Self {
            bounds,
            mode,
            quality,
            opacity: 1.0,
        }
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }
}

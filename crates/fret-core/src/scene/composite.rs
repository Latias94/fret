use super::{EffectQuality, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Premultiplied alpha-over (the baseline compositing contract; ADR 0040).
    Over,
    /// Additive blending (used for glow/beam).
    Add,
    /// Multiply blending (used for grain/darken overlays).
    Multiply,
    /// Screen blending (used for light overlays).
    Screen,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Over
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
}

impl CompositeGroupDesc {
    pub const fn new(bounds: Rect, mode: BlendMode, quality: EffectQuality) -> Self {
        Self {
            bounds,
            mode,
            quality,
        }
    }
}

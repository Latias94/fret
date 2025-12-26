use crate::ids::SvgId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SvgFit {
    /// Uniformly scale to fully fit inside the target rect (no cropping).
    #[default]
    Contain,
    /// Uniformly scale based on target width (height may overflow and should be clipped).
    Width,
    /// Non-uniformly scale to match the target rect (may distort).
    Stretch,
}

/// SVG asset registration service.
///
/// This keeps `SceneOp` cheap (it stores `SvgId`, not raw bytes) while allowing renderers to
/// rasterize/cache SVGs at paint time using their GPU context.
pub trait SvgService {
    /// Register SVG bytes and return a stable `SvgId`.
    ///
    /// Implementations may deduplicate repeated registrations.
    fn register_svg(&mut self, bytes: &[u8]) -> SvgId;

    /// Unregister a previously registered SVG.
    fn unregister_svg(&mut self, svg: SvgId) -> bool;
}

use crate::ids::SvgId;

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

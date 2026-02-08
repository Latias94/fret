//! Compatibility facade for Fret's default renderer backend.
//!
//! Today the default renderer is wgpu-based (`fret-render-wgpu`). This crate exists to keep the
//! historical `fret-render` crate name stable while we split backend implementations into
//! explicit crates.

pub use fret_render_wgpu::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_reexports_default_renderer_surface() {
        let _ = std::mem::size_of::<Renderer>();
        let _ = std::mem::size_of::<SurfaceState>();
        let _ = std::mem::size_of::<RenderSceneParams>();
        let _ = std::mem::size_of::<RenderError>();
        let _ = std::mem::size_of::<RenderTargetRegistry>();
        let _ = std::mem::size_of::<TextFontFamilyConfig>();
        let _ = std::mem::size_of::<RenderTargetColorSpace>();
    }
}

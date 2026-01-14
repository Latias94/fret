//! Bundled font assets for `fret`.
//!
//! Notes:
//! - Web/WASM cannot access system fonts, so applications must provide font bytes.
//! - This crate provides a small default font to bootstrap demos and simple apps.

/// Returns the default font bytes (TTF/OTF/TTC) that can be fed to `Effect::TextAddFonts`.
pub fn default_fonts() -> &'static [&'static [u8]] {
    &[
        #[cfg(feature = "bootstrap")]
        include_bytes!("../assets/Inter-roman.ttf"),
        #[cfg(feature = "bootstrap")]
        include_bytes!("../assets/Inter-italic.ttf"),
        #[cfg(feature = "bootstrap")]
        include_bytes!("../assets/JetBrainsMono-roman.ttf"),
        #[cfg(feature = "bootstrap")]
        include_bytes!("../assets/JetBrainsMono-italic.ttf"),
        include_bytes!("../assets/FiraMono-subset.ttf"),
    ]
}

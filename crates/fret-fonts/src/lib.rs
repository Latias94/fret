//! Bundled font assets for `fret`.
//!
//! Notes:
//! - Web/WASM cannot access system fonts, so applications must provide font bytes.
//! - This crate provides a small default font to bootstrap demos and simple apps.

#[cfg(feature = "emoji")]
static EMOJI_FONTS: &[&[u8]] = &[include_bytes!("../assets/NotoColorEmoji.ttf")];

#[cfg(feature = "bootstrap-full")]
static BOOTSTRAP_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/Inter-roman.ttf"),
    include_bytes!("../assets/Inter-italic.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
static BOOTSTRAP_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/Inter-roman-subset.ttf"),
    include_bytes!("../assets/Inter-italic-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(feature = "emoji")]
static DEFAULT_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/NotoColorEmoji.ttf"),
    #[cfg(feature = "bootstrap-full")]
    include_bytes!("../assets/Inter-roman.ttf"),
    #[cfg(feature = "bootstrap-full")]
    include_bytes!("../assets/Inter-italic.ttf"),
    #[cfg(feature = "bootstrap-full")]
    include_bytes!("../assets/JetBrainsMono-roman.ttf"),
    #[cfg(feature = "bootstrap-full")]
    include_bytes!("../assets/JetBrainsMono-italic.ttf"),
    #[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
    include_bytes!("../assets/Inter-roman-subset.ttf"),
    #[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
    include_bytes!("../assets/Inter-italic-subset.ttf"),
    #[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf"),
    #[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(not(feature = "emoji"))]
static DEFAULT_FONTS: &[&[u8]] = &[
    #[cfg(feature = "bootstrap-full")]
    include_bytes!("../assets/Inter-roman.ttf"),
    #[cfg(feature = "bootstrap-full")]
    include_bytes!("../assets/Inter-italic.ttf"),
    #[cfg(feature = "bootstrap-full")]
    include_bytes!("../assets/JetBrainsMono-roman.ttf"),
    #[cfg(feature = "bootstrap-full")]
    include_bytes!("../assets/JetBrainsMono-italic.ttf"),
    #[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
    include_bytes!("../assets/Inter-roman-subset.ttf"),
    #[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
    include_bytes!("../assets/Inter-italic-subset.ttf"),
    #[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf"),
    #[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

/// Returns the default font bytes (TTF/OTF/TTC) that can be fed to `Effect::TextAddFonts`.
pub fn default_fonts() -> &'static [&'static [u8]] {
    DEFAULT_FONTS
}

pub fn bootstrap_fonts() -> &'static [&'static [u8]] {
    BOOTSTRAP_FONTS
}

#[cfg(feature = "emoji")]
pub fn emoji_fonts() -> &'static [&'static [u8]] {
    EMOJI_FONTS
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_fonts_are_non_empty() {
        for font in super::default_fonts() {
            assert!(font.len() > 1024);
        }
    }

    #[cfg(feature = "emoji")]
    #[test]
    fn bundles_add_up_when_emoji_is_enabled() {
        assert_eq!(
            super::default_fonts().len(),
            super::bootstrap_fonts().len() + super::emoji_fonts().len()
        );
        assert_eq!(super::emoji_fonts().len(), 1);
    }

    #[test]
    fn default_fonts_total_size_is_reasonable() {
        let total: usize = super::default_fonts().iter().map(|b| b.len()).sum();

        #[cfg(not(feature = "emoji"))]
        assert!(
            total < 2_000_000,
            "default fonts too large (expected subset bootstrap): {total}"
        );

        #[cfg(feature = "emoji")]
        assert!(
            total < 15_000_000,
            "default fonts too large (emoji bundle enabled): {total}"
        );
    }
}

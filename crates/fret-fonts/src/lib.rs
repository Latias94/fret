//! Bundled font assets for `fret`.
//!
//! Notes:
//! - Web/WASM cannot access system fonts, so applications must provide font bytes.
//! - This crate provides a small default font to bootstrap demos and simple apps.

#[cfg(feature = "emoji")]
static EMOJI_FONTS: &[&[u8]] = &[include_bytes!("../assets/NotoColorEmoji.ttf")];

#[cfg(feature = "cjk-lite")]
static CJK_LITE_FONTS: &[&[u8]] = &[include_bytes!(
    "../assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf"
)];

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
    #[cfg(feature = "cjk-lite")]
    include_bytes!("../assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf"),
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
    #[cfg(feature = "cjk-lite")]
    include_bytes!("../assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf"),
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

#[cfg(feature = "cjk-lite")]
pub fn cjk_lite_fonts() -> &'static [&'static [u8]] {
    CJK_LITE_FONTS
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_fonts_are_non_empty() {
        for font in super::default_fonts() {
            assert!(font.len() > 1024);
        }
    }

    #[cfg(all(feature = "emoji", not(feature = "cjk-lite")))]
    #[test]
    fn bundles_add_up_when_emoji_is_enabled() {
        assert_eq!(
            super::default_fonts().len(),
            super::bootstrap_fonts().len() + super::emoji_fonts().len()
        );
        assert_eq!(super::emoji_fonts().len(), 1);
    }

    #[cfg(all(feature = "cjk-lite", not(feature = "emoji")))]
    #[test]
    fn bundles_add_up_when_cjk_lite_is_enabled() {
        assert_eq!(
            super::default_fonts().len(),
            super::bootstrap_fonts().len() + super::cjk_lite_fonts().len()
        );
        assert_eq!(super::cjk_lite_fonts().len(), 1);
    }

    #[cfg(all(feature = "emoji", feature = "cjk-lite"))]
    #[test]
    fn bundles_add_up_when_emoji_and_cjk_lite_are_enabled() {
        assert_eq!(
            super::default_fonts().len(),
            super::bootstrap_fonts().len()
                + super::emoji_fonts().len()
                + super::cjk_lite_fonts().len()
        );
        assert_eq!(super::emoji_fonts().len(), 1);
        assert_eq!(super::cjk_lite_fonts().len(), 1);
    }

    #[test]
    fn default_fonts_total_size_is_reasonable() {
        let total: usize = super::default_fonts().iter().map(|b| b.len()).sum();

        #[cfg(all(not(feature = "emoji"), not(feature = "cjk-lite")))]
        assert!(
            total < 2_000_000,
            "default fonts too large (expected subset bootstrap): {total}"
        );

        #[cfg(all(feature = "emoji", not(feature = "cjk-lite")))]
        assert!(
            total < 15_000_000,
            "default fonts too large (emoji bundle enabled): {total}"
        );

        #[cfg(all(not(feature = "emoji"), feature = "cjk-lite"))]
        assert!(
            total < 12_000_000,
            "default fonts too large (cjk-lite bundle enabled): {total}"
        );

        #[cfg(all(feature = "emoji", feature = "cjk-lite"))]
        assert!(
            total < 25_000_000,
            "default fonts too large (emoji + cjk-lite bundles enabled): {total}"
        );
    }
}

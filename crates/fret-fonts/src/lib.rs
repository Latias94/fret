//! Bundled font assets for `fret`.
//!
//! Notes:
//! - Web/WASM cannot access system fonts, so applications must provide font bytes.
//! - This crate provides a small default font to bootstrap demos and simple apps.

#[cfg(feature = "bootstrap-full")]
const BOOTSTRAP_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/Inter-roman.ttf"),
    include_bytes!("../assets/Inter-italic.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const BOOTSTRAP_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/Inter-roman-subset.ttf"),
    include_bytes!("../assets/Inter-italic-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(all(not(feature = "bootstrap-subset"), not(feature = "bootstrap-full")))]
const BOOTSTRAP_FONTS: &[&[u8]] = &[include_bytes!("../assets/FiraMono-subset.ttf")];

#[cfg(all(feature = "emoji", feature = "bootstrap-full"))]
const DEFAULT_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/NotoColorEmoji.ttf"),
    include_bytes!("../assets/Inter-roman.ttf"),
    include_bytes!("../assets/Inter-italic.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(all(not(feature = "emoji"), feature = "bootstrap-full"))]
const DEFAULT_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/Inter-roman.ttf"),
    include_bytes!("../assets/Inter-italic.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(all(
    feature = "emoji",
    feature = "bootstrap-subset",
    not(feature = "bootstrap-full")
))]
const DEFAULT_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/NotoColorEmoji.ttf"),
    include_bytes!("../assets/Inter-roman-subset.ttf"),
    include_bytes!("../assets/Inter-italic-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(all(
    not(feature = "emoji"),
    feature = "bootstrap-subset",
    not(feature = "bootstrap-full")
))]
const DEFAULT_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/Inter-roman-subset.ttf"),
    include_bytes!("../assets/Inter-italic-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(all(
    feature = "emoji",
    not(feature = "bootstrap-subset"),
    not(feature = "bootstrap-full")
))]
const DEFAULT_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/NotoColorEmoji.ttf"),
    include_bytes!("../assets/FiraMono-subset.ttf"),
];

#[cfg(all(
    not(feature = "emoji"),
    not(feature = "bootstrap-subset"),
    not(feature = "bootstrap-full")
))]
const DEFAULT_FONTS: &[&[u8]] = &[include_bytes!("../assets/FiraMono-subset.ttf")];

/// Returns the default font bytes (TTF/OTF/TTC) that can be fed to `Effect::TextAddFonts`.
pub fn default_fonts() -> &'static [&'static [u8]] {
    DEFAULT_FONTS
}

pub fn bootstrap_fonts() -> &'static [&'static [u8]] {
    BOOTSTRAP_FONTS
}

#[cfg(feature = "emoji")]
pub fn emoji_fonts() -> &'static [&'static [u8]] {
    &[include_bytes!("../assets/NotoColorEmoji.ttf")]
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_fonts_are_non_empty() {
        for font in super::default_fonts() {
            assert!(font.len() > 1024);
        }
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

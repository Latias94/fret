//! Bundled font assets for `fret`.
//!
//! Notes:
//! - Web/WASM cannot access system fonts, so applications must provide font bytes.
//! - This crate provides a small default font to bootstrap demos and simple apps.

const FIRA_MONO_SUBSET: &[u8] = include_bytes!("../assets/FiraMono-subset.ttf");

#[cfg(feature = "emoji")]
const NOTO_COLOR_EMOJI: &[u8] = include_bytes!("../assets/NotoColorEmoji.ttf");

#[cfg(feature = "cjk-lite")]
const NOTO_SANS_CJK_SC_LITE_SUBSET: &[u8] =
    include_bytes!("../assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf");

#[cfg(feature = "bootstrap-full")]
const BOOTSTRAP_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/Inter-roman.ttf"),
    include_bytes!("../assets/Inter-italic.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic.ttf"),
    FIRA_MONO_SUBSET,
];

#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const BOOTSTRAP_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/Inter-roman-subset.ttf"),
    include_bytes!("../assets/Inter-italic-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf"),
    FIRA_MONO_SUBSET,
];

#[cfg(all(not(feature = "bootstrap-subset"), not(feature = "bootstrap-full")))]
const BOOTSTRAP_FONTS: &[&[u8]] = &[FIRA_MONO_SUBSET];

#[cfg(all(feature = "emoji", feature = "bootstrap-full"))]
const DEFAULT_FONTS: &[&[u8]] = &[
    NOTO_COLOR_EMOJI,
    #[cfg(feature = "cjk-lite")]
    NOTO_SANS_CJK_SC_LITE_SUBSET,
    include_bytes!("../assets/Inter-roman.ttf"),
    include_bytes!("../assets/Inter-italic.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic.ttf"),
    FIRA_MONO_SUBSET,
];

#[cfg(all(not(feature = "emoji"), feature = "bootstrap-full"))]
const DEFAULT_FONTS: &[&[u8]] = &[
    #[cfg(feature = "cjk-lite")]
    NOTO_SANS_CJK_SC_LITE_SUBSET,
    include_bytes!("../assets/Inter-roman.ttf"),
    include_bytes!("../assets/Inter-italic.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic.ttf"),
    FIRA_MONO_SUBSET,
];

#[cfg(all(
    feature = "emoji",
    feature = "bootstrap-subset",
    not(feature = "bootstrap-full")
))]
const DEFAULT_FONTS: &[&[u8]] = &[
    NOTO_COLOR_EMOJI,
    #[cfg(feature = "cjk-lite")]
    NOTO_SANS_CJK_SC_LITE_SUBSET,
    include_bytes!("../assets/Inter-roman-subset.ttf"),
    include_bytes!("../assets/Inter-italic-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf"),
    FIRA_MONO_SUBSET,
];

#[cfg(all(
    not(feature = "emoji"),
    feature = "bootstrap-subset",
    not(feature = "bootstrap-full")
))]
const DEFAULT_FONTS: &[&[u8]] = &[
    #[cfg(feature = "cjk-lite")]
    NOTO_SANS_CJK_SC_LITE_SUBSET,
    include_bytes!("../assets/Inter-roman-subset.ttf"),
    include_bytes!("../assets/Inter-italic-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf"),
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf"),
    FIRA_MONO_SUBSET,
];

#[cfg(all(
    feature = "emoji",
    not(feature = "bootstrap-subset"),
    not(feature = "bootstrap-full")
))]
const DEFAULT_FONTS: &[&[u8]] = &[
    NOTO_COLOR_EMOJI,
    #[cfg(feature = "cjk-lite")]
    NOTO_SANS_CJK_SC_LITE_SUBSET,
    FIRA_MONO_SUBSET,
];

#[cfg(all(
    not(feature = "emoji"),
    not(feature = "bootstrap-subset"),
    not(feature = "bootstrap-full")
))]
const DEFAULT_FONTS: &[&[u8]] = &[
    #[cfg(feature = "cjk-lite")]
    NOTO_SANS_CJK_SC_LITE_SUBSET,
    FIRA_MONO_SUBSET,
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
    &[NOTO_COLOR_EMOJI]
}

#[cfg(feature = "cjk-lite")]
pub fn cjk_lite_fonts() -> &'static [&'static [u8]] {
    &[NOTO_SANS_CJK_SC_LITE_SUBSET]
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

        #[cfg(all(
            not(feature = "emoji"),
            not(feature = "cjk-lite"),
            feature = "bootstrap-full"
        ))]
        assert!(
            total < 8_000_000,
            "default fonts too large (expected full bootstrap): {total}"
        );

        #[cfg(all(
            not(feature = "emoji"),
            not(feature = "cjk-lite"),
            not(feature = "bootstrap-full")
        ))]
        assert!(
            total < 2_000_000,
            "default fonts too large (expected subset bootstrap): {total}"
        );

        #[cfg(all(
            feature = "emoji",
            not(feature = "cjk-lite"),
            feature = "bootstrap-full"
        ))]
        assert!(
            total < 20_000_000,
            "default fonts too large (emoji + full bootstrap enabled): {total}"
        );

        #[cfg(all(
            feature = "emoji",
            not(feature = "cjk-lite"),
            not(feature = "bootstrap-full")
        ))]
        assert!(
            total < 15_000_000,
            "default fonts too large (emoji bundle enabled): {total}"
        );

        #[cfg(all(
            not(feature = "emoji"),
            feature = "cjk-lite",
            feature = "bootstrap-full"
        ))]
        assert!(
            total < 15_000_000,
            "default fonts too large (cjk-lite + full bootstrap enabled): {total}"
        );

        #[cfg(all(
            not(feature = "emoji"),
            feature = "cjk-lite",
            not(feature = "bootstrap-full")
        ))]
        assert!(
            total < 12_000_000,
            "default fonts too large (cjk-lite bundle enabled): {total}"
        );

        #[cfg(all(feature = "emoji", feature = "cjk-lite", feature = "bootstrap-full"))]
        assert!(
            total < 30_000_000,
            "default fonts too large (emoji + cjk-lite + full bootstrap enabled): {total}"
        );

        #[cfg(all(
            feature = "emoji",
            feature = "cjk-lite",
            not(feature = "bootstrap-full")
        ))]
        assert!(
            total < 25_000_000,
            "default fonts too large (emoji + cjk-lite bundles enabled): {total}"
        );
    }
}

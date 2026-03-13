//! Bundled font assets for `fret`.
//!
//! Notes:
//! - Web/WASM cannot access system fonts, so applications must provide font bytes.
//! - This crate exposes both the bytes and a small manifest describing which bundled profile
//!   guarantees which family/role surface.

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BundledFontRole {
    UiSans,
    UiSerif,
    UiMonospace,
    EmojiFallback,
    CjkFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BundledGenericFamily {
    Sans,
    Serif,
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundledFontFaceSpec {
    pub family: &'static str,
    pub roles: &'static [BundledFontRole],
    pub bytes: &'static [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundledFontProfile {
    pub name: &'static str,
    pub faces: &'static [BundledFontFaceSpec],
    pub provided_roles: &'static [BundledFontRole],
    pub expected_family_names: &'static [&'static str],
    pub guaranteed_generic_families: &'static [BundledGenericFamily],
    pub ui_sans_families: &'static [&'static str],
    pub ui_serif_families: &'static [&'static str],
    pub ui_mono_families: &'static [&'static str],
    pub common_fallback_families: &'static [&'static str],
}

impl BundledFontProfile {
    pub fn font_bytes(&self) -> impl ExactSizeIterator<Item = &'static [u8]> + '_ {
        self.faces.iter().map(|face| face.bytes)
    }

    pub fn supports_role(&self, role: BundledFontRole) -> bool {
        self.faces.iter().any(|face| face.roles.contains(&role))
    }

    pub fn guarantees_generic_family(&self, family: BundledGenericFamily) -> bool {
        self.guaranteed_generic_families.contains(&family)
    }
}

const ROLE_UI_SANS: &[BundledFontRole] = &[BundledFontRole::UiSans];
const ROLE_UI_MONO: &[BundledFontRole] = &[BundledFontRole::UiMonospace];
#[cfg(feature = "emoji")]
const ROLE_EMOJI: &[BundledFontRole] = &[BundledFontRole::EmojiFallback];
#[cfg(feature = "cjk-lite")]
const ROLE_CJK: &[BundledFontRole] = &[BundledFontRole::CjkFallback];

const FIRA_MONO_SUBSET: &[u8] = include_bytes!("../assets/FiraMono-subset.ttf");

#[cfg(feature = "emoji")]
const NOTO_COLOR_EMOJI: &[u8] = include_bytes!("../assets/NotoColorEmoji.ttf");

#[cfg(feature = "cjk-lite")]
const NOTO_SANS_CJK_SC_LITE_SUBSET: &[u8] =
    include_bytes!("../assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf");

#[cfg(feature = "bootstrap-full")]
const INTER_ROMAN_BYTES: &[u8] = include_bytes!("../assets/Inter-roman.ttf");
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const INTER_ROMAN_BYTES: &[u8] = include_bytes!("../assets/Inter-roman-subset.ttf");

#[cfg(feature = "bootstrap-full")]
const INTER_ITALIC_BYTES: &[u8] = include_bytes!("../assets/Inter-italic.ttf");
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const INTER_ITALIC_BYTES: &[u8] = include_bytes!("../assets/Inter-italic-subset.ttf");

#[cfg(feature = "bootstrap-full")]
const JETBRAINS_MONO_ROMAN_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-roman.ttf");
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const JETBRAINS_MONO_ROMAN_BYTES: &[u8] =
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf");

#[cfg(feature = "bootstrap-full")]
const JETBRAINS_MONO_ITALIC_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-italic.ttf");
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const JETBRAINS_MONO_ITALIC_BYTES: &[u8] =
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf");

const FIRA_MONO_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Fira Mono",
    roles: ROLE_UI_MONO,
    bytes: FIRA_MONO_SUBSET,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const INTER_ROMAN_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Inter",
    roles: ROLE_UI_SANS,
    bytes: INTER_ROMAN_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const INTER_ITALIC_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Inter",
    roles: ROLE_UI_SANS,
    bytes: INTER_ITALIC_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const JETBRAINS_MONO_ROMAN_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "JetBrains Mono",
    roles: ROLE_UI_MONO,
    bytes: JETBRAINS_MONO_ROMAN_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const JETBRAINS_MONO_ITALIC_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "JetBrains Mono",
    roles: ROLE_UI_MONO,
    bytes: JETBRAINS_MONO_ITALIC_BYTES,
};

#[cfg(feature = "emoji")]
const NOTO_COLOR_EMOJI_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Noto Color Emoji",
    roles: ROLE_EMOJI,
    bytes: NOTO_COLOR_EMOJI,
};

#[cfg(feature = "cjk-lite")]
const NOTO_SANS_CJK_SC_LITE_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Noto Sans CJK SC",
    roles: ROLE_CJK,
    bytes: NOTO_SANS_CJK_SC_LITE_SUBSET,
};

const BOOTSTRAP_FACES: &[BundledFontFaceSpec] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    INTER_ROMAN_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    INTER_ITALIC_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    JETBRAINS_MONO_ROMAN_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    JETBRAINS_MONO_ITALIC_FACE,
    FIRA_MONO_FACE,
];

const DEFAULT_FACES: &[BundledFontFaceSpec] = &[
    #[cfg(feature = "emoji")]
    NOTO_COLOR_EMOJI_FACE,
    #[cfg(feature = "cjk-lite")]
    NOTO_SANS_CJK_SC_LITE_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    INTER_ROMAN_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    INTER_ITALIC_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    JETBRAINS_MONO_ROMAN_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    JETBRAINS_MONO_ITALIC_FACE,
    FIRA_MONO_FACE,
];

const BOOTSTRAP_EXPECTED_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Inter",
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "JetBrains Mono",
    "Fira Mono",
];

const DEFAULT_EXPECTED_FAMILIES: &[&str] = &[
    #[cfg(feature = "emoji")]
    "Noto Color Emoji",
    #[cfg(feature = "cjk-lite")]
    "Noto Sans CJK SC",
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Inter",
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "JetBrains Mono",
    "Fira Mono",
];

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const BOOTSTRAP_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] =
    &[BundledGenericFamily::Sans, BundledGenericFamily::Monospace];
#[cfg(not(any(feature = "bootstrap-subset", feature = "bootstrap-full")))]
const BOOTSTRAP_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] =
    &[BundledGenericFamily::Monospace];

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const DEFAULT_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] =
    &[BundledGenericFamily::Sans, BundledGenericFamily::Monospace];
#[cfg(not(any(feature = "bootstrap-subset", feature = "bootstrap-full")))]
const DEFAULT_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] =
    &[BundledGenericFamily::Monospace];

const BOOTSTRAP_PROVIDED_ROLES: &[BundledFontRole] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    BundledFontRole::UiSans,
    BundledFontRole::UiMonospace,
];

const DEFAULT_PROVIDED_ROLES: &[BundledFontRole] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    BundledFontRole::UiSans,
    BundledFontRole::UiMonospace,
    #[cfg(feature = "emoji")]
    BundledFontRole::EmojiFallback,
    #[cfg(feature = "cjk-lite")]
    BundledFontRole::CjkFallback,
];

const BOOTSTRAP_UI_SANS_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Inter",
];
const BOOTSTRAP_UI_SERIF_FAMILIES: &[&str] = &[];
const BOOTSTRAP_UI_MONO_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "JetBrains Mono",
    "Fira Mono",
];

const DEFAULT_UI_SANS_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Inter",
];
const DEFAULT_UI_SERIF_FAMILIES: &[&str] = &[];
const DEFAULT_UI_MONO_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "JetBrains Mono",
    "Fira Mono",
];
const DEFAULT_COMMON_FALLBACK_FAMILIES: &[&str] = &[
    #[cfg(feature = "cjk-lite")]
    "Noto Sans CJK SC",
    #[cfg(feature = "emoji")]
    "Noto Color Emoji",
];

#[cfg(feature = "bootstrap-full")]
const BOOTSTRAP_PROFILE_NAME: &str = "bootstrap-full";
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const BOOTSTRAP_PROFILE_NAME: &str = "bootstrap-subset";
#[cfg(all(not(feature = "bootstrap-subset"), not(feature = "bootstrap-full")))]
const BOOTSTRAP_PROFILE_NAME: &str = "mono-fallback-minimal";

#[cfg(all(feature = "bootstrap-full", feature = "emoji", feature = "cjk-lite"))]
const DEFAULT_PROFILE_NAME: &str = "default-full+emoji+cjk-lite";
#[cfg(all(
    feature = "bootstrap-full",
    feature = "emoji",
    not(feature = "cjk-lite")
))]
const DEFAULT_PROFILE_NAME: &str = "default-full+emoji";
#[cfg(all(
    feature = "bootstrap-full",
    not(feature = "emoji"),
    feature = "cjk-lite"
))]
const DEFAULT_PROFILE_NAME: &str = "default-full+cjk-lite";
#[cfg(all(
    feature = "bootstrap-full",
    not(feature = "emoji"),
    not(feature = "cjk-lite")
))]
const DEFAULT_PROFILE_NAME: &str = "default-full";
#[cfg(all(
    feature = "bootstrap-subset",
    not(feature = "bootstrap-full"),
    feature = "emoji",
    feature = "cjk-lite"
))]
const DEFAULT_PROFILE_NAME: &str = "default-subset+emoji+cjk-lite";
#[cfg(all(
    feature = "bootstrap-subset",
    not(feature = "bootstrap-full"),
    feature = "emoji",
    not(feature = "cjk-lite")
))]
const DEFAULT_PROFILE_NAME: &str = "default-subset+emoji";
#[cfg(all(
    feature = "bootstrap-subset",
    not(feature = "bootstrap-full"),
    not(feature = "emoji"),
    feature = "cjk-lite"
))]
const DEFAULT_PROFILE_NAME: &str = "default-subset+cjk-lite";
#[cfg(all(
    feature = "bootstrap-subset",
    not(feature = "bootstrap-full"),
    not(feature = "emoji"),
    not(feature = "cjk-lite")
))]
const DEFAULT_PROFILE_NAME: &str = "default-subset";
#[cfg(all(
    not(feature = "bootstrap-subset"),
    not(feature = "bootstrap-full"),
    feature = "emoji",
    feature = "cjk-lite"
))]
const DEFAULT_PROFILE_NAME: &str = "default-minimal+emoji+cjk-lite";
#[cfg(all(
    not(feature = "bootstrap-subset"),
    not(feature = "bootstrap-full"),
    feature = "emoji",
    not(feature = "cjk-lite")
))]
const DEFAULT_PROFILE_NAME: &str = "default-minimal+emoji";
#[cfg(all(
    not(feature = "bootstrap-subset"),
    not(feature = "bootstrap-full"),
    not(feature = "emoji"),
    feature = "cjk-lite"
))]
const DEFAULT_PROFILE_NAME: &str = "default-minimal+cjk-lite";
#[cfg(all(
    not(feature = "bootstrap-subset"),
    not(feature = "bootstrap-full"),
    not(feature = "emoji"),
    not(feature = "cjk-lite")
))]
const DEFAULT_PROFILE_NAME: &str = "default-minimal";

const BOOTSTRAP_PROFILE: BundledFontProfile = BundledFontProfile {
    name: BOOTSTRAP_PROFILE_NAME,
    faces: BOOTSTRAP_FACES,
    provided_roles: BOOTSTRAP_PROVIDED_ROLES,
    expected_family_names: BOOTSTRAP_EXPECTED_FAMILIES,
    guaranteed_generic_families: BOOTSTRAP_GUARANTEED_GENERIC_FAMILIES,
    ui_sans_families: BOOTSTRAP_UI_SANS_FAMILIES,
    ui_serif_families: BOOTSTRAP_UI_SERIF_FAMILIES,
    ui_mono_families: BOOTSTRAP_UI_MONO_FAMILIES,
    common_fallback_families: &[],
};

const DEFAULT_PROFILE: BundledFontProfile = BundledFontProfile {
    name: DEFAULT_PROFILE_NAME,
    faces: DEFAULT_FACES,
    provided_roles: DEFAULT_PROVIDED_ROLES,
    expected_family_names: DEFAULT_EXPECTED_FAMILIES,
    guaranteed_generic_families: DEFAULT_GUARANTEED_GENERIC_FAMILIES,
    ui_sans_families: DEFAULT_UI_SANS_FAMILIES,
    ui_serif_families: DEFAULT_UI_SERIF_FAMILIES,
    ui_mono_families: DEFAULT_UI_MONO_FAMILIES,
    common_fallback_families: DEFAULT_COMMON_FALLBACK_FAMILIES,
};

pub fn bootstrap_profile() -> &'static BundledFontProfile {
    &BOOTSTRAP_PROFILE
}

pub fn default_profile() -> &'static BundledFontProfile {
    &DEFAULT_PROFILE
}

fn collect_font_bytes(faces: &'static [BundledFontFaceSpec]) -> Box<[&'static [u8]]> {
    faces
        .iter()
        .map(|face| face.bytes)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn collect_font_bytes_for_role(
    faces: &'static [BundledFontFaceSpec],
    role: BundledFontRole,
) -> Box<[&'static [u8]]> {
    faces
        .iter()
        .filter(|face| face.roles.contains(&role))
        .map(|face| face.bytes)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

/// Returns the default font bytes (TTF/OTF/TTC) that can be fed to `Effect::TextAddFonts`.
pub fn default_fonts() -> &'static [&'static [u8]] {
    static BYTES: OnceLock<Box<[&'static [u8]]>> = OnceLock::new();
    BYTES.get_or_init(|| collect_font_bytes(default_profile().faces))
}

pub fn bootstrap_fonts() -> &'static [&'static [u8]] {
    static BYTES: OnceLock<Box<[&'static [u8]]>> = OnceLock::new();
    BYTES.get_or_init(|| collect_font_bytes(bootstrap_profile().faces))
}

#[cfg(feature = "emoji")]
pub fn emoji_fonts() -> &'static [&'static [u8]] {
    static BYTES: OnceLock<Box<[&'static [u8]]>> = OnceLock::new();
    BYTES.get_or_init(|| {
        collect_font_bytes_for_role(default_profile().faces, BundledFontRole::EmojiFallback)
    })
}

#[cfg(feature = "cjk-lite")]
pub fn cjk_lite_fonts() -> &'static [&'static [u8]] {
    static BYTES: OnceLock<Box<[&'static [u8]]>> = OnceLock::new();
    BYTES.get_or_init(|| {
        collect_font_bytes_for_role(default_profile().faces, BundledFontRole::CjkFallback)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_fonts_are_non_empty() {
        for font in super::default_fonts() {
            assert!(font.len() > 1024);
        }
    }

    #[test]
    fn default_profile_matches_default_fonts() {
        let profile = super::default_profile();
        let bytes_from_profile = profile.font_bytes().collect::<Vec<_>>();
        assert_eq!(super::default_fonts(), bytes_from_profile.as_slice());
        assert_eq!(profile.faces.len(), super::default_fonts().len());
        assert_eq!(profile.expected_family_names, DEFAULT_EXPECTED_FAMILIES);
        assert_eq!(profile.provided_roles, DEFAULT_PROVIDED_ROLES);
    }

    #[test]
    fn bootstrap_profile_declares_expected_generic_guarantees() {
        let profile = super::bootstrap_profile();
        assert!(profile.guarantees_generic_family(BundledGenericFamily::Monospace));
        #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
        assert!(profile.guarantees_generic_family(BundledGenericFamily::Sans));
        #[cfg(not(any(feature = "bootstrap-subset", feature = "bootstrap-full")))]
        assert!(!profile.guarantees_generic_family(BundledGenericFamily::Sans));
        assert!(!profile.guarantees_generic_family(BundledGenericFamily::Serif));
    }

    #[test]
    fn default_profile_explicitly_does_not_guarantee_serif() {
        let profile = super::default_profile();
        assert!(profile.ui_serif_families.is_empty());
        assert!(!profile.guarantees_generic_family(BundledGenericFamily::Serif));
    }

    #[cfg(feature = "emoji")]
    #[test]
    fn default_profile_declares_emoji_role_when_enabled() {
        let profile = super::default_profile();
        assert!(profile.supports_role(BundledFontRole::EmojiFallback));
        assert!(
            profile
                .common_fallback_families
                .contains(&"Noto Color Emoji")
        );
    }

    #[cfg(feature = "cjk-lite")]
    #[test]
    fn default_profile_declares_cjk_role_when_enabled() {
        let profile = super::default_profile();
        assert!(profile.supports_role(BundledFontRole::CjkFallback));
        assert!(
            profile
                .common_fallback_families
                .contains(&"Noto Sans CJK SC")
        );
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

use std::sync::OnceLock;

use crate::{
    BundledFontFaceSpec, BundledFontProfile, BundledFontRole, BundledGenericFamily,
    assets::{BOOTSTRAP_FACES, DEFAULT_FACES},
};

pub(crate) const BOOTSTRAP_EXPECTED_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Inter",
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "JetBrains Mono",
    "Fira Mono",
];

pub(crate) const DEFAULT_EXPECTED_FAMILIES: &[&str] = &[
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

pub(crate) const DEFAULT_PROVIDED_ROLES: &[BundledFontRole] = &[
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

#[cfg(any(feature = "emoji", feature = "cjk-lite"))]
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

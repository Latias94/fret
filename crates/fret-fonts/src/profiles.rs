use crate::{
    BundledFontProfile, BundledFontRole, BundledGenericFamily,
    assets::{BOOTSTRAP_FACES, DEFAULT_FACES},
};

pub(crate) const BOOTSTRAP_EXPECTED_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Inter",
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Roboto Slab",
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
    "Roboto Slab",
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "JetBrains Mono",
    "Fira Mono",
];

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const BOOTSTRAP_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] = &[
    BundledGenericFamily::Sans,
    BundledGenericFamily::Serif,
    BundledGenericFamily::Monospace,
];
#[cfg(not(any(feature = "bootstrap-subset", feature = "bootstrap-full")))]
const BOOTSTRAP_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] =
    &[BundledGenericFamily::Monospace];

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const DEFAULT_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] = &[
    BundledGenericFamily::Sans,
    BundledGenericFamily::Serif,
    BundledGenericFamily::Monospace,
];
#[cfg(not(any(feature = "bootstrap-subset", feature = "bootstrap-full")))]
const DEFAULT_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] =
    &[BundledGenericFamily::Monospace];

const BOOTSTRAP_PROVIDED_ROLES: &[BundledFontRole] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    BundledFontRole::UiSans,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    BundledFontRole::UiSerif,
    BundledFontRole::UiMonospace,
];

pub(crate) const DEFAULT_PROVIDED_ROLES: &[BundledFontRole] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    BundledFontRole::UiSans,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    BundledFontRole::UiSerif,
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
const BOOTSTRAP_UI_SERIF_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Roboto Slab",
];
const BOOTSTRAP_UI_MONO_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "JetBrains Mono",
    "Fira Mono",
];

const DEFAULT_UI_SANS_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Inter",
];
const DEFAULT_UI_SERIF_FAMILIES: &[&str] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    "Roboto Slab",
];
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

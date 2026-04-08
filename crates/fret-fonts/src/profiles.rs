use crate::{
    BundledFontProfile, BundledFontRole, BundledGenericFamily,
    assets::{BOOTSTRAP_FACES, DEFAULT_FACES},
};

pub(crate) const BOOTSTRAP_EXPECTED_FAMILIES: &[&str] =
    &["Inter", "Roboto Slab", "JetBrains Mono", "Fira Mono"];

pub(crate) const DEFAULT_EXPECTED_FAMILIES: &[&str] = BOOTSTRAP_EXPECTED_FAMILIES;

const BOOTSTRAP_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] = &[
    BundledGenericFamily::Sans,
    BundledGenericFamily::Serif,
    BundledGenericFamily::Monospace,
];
const DEFAULT_GUARANTEED_GENERIC_FAMILIES: &[BundledGenericFamily] =
    BOOTSTRAP_GUARANTEED_GENERIC_FAMILIES;

const BOOTSTRAP_PROVIDED_ROLES: &[BundledFontRole] = &[
    BundledFontRole::UiSans,
    BundledFontRole::UiSerif,
    BundledFontRole::UiMonospace,
];

pub(crate) const DEFAULT_PROVIDED_ROLES: &[BundledFontRole] = BOOTSTRAP_PROVIDED_ROLES;

const BOOTSTRAP_UI_SANS_FAMILIES: &[&str] = &["Inter"];
const BOOTSTRAP_UI_SERIF_FAMILIES: &[&str] = &["Roboto Slab"];
const BOOTSTRAP_UI_MONO_FAMILIES: &[&str] = &["JetBrains Mono", "Fira Mono"];

const DEFAULT_UI_SANS_FAMILIES: &[&str] = BOOTSTRAP_UI_SANS_FAMILIES;
const DEFAULT_UI_SERIF_FAMILIES: &[&str] = BOOTSTRAP_UI_SERIF_FAMILIES;
const DEFAULT_UI_MONO_FAMILIES: &[&str] = BOOTSTRAP_UI_MONO_FAMILIES;
const DEFAULT_COMMON_FALLBACK_FAMILIES: &[&str] = &[];

const BOOTSTRAP_PROFILE_NAME: &str = "bootstrap-subset";
const DEFAULT_PROFILE_NAME: &str = "default-subset";

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

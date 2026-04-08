use crate::{BundledFontFaceSpec, BundledFontRole};

const FONT_TTF_MEDIA_TYPE: &str = "font/ttf";
const ROLE_UI_SANS: &[BundledFontRole] = &[BundledFontRole::UiSans];
const ROLE_UI_SERIF: &[BundledFontRole] = &[BundledFontRole::UiSerif];
const ROLE_UI_MONO: &[BundledFontRole] = &[BundledFontRole::UiMonospace];

const FIRA_MONO_SUBSET: &[u8] = include_bytes!("../assets/FiraMono-subset.ttf");
const FIRA_MONO_ASSET_KEY: &str = "fonts/FiraMono-subset.ttf";

const INTER_ROMAN_BYTES: &[u8] = include_bytes!("../assets/Inter-roman-subset.ttf");
const INTER_ROMAN_ASSET_KEY: &str = "fonts/Inter-roman-subset.ttf";

const INTER_ITALIC_BYTES: &[u8] = include_bytes!("../assets/Inter-italic-subset.ttf");
const INTER_ITALIC_ASSET_KEY: &str = "fonts/Inter-italic-subset.ttf";

const ROBOTO_SLAB_VARIABLE_BYTES: &[u8] =
    include_bytes!("../assets/RobotoSlab-VariableFont_wght.ttf");
const ROBOTO_SLAB_VARIABLE_ASSET_KEY: &str = "fonts/RobotoSlab-VariableFont_wght.ttf";

const JETBRAINS_MONO_ROMAN_BYTES: &[u8] =
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf");
const JETBRAINS_MONO_ROMAN_ASSET_KEY: &str = "fonts/JetBrainsMono-roman-subset.ttf";

const JETBRAINS_MONO_ITALIC_BYTES: &[u8] =
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf");
const JETBRAINS_MONO_ITALIC_ASSET_KEY: &str = "fonts/JetBrainsMono-italic-subset.ttf";

pub(crate) const FIRA_MONO_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    bundle_name: env!("CARGO_PKG_NAME"),
    family: "Fira Mono",
    roles: ROLE_UI_MONO,
    asset_key: FIRA_MONO_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: FIRA_MONO_SUBSET,
};

pub(crate) const INTER_ROMAN_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    bundle_name: env!("CARGO_PKG_NAME"),
    family: "Inter",
    roles: ROLE_UI_SANS,
    asset_key: INTER_ROMAN_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: INTER_ROMAN_BYTES,
};

pub(crate) const INTER_ITALIC_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    bundle_name: env!("CARGO_PKG_NAME"),
    family: "Inter",
    roles: ROLE_UI_SANS,
    asset_key: INTER_ITALIC_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: INTER_ITALIC_BYTES,
};

pub(crate) const ROBOTO_SLAB_VARIABLE_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    bundle_name: env!("CARGO_PKG_NAME"),
    family: "Roboto Slab",
    roles: ROLE_UI_SERIF,
    asset_key: ROBOTO_SLAB_VARIABLE_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: ROBOTO_SLAB_VARIABLE_BYTES,
};

pub(crate) const JETBRAINS_MONO_ROMAN_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    bundle_name: env!("CARGO_PKG_NAME"),
    family: "JetBrains Mono",
    roles: ROLE_UI_MONO,
    asset_key: JETBRAINS_MONO_ROMAN_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: JETBRAINS_MONO_ROMAN_BYTES,
};

pub(crate) const JETBRAINS_MONO_ITALIC_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    bundle_name: env!("CARGO_PKG_NAME"),
    family: "JetBrains Mono",
    roles: ROLE_UI_MONO,
    asset_key: JETBRAINS_MONO_ITALIC_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: JETBRAINS_MONO_ITALIC_BYTES,
};

pub(crate) const BOOTSTRAP_FACES: &[BundledFontFaceSpec] = &[
    INTER_ROMAN_FACE,
    INTER_ITALIC_FACE,
    ROBOTO_SLAB_VARIABLE_FACE,
    JETBRAINS_MONO_ROMAN_FACE,
    JETBRAINS_MONO_ITALIC_FACE,
    FIRA_MONO_FACE,
];

pub(crate) const DEFAULT_FACES: &[BundledFontFaceSpec] = BOOTSTRAP_FACES;

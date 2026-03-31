use crate::{BundledFontFaceSpec, BundledFontRole};

const FONT_TTF_MEDIA_TYPE: &str = "font/ttf";
#[cfg(feature = "cjk-lite")]
const FONT_OTF_MEDIA_TYPE: &str = "font/otf";

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const ROLE_UI_SANS: &[BundledFontRole] = &[BundledFontRole::UiSans];
#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const ROLE_UI_SERIF: &[BundledFontRole] = &[BundledFontRole::UiSerif];
const ROLE_UI_MONO: &[BundledFontRole] = &[BundledFontRole::UiMonospace];
#[cfg(feature = "emoji")]
const ROLE_EMOJI: &[BundledFontRole] = &[BundledFontRole::EmojiFallback];
#[cfg(feature = "cjk-lite")]
const ROLE_CJK: &[BundledFontRole] = &[BundledFontRole::CjkFallback];

const FIRA_MONO_SUBSET: &[u8] = include_bytes!("../assets/FiraMono-subset.ttf");
const FIRA_MONO_ASSET_KEY: &str = "fonts/FiraMono-subset.ttf";

#[cfg(feature = "emoji")]
const NOTO_COLOR_EMOJI: &[u8] = include_bytes!("../assets/NotoColorEmoji.ttf");
#[cfg(feature = "emoji")]
const NOTO_COLOR_EMOJI_ASSET_KEY: &str = "fonts/NotoColorEmoji.ttf";

#[cfg(feature = "cjk-lite")]
const NOTO_SANS_CJK_SC_LITE_SUBSET: &[u8] =
    include_bytes!("../assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf");
#[cfg(feature = "cjk-lite")]
const NOTO_SANS_CJK_SC_LITE_ASSET_KEY: &str = "fonts/NotoSansCJKsc-Regular-cjk-lite-subset.otf";

#[cfg(feature = "bootstrap-full")]
const INTER_ROMAN_BYTES: &[u8] = include_bytes!("../assets/Inter-roman.ttf");
#[cfg(feature = "bootstrap-full")]
const INTER_ROMAN_ASSET_KEY: &str = "fonts/Inter-roman.ttf";
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const INTER_ROMAN_BYTES: &[u8] = include_bytes!("../assets/Inter-roman-subset.ttf");
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const INTER_ROMAN_ASSET_KEY: &str = "fonts/Inter-roman-subset.ttf";

#[cfg(feature = "bootstrap-full")]
const INTER_ITALIC_BYTES: &[u8] = include_bytes!("../assets/Inter-italic.ttf");
#[cfg(feature = "bootstrap-full")]
const INTER_ITALIC_ASSET_KEY: &str = "fonts/Inter-italic.ttf";
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const INTER_ITALIC_BYTES: &[u8] = include_bytes!("../assets/Inter-italic-subset.ttf");
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const INTER_ITALIC_ASSET_KEY: &str = "fonts/Inter-italic-subset.ttf";

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const ROBOTO_SLAB_VARIABLE_BYTES: &[u8] =
    include_bytes!("../assets/RobotoSlab-VariableFont_wght.ttf");
#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
const ROBOTO_SLAB_VARIABLE_ASSET_KEY: &str = "fonts/RobotoSlab-VariableFont_wght.ttf";

#[cfg(feature = "bootstrap-full")]
const JETBRAINS_MONO_ROMAN_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-roman.ttf");
#[cfg(feature = "bootstrap-full")]
const JETBRAINS_MONO_ROMAN_ASSET_KEY: &str = "fonts/JetBrainsMono-roman.ttf";
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const JETBRAINS_MONO_ROMAN_BYTES: &[u8] =
    include_bytes!("../assets/JetBrainsMono-roman-subset.ttf");
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const JETBRAINS_MONO_ROMAN_ASSET_KEY: &str = "fonts/JetBrainsMono-roman-subset.ttf";

#[cfg(feature = "bootstrap-full")]
const JETBRAINS_MONO_ITALIC_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-italic.ttf");
#[cfg(feature = "bootstrap-full")]
const JETBRAINS_MONO_ITALIC_ASSET_KEY: &str = "fonts/JetBrainsMono-italic.ttf";
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const JETBRAINS_MONO_ITALIC_BYTES: &[u8] =
    include_bytes!("../assets/JetBrainsMono-italic-subset.ttf");
#[cfg(all(feature = "bootstrap-subset", not(feature = "bootstrap-full")))]
const JETBRAINS_MONO_ITALIC_ASSET_KEY: &str = "fonts/JetBrainsMono-italic-subset.ttf";

pub(crate) const FIRA_MONO_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Fira Mono",
    roles: ROLE_UI_MONO,
    asset_key: FIRA_MONO_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: FIRA_MONO_SUBSET,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
pub(crate) const INTER_ROMAN_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Inter",
    roles: ROLE_UI_SANS,
    asset_key: INTER_ROMAN_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: INTER_ROMAN_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
pub(crate) const INTER_ITALIC_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Inter",
    roles: ROLE_UI_SANS,
    asset_key: INTER_ITALIC_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: INTER_ITALIC_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
pub(crate) const ROBOTO_SLAB_VARIABLE_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Roboto Slab",
    roles: ROLE_UI_SERIF,
    asset_key: ROBOTO_SLAB_VARIABLE_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: ROBOTO_SLAB_VARIABLE_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
pub(crate) const JETBRAINS_MONO_ROMAN_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "JetBrains Mono",
    roles: ROLE_UI_MONO,
    asset_key: JETBRAINS_MONO_ROMAN_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: JETBRAINS_MONO_ROMAN_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
pub(crate) const JETBRAINS_MONO_ITALIC_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "JetBrains Mono",
    roles: ROLE_UI_MONO,
    asset_key: JETBRAINS_MONO_ITALIC_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: JETBRAINS_MONO_ITALIC_BYTES,
};

#[cfg(feature = "emoji")]
pub(crate) const NOTO_COLOR_EMOJI_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Noto Color Emoji",
    roles: ROLE_EMOJI,
    asset_key: NOTO_COLOR_EMOJI_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: NOTO_COLOR_EMOJI,
};

#[cfg(feature = "cjk-lite")]
pub(crate) const NOTO_SANS_CJK_SC_LITE_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Noto Sans CJK SC",
    roles: ROLE_CJK,
    asset_key: NOTO_SANS_CJK_SC_LITE_ASSET_KEY,
    media_type: FONT_OTF_MEDIA_TYPE,
    bytes: NOTO_SANS_CJK_SC_LITE_SUBSET,
};

pub(crate) const BOOTSTRAP_FACES: &[BundledFontFaceSpec] = &[
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    INTER_ROMAN_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    INTER_ITALIC_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    ROBOTO_SLAB_VARIABLE_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    JETBRAINS_MONO_ROMAN_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    JETBRAINS_MONO_ITALIC_FACE,
    FIRA_MONO_FACE,
];

pub(crate) const DEFAULT_FACES: &[BundledFontFaceSpec] = &[
    #[cfg(feature = "emoji")]
    NOTO_COLOR_EMOJI_FACE,
    #[cfg(feature = "cjk-lite")]
    NOTO_SANS_CJK_SC_LITE_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    INTER_ROMAN_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    INTER_ITALIC_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    ROBOTO_SLAB_VARIABLE_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    JETBRAINS_MONO_ROMAN_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    JETBRAINS_MONO_ITALIC_FACE,
    FIRA_MONO_FACE,
];

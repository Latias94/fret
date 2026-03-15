use crate::{BundledFontFaceSpec, BundledFontRole};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
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

pub(crate) const FIRA_MONO_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Fira Mono",
    roles: ROLE_UI_MONO,
    bytes: FIRA_MONO_SUBSET,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
pub(crate) const INTER_ROMAN_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Inter",
    roles: ROLE_UI_SANS,
    bytes: INTER_ROMAN_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
pub(crate) const INTER_ITALIC_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Inter",
    roles: ROLE_UI_SANS,
    bytes: INTER_ITALIC_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
pub(crate) const JETBRAINS_MONO_ROMAN_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "JetBrains Mono",
    roles: ROLE_UI_MONO,
    bytes: JETBRAINS_MONO_ROMAN_BYTES,
};

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
pub(crate) const JETBRAINS_MONO_ITALIC_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "JetBrains Mono",
    roles: ROLE_UI_MONO,
    bytes: JETBRAINS_MONO_ITALIC_BYTES,
};

#[cfg(feature = "emoji")]
pub(crate) const NOTO_COLOR_EMOJI_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Noto Color Emoji",
    roles: ROLE_EMOJI,
    bytes: NOTO_COLOR_EMOJI,
};

#[cfg(feature = "cjk-lite")]
pub(crate) const NOTO_SANS_CJK_SC_LITE_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    family: "Noto Sans CJK SC",
    roles: ROLE_CJK,
    bytes: NOTO_SANS_CJK_SC_LITE_SUBSET,
};

pub(crate) const BOOTSTRAP_FACES: &[BundledFontFaceSpec] = &[
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
    JETBRAINS_MONO_ROMAN_FACE,
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    JETBRAINS_MONO_ITALIC_FACE,
    FIRA_MONO_FACE,
];

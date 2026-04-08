use fret_assets::AssetBundleId;

pub use fret_fonts::{BundledFontFaceSpec, BundledFontProfile, BundledFontRole};

#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    use fret_fonts::BundledFontFaceSpec;

    pub fn face_blobs<'a, I>(faces: I) -> impl Iterator<Item = Vec<u8>> + 'a
    where
        I: IntoIterator<Item = &'a BundledFontFaceSpec>,
        I::IntoIter: 'a,
    {
        faces.into_iter().map(|face| face.bytes.to_vec())
    }
}

#[cfg(test)]
mod tests;

const FONT_TTF_MEDIA_TYPE: &str = "font/ttf";
const ROLE_EMOJI: &[BundledFontRole] = &[BundledFontRole::EmojiFallback];

const NOTO_COLOR_EMOJI: &[u8] = include_bytes!("../assets/NotoColorEmoji.ttf");
const NOTO_COLOR_EMOJI_ASSET_KEY: &str = "fonts/NotoColorEmoji.ttf";

const EMOJI_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    bundle_name: env!("CARGO_PKG_NAME"),
    family: "Noto Color Emoji",
    roles: ROLE_EMOJI,
    asset_key: NOTO_COLOR_EMOJI_ASSET_KEY,
    media_type: FONT_TTF_MEDIA_TYPE,
    bytes: NOTO_COLOR_EMOJI,
};

const EMOJI_PROFILE: BundledFontProfile = BundledFontProfile {
    name: "emoji",
    faces: &[EMOJI_FACE],
    provided_roles: &[BundledFontRole::EmojiFallback],
    expected_family_names: &["Noto Color Emoji"],
    guaranteed_generic_families: &[],
    ui_sans_families: &[],
    ui_serif_families: &[],
    ui_mono_families: &[],
    common_fallback_families: &["Noto Color Emoji"],
};

pub fn bundled_asset_bundle() -> AssetBundleId {
    AssetBundleId::package(env!("CARGO_PKG_NAME"))
}

pub fn default_profile() -> &'static BundledFontProfile {
    &EMOJI_PROFILE
}

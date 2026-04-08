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

const FONT_OTF_MEDIA_TYPE: &str = "font/otf";
const ROLE_CJK: &[BundledFontRole] = &[BundledFontRole::CjkFallback];

const NOTO_SANS_CJK_SC_LITE_SUBSET: &[u8] =
    include_bytes!("../assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf");
const NOTO_SANS_CJK_SC_LITE_ASSET_KEY: &str = "fonts/NotoSansCJKsc-Regular-cjk-lite-subset.otf";

const CJK_LITE_FACE: BundledFontFaceSpec = BundledFontFaceSpec {
    bundle_name: env!("CARGO_PKG_NAME"),
    family: "Noto Sans CJK SC",
    roles: ROLE_CJK,
    asset_key: NOTO_SANS_CJK_SC_LITE_ASSET_KEY,
    media_type: FONT_OTF_MEDIA_TYPE,
    bytes: NOTO_SANS_CJK_SC_LITE_SUBSET,
};

const CJK_LITE_PROFILE: BundledFontProfile = BundledFontProfile {
    name: "cjk-lite",
    faces: &[CJK_LITE_FACE],
    provided_roles: &[BundledFontRole::CjkFallback],
    expected_family_names: &["Noto Sans CJK SC"],
    guaranteed_generic_families: &[],
    ui_sans_families: &[],
    ui_serif_families: &[],
    ui_mono_families: &[],
    common_fallback_families: &["Noto Sans CJK SC"],
};

pub fn bundled_asset_bundle() -> AssetBundleId {
    AssetBundleId::package(env!("CARGO_PKG_NAME"))
}

pub fn default_profile() -> &'static BundledFontProfile {
    &CJK_LITE_PROFILE
}

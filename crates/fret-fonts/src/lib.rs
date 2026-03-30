//! Bundled font assets for `fret`.
//!
//! Notes:
//! - Web/WASM cannot access system fonts, so applications must provide font bytes.
//! - This crate exposes both the bytes and a small manifest describing which bundled profile
//!   guarantees which family/role surface.
//! - Framework-owned startup baselines should publish bundled asset identity through the shared
//!   runtime asset contract and resolve startup bytes from that identity before renderer
//!   injection.
//! - The shipped bootstrap/default profiles currently guarantee `sans` and `monospace`, but they
//!   do not guarantee `serif`; app shells that need deterministic serif typography on Web/WASM
//!   must bundle and register serif-capable fonts explicitly.

use fret_assets::{
    AssetBundleId, AssetKindHint, AssetLocator, AssetRequest, AssetRevision, StaticAssetEntry,
};

mod assets;
mod profiles;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

#[cfg(test)]
mod tests;

pub use profiles::{bootstrap_profile, default_profile};

pub fn bundled_asset_bundle() -> AssetBundleId {
    AssetBundleId::package(env!("CARGO_PKG_NAME"))
}

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
    pub asset_key: &'static str,
    pub media_type: &'static str,
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
    pub fn asset_entries(&self) -> impl ExactSizeIterator<Item = StaticAssetEntry> + '_ {
        self.faces.iter().map(BundledFontFaceSpec::asset_entry)
    }

    pub fn faces_for_role(
        &self,
        role: BundledFontRole,
    ) -> impl Iterator<Item = &BundledFontFaceSpec> + '_ {
        self.faces
            .iter()
            .filter(move |face| face.supports_role(role))
    }

    pub fn supports_role(&self, role: BundledFontRole) -> bool {
        self.faces_for_role(role).next().is_some()
    }

    pub fn guarantees_generic_family(&self, family: BundledGenericFamily) -> bool {
        self.guaranteed_generic_families.contains(&family)
    }

    pub fn face_for_asset_key(&self, key: &str) -> Option<&BundledFontFaceSpec> {
        self.faces.iter().find(|face| face.asset_key == key)
    }
}

impl BundledFontFaceSpec {
    pub fn supports_role(&self, role: BundledFontRole) -> bool {
        self.roles.contains(&role)
    }

    pub fn asset_locator(&self) -> AssetLocator {
        AssetLocator::bundle(bundled_asset_bundle(), self.asset_key)
    }

    pub fn asset_request(&self) -> AssetRequest {
        AssetRequest::new(self.asset_locator()).with_kind_hint(AssetKindHint::Font)
    }

    pub fn asset_entry(&self) -> StaticAssetEntry {
        StaticAssetEntry::new(
            self.asset_key,
            stable_font_asset_revision(self.asset_key, self.bytes),
            self.bytes,
        )
        .with_media_type(self.media_type)
    }
}

fn stable_font_asset_revision(key: &str, bytes: &[u8]) -> AssetRevision {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in key.as_bytes().iter().chain(bytes.iter()) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    AssetRevision(if hash == 0 { 1 } else { hash })
}

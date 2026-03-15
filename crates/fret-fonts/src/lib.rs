//! Bundled font assets for `fret`.
//!
//! Notes:
//! - Web/WASM cannot access system fonts, so applications must provide font bytes.
//! - This crate exposes both the bytes and a small manifest describing which bundled profile
//!   guarantees which family/role surface.

mod assets;
mod profiles;

#[cfg(test)]
mod tests;

pub use profiles::{bootstrap_fonts, bootstrap_profile, default_fonts, default_profile};

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

    pub fn font_bytes_for_role(
        &self,
        role: BundledFontRole,
    ) -> impl Iterator<Item = &'static [u8]> + '_ {
        self.faces
            .iter()
            .filter(move |face| face.roles.contains(&role))
            .map(|face| face.bytes)
    }

    pub fn supports_role(&self, role: BundledFontRole) -> bool {
        self.faces.iter().any(|face| face.roles.contains(&role))
    }

    pub fn guarantees_generic_family(&self, family: BundledGenericFamily) -> bool {
        self.guaranteed_generic_families.contains(&family)
    }
}

//! Bundled font assets for `fret`.
//!
//! Notes:
//! - Web/WASM cannot access system fonts, so applications must provide font bytes.
//! - This crate exposes both the bytes and a small manifest describing which bundled profile
//!   guarantees which family/role surface.
//! - Framework-owned startup baselines should publish bundled asset identity through the shared
//!   runtime asset contract and resolve startup bytes from that identity before renderer
//!   injection.
//! - The shipped bootstrap/default profiles now guarantee `sans`, `serif`, and `monospace`
//!   whenever the bootstrap font features are enabled; the intentionally minimal mono-only build
//!   still avoids promising `sans`/`serif`.

use fret_assets::{
    AssetBundleId, AssetCapabilities, AssetKindHint, AssetLoadError, AssetLocator, AssetMemoryKey,
    AssetRequest, AssetResolver, AssetRevision, ResolvedAssetBytes, StaticAssetEntry,
};
use std::collections::HashMap;
use std::sync::RwLock;

mod assets;
mod profiles;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

#[cfg(test)]
mod tests;

pub use profiles::{bootstrap_profile, default_profile};

/// File extensions that first-party file dialogs should advertise for user-provided font import.
pub const SUPPORTED_USER_FONT_IMPORT_EXTENSIONS: &[&str] = &["ttf", "otf", "ttc"];

/// Result of preparing user-provided font files for the runtime memory-asset lane.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportedFontAssetBatch {
    pub requests: Vec<AssetRequest>,
    pub resolved: Vec<ResolvedAssetBytes>,
    pub rejected_files: usize,
}

/// Mutable memory resolver used by first-party and app-level local font import flows.
#[derive(Debug, Default)]
pub struct ImportedFontAssetResolver {
    entries: RwLock<HashMap<AssetLocator, ResolvedAssetBytes>>,
}

impl ImportedFontAssetResolver {
    pub fn replace_entries(&self, entries: impl IntoIterator<Item = ResolvedAssetBytes>) {
        let next = entries
            .into_iter()
            .map(|resolved| (resolved.locator.clone(), resolved))
            .collect::<HashMap<_, _>>();
        *self
            .entries
            .write()
            .expect("poisoned ImportedFontAssetResolver entries lock") = next;
    }

    pub fn replace_batch(&self, batch: &ImportedFontAssetBatch) {
        self.replace_entries(batch.resolved.iter().cloned());
    }
}

impl AssetResolver for ImportedFontAssetResolver {
    fn capabilities(&self) -> AssetCapabilities {
        AssetCapabilities {
            memory: true,
            embedded: false,
            bundle_asset: false,
            file: false,
            url: false,
            file_watch: false,
            system_font_scan: false,
        }
    }

    fn resolve_bytes(&self, request: &AssetRequest) -> Result<ResolvedAssetBytes, AssetLoadError> {
        if !matches!(request.locator, AssetLocator::Memory(_)) {
            return Err(AssetLoadError::UnsupportedLocatorKind {
                kind: request.locator.kind(),
            });
        }

        self.entries
            .read()
            .expect("poisoned ImportedFontAssetResolver entries lock")
            .get(&request.locator)
            .cloned()
            .ok_or(AssetLoadError::NotFound)
    }
}

/// Returns true when the bytes look like a TTF/OTF/TTC payload accepted by the raw font lane.
pub fn is_supported_user_font_bytes(bytes: &[u8]) -> bool {
    bytes.starts_with(b"OTTO")
        || bytes.starts_with(b"ttcf")
        || bytes
            .get(0..4)
            .is_some_and(|header| header == [0x00, 0x01, 0x00, 0x00])
}

/// Prepares user-provided font files for the runtime memory-asset lane.
///
/// Each accepted file is assigned a stable session-local memory locator and a `Font` kind hint so
/// first-party import flows can stay on the shared asset identity contract instead of injecting raw
/// byte vectors directly into the renderer path.
pub fn build_imported_font_asset_batch<I, N, B>(files: I) -> ImportedFontAssetBatch
where
    I: IntoIterator<Item = (N, B)>,
    N: AsRef<str>,
    B: AsRef<[u8]>,
{
    let mut batch = ImportedFontAssetBatch::default();
    let mut duplicate_counts = HashMap::<String, usize>::new();

    for (name, bytes) in files {
        let name = name.as_ref();
        let bytes = bytes.as_ref();
        if !is_supported_user_font_bytes(bytes) {
            batch.rejected_files += 1;
            continue;
        }

        let memory_key = imported_font_memory_key(name, bytes, &mut duplicate_counts);
        let revision = stable_font_asset_revision(memory_key.as_str(), bytes);
        let locator = AssetLocator::memory(memory_key);
        let request = AssetRequest::new(locator.clone()).with_kind_hint(AssetKindHint::Font);
        let mut resolved = ResolvedAssetBytes::new(locator, revision, bytes.to_vec());
        if let Some(media_type) = supported_user_font_media_type(bytes) {
            resolved = resolved.with_media_type(media_type);
        }

        batch.requests.push(request);
        batch.resolved.push(resolved);
    }

    batch
}

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
    let hash = stable_font_asset_hash(key.as_bytes().iter().copied().chain(bytes.iter().copied()));
    AssetRevision(if hash == 0 { 1 } else { hash })
}

fn stable_font_asset_hash(bytes: impl IntoIterator<Item = u8>) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn imported_font_memory_key(
    name: &str,
    bytes: &[u8],
    duplicate_counts: &mut HashMap<String, usize>,
) -> AssetMemoryKey {
    let base = imported_font_memory_key_base(name, bytes);
    let duplicates = duplicate_counts.entry(base.clone()).or_insert(0);
    let key = if *duplicates == 0 {
        base
    } else {
        format!("{base}-dup{duplicates}")
    };
    *duplicates += 1;
    AssetMemoryKey::new(key)
}

fn imported_font_memory_key_base(name: &str, bytes: &[u8]) -> String {
    let name = sanitize_imported_font_name(name);
    let hash = stable_font_asset_hash(
        name.as_bytes()
            .iter()
            .copied()
            .chain(std::iter::once(0xff))
            .chain(bytes.iter().copied()),
    );
    format!("user-font/{hash:016x}-{name}")
}

fn sanitize_imported_font_name(name: &str) -> String {
    let name = name.trim();
    let name = if name.is_empty() { "font" } else { name };
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn supported_user_font_media_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"OTTO") {
        Some("font/otf")
    } else if bytes.starts_with(b"ttcf") {
        Some("font/collection")
    } else if bytes
        .get(0..4)
        .is_some_and(|header| header == [0x00, 0x01, 0x00, 0x00])
    {
        Some("font/ttf")
    } else {
        None
    }
}

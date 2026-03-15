//! Portable asset contract vocabulary for the Fret workspace.
//!
//! This crate intentionally defines only stable, dependency-light asset contract types:
//!
//! - logical asset identity (`AssetBundleId`, `AssetKey`, `AssetLocator`),
//! - capability reporting (`AssetCapabilities`),
//! - revisioning (`AssetRevision`),
//! - and small request/result/error types for higher layers to build on.
//!
//! It does not own:
//!
//! - packaging policy,
//! - async loading orchestration,
//! - cache lifetimes,
//! - UI invalidation,
//! - or platform-specific resolver implementations.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetLocatorKind {
    Memory,
    Embedded,
    BundleAsset,
    File,
    Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetBundleId(SmolStr);

impl AssetBundleId {
    pub fn new(value: impl Into<SmolStr>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for AssetBundleId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AssetBundleId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<SmolStr> for AssetBundleId {
    fn from(value: SmolStr) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetKey(SmolStr);

impl AssetKey {
    pub fn new(value: impl Into<SmolStr>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for AssetKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AssetKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<SmolStr> for AssetKey {
    fn from(value: SmolStr) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetMemoryKey(SmolStr);

impl AssetMemoryKey {
    pub fn new(value: impl Into<SmolStr>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for AssetMemoryKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AssetMemoryKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<SmolStr> for AssetMemoryKey {
    fn from(value: SmolStr) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmbeddedAssetLocator {
    pub owner: AssetBundleId,
    pub key: AssetKey,
}

impl EmbeddedAssetLocator {
    pub fn new(owner: impl Into<AssetBundleId>, key: impl Into<AssetKey>) -> Self {
        Self {
            owner: owner.into(),
            key: key.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BundleAssetLocator {
    pub bundle: AssetBundleId,
    pub key: AssetKey,
}

impl BundleAssetLocator {
    pub fn new(bundle: impl Into<AssetBundleId>, key: impl Into<AssetKey>) -> Self {
        Self {
            bundle: bundle.into(),
            key: key.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileAssetLocator {
    pub path: PathBuf,
}

impl FileAssetLocator {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UrlAssetLocator {
    pub url: SmolStr,
}

impl UrlAssetLocator {
    pub fn new(url: impl Into<SmolStr>) -> Self {
        Self { url: url.into() }
    }

    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetLocator {
    Memory(AssetMemoryKey),
    Embedded(EmbeddedAssetLocator),
    BundleAsset(BundleAssetLocator),
    File(FileAssetLocator),
    Url(UrlAssetLocator),
}

impl AssetLocator {
    pub fn kind(&self) -> AssetLocatorKind {
        match self {
            Self::Memory(_) => AssetLocatorKind::Memory,
            Self::Embedded(_) => AssetLocatorKind::Embedded,
            Self::BundleAsset(_) => AssetLocatorKind::BundleAsset,
            Self::File(_) => AssetLocatorKind::File,
            Self::Url(_) => AssetLocatorKind::Url,
        }
    }

    pub fn memory(key: impl Into<AssetMemoryKey>) -> Self {
        Self::Memory(key.into())
    }

    pub fn embedded(owner: impl Into<AssetBundleId>, key: impl Into<AssetKey>) -> Self {
        Self::Embedded(EmbeddedAssetLocator::new(owner, key))
    }

    pub fn bundle(bundle: impl Into<AssetBundleId>, key: impl Into<AssetKey>) -> Self {
        Self::BundleAsset(BundleAssetLocator::new(bundle, key))
    }

    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(FileAssetLocator::new(path))
    }

    pub fn url(url: impl Into<SmolStr>) -> Self {
        Self::Url(UrlAssetLocator::new(url))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetRevision(pub u64);

impl AssetRevision {
    pub const ZERO: Self = Self(0);

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetKindHint {
    Binary,
    Image,
    Svg,
    Font,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetMediaType(SmolStr);

impl AssetMediaType {
    pub fn new(value: impl Into<SmolStr>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for AssetMediaType {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AssetMediaType {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<SmolStr> for AssetMediaType {
    fn from(value: SmolStr) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AssetCapabilities {
    pub memory: bool,
    pub embedded: bool,
    pub bundle_asset: bool,
    pub file: bool,
    pub url: bool,
    pub file_watch: bool,
    pub system_font_scan: bool,
}

impl AssetCapabilities {
    pub fn supports_kind(&self, kind: AssetLocatorKind) -> bool {
        match kind {
            AssetLocatorKind::Memory => self.memory,
            AssetLocatorKind::Embedded => self.embedded,
            AssetLocatorKind::BundleAsset => self.bundle_asset,
            AssetLocatorKind::File => self.file,
            AssetLocatorKind::Url => self.url,
        }
    }

    pub fn supports(&self, locator: &AssetLocator) -> bool {
        self.supports_kind(locator.kind())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRequest {
    pub locator: AssetLocator,
    pub kind_hint: Option<AssetKindHint>,
}

impl AssetRequest {
    pub fn new(locator: AssetLocator) -> Self {
        Self {
            locator,
            kind_hint: None,
        }
    }

    pub fn with_kind_hint(mut self, kind_hint: AssetKindHint) -> Self {
        self.kind_hint = Some(kind_hint);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAssetBytes {
    pub locator: AssetLocator,
    pub revision: AssetRevision,
    pub media_type: Option<AssetMediaType>,
    pub bytes: Arc<[u8]>,
}

impl ResolvedAssetBytes {
    pub fn new(
        locator: AssetLocator,
        revision: AssetRevision,
        bytes: impl Into<Arc<[u8]>>,
    ) -> Self {
        Self {
            locator,
            revision,
            media_type: None,
            bytes: bytes.into(),
        }
    }

    pub fn with_media_type(mut self, media_type: impl Into<AssetMediaType>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }
}

pub trait AssetResolver: 'static + Send + Sync {
    fn capabilities(&self) -> AssetCapabilities;
    fn resolve_bytes(&self, request: &AssetRequest) -> Result<ResolvedAssetBytes, AssetLoadError>;
}

impl dyn AssetResolver + '_ {
    pub fn supports(&self, locator: &AssetLocator) -> bool {
        self.capabilities().supports(locator)
    }

    pub fn resolve_locator_bytes(
        &self,
        locator: AssetLocator,
    ) -> Result<ResolvedAssetBytes, AssetLoadError> {
        self.resolve_bytes(&AssetRequest::new(locator))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum AssetLoadError {
    #[error("asset locator kind {kind:?} is not supported on this host")]
    UnsupportedLocatorKind { kind: AssetLocatorKind },
    #[error("asset not found")]
    NotFound,
    #[error("asset access denied")]
    AccessDenied,
    #[error("asset load failed: {message}")]
    Message { message: SmolStr },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_kind_matches_variant() {
        assert_eq!(
            AssetLocator::memory("framebuffer-snapshot").kind(),
            AssetLocatorKind::Memory
        );
        assert_eq!(
            AssetLocator::embedded("fret-ui-shadcn", "icons/search.svg").kind(),
            AssetLocatorKind::Embedded
        );
        assert_eq!(
            AssetLocator::bundle("app", "images/logo.png").kind(),
            AssetLocatorKind::BundleAsset
        );
        assert_eq!(
            AssetLocator::file("assets/logo.png").kind(),
            AssetLocatorKind::File
        );
        assert_eq!(
            AssetLocator::url("https://example.com/logo.png").kind(),
            AssetLocatorKind::Url
        );
    }

    #[test]
    fn capabilities_report_support_per_locator_kind() {
        let caps = AssetCapabilities {
            memory: true,
            embedded: true,
            bundle_asset: true,
            file: false,
            url: true,
            file_watch: false,
            system_font_scan: false,
        };

        assert!(caps.supports(&AssetLocator::bundle("app", "images/logo.png")));
        assert!(caps.supports(&AssetLocator::embedded("ui-kit", "icons/close.svg")));
        assert!(!caps.supports(&AssetLocator::file("assets/logo.png")));
    }

    #[test]
    fn resolved_asset_bytes_can_attach_media_type() {
        let resolved = ResolvedAssetBytes::new(
            AssetLocator::bundle("app", "images/logo.png"),
            AssetRevision(7),
            Arc::<[u8]>::from([1u8, 2, 3]),
        )
        .with_media_type("image/png");

        assert_eq!(resolved.revision, AssetRevision(7));
        assert_eq!(
            resolved.media_type.as_ref().map(AssetMediaType::as_str),
            Some("image/png")
        );
        assert_eq!(resolved.bytes.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn asset_resolver_supports_capability_queries() {
        struct TestResolver;

        impl AssetResolver for TestResolver {
            fn capabilities(&self) -> AssetCapabilities {
                AssetCapabilities {
                    memory: true,
                    embedded: true,
                    bundle_asset: true,
                    file: false,
                    url: false,
                    file_watch: false,
                    system_font_scan: false,
                }
            }

            fn resolve_bytes(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetBytes, AssetLoadError> {
                Ok(ResolvedAssetBytes::new(
                    request.locator.clone(),
                    AssetRevision(1),
                    Arc::<[u8]>::from([9u8, 8, 7]),
                ))
            }
        }

        let resolver = TestResolver;
        let dyn_resolver: &dyn AssetResolver = &resolver;

        assert!(dyn_resolver.supports(&AssetLocator::bundle("app", "images/logo.png")));
        assert!(!dyn_resolver.supports(&AssetLocator::file("assets/logo.png")));

        let resolved = dyn_resolver
            .resolve_locator_bytes(AssetLocator::bundle("app", "images/logo.png"))
            .expect("bundle asset should resolve");
        assert_eq!(resolved.revision, AssetRevision(1));
        assert_eq!(resolved.bytes.as_ref(), &[9, 8, 7]);
    }
}

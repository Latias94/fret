#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::hash::{DefaultHasher, Hash, Hasher};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{AssetBundleId, AssetKey, AssetMediaType};
#[cfg(not(target_arch = "wasm32"))]
use crate::{
    AssetCapabilities, AssetLoadError, AssetLocator, AssetRequest, AssetResolver, AssetRevision,
    ResolvedAssetBytes,
};

pub const FILE_ASSET_MANIFEST_KIND_V1: &str = "fret_file_asset_manifest";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileAssetManifestV1 {
    pub schema_version: u32,
    pub kind: SmolStr,
    pub bundles: Vec<FileAssetManifestBundleV1>,
}

impl FileAssetManifestV1 {
    pub const SCHEMA_VERSION: u32 = 1;

    pub fn new(bundles: impl IntoIterator<Item = FileAssetManifestBundleV1>) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            kind: FILE_ASSET_MANIFEST_KIND_V1.into(),
            bundles: bundles.into_iter().collect(),
        }
    }

    pub fn validate(&self) -> Result<(), AssetManifestLoadError> {
        if self.schema_version != Self::SCHEMA_VERSION {
            return Err(AssetManifestLoadError::InvalidManifest {
                message: format!(
                    "invalid schema_version {} (expected {})",
                    self.schema_version,
                    Self::SCHEMA_VERSION
                )
                .into(),
            });
        }

        if self.kind.as_str() != FILE_ASSET_MANIFEST_KIND_V1 {
            return Err(AssetManifestLoadError::InvalidManifest {
                message: format!(
                    "invalid kind {:?} (expected {FILE_ASSET_MANIFEST_KIND_V1:?})",
                    self.kind
                )
                .into(),
            });
        }

        let mut seen = std::collections::HashSet::new();
        for bundle in &self.bundles {
            if bundle.id.as_str().trim().is_empty() {
                return Err(AssetManifestLoadError::InvalidManifest {
                    message: "bundle id must not be empty".into(),
                });
            }

            for entry in &bundle.entries {
                if entry.key.as_str().trim().is_empty() {
                    return Err(AssetManifestLoadError::InvalidManifest {
                        message: format!("bundle {:?} contains an empty asset key", bundle.id)
                            .into(),
                    });
                }

                let duplicate_key = (bundle.id.clone(), entry.key.clone());
                if !seen.insert(duplicate_key.clone()) {
                    return Err(AssetManifestLoadError::DuplicateBundleKey {
                        bundle: duplicate_key.0,
                        key: duplicate_key.1,
                    });
                }
            }
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_json_path(path: impl AsRef<Path>) -> Result<Self, AssetManifestLoadError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| AssetManifestLoadError::ReadManifest {
            path: path.to_path_buf(),
            source,
        })?;
        let manifest = serde_json::from_slice::<Self>(&bytes).map_err(|source| {
            AssetManifestLoadError::ParseManifest {
                path: path.to_path_buf(),
                source,
            }
        })?;
        manifest.validate()?;
        Ok(manifest)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_json_path(&self, path: impl AsRef<Path>) -> Result<(), AssetManifestLoadError> {
        self.validate()?;

        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| {
                AssetManifestLoadError::WriteManifest {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
        }

        let bytes = serde_json::to_vec_pretty(self).map_err(|source| {
            AssetManifestLoadError::SerializeManifest {
                path: path.to_path_buf(),
                source,
            }
        })?;
        std::fs::write(path, bytes).map_err(|source| AssetManifestLoadError::WriteManifest {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileAssetManifestBundleV1 {
    pub id: AssetBundleId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<PathBuf>,
    #[serde(default)]
    pub entries: Vec<FileAssetManifestEntryV1>,
}

impl FileAssetManifestBundleV1 {
    pub fn new(
        id: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = FileAssetManifestEntryV1>,
    ) -> Self {
        Self {
            id: id.into(),
            root: None,
            entries: entries.into_iter().collect(),
        }
    }

    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn scan_dir(
        id: impl Into<AssetBundleId>,
        root: impl AsRef<Path>,
    ) -> Result<Self, AssetManifestLoadError> {
        let id = id.into();
        let root = root.as_ref();
        let metadata =
            std::fs::metadata(root).map_err(|source| AssetManifestLoadError::ReadBundleRoot {
                path: root.to_path_buf(),
                source,
            })?;
        if !metadata.is_dir() {
            return Err(AssetManifestLoadError::InvalidManifest {
                message: format!("bundle root is not a directory: {}", root.display()).into(),
            });
        }

        let mut files = Vec::new();
        collect_bundle_files(root, &mut files)?;
        files.sort();

        let entries = files
            .into_iter()
            .map(|path| {
                let rel = path.strip_prefix(root).map_err(|_| {
                    AssetManifestLoadError::InvalidManifest {
                        message: format!(
                            "failed to strip bundle root {} from {}",
                            root.display(),
                            path.display()
                        )
                        .into(),
                    }
                })?;
                let key = rel.to_string_lossy().replace('\\', "/");
                Ok(FileAssetManifestEntryV1::new(key))
            })
            .collect::<Result<Vec<_>, AssetManifestLoadError>>()?;

        Ok(Self::new(id, entries).with_root(root.to_path_buf()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileAssetManifestEntryV1 {
    pub key: AssetKey,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<AssetMediaType>,
}

impl FileAssetManifestEntryV1 {
    pub fn new(key: impl Into<AssetKey>) -> Self {
        Self {
            key: key.into(),
            path: None,
            media_type: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_media_type(mut self, media_type: impl Into<AssetMediaType>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssetManifestLoadError {
    #[error("failed to read asset manifest {path}: {source}")]
    ReadManifest {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse asset manifest {path}: {source}")]
    ParseManifest {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to serialize asset manifest {path}: {source}")]
    SerializeManifest {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write asset manifest {path}: {source}")]
    WriteManifest {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read asset bundle root {path}: {source}")]
    ReadBundleRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid asset manifest: {message}")]
    InvalidManifest { message: SmolStr },
    #[error("duplicate asset manifest entry for bundle {bundle:?} key {key:?}")]
    DuplicateBundleKey {
        bundle: AssetBundleId,
        key: AssetKey,
    },
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct FileAssetManifestResolver {
    manifest_path: PathBuf,
    entries: HashMap<AssetLocator, FileAssetManifestResolvedEntry>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct FileAssetManifestResolvedEntry {
    path: PathBuf,
    media_type: Option<AssetMediaType>,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileAssetManifestResolver {
    pub fn from_manifest_path(path: impl AsRef<Path>) -> Result<Self, AssetManifestLoadError> {
        let path = path.as_ref();
        let manifest = FileAssetManifestV1::load_json_path(path)?;
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
        Self::from_manifest_with_base_dir(manifest, base_dir, path.to_path_buf())
    }

    pub fn from_bundle_dir(
        bundle: impl Into<AssetBundleId>,
        root: impl AsRef<Path>,
    ) -> Result<Self, AssetManifestLoadError> {
        let root = root.as_ref();
        let manifest =
            FileAssetManifestV1::new([FileAssetManifestBundleV1::scan_dir(bundle, root)?]);
        Self::from_manifest_with_base_dir(manifest, PathBuf::new(), root.to_path_buf())
    }

    pub fn from_manifest_with_base_dir(
        manifest: FileAssetManifestV1,
        base_dir: impl Into<PathBuf>,
        manifest_path: impl Into<PathBuf>,
    ) -> Result<Self, AssetManifestLoadError> {
        manifest.validate()?;

        let base_dir = base_dir.into();
        let mut entries = HashMap::new();
        for bundle in manifest.bundles {
            let bundle_root = bundle.root.unwrap_or_default();
            for entry in bundle.entries {
                let locator = AssetLocator::bundle(bundle.id.clone(), entry.key.clone());
                let entry_path = entry
                    .path
                    .unwrap_or_else(|| PathBuf::from(entry.key.as_str()));
                let path = resolve_manifest_path(&base_dir, &bundle_root, &entry_path);
                entries.insert(
                    locator,
                    FileAssetManifestResolvedEntry {
                        path,
                        media_type: entry.media_type,
                    },
                );
            }
        }

        Ok(Self {
            manifest_path: manifest_path.into(),
            entries,
        })
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl AssetResolver for FileAssetManifestResolver {
    fn capabilities(&self) -> AssetCapabilities {
        AssetCapabilities {
            memory: false,
            embedded: false,
            bundle_asset: true,
            file: false,
            url: false,
            file_watch: false,
            system_font_scan: false,
        }
    }

    fn resolve_bytes(&self, request: &AssetRequest) -> Result<ResolvedAssetBytes, AssetLoadError> {
        let Some(entry) = self.entries.get(&request.locator) else {
            return Err(match request.locator {
                AssetLocator::BundleAsset(_) => AssetLoadError::NotFound,
                _ => AssetLoadError::UnsupportedLocatorKind {
                    kind: request.locator.kind(),
                },
            });
        };

        let bytes = std::fs::read(&entry.path).map_err(map_fs_read_error)?;
        let mut resolved = ResolvedAssetBytes::new(
            request.locator.clone(),
            AssetRevision(hash_bytes(&bytes)),
            bytes,
        );
        if let Some(media_type) = &entry.media_type {
            resolved = resolved.with_media_type(media_type.clone());
        }
        Ok(resolved)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_manifest_path(base_dir: &Path, bundle_root: &Path, entry_path: &Path) -> PathBuf {
    if entry_path.is_absolute() {
        return entry_path.to_path_buf();
    }

    let joined_root = if bundle_root.is_absolute() {
        bundle_root.to_path_buf()
    } else {
        base_dir.join(bundle_root)
    };
    joined_root.join(entry_path)
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_bundle_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), AssetManifestLoadError> {
    let mut entries =
        std::fs::read_dir(dir).map_err(|source| AssetManifestLoadError::ReadBundleRoot {
            path: dir.to_path_buf(),
            source,
        })?;
    let mut paths = Vec::new();
    while let Some(entry) = entries.next() {
        let entry = entry.map_err(|source| AssetManifestLoadError::ReadBundleRoot {
            path: dir.to_path_buf(),
            source,
        })?;
        paths.push(entry.path());
    }
    paths.sort();

    for path in paths {
        let metadata =
            std::fs::metadata(&path).map_err(|source| AssetManifestLoadError::ReadBundleRoot {
                path: path.clone(),
                source,
            })?;
        if metadata.is_dir() {
            collect_bundle_files(&path, out)?;
        } else if metadata.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn map_fs_read_error(source: std::io::Error) -> AssetLoadError {
    use std::io::ErrorKind;

    match source.kind() {
        ErrorKind::NotFound => AssetLoadError::NotFound,
        ErrorKind::PermissionDenied => AssetLoadError::AccessDenied,
        _ => AssetLoadError::Message {
            message: source.to_string().into(),
        },
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    use std::time::{SystemTime, UNIX_EPOCH};

    fn app_bundle() -> AssetBundleId {
        AssetBundleId::app("demo-app")
    }

    #[test]
    fn manifest_validation_rejects_duplicate_bundle_keys() {
        let manifest = FileAssetManifestV1::new([FileAssetManifestBundleV1::new(
            app_bundle(),
            [
                FileAssetManifestEntryV1::new("images/logo.png"),
                FileAssetManifestEntryV1::new("images/logo.png"),
            ],
        )]);

        assert!(matches!(
            manifest.validate(),
            Err(AssetManifestLoadError::DuplicateBundleKey { .. })
        ));
    }

    #[test]
    fn manifest_entries_default_to_key_as_file_path() {
        let manifest = FileAssetManifestV1::new([FileAssetManifestBundleV1::new(
            app_bundle(),
            [FileAssetManifestEntryV1::new("images/logo.png")],
        )
        .with_root("assets")]);
        manifest.validate().expect("manifest should validate");

        let bundle = &manifest.bundles[0];
        let entry = &bundle.entries[0];
        assert_eq!(entry.path, None);
        assert_eq!(bundle.root.as_deref(), Some(Path::new("assets")));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn file_manifest_resolver_loads_manifest_and_resolves_bundle_bytes() {
        let root = make_temp_dir("fret-assets-file-manifest");
        let assets_dir = root.join("assets").join("images");
        std::fs::create_dir_all(&assets_dir).expect("create assets dir");
        let logo_path = assets_dir.join("logo.txt");
        std::fs::write(&logo_path, b"hello-manifest").expect("write asset file");

        let manifest = FileAssetManifestV1::new([FileAssetManifestBundleV1::new(
            app_bundle(),
            [FileAssetManifestEntryV1::new("images/logo.png")
                .with_path("images/logo.txt")
                .with_media_type("text/plain")],
        )
        .with_root("assets")]);
        let manifest_path = root.join("assets.manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
        )
        .expect("write manifest");

        let resolver = FileAssetManifestResolver::from_manifest_path(&manifest_path)
            .expect("manifest resolver should load");
        let resolved = resolver
            .resolve_bytes(&AssetRequest::new(AssetLocator::bundle(
                app_bundle(),
                "images/logo.png",
            )))
            .expect("bundle asset should resolve");

        assert_eq!(resolver.entry_count(), 1);
        assert_eq!(resolver.manifest_path(), manifest_path.as_path());
        assert_eq!(resolved.bytes.as_ref(), b"hello-manifest");
        assert_eq!(
            resolved.media_type.as_ref().map(AssetMediaType::as_str),
            Some("text/plain")
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn file_manifest_resolver_uses_key_path_when_entry_path_is_omitted() {
        let root = make_temp_dir("fret-assets-file-manifest-key-default");
        let asset_dir = root.join("assets").join("icons");
        std::fs::create_dir_all(&asset_dir).expect("create icons dir");
        let icon_path = asset_dir.join("search.svg");
        std::fs::write(&icon_path, br#"<svg></svg>"#).expect("write icon file");

        let manifest = FileAssetManifestV1::new([FileAssetManifestBundleV1::new(
            app_bundle(),
            [FileAssetManifestEntryV1::new("icons/search.svg").with_media_type("image/svg+xml")],
        )
        .with_root("assets")]);
        let resolver = FileAssetManifestResolver::from_manifest_with_base_dir(
            manifest,
            &root,
            root.join("inline.manifest.json"),
        )
        .expect("manifest resolver should build");

        let resolved = resolver
            .resolve_bytes(&AssetRequest::new(AssetLocator::bundle(
                app_bundle(),
                "icons/search.svg",
            )))
            .expect("bundle asset should resolve");
        assert_eq!(resolved.bytes.as_ref(), br#"<svg></svg>"#);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn scan_dir_builds_entries_from_bundle_root() {
        let root = make_temp_dir("fret-assets-scan-dir");
        std::fs::create_dir_all(root.join("icons")).expect("create icons dir");
        std::fs::create_dir_all(root.join("images")).expect("create images dir");
        std::fs::write(root.join("icons/search.svg"), br#"<svg></svg>"#).expect("write svg");
        std::fs::write(root.join("images/logo.png"), b"png").expect("write png");

        let bundle = FileAssetManifestBundleV1::scan_dir(app_bundle(), &root)
            .expect("scan dir should build bundle");

        assert_eq!(bundle.root.as_deref(), Some(root.as_path()));
        assert_eq!(bundle.entries.len(), 2);
        assert_eq!(bundle.entries[0].key.as_str(), "icons/search.svg");
        assert_eq!(bundle.entries[1].key.as_str(), "images/logo.png");
        assert!(bundle.entries.iter().all(|entry| entry.path.is_none()));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn write_json_path_round_trips_generated_manifest() {
        let root = make_temp_dir("fret-assets-write-json");
        std::fs::create_dir_all(root.join("images")).expect("create images dir");
        std::fs::write(root.join("images/logo.png"), b"png").expect("write asset");

        let manifest =
            FileAssetManifestV1::new([FileAssetManifestBundleV1::scan_dir(app_bundle(), &root)
                .expect("scan dir should succeed")]);
        let manifest_path = root.join("out").join("assets.manifest.json");
        manifest
            .write_json_path(&manifest_path)
            .expect("write json should succeed");

        let loaded = FileAssetManifestV1::load_json_path(&manifest_path)
            .expect("written manifest should parse");
        assert_eq!(loaded, manifest);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn file_manifest_resolver_can_build_directly_from_bundle_dir() {
        let root = make_temp_dir("fret-assets-bundle-dir-resolver");
        std::fs::create_dir_all(root.join("images")).expect("create images dir");
        std::fs::write(root.join("images/logo.png"), b"bundle-dir").expect("write asset");

        let resolver = FileAssetManifestResolver::from_bundle_dir(app_bundle(), &root)
            .expect("bundle dir should build resolver");
        let resolved = resolver
            .resolve_bytes(&AssetRequest::new(AssetLocator::bundle(
                app_bundle(),
                "images/logo.png",
            )))
            .expect("bundle dir asset should resolve");

        assert_eq!(resolved.bytes.as_ref(), b"bundle-dir");
        assert_eq!(resolver.entry_count(), 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn make_temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }
}

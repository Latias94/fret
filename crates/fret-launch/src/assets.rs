use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub use fret_assets::FileAssetManifestResolver;
/// Explicit logical asset vocabulary and host registration helpers for `fret-launch` users.
pub use fret_assets::{
    AssetBundleId, AssetBundleNamespace, AssetCapabilities, AssetExternalReference, AssetKey,
    AssetKindHint, AssetLoadError, AssetLocator, AssetLocatorKind, AssetManifestLoadError,
    AssetMediaType, AssetMemoryKey, AssetRequest, AssetResolver, AssetRevision,
    FILE_ASSET_MANIFEST_KIND_V1, FileAssetManifestBundleV1, FileAssetManifestEntryV1,
    FileAssetManifestV1, ResolvedAssetBytes, ResolvedAssetReference, StaticAssetEntry,
    UrlPassthroughAssetResolver, asset_app_bundle_id, asset_package_bundle_id,
};
pub use fret_runtime::{
    AssetReloadBackendKind, AssetReloadEpoch, AssetReloadFallbackReason, AssetReloadStatus,
    AssetReloadSupport, AssetResolverService, asset_reload_epoch, asset_reload_status,
    asset_reload_support, bump_asset_reload_epoch,
};

/// Install or replace the primary resolver layer for the current host.
pub use fret_runtime::set_asset_resolver as set_primary_resolver;

/// Add an additional resolver layer without replacing earlier registrations.
pub use fret_runtime::register_asset_resolver as register_resolver;

pub(crate) fn ensure_default_url_passthrough_resolver(host: &mut impl fret_runtime::GlobalsHost) {
    if capabilities(host).is_some_and(|caps| caps.url) {
        return;
    }
    register_resolver(host, Arc::new(UrlPassthroughAssetResolver::new()));
}

/// Register static bundle-scoped entries on the current host.
pub use fret_runtime::register_bundle_asset_entries as register_bundle_entries;

/// Register static embedded entries owned by a specific bundle or crate.
pub use fret_runtime::register_embedded_asset_entries as register_embedded_entries;

/// Inspect the composed asset resolver service installed on the current host.
pub use fret_runtime::asset_resolver as resolver;

/// Report the current host's aggregated asset capabilities.
pub use fret_runtime::asset_capabilities as capabilities;

/// Selects which asset-publication lane a startup plan should apply.
///
/// `Development` keeps real-file manifest or bundle-directory mounts on the builder path.
/// `Packaged` keeps compile-time/static bundle or embedded entries on the builder path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetStartupMode {
    Development,
    Packaged,
}

impl AssetStartupMode {
    /// First-party default startup selection for launch/bootstrap-facing app startup.
    ///
    /// Native debug builds stay on the file-backed development lane for quick iteration, while
    /// packaged targets (including web/mobile and native release builds) stay on compiled bundle
    /// or embedded bytes.
    pub const fn preferred() -> Self {
        #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
        {
            Self::Development
        }
        #[cfg(not(all(not(target_arch = "wasm32"), debug_assertions)))]
        {
            Self::Packaged
        }
    }
}

/// Development asset-reload policy applied on top of file-backed startup mounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssetReloadPolicy {
    /// Poll native file metadata for builder-mounted manifests/directories and bump the shared
    /// reload epoch when their observed stamp set changes.
    PollMetadata { interval: Duration },
    /// Use a native filesystem watcher when available and fall back to metadata polling if the
    /// watcher backend cannot be installed for the current host or watch roots.
    NativeWatcher { fallback_poll_interval: Duration },
}

impl AssetReloadPolicy {
    pub const fn poll_metadata(interval: Duration) -> Self {
        Self::PollMetadata { interval }
    }

    pub const fn native_watcher(fallback_poll_interval: Duration) -> Self {
        Self::NativeWatcher {
            fallback_poll_interval,
        }
    }

    pub fn development_default() -> Self {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            Self::NativeWatcher {
                fallback_poll_interval: Duration::from_millis(250),
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Self::PollMetadata {
                interval: Duration::from_millis(250),
            }
        }
    }
}

// WASM/web currently carries the startup-plan shape for parity and builder reuse even though only
// native consumes these mount specs today.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Debug, Clone)]
enum AssetStartupBundleTarget {
    App,
    Explicit(AssetBundleId),
}

impl AssetStartupBundleTarget {
    #[cfg(not(target_arch = "wasm32"))]
    fn resolve(self, app_bundle: AssetBundleId) -> AssetBundleId {
        match self {
            Self::App => app_bundle,
            Self::Explicit(bundle) => bundle,
        }
    }
}

// WASM/web currently stores these specs without consuming them into runner mounts.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Debug, Clone)]
enum AssetStartupMountSpec {
    Manifest {
        path: PathBuf,
    },
    Dir {
        bundle: AssetStartupBundleTarget,
        dir: PathBuf,
    },
    BundleEntries {
        bundle: AssetStartupBundleTarget,
        entries: Vec<StaticAssetEntry>,
    },
    EmbeddedEntries {
        owner: AssetBundleId,
        entries: Vec<StaticAssetEntry>,
    },
}

/// Explicit startup plan that separates development asset mounts from packaged asset mounts.
#[derive(Debug, Clone, Default)]
pub struct AssetStartupPlan {
    development: Vec<AssetStartupMountSpec>,
    packaged: Vec<AssetStartupMountSpec>,
}

impl AssetStartupPlan {
    /// Create an empty startup plan.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a native/package-dev manifest artifact to the development lane.
    pub fn development_manifest(mut self, manifest_path: impl Into<PathBuf>) -> Self {
        self.development.push(AssetStartupMountSpec::Manifest {
            path: manifest_path.into(),
        });
        self
    }

    /// Add a native/package-dev directory scan under the default app bundle id.
    pub fn development_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.development.push(AssetStartupMountSpec::Dir {
            bundle: AssetStartupBundleTarget::App,
            dir: dir.into(),
        });
        self
    }

    /// Add a native/package-dev directory scan under an explicit bundle id.
    pub fn development_bundle_dir(
        mut self,
        bundle: impl Into<AssetBundleId>,
        dir: impl Into<PathBuf>,
    ) -> Self {
        self.development.push(AssetStartupMountSpec::Dir {
            bundle: AssetStartupBundleTarget::Explicit(bundle.into()),
            dir: dir.into(),
        });
        self
    }

    /// Add a development bundle-directory lane on native targets and no-op on wasm.
    pub fn development_bundle_dir_if_native(
        self,
        bundle: impl Into<AssetBundleId>,
        dir: impl Into<PathBuf>,
    ) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.development_bundle_dir(bundle, dir)
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = bundle.into();
            let _ = dir.into();
            self
        }
    }

    /// Add compile-time/static entries under the default app bundle id to the packaged lane.
    pub fn packaged_entries(mut self, entries: impl IntoIterator<Item = StaticAssetEntry>) -> Self {
        self.packaged.push(AssetStartupMountSpec::BundleEntries {
            bundle: AssetStartupBundleTarget::App,
            entries: entries.into_iter().collect(),
        });
        self
    }

    /// Add compile-time/static entries under an explicit bundle id to the packaged lane.
    pub fn packaged_bundle_entries(
        mut self,
        bundle: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) -> Self {
        self.packaged.push(AssetStartupMountSpec::BundleEntries {
            bundle: AssetStartupBundleTarget::Explicit(bundle.into()),
            entries: entries.into_iter().collect(),
        });
        self
    }

    /// Add owner-scoped embedded bytes to the packaged lane.
    pub fn packaged_embedded_entries(
        mut self,
        owner: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) -> Self {
        self.packaged.push(AssetStartupMountSpec::EmbeddedEntries {
            owner: owner.into(),
            entries: entries.into_iter().collect(),
        });
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn into_mounts(
        self,
        app_bundle: AssetBundleId,
        mode: AssetStartupMode,
    ) -> Result<Vec<AssetMount>, AssetStartupPlanError> {
        let mounts = match mode {
            AssetStartupMode::Development => self.development,
            AssetStartupMode::Packaged => self.packaged,
        };

        if mounts.is_empty() {
            return Err(match mode {
                AssetStartupMode::Development => AssetStartupPlanError::MissingDevelopmentLane,
                AssetStartupMode::Packaged => AssetStartupPlanError::MissingPackagedLane,
            });
        }

        Ok(mounts
            .into_iter()
            .map(|mount| mount.into_mount(app_bundle.clone()))
            .collect())
    }
}

impl AssetStartupMountSpec {
    #[cfg(not(target_arch = "wasm32"))]
    fn into_mount(self, app_bundle: AssetBundleId) -> AssetMount {
        match self {
            Self::Manifest { path } => AssetMount::Manifest { path },
            Self::Dir { bundle, dir } => AssetMount::Dir {
                bundle: bundle.resolve(app_bundle),
                dir,
            },
            Self::BundleEntries { bundle, entries } => AssetMount::BundleEntries {
                bundle: bundle.resolve(app_bundle),
                entries,
            },
            Self::EmbeddedEntries { owner, entries } => {
                AssetMount::EmbeddedEntries { owner, entries }
            }
        }
    }
}

/// Reported when a startup plan selects a lane that was never configured.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AssetStartupPlanError {
    #[error(
        "asset startup plan selected development mode but no development manifest/directory lane was configured"
    )]
    MissingDevelopmentLane,
    #[error(
        "asset startup plan selected packaged mode but no packaged bundle/embedded entries were configured"
    )]
    MissingPackagedLane,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub(crate) enum AssetMount {
    Dir {
        bundle: AssetBundleId,
        dir: PathBuf,
    },
    Manifest {
        path: PathBuf,
    },
    BundleEntries {
        bundle: AssetBundleId,
        entries: Vec<StaticAssetEntry>,
    },
    EmbeddedEntries {
        owner: AssetBundleId,
        entries: Vec<StaticAssetEntry>,
    },
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub(crate) enum AssetReloadTarget {
    Manifest { path: PathBuf },
    Dir { path: PathBuf },
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    use super::{AssetBundleId, AssetMount, AssetRevision, StaticAssetEntry};
    use super::{AssetStartupMode, AssetStartupPlan, ensure_default_url_passthrough_resolver};
    use fret_app::App;
    use fret_assets::{
        AssetCapabilities, AssetExternalReference, AssetLoadError, AssetLocator, AssetRequest,
        AssetResolver, ResolvedAssetBytes, ResolvedAssetReference,
    };
    #[cfg(not(target_arch = "wasm32"))]
    use std::path::PathBuf;
    use std::sync::Arc;
    #[cfg(not(target_arch = "wasm32"))]
    use std::sync::atomic::{AtomicU64, Ordering};

    #[cfg(not(target_arch = "wasm32"))]
    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    #[cfg(not(target_arch = "wasm32"))]
    fn make_temp_dir(tag: &str) -> PathBuf {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("fret-launch-{tag}-{}-{id}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn write_asset_dir_fixture(tag: &str) -> PathBuf {
        let dir = make_temp_dir(tag).join("assets");
        std::fs::create_dir_all(dir.join("images")).expect("create images dir");
        std::fs::write(dir.join("images/logo.png"), b"launch-bytes").expect("write asset");
        dir
    }

    #[test]
    fn asset_startup_mode_preferred_matches_current_target_defaults() {
        #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
        assert_eq!(AssetStartupMode::preferred(), AssetStartupMode::Development);

        #[cfg(not(all(not(target_arch = "wasm32"), debug_assertions)))]
        assert_eq!(AssetStartupMode::preferred(), AssetStartupMode::Packaged);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn asset_startup_plan_development_bundle_dir_if_native_populates_development_lane() {
        let asset_dir =
            write_asset_dir_fixture("asset-startup-plan-development-bundle-dir-if-native");
        let app_bundle = AssetBundleId::app("asset-startup-plan-development-bundle-dir-if-native");
        let plan = AssetStartupPlan::new()
            .packaged_entries([StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(1),
                b"builder-bytes",
            )])
            .development_bundle_dir_if_native(app_bundle.clone(), &asset_dir);

        let mounts = plan
            .into_mounts(app_bundle.clone(), AssetStartupMode::Development)
            .expect("native helper should populate the development lane");

        assert!(matches!(
            mounts.as_slice(),
            [AssetMount::Dir { bundle, dir }] if bundle == &app_bundle && dir == &asset_dir
        ));
    }

    #[test]
    fn ensure_default_url_passthrough_resolver_installs_url_capability_when_missing() {
        let mut app = App::new();
        ensure_default_url_passthrough_resolver(&mut app);

        assert_eq!(
            super::capabilities(&app),
            Some(AssetCapabilities {
                url: true,
                ..AssetCapabilities::default()
            })
        );

        let resolved = fret_runtime::resolve_asset_reference(
            &app,
            &AssetRequest::new(AssetLocator::url("https://example.com/logo.png")),
        )
        .expect("url passthrough resolver should resolve references");

        assert_eq!(
            resolved.reference,
            AssetExternalReference::url("https://example.com/logo.png")
        );
    }

    #[test]
    fn ensure_default_url_passthrough_resolver_respects_existing_url_layers() {
        struct ExistingUrlResolver;

        impl AssetResolver for ExistingUrlResolver {
            fn capabilities(&self) -> AssetCapabilities {
                AssetCapabilities {
                    url: true,
                    ..AssetCapabilities::default()
                }
            }

            fn resolve_bytes(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetBytes, AssetLoadError> {
                Err(AssetLoadError::ReferenceOnlyLocator {
                    kind: request.locator.kind(),
                })
            }

            fn resolve_reference(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetReference, AssetLoadError> {
                Ok(ResolvedAssetReference::new(
                    request.locator.clone(),
                    AssetRevision(99),
                    AssetExternalReference::url("https://example.com/custom.png"),
                ))
            }
        }

        let mut app = App::new();
        super::register_resolver(&mut app, Arc::new(ExistingUrlResolver));
        ensure_default_url_passthrough_resolver(&mut app);

        let resolved = fret_runtime::resolve_asset_reference(
            &app,
            &AssetRequest::new(AssetLocator::url("https://example.com/logo.png")),
        )
        .expect("existing url resolver should remain authoritative");

        assert_eq!(resolved.revision, AssetRevision(99));
        assert_eq!(
            resolved.reference,
            AssetExternalReference::url("https://example.com/custom.png")
        );
    }
}

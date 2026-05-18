use std::path::PathBuf;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub use fret_assets::FileAssetManifestResolver;
/// Explicit logical asset vocabulary and host registration helpers for `fret-bootstrap` users.
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

/// Register static bundle-scoped entries on the current host.
pub use fret_runtime::register_bundle_asset_entries as register_bundle_entries;

/// Register static embedded entries owned by a specific bundle or crate.
pub use fret_runtime::register_embedded_asset_entries as register_embedded_entries;

/// Inspect the composed asset resolver service installed on the current host.
pub use fret_runtime::asset_resolver as resolver;

/// Report the current host's aggregated asset capabilities.
pub use fret_runtime::asset_capabilities as capabilities;

/// Resolve bytes for a logical asset request through the host-installed resolver chain.
pub use fret_runtime::resolve_asset_bytes as resolve_bytes;

/// Resolve bytes for a single locator through the host-installed resolver chain.
pub use fret_runtime::resolve_asset_locator_bytes as resolve_locator;

/// Resolve an external file/URL reference for a logical asset request through the
/// host-installed resolver chain.
pub use fret_runtime::resolve_asset_reference as resolve_reference;

/// Resolve an external file/URL reference for a single locator through the host-installed resolver
/// chain.
pub use fret_runtime::resolve_asset_locator_reference as resolve_locator_reference;

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

    #[cfg(all(not(target_arch = "wasm32"), feature = "launch"))]
    pub(crate) fn into_launch(self) -> fret_launch::assets::AssetReloadPolicy {
        match self {
            Self::PollMetadata { interval } => {
                fret_launch::assets::AssetReloadPolicy::poll_metadata(interval)
            }
            Self::NativeWatcher {
                fallback_poll_interval,
            } => fret_launch::assets::AssetReloadPolicy::native_watcher(fallback_poll_interval),
        }
    }
}

#[cfg_attr(
    not(all(not(target_arch = "wasm32"), feature = "launch")),
    allow(dead_code)
)]
#[derive(Debug, Clone)]
enum AssetStartupBundleTarget {
    App,
    Explicit(AssetBundleId),
}

impl AssetStartupBundleTarget {
    #[cfg(all(not(target_arch = "wasm32"), feature = "launch"))]
    fn resolve(self, app_bundle: AssetBundleId) -> AssetBundleId {
        match self {
            Self::App => app_bundle,
            Self::Explicit(bundle) => bundle,
        }
    }
}

#[cfg_attr(
    not(all(not(target_arch = "wasm32"), feature = "launch")),
    allow(dead_code)
)]
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

    #[cfg(all(not(target_arch = "wasm32"), feature = "launch"))]
    pub(crate) fn into_launch(
        self,
        app_bundle: AssetBundleId,
        mode: AssetStartupMode,
    ) -> Result<fret_launch::assets::AssetStartupPlan, AssetStartupPlanError> {
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

        let mut plan = fret_launch::assets::AssetStartupPlan::new();
        for mount in mounts {
            plan = mount.apply_to_launch(plan, app_bundle.clone());
        }
        Ok(plan)
    }
}

impl AssetStartupMountSpec {
    #[cfg(all(not(target_arch = "wasm32"), feature = "launch"))]
    fn apply_to_launch(
        self,
        plan: fret_launch::assets::AssetStartupPlan,
        app_bundle: AssetBundleId,
    ) -> fret_launch::assets::AssetStartupPlan {
        match self {
            Self::Manifest { path } => plan.development_manifest(path),
            Self::Dir { bundle, dir } => {
                plan.development_bundle_dir(bundle.resolve(app_bundle), dir)
            }
            Self::BundleEntries { bundle, entries } => {
                plan.packaged_bundle_entries(bundle.resolve(app_bundle), entries)
            }
            Self::EmbeddedEntries { owner, entries } => {
                plan.packaged_embedded_entries(owner, entries)
            }
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "launch"))]
impl From<AssetStartupMode> for fret_launch::assets::AssetStartupMode {
    fn from(mode: AssetStartupMode) -> Self {
        match mode {
            AssetStartupMode::Development => Self::Development,
            AssetStartupMode::Packaged => Self::Packaged,
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

#[cfg(all(not(target_arch = "wasm32"), feature = "launch"))]
impl From<fret_launch::assets::AssetStartupPlanError> for AssetStartupPlanError {
    fn from(error: fret_launch::assets::AssetStartupPlanError) -> Self {
        match error {
            fret_launch::assets::AssetStartupPlanError::MissingDevelopmentLane => {
                Self::MissingDevelopmentLane
            }
            fret_launch::assets::AssetStartupPlanError::MissingPackagedLane => {
                Self::MissingPackagedLane
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AssetStartupMode, AssetStartupPlan};

    #[test]
    fn asset_startup_mode_preferred_matches_current_target_defaults() {
        #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
        assert_eq!(AssetStartupMode::preferred(), AssetStartupMode::Development);

        #[cfg(not(all(not(target_arch = "wasm32"), debug_assertions)))]
        assert_eq!(AssetStartupMode::preferred(), AssetStartupMode::Packaged);
    }

    #[test]
    fn startup_plan_is_available_without_launch_adapter() {
        let _plan = AssetStartupPlan::new()
            .development_manifest("assets.manifest.json")
            .development_dir("assets")
            .packaged_entries(std::iter::empty());
    }
}

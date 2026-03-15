use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};

use fret_assets::{
    AssetBundleId, AssetCapabilities, AssetLoadError, AssetLocator, AssetLocatorKind, AssetRequest,
    AssetResolver, AssetRevision, InMemoryAssetResolver, ResolvedAssetBytes,
    ResolvedAssetReference, StaticAssetEntry,
};

use crate::GlobalsHost;

const MAX_ASSET_LOAD_RECENT_EVENTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetLoadAccessKind {
    Bytes,
    ExternalReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetLoadOutcomeKind {
    Resolved,
    Missing,
    StaleManifest,
    UnsupportedLocatorKind,
    ExternalReferenceUnavailable,
    ResolverUnavailable,
    AccessDenied,
    Message,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetRevisionTransitionKind {
    Initial,
    Stable,
    Changed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetLoadDiagnosticEvent {
    pub access_kind: AssetLoadAccessKind,
    pub locator_kind: AssetLocatorKind,
    pub locator_debug: String,
    pub outcome_kind: AssetLoadOutcomeKind,
    pub revision: Option<AssetRevision>,
    pub previous_revision: Option<AssetRevision>,
    pub revision_transition: Option<AssetRevisionTransitionKind>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssetLoadDiagnosticsSnapshot {
    pub total_requests: u64,
    pub bytes_requests: u64,
    pub reference_requests: u64,
    pub missing_bundle_asset_requests: u64,
    pub stale_manifest_requests: u64,
    pub unsupported_file_requests: u64,
    pub unsupported_url_requests: u64,
    pub external_reference_unavailable_requests: u64,
    pub revision_change_requests: u64,
    pub recent: Vec<AssetLoadDiagnosticEvent>,
}

#[derive(Default)]
struct AssetLoadDiagnosticsState {
    snapshot: AssetLoadDiagnosticsSnapshot,
    recent: VecDeque<AssetLoadDiagnosticEvent>,
    last_seen_revisions: HashMap<AssetLocator, AssetRevision>,
}

#[derive(Default)]
struct AssetLoadDiagnosticsStore {
    state: Mutex<AssetLoadDiagnosticsState>,
}

trait AssetLoadResolvedMetadata {
    fn revision(&self) -> AssetRevision;
}

impl AssetLoadResolvedMetadata for ResolvedAssetBytes {
    fn revision(&self) -> AssetRevision {
        self.revision
    }
}

impl AssetLoadResolvedMetadata for ResolvedAssetReference {
    fn revision(&self) -> AssetRevision {
        self.revision
    }
}

impl AssetLoadDiagnosticsStore {
    fn snapshot(&self) -> AssetLoadDiagnosticsSnapshot {
        let state = self
            .state
            .lock()
            .expect("poisoned AssetLoadDiagnosticsStore state lock");
        let mut snapshot = state.snapshot.clone();
        snapshot.recent = state.recent.iter().cloned().collect();
        snapshot
    }

    fn record_bytes_result(
        &self,
        request: &AssetRequest,
        result: &Result<ResolvedAssetBytes, AssetLoadError>,
    ) {
        self.record_result(AssetLoadAccessKind::Bytes, &request.locator, result);
    }

    fn record_reference_result(
        &self,
        request: &AssetRequest,
        result: &Result<ResolvedAssetReference, AssetLoadError>,
    ) {
        self.record_result(
            AssetLoadAccessKind::ExternalReference,
            &request.locator,
            result,
        );
    }

    fn record_result<T>(
        &self,
        access_kind: AssetLoadAccessKind,
        locator: &AssetLocator,
        result: &Result<T, AssetLoadError>,
    ) where
        T: AssetLoadResolvedMetadata,
    {
        let mut state = self
            .state
            .lock()
            .expect("poisoned AssetLoadDiagnosticsStore state lock");
        state.snapshot.total_requests = state.snapshot.total_requests.saturating_add(1);
        match access_kind {
            AssetLoadAccessKind::Bytes => {
                state.snapshot.bytes_requests = state.snapshot.bytes_requests.saturating_add(1);
            }
            AssetLoadAccessKind::ExternalReference => {
                state.snapshot.reference_requests =
                    state.snapshot.reference_requests.saturating_add(1);
            }
        }

        let (outcome_kind, revision, previous_revision, revision_transition, message) = match result
        {
            Ok(resolved) => {
                let revision = resolved.revision();
                let previous_revision = state.last_seen_revisions.insert(locator.clone(), revision);
                let revision_transition = match previous_revision {
                    None => Some(AssetRevisionTransitionKind::Initial),
                    Some(prev) if prev == revision => Some(AssetRevisionTransitionKind::Stable),
                    Some(_) => {
                        state.snapshot.revision_change_requests =
                            state.snapshot.revision_change_requests.saturating_add(1);
                        Some(AssetRevisionTransitionKind::Changed)
                    }
                };
                (
                    AssetLoadOutcomeKind::Resolved,
                    Some(revision),
                    previous_revision,
                    revision_transition,
                    None,
                )
            }
            Err(err) => {
                let outcome_kind = match err {
                    AssetLoadError::NotFound => AssetLoadOutcomeKind::Missing,
                    AssetLoadError::StaleManifestMapping { .. } => {
                        state.snapshot.stale_manifest_requests =
                            state.snapshot.stale_manifest_requests.saturating_add(1);
                        AssetLoadOutcomeKind::StaleManifest
                    }
                    AssetLoadError::UnsupportedLocatorKind { kind } => {
                        match kind {
                            AssetLocatorKind::File => {
                                state.snapshot.unsupported_file_requests =
                                    state.snapshot.unsupported_file_requests.saturating_add(1);
                            }
                            AssetLocatorKind::Url => {
                                state.snapshot.unsupported_url_requests =
                                    state.snapshot.unsupported_url_requests.saturating_add(1);
                            }
                            _ => {}
                        }
                        AssetLoadOutcomeKind::UnsupportedLocatorKind
                    }
                    AssetLoadError::ExternalReferenceUnavailable { .. } => {
                        state.snapshot.external_reference_unavailable_requests = state
                            .snapshot
                            .external_reference_unavailable_requests
                            .saturating_add(1);
                        AssetLoadOutcomeKind::ExternalReferenceUnavailable
                    }
                    AssetLoadError::ResolverUnavailable => {
                        AssetLoadOutcomeKind::ResolverUnavailable
                    }
                    AssetLoadError::AccessDenied => AssetLoadOutcomeKind::AccessDenied,
                    AssetLoadError::Message { .. } => AssetLoadOutcomeKind::Message,
                };

                if matches!(err, AssetLoadError::NotFound)
                    && locator.kind() == AssetLocatorKind::BundleAsset
                {
                    state.snapshot.missing_bundle_asset_requests = state
                        .snapshot
                        .missing_bundle_asset_requests
                        .saturating_add(1);
                }

                (
                    outcome_kind,
                    None,
                    state.last_seen_revisions.get(locator).copied(),
                    None,
                    match err {
                        AssetLoadError::StaleManifestMapping { path } => Some(path.to_string()),
                        AssetLoadError::Message { message } => Some(message.to_string()),
                        _ => None,
                    },
                )
            }
        };

        push_recent_event(
            &mut state.recent,
            AssetLoadDiagnosticEvent {
                access_kind,
                locator_kind: locator.kind(),
                locator_debug: debug_asset_locator(locator),
                outcome_kind,
                revision,
                previous_revision,
                revision_transition,
                message,
            },
        );
    }
}

fn push_recent_event(
    recent: &mut VecDeque<AssetLoadDiagnosticEvent>,
    event: AssetLoadDiagnosticEvent,
) {
    if recent.len() >= MAX_ASSET_LOAD_RECENT_EVENTS {
        let _ = recent.pop_front();
    }
    recent.push_back(event);
}

fn debug_asset_locator(locator: &AssetLocator) -> String {
    match locator {
        AssetLocator::Memory(key) => format!("memory:{}", key.as_str()),
        AssetLocator::Embedded(locator) => {
            format!(
                "embedded:{}:{}",
                locator.owner.as_str(),
                locator.key.as_str()
            )
        }
        AssetLocator::BundleAsset(locator) => {
            format!(
                "bundle:{}:{}",
                locator.bundle.as_str(),
                locator.key.as_str()
            )
        }
        AssetLocator::File(locator) => format!("file:{}", locator.path.to_string_lossy()),
        AssetLocator::Url(locator) => format!("url:{}", locator.as_str()),
    }
}

struct AssetResolverServiceState {
    layers: RwLock<Vec<AssetResolverLayer>>,
    diagnostics: AssetLoadDiagnosticsStore,
}

#[derive(Clone)]
enum AssetResolverLayer {
    Primary(Arc<dyn AssetResolver>),
    Registered(Arc<dyn AssetResolver>),
}

impl AssetResolverLayer {
    fn resolver(&self) -> &Arc<dyn AssetResolver> {
        match self {
            Self::Primary(resolver) | Self::Registered(resolver) => resolver,
        }
    }
}

impl Default for AssetResolverServiceState {
    fn default() -> Self {
        Self {
            layers: RwLock::default(),
            diagnostics: AssetLoadDiagnosticsStore::default(),
        }
    }
}

#[derive(Clone)]
pub struct AssetResolverService {
    state: Arc<AssetResolverServiceState>,
}

impl AssetResolverService {
    pub fn new(resolver: Arc<dyn AssetResolver>) -> Self {
        let service = Self::default();
        service.set_primary_resolver(resolver);
        service
    }

    pub fn primary_resolver(&self) -> Option<Arc<dyn AssetResolver>> {
        self.state
            .layers
            .read()
            .expect("poisoned AssetResolverService layers lock")
            .iter()
            .find_map(|layer| match layer {
                AssetResolverLayer::Primary(resolver) => Some(resolver.clone()),
                AssetResolverLayer::Registered(_) => None,
            })
    }

    pub fn layered_resolvers(&self) -> Vec<Arc<dyn AssetResolver>> {
        self.state
            .layers
            .read()
            .expect("poisoned AssetResolverService layers lock")
            .iter()
            .filter_map(|layer| match layer {
                AssetResolverLayer::Primary(_) => None,
                AssetResolverLayer::Registered(resolver) => Some(resolver.clone()),
            })
            .collect()
    }

    pub fn set_primary_resolver(&self, resolver: Arc<dyn AssetResolver>) {
        let mut layers = self
            .state
            .layers
            .write()
            .expect("poisoned AssetResolverService layers lock");
        if let Some(layer) = layers
            .iter_mut()
            .find(|layer| matches!(layer, AssetResolverLayer::Primary(_)))
        {
            *layer = AssetResolverLayer::Primary(resolver);
        } else {
            layers.push(AssetResolverLayer::Primary(resolver));
        }
    }

    pub fn register_resolver(&self, resolver: Arc<dyn AssetResolver>) {
        self.state
            .layers
            .write()
            .expect("poisoned AssetResolverService layers lock")
            .push(AssetResolverLayer::Registered(resolver));
    }

    pub fn register_bundle_entries(
        &self,
        bundle: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) {
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_bundle_entries(bundle, entries);
        self.register_resolver(Arc::new(resolver));
    }

    pub fn register_embedded_entries(
        &self,
        owner: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) {
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_embedded_entries(owner, entries);
        self.register_resolver(Arc::new(resolver));
    }

    fn resolver_layers(&self) -> Vec<AssetResolverLayer> {
        self.state
            .layers
            .read()
            .expect("poisoned AssetResolverService layers lock")
            .clone()
    }

    pub fn capabilities(&self) -> AssetCapabilities {
        let mut caps = AssetCapabilities::default();

        for layer in self.resolver_layers() {
            union_capabilities(&mut caps, layer.resolver().capabilities());
        }
        caps
    }

    pub fn supports(&self, locator: &AssetLocator) -> bool {
        self.capabilities().supports(locator)
    }

    pub fn diagnostics_snapshot(&self) -> AssetLoadDiagnosticsSnapshot {
        self.state.diagnostics.snapshot()
    }

    pub fn resolve_bytes(
        &self,
        request: &AssetRequest,
    ) -> Result<ResolvedAssetBytes, AssetLoadError> {
        let mut saw_supported = false;

        for layer in self.resolver_layers().into_iter().rev() {
            let resolver = layer.resolver();
            match try_resolver_layer(resolver.as_ref(), request) {
                Ok(Some(resolved)) => {
                    let result = Ok(resolved);
                    self.state.diagnostics.record_bytes_result(request, &result);
                    return result;
                }
                Ok(None) => saw_supported |= resolver.supports(&request.locator),
                Err(err) => {
                    let result = Err(err);
                    self.state.diagnostics.record_bytes_result(request, &result);
                    return result;
                }
            }
        }

        let result = if saw_supported {
            Err(AssetLoadError::NotFound)
        } else {
            Err(AssetLoadError::UnsupportedLocatorKind {
                kind: request.locator.kind(),
            })
        };
        self.state.diagnostics.record_bytes_result(request, &result);
        result
    }

    pub fn resolve_locator_bytes(
        &self,
        locator: AssetLocator,
    ) -> Result<ResolvedAssetBytes, AssetLoadError> {
        self.resolve_bytes(&AssetRequest::new(locator))
    }

    pub fn resolve_reference(
        &self,
        request: &AssetRequest,
    ) -> Result<ResolvedAssetReference, AssetLoadError> {
        let mut saw_supported = false;

        for layer in self.resolver_layers().into_iter().rev() {
            let resolver = layer.resolver();
            match try_resolver_reference_layer(resolver.as_ref(), request) {
                Ok(Some(resolved)) => {
                    let result = Ok(resolved);
                    self.state
                        .diagnostics
                        .record_reference_result(request, &result);
                    return result;
                }
                Ok(None) => saw_supported |= resolver.supports(&request.locator),
                Err(err) => {
                    let result = Err(err);
                    self.state
                        .diagnostics
                        .record_reference_result(request, &result);
                    return result;
                }
            }
        }

        let result = if saw_supported {
            Err(AssetLoadError::NotFound)
        } else {
            Err(AssetLoadError::UnsupportedLocatorKind {
                kind: request.locator.kind(),
            })
        };
        self.state
            .diagnostics
            .record_reference_result(request, &result);
        result
    }

    pub fn resolve_locator_reference(
        &self,
        locator: AssetLocator,
    ) -> Result<ResolvedAssetReference, AssetLoadError> {
        self.resolve_reference(&AssetRequest::new(locator))
    }
}

impl fmt::Debug for AssetResolverService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetResolverService")
            .field("capabilities", &self.capabilities())
            .field("has_primary", &self.primary_resolver().is_some())
            .field("layered_resolvers", &self.layered_resolvers().len())
            .finish_non_exhaustive()
    }
}

impl Default for AssetResolverService {
    fn default() -> Self {
        Self {
            state: Arc::new(AssetResolverServiceState::default()),
        }
    }
}

impl From<Arc<dyn AssetResolver>> for AssetResolverService {
    fn from(resolver: Arc<dyn AssetResolver>) -> Self {
        Self::new(resolver)
    }
}

pub fn set_asset_resolver(host: &mut impl GlobalsHost, resolver: Arc<dyn AssetResolver>) {
    host.with_global_mut(AssetResolverService::default, |service, _host| {
        service.set_primary_resolver(resolver);
    });
}

pub fn register_asset_resolver(host: &mut impl GlobalsHost, resolver: Arc<dyn AssetResolver>) {
    host.with_global_mut(AssetResolverService::default, |service, _host| {
        service.register_resolver(resolver);
    });
}

pub fn register_bundle_asset_entries(
    host: &mut impl GlobalsHost,
    bundle: impl Into<AssetBundleId>,
    entries: impl IntoIterator<Item = StaticAssetEntry>,
) {
    let bundle = bundle.into();
    let entries = entries.into_iter().collect::<Vec<_>>();
    host.with_global_mut(AssetResolverService::default, move |service, _host| {
        service.register_bundle_entries(bundle, entries);
    });
}

pub fn register_embedded_asset_entries(
    host: &mut impl GlobalsHost,
    owner: impl Into<AssetBundleId>,
    entries: impl IntoIterator<Item = StaticAssetEntry>,
) {
    let owner = owner.into();
    let entries = entries.into_iter().collect::<Vec<_>>();
    host.with_global_mut(AssetResolverService::default, move |service, _host| {
        service.register_embedded_entries(owner, entries);
    });
}

pub fn asset_resolver(host: &impl GlobalsHost) -> Option<&AssetResolverService> {
    host.global::<AssetResolverService>()
}

pub fn asset_capabilities(host: &impl GlobalsHost) -> Option<AssetCapabilities> {
    asset_resolver(host).map(AssetResolverService::capabilities)
}

pub fn resolve_asset_bytes(
    host: &impl GlobalsHost,
    request: &AssetRequest,
) -> Result<ResolvedAssetBytes, AssetLoadError> {
    asset_resolver(host)
        .ok_or(AssetLoadError::ResolverUnavailable)?
        .resolve_bytes(request)
}

pub fn resolve_asset_locator_bytes(
    host: &impl GlobalsHost,
    locator: AssetLocator,
) -> Result<ResolvedAssetBytes, AssetLoadError> {
    resolve_asset_bytes(host, &AssetRequest::new(locator))
}

pub fn resolve_asset_reference(
    host: &impl GlobalsHost,
    request: &AssetRequest,
) -> Result<ResolvedAssetReference, AssetLoadError> {
    asset_resolver(host)
        .ok_or(AssetLoadError::ResolverUnavailable)?
        .resolve_reference(request)
}

pub fn resolve_asset_locator_reference(
    host: &impl GlobalsHost,
    locator: AssetLocator,
) -> Result<ResolvedAssetReference, AssetLoadError> {
    resolve_asset_reference(host, &AssetRequest::new(locator))
}

fn union_capabilities(dst: &mut AssetCapabilities, src: AssetCapabilities) {
    dst.memory |= src.memory;
    dst.embedded |= src.embedded;
    dst.bundle_asset |= src.bundle_asset;
    dst.file |= src.file;
    dst.url |= src.url;
    dst.file_watch |= src.file_watch;
    dst.system_font_scan |= src.system_font_scan;
}

fn try_resolver_layer(
    resolver: &dyn AssetResolver,
    request: &AssetRequest,
) -> Result<Option<ResolvedAssetBytes>, AssetLoadError> {
    if !resolver.supports(&request.locator) {
        return Ok(None);
    }

    match resolver.resolve_bytes(request) {
        Ok(resolved) => Ok(Some(resolved)),
        Err(AssetLoadError::NotFound) => Ok(None),
        Err(AssetLoadError::UnsupportedLocatorKind { .. }) => Ok(None),
        Err(err) => Err(err),
    }
}

fn try_resolver_reference_layer(
    resolver: &dyn AssetResolver,
    request: &AssetRequest,
) -> Result<Option<ResolvedAssetReference>, AssetLoadError> {
    if !resolver.supports(&request.locator) {
        return Ok(None);
    }

    match resolver.resolve_reference(request) {
        Ok(resolved) => Ok(Some(resolved)),
        Err(AssetLoadError::NotFound) => Ok(None),
        Err(AssetLoadError::UnsupportedLocatorKind { .. }) => Ok(None),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_assets::{AssetLocator, AssetRevision, InMemoryAssetResolver};

    use super::*;

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
    }

    impl GlobalsHost for TestHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals.get(&TypeId::of::<T>())?.downcast_ref::<T>()
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let mut value = match self.globals.remove(&type_id) {
                None => init(),
                Some(value) => *value.downcast::<T>().expect("global type id must match"),
            };
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    #[test]
    fn resolve_asset_bytes_requires_installed_service() {
        let host = TestHost::default();
        let err =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect_err("missing service should fail");

        assert_eq!(err, AssetLoadError::ResolverUnavailable);
    }

    #[test]
    fn installed_service_resolves_bundle_assets() {
        let mut host = TestHost::default();
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_bundle("app", "images/logo.png", AssetRevision(7), [1u8, 2, 3]);
        set_asset_resolver(&mut host, Arc::new(resolver));

        let caps = asset_capabilities(&host).expect("resolver caps should exist");
        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("bundle asset should resolve");

        assert!(caps.bundle_asset);
        assert_eq!(resolved.revision, AssetRevision(7));
        assert_eq!(resolved.bytes.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn diagnostics_snapshot_records_initial_stable_and_changed_revisions() {
        let mut host = TestHost::default();
        register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(1),
                b"v1",
            )],
        );

        let _ = resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
            .expect("first bundle resolution should succeed");
        let _ = resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
            .expect("second bundle resolution should succeed");
        register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(9),
                b"v9",
            )],
        );
        let _ = resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
            .expect("updated bundle resolution should succeed");

        let snapshot = asset_resolver(&host)
            .expect("resolver service")
            .diagnostics_snapshot();

        assert_eq!(snapshot.total_requests, 3);
        assert_eq!(snapshot.bytes_requests, 3);
        assert_eq!(snapshot.revision_change_requests, 1);
        assert_eq!(snapshot.recent.len(), 3);
        assert_eq!(
            snapshot.recent[0].revision_transition,
            Some(AssetRevisionTransitionKind::Initial)
        );
        assert_eq!(
            snapshot.recent[1].revision_transition,
            Some(AssetRevisionTransitionKind::Stable)
        );
        assert_eq!(
            snapshot.recent[2].revision_transition,
            Some(AssetRevisionTransitionKind::Changed)
        );
        assert_eq!(snapshot.recent[2].previous_revision, Some(AssetRevision(1)));
        assert_eq!(snapshot.recent[2].revision, Some(AssetRevision(9)));
    }

    #[test]
    fn diagnostics_snapshot_counts_missing_bundle_assets() {
        let mut host = TestHost::default();
        let resolver = InMemoryAssetResolver::new().with_capabilities(AssetCapabilities {
            memory: false,
            embedded: false,
            bundle_asset: true,
            file: false,
            url: false,
            file_watch: false,
            system_font_scan: false,
        });
        set_asset_resolver(&mut host, Arc::new(resolver));

        let err =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/missing.png"))
                .expect_err("missing bundle asset should fail");

        assert_eq!(err, AssetLoadError::NotFound);
        let snapshot = asset_resolver(&host)
            .expect("resolver service")
            .diagnostics_snapshot();
        assert_eq!(snapshot.missing_bundle_asset_requests, 1);
        assert_eq!(snapshot.recent.len(), 1);
        assert_eq!(
            snapshot.recent[0].outcome_kind,
            AssetLoadOutcomeKind::Missing
        );
        assert_eq!(
            snapshot.recent[0].locator_kind,
            AssetLocatorKind::BundleAsset
        );
    }

    #[test]
    fn diagnostics_snapshot_counts_unsupported_file_capability_requests() {
        let mut host = TestHost::default();
        register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(1),
                b"png",
            )],
        );

        let err = resolve_asset_locator_bytes(&host, AssetLocator::file("assets/logo.png"))
            .expect_err("unsupported file locator should fail");

        assert_eq!(
            err,
            AssetLoadError::UnsupportedLocatorKind {
                kind: AssetLocatorKind::File,
            }
        );
        let snapshot = asset_resolver(&host)
            .expect("resolver service")
            .diagnostics_snapshot();
        assert_eq!(snapshot.unsupported_file_requests, 1);
        assert_eq!(
            snapshot.recent[0].outcome_kind,
            AssetLoadOutcomeKind::UnsupportedLocatorKind
        );
        assert_eq!(snapshot.recent[0].locator_kind, AssetLocatorKind::File);
    }

    #[test]
    fn diagnostics_snapshot_counts_external_reference_unavailable_requests() {
        let mut host = TestHost::default();
        register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "icons/search.svg",
                AssetRevision(2),
                br#"<svg viewBox="0 0 1 1"></svg>"#,
            )],
        );

        let err =
            resolve_asset_locator_reference(&host, AssetLocator::bundle("app", "icons/search.svg"))
                .expect_err("byte-only bundle asset should not resolve to external reference");

        assert_eq!(
            err,
            AssetLoadError::ExternalReferenceUnavailable {
                kind: AssetLocatorKind::BundleAsset,
            }
        );
        let snapshot = asset_resolver(&host)
            .expect("resolver service")
            .diagnostics_snapshot();
        assert_eq!(snapshot.reference_requests, 1);
        assert_eq!(snapshot.external_reference_unavailable_requests, 1);
        assert_eq!(
            snapshot.recent[0].access_kind,
            AssetLoadAccessKind::ExternalReference
        );
        assert_eq!(
            snapshot.recent[0].outcome_kind,
            AssetLoadOutcomeKind::ExternalReferenceUnavailable
        );
    }

    #[test]
    fn asset_capabilities_union_optional_escape_hatches_across_layers() {
        let mut host = TestHost::default();
        let primary = InMemoryAssetResolver::new().with_capabilities(AssetCapabilities {
            memory: false,
            embedded: false,
            bundle_asset: false,
            file: true,
            url: false,
            file_watch: true,
            system_font_scan: false,
        });
        let secondary = InMemoryAssetResolver::new().with_capabilities(AssetCapabilities {
            memory: false,
            embedded: false,
            bundle_asset: false,
            file: false,
            url: true,
            file_watch: false,
            system_font_scan: true,
        });

        set_asset_resolver(&mut host, Arc::new(primary));
        register_asset_resolver(&mut host, Arc::new(secondary));

        assert_eq!(
            asset_capabilities(&host).expect("resolver caps should exist"),
            AssetCapabilities {
                memory: false,
                embedded: false,
                bundle_asset: false,
                file: true,
                url: true,
                file_watch: true,
                system_font_scan: true,
            }
        );
    }

    #[test]
    fn unsupported_file_locator_kind_stays_unsupported_even_when_other_locators_exist() {
        let mut host = TestHost::default();
        register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(1),
                b"png",
            )],
        );

        let err = resolve_asset_locator_bytes(&host, AssetLocator::file("assets/logo.png"))
            .expect_err("unsupported file locator should not be downgraded to not-found");

        assert_eq!(
            err,
            AssetLoadError::UnsupportedLocatorKind {
                kind: fret_assets::AssetLocatorKind::File,
            }
        );
    }

    #[test]
    fn supported_but_missing_file_locator_returns_not_found() {
        let mut host = TestHost::default();
        let resolver = InMemoryAssetResolver::new().with_capabilities(AssetCapabilities {
            memory: false,
            embedded: false,
            bundle_asset: false,
            file: true,
            url: false,
            file_watch: true,
            system_font_scan: false,
        });
        set_asset_resolver(&mut host, Arc::new(resolver));

        let err = resolve_asset_locator_bytes(&host, AssetLocator::file("assets/missing.png"))
            .expect_err("supported but missing file locator should report not-found");

        assert_eq!(err, AssetLoadError::NotFound);
    }

    #[test]
    fn register_bundle_asset_entries_adds_composable_static_assets() {
        let mut host = TestHost::default();
        register_bundle_asset_entries(
            &mut host,
            "app",
            [
                StaticAssetEntry::new("images/logo.png", AssetRevision(2), b"png")
                    .with_media_type("image/png"),
            ],
        );

        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("bundle asset should resolve");

        assert_eq!(resolved.revision, AssetRevision(2));
        assert_eq!(
            resolved.media_type.as_ref().map(|v| v.as_str()),
            Some("image/png")
        );
    }

    #[test]
    fn register_embedded_asset_entries_adds_namespaced_assets() {
        let mut host = TestHost::default();
        register_embedded_asset_entries(
            &mut host,
            "fret-ui-shadcn",
            [StaticAssetEntry::new(
                "icons/search.svg",
                AssetRevision(5),
                br#"<svg viewBox="0 0 1 1"></svg>"#,
            )
            .with_media_type("image/svg+xml")],
        );

        let resolved = resolve_asset_locator_bytes(
            &host,
            AssetLocator::embedded("fret-ui-shadcn", "icons/search.svg"),
        )
        .expect("embedded asset should resolve");

        assert_eq!(resolved.revision, AssetRevision(5));
        assert_eq!(
            resolved.media_type.as_ref().map(|v| v.as_str()),
            Some("image/svg+xml")
        );
    }

    #[test]
    fn resolve_asset_reference_requires_installed_service() {
        let host = TestHost::default();
        let err = resolve_asset_locator_reference(&host, AssetLocator::bundle("app", "logo.png"))
            .expect_err("missing service should fail");

        assert_eq!(err, AssetLoadError::ResolverUnavailable);
    }

    #[test]
    fn layered_resolvers_preserve_existing_sources() {
        let mut host = TestHost::default();
        register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(1),
                b"png",
            )],
        );

        let mut embedded = InMemoryAssetResolver::new();
        embedded.insert_embedded(
            "fret-ui-shadcn",
            "icons/search.svg",
            AssetRevision(4),
            [9u8, 8, 7],
        );
        register_asset_resolver(&mut host, Arc::new(embedded));

        let bundle =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("bundle asset should resolve");
        let embedded = resolve_asset_locator_bytes(
            &host,
            AssetLocator::embedded("fret-ui-shadcn", "icons/search.svg"),
        )
        .expect("embedded asset should resolve");

        assert_eq!(bundle.revision, AssetRevision(1));
        assert_eq!(embedded.revision, AssetRevision(4));
    }

    #[test]
    fn later_missing_bundle_layer_falls_back_to_earlier_bundle_asset() {
        let mut host = TestHost::default();

        let mut earlier = InMemoryAssetResolver::new();
        earlier.insert_bundle("app", "images/logo.png", AssetRevision(1), [1u8, 2, 3]);
        register_asset_resolver(&mut host, Arc::new(earlier));

        let later = InMemoryAssetResolver::new().with_capabilities(AssetCapabilities {
            memory: false,
            embedded: false,
            bundle_asset: true,
            file: false,
            url: false,
            file_watch: false,
            system_font_scan: false,
        });
        register_asset_resolver(&mut host, Arc::new(later));

        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("later missing layer should fall back to earlier bytes");

        assert_eq!(resolved.revision, AssetRevision(1));
        assert_eq!(resolved.bytes.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn later_missing_reference_layer_falls_back_to_earlier_reference_handoff() {
        let mut host = TestHost::default();

        struct FileReferenceResolver;

        impl AssetResolver for FileReferenceResolver {
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

            fn resolve_bytes(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetBytes, AssetLoadError> {
                Ok(ResolvedAssetBytes::new(
                    request.locator.clone(),
                    AssetRevision(1),
                    b"earlier".as_slice(),
                ))
            }

            fn resolve_reference(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetReference, AssetLoadError> {
                Ok(ResolvedAssetReference::new(
                    request.locator.clone(),
                    AssetRevision(1),
                    fret_assets::AssetExternalReference::file_path("assets/earlier.png"),
                ))
            }
        }

        register_asset_resolver(&mut host, Arc::new(FileReferenceResolver));
        register_asset_resolver(
            &mut host,
            Arc::new(
                InMemoryAssetResolver::new().with_capabilities(AssetCapabilities {
                    memory: false,
                    embedded: false,
                    bundle_asset: true,
                    file: false,
                    url: false,
                    file_watch: false,
                    system_font_scan: false,
                }),
            ),
        );

        let resolved =
            resolve_asset_locator_reference(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("later missing reference layer should fall back to earlier handoff");

        assert_eq!(resolved.revision, AssetRevision(1));
        assert_eq!(
            resolved.reference,
            fret_assets::AssetExternalReference::file_path("assets/earlier.png")
        );
    }

    #[test]
    fn stale_manifest_layer_blocks_earlier_bundle_fallback_and_is_counted() {
        let mut host = TestHost::default();

        let mut earlier = InMemoryAssetResolver::new();
        earlier.insert_bundle("app", "images/logo.png", AssetRevision(1), [1u8, 2, 3]);
        register_asset_resolver(&mut host, Arc::new(earlier));

        struct StaleManifestResolver;

        impl AssetResolver for StaleManifestResolver {
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

            fn resolve_bytes(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetBytes, AssetLoadError> {
                Err(AssetLoadError::StaleManifestMapping {
                    path: format!("/tmp/stale/{}", debug_asset_locator(&request.locator)).into(),
                })
            }
        }

        register_asset_resolver(&mut host, Arc::new(StaleManifestResolver));

        let err =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect_err("stale manifest layer should block fallback");

        assert!(matches!(err, AssetLoadError::StaleManifestMapping { .. }));
        let snapshot = asset_resolver(&host)
            .expect("resolver service")
            .diagnostics_snapshot();
        assert_eq!(snapshot.stale_manifest_requests, 1);
        assert_eq!(snapshot.missing_bundle_asset_requests, 0);
        assert_eq!(
            snapshot.recent[0].outcome_kind,
            AssetLoadOutcomeKind::StaleManifest
        );
    }

    #[test]
    fn later_layer_without_reference_blocks_earlier_reference_handoff() {
        let mut host = TestHost::default();

        struct FileReferenceResolver;

        impl AssetResolver for FileReferenceResolver {
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

            fn resolve_bytes(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetBytes, AssetLoadError> {
                Ok(ResolvedAssetBytes::new(
                    request.locator.clone(),
                    AssetRevision(1),
                    b"earlier".as_slice(),
                ))
            }

            fn resolve_reference(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetReference, AssetLoadError> {
                Ok(ResolvedAssetReference::new(
                    request.locator.clone(),
                    AssetRevision(1),
                    fret_assets::AssetExternalReference::file_path("assets/earlier.png"),
                ))
            }
        }

        register_asset_resolver(&mut host, Arc::new(FileReferenceResolver));
        register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(9),
                b"override",
            )],
        );

        let err =
            resolve_asset_locator_reference(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect_err("later static entry should shadow earlier file reference");

        assert_eq!(
            err,
            AssetLoadError::ExternalReferenceUnavailable {
                kind: fret_assets::AssetLocatorKind::BundleAsset,
            }
        );
    }

    #[test]
    fn later_layered_resolvers_override_earlier_layers_for_the_same_locator() {
        let mut host = TestHost::default();

        let mut earlier = InMemoryAssetResolver::new();
        earlier.insert_bundle("app", "images/logo.png", AssetRevision(1), [1u8, 2, 3]);
        register_asset_resolver(&mut host, Arc::new(earlier));

        let mut later = InMemoryAssetResolver::new();
        later.insert_bundle("app", "images/logo.png", AssetRevision(9), [9u8, 8, 7]);
        register_asset_resolver(&mut host, Arc::new(later));

        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("later layered resolver should win");

        assert_eq!(resolved.revision, AssetRevision(9));
        assert_eq!(resolved.bytes.as_ref(), &[9, 8, 7]);
    }

    #[test]
    fn later_static_entry_layers_override_earlier_resolver_layers_for_the_same_locator() {
        let mut host = TestHost::default();

        let mut earlier = InMemoryAssetResolver::new();
        earlier.insert_bundle("app", "images/logo.png", AssetRevision(1), [1u8, 2, 3]);
        register_asset_resolver(&mut host, Arc::new(earlier));

        register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(9),
                b"override",
            )],
        );

        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("later static entry layer should win");

        assert_eq!(resolved.revision, AssetRevision(9));
        assert_eq!(resolved.bytes.as_ref(), b"override");
    }

    #[test]
    fn later_resolver_layers_override_earlier_static_entry_layers_for_the_same_locator() {
        let mut host = TestHost::default();

        register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(1),
                b"earlier",
            )],
        );

        let mut later = InMemoryAssetResolver::new();
        later.insert_bundle("app", "images/logo.png", AssetRevision(9), [9u8, 8, 7]);
        register_asset_resolver(&mut host, Arc::new(later));

        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("later resolver layer should win");

        assert_eq!(resolved.revision, AssetRevision(9));
        assert_eq!(resolved.bytes.as_ref(), &[9, 8, 7]);
    }

    #[test]
    fn primary_resolver_replacement_keeps_its_existing_layer_position() {
        let mut host = TestHost::default();

        let mut earlier = InMemoryAssetResolver::new();
        earlier.insert_bundle("app", "images/logo.png", AssetRevision(1), [1u8, 2, 3]);
        register_asset_resolver(&mut host, Arc::new(earlier));

        let mut first_primary = InMemoryAssetResolver::new();
        first_primary.insert_bundle("app", "images/logo.png", AssetRevision(4), [4u8, 4, 4]);
        set_asset_resolver(&mut host, Arc::new(first_primary));

        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("first primary should win when it is the latest registration");
        assert_eq!(resolved.revision, AssetRevision(4));

        let mut later = InMemoryAssetResolver::new();
        later.insert_bundle("app", "images/logo.png", AssetRevision(9), [9u8, 8, 7]);
        register_asset_resolver(&mut host, Arc::new(later));

        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("later layered resolver should win");
        assert_eq!(resolved.revision, AssetRevision(9));

        let mut replacement_primary = InMemoryAssetResolver::new();
        replacement_primary.insert_bundle("app", "images/logo.png", AssetRevision(7), [7u8, 7, 7]);
        set_asset_resolver(&mut host, Arc::new(replacement_primary));

        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("replacing primary should not jump ahead of later layers");

        assert_eq!(resolved.revision, AssetRevision(9));
        assert_eq!(resolved.bytes.as_ref(), &[9, 8, 7]);
    }
}

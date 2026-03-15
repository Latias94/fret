use std::fmt;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};

use fret_assets::{
    AssetBundleId, AssetCapabilities, AssetLoadError, AssetLocator, AssetRequest, AssetResolver,
    InMemoryAssetResolver, ResolvedAssetBytes, StaticAssetEntry,
};

use crate::GlobalsHost;

struct AssetResolverServiceState {
    primary: RwLock<Option<Arc<dyn AssetResolver>>>,
    layered: RwLock<Vec<Arc<dyn AssetResolver>>>,
    static_assets: Mutex<InMemoryAssetResolver>,
}

impl Default for AssetResolverServiceState {
    fn default() -> Self {
        Self {
            primary: RwLock::default(),
            layered: RwLock::default(),
            static_assets: Mutex::new(InMemoryAssetResolver::new()),
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
            .primary
            .read()
            .expect("poisoned AssetResolverService primary lock")
            .clone()
    }

    pub fn layered_resolvers(&self) -> Vec<Arc<dyn AssetResolver>> {
        self.state
            .layered
            .read()
            .expect("poisoned AssetResolverService layered lock")
            .clone()
    }

    pub fn set_primary_resolver(&self, resolver: Arc<dyn AssetResolver>) {
        *self
            .state
            .primary
            .write()
            .expect("poisoned AssetResolverService primary lock") = Some(resolver);
    }

    pub fn register_resolver(&self, resolver: Arc<dyn AssetResolver>) {
        self.state
            .layered
            .write()
            .expect("poisoned AssetResolverService layered lock")
            .push(resolver);
    }

    pub fn register_bundle_entries(
        &self,
        bundle: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) {
        self.state
            .static_assets
            .lock()
            .expect("poisoned AssetResolverService static-assets lock")
            .insert_bundle_entries(bundle, entries);
    }

    pub fn register_embedded_entries(
        &self,
        owner: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) {
        self.state
            .static_assets
            .lock()
            .expect("poisoned AssetResolverService static-assets lock")
            .insert_embedded_entries(owner, entries);
    }

    pub fn capabilities(&self) -> AssetCapabilities {
        let mut caps = self
            .state
            .static_assets
            .lock()
            .expect("poisoned AssetResolverService static-assets lock")
            .capabilities();

        if let Some(primary) = self.primary_resolver() {
            union_capabilities(&mut caps, primary.capabilities());
        }
        for resolver in self.layered_resolvers() {
            union_capabilities(&mut caps, resolver.capabilities());
        }
        caps
    }

    pub fn supports(&self, locator: &AssetLocator) -> bool {
        self.capabilities().supports(locator)
    }

    pub fn resolve_bytes(
        &self,
        request: &AssetRequest,
    ) -> Result<ResolvedAssetBytes, AssetLoadError> {
        if let Some(resolved) = self.try_static_assets(request)? {
            return Ok(resolved);
        }

        let mut saw_supported = false;

        if let Some(primary) = self.primary_resolver() {
            match try_resolver_layer(primary.as_ref(), request) {
                Ok(Some(resolved)) => return Ok(resolved),
                Ok(None) => saw_supported |= primary.supports(&request.locator),
                Err(err) => return Err(err),
            }
        }

        for resolver in self.layered_resolvers() {
            match try_resolver_layer(resolver.as_ref(), request) {
                Ok(Some(resolved)) => return Ok(resolved),
                Ok(None) => saw_supported |= resolver.supports(&request.locator),
                Err(err) => return Err(err),
            }
        }

        if saw_supported || self.supports(&request.locator) {
            Err(AssetLoadError::NotFound)
        } else {
            Err(AssetLoadError::UnsupportedLocatorKind {
                kind: request.locator.kind(),
            })
        }
    }

    pub fn resolve_locator_bytes(
        &self,
        locator: AssetLocator,
    ) -> Result<ResolvedAssetBytes, AssetLoadError> {
        self.resolve_bytes(&AssetRequest::new(locator))
    }

    fn try_static_assets(
        &self,
        request: &AssetRequest,
    ) -> Result<Option<ResolvedAssetBytes>, AssetLoadError> {
        let static_assets = self
            .state
            .static_assets
            .lock()
            .expect("poisoned AssetResolverService static-assets lock");

        if !static_assets.capabilities().supports(&request.locator) {
            return Ok(None);
        }

        match static_assets.resolve_bytes(request) {
            Ok(resolved) => Ok(Some(resolved)),
            Err(AssetLoadError::NotFound) => Ok(None),
            Err(AssetLoadError::UnsupportedLocatorKind { .. }) => Ok(None),
            Err(err) => Err(err),
        }
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
}

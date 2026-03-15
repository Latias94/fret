use crate::ui_host::GlobalsHost;

/// Global epoch observed by asset-consuming code when logical assets should be reloaded.
///
/// This is intentionally generic: UI code, diagnostics, and future non-UI integrations should all
/// observe the same invalidation token instead of inventing asset-class-specific reload globals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AssetReloadEpoch(pub u64);

impl AssetReloadEpoch {
    pub fn bump(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

/// Host-level support flags for development asset reload automation.
///
/// `file_watch` means the current host/startup stack will automatically detect native file-backed
/// asset changes and publish new [`AssetReloadEpoch`] values without requiring an app-local manual
/// bump.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AssetReloadSupport {
    pub file_watch: bool,
}

pub fn asset_reload_epoch(host: &impl GlobalsHost) -> Option<AssetReloadEpoch> {
    host.global::<AssetReloadEpoch>().copied()
}

pub fn bump_asset_reload_epoch(host: &mut impl GlobalsHost) {
    host.with_global_mut(AssetReloadEpoch::default, |epoch, _host| {
        epoch.bump();
    });
}

pub fn asset_reload_support(host: &impl GlobalsHost) -> Option<AssetReloadSupport> {
    host.global::<AssetReloadSupport>().copied()
}

pub fn set_asset_reload_support(host: &mut impl GlobalsHost, support: AssetReloadSupport) {
    host.set_global(support);
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use super::{
        AssetReloadEpoch, AssetReloadSupport, asset_reload_epoch, asset_reload_support,
        bump_asset_reload_epoch, set_asset_reload_support,
    };
    use crate::ui_host::GlobalsHost;

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
                Some(v) => *v.downcast::<T>().expect("global type id must match"),
            };
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    #[test]
    fn bump_asset_reload_epoch_initializes_and_increments_global() {
        let mut host = TestHost::default();
        assert_eq!(asset_reload_epoch(&host), None);

        bump_asset_reload_epoch(&mut host);
        assert_eq!(asset_reload_epoch(&host), Some(AssetReloadEpoch(1)));

        bump_asset_reload_epoch(&mut host);
        assert_eq!(asset_reload_epoch(&host), Some(AssetReloadEpoch(2)));
    }

    #[test]
    fn set_asset_reload_support_publishes_host_support_flags() {
        let mut host = TestHost::default();
        assert_eq!(asset_reload_support(&host), None);

        set_asset_reload_support(&mut host, AssetReloadSupport { file_watch: true });
        assert_eq!(
            asset_reload_support(&host),
            Some(AssetReloadSupport { file_watch: true })
        );
    }
}

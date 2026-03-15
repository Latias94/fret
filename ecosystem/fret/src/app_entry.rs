//! `fret::FretApp` builder-chain entry points.
//!
//! This module provides an ergonomic, desktop-first entry surface (ecosystem-level) while
//! preserving the golden-path driver's hotpatch-friendly posture (function-pointer hooks).

use std::path::PathBuf;

use crate::{
    AssetMount, Defaults, Result, UiAppBuilder, UiAppDriver,
    assets::{AssetBundleId, StaticAssetEntry},
    integration::InstallIntoApp,
};

type AppSetupHook = Box<dyn FnOnce(&mut crate::app::App)>;

/// Builder-chain facade for creating and running a desktop-first Fret UI app.
///
/// Notes:
/// - This is an ecosystem-level convenience layer (not a kernel contract).
/// - The builder composes existing `fret` entry points (the view/runtime + driver wiring) and
///   applies a default main window if none is configured.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct FretApp {
    root_name: &'static str,
    main_window: Option<(String, (f64, f64))>,
    defaults: Defaults,
    command_palette: bool,
    asset_mounts: Vec<AssetMount>,
    setup_hooks: Vec<AppSetupHook>,
    install_hooks: Vec<fn(&mut crate::app::App, &mut dyn fret_core::UiServices)>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl FretApp {
    /// Create a new app builder with a stable root name.
    ///
    /// `root_name` is used by the golden-path driver for IDs, diagnostics, and dev tooling.
    pub fn new(root_name: &'static str) -> Self {
        Self {
            root_name,
            main_window: None,
            defaults: Defaults::default(),
            command_palette: false,
            asset_mounts: Vec::new(),
            setup_hooks: Vec::new(),
            install_hooks: Vec::new(),
        }
    }

    /// Override the default runtime defaults applied by the `fret` facade.
    pub fn defaults(mut self, defaults: Defaults) -> Self {
        self.defaults = defaults;
        self
    }

    /// Apply the minimal defaults preset (no config files, no diagnostics, no shadcn integration).
    pub fn minimal_defaults(mut self) -> Self {
        self.defaults = Defaults::minimal();
        self
    }

    /// Enable/disable layered `.fret/*` config file loading.
    pub fn config_files(mut self, enabled: bool) -> Self {
        self.defaults.config_files = enabled;
        self
    }

    /// Override UI assets budgets and enable UI assets caches.
    pub fn ui_assets_budgets(
        mut self,
        image_budget_bytes: u64,
        image_max_ready_entries: usize,
        svg_budget_bytes: u64,
        svg_max_ready_entries: usize,
    ) -> Self {
        self.defaults = self.defaults.with_ui_assets_budgets(
            image_budget_bytes,
            image_max_ready_entries,
            svg_budget_bytes,
            svg_max_ready_entries,
        );
        self
    }

    /// Register a native/package-dev asset manifest on the default app builder path.
    ///
    /// This keeps logical bundle keys on the builder surface instead of teaching ad-hoc runtime
    /// resolver registration inside app setup or widget code. Asset registrations preserve the
    /// builder call order, so later calls can intentionally override earlier ones for the same
    /// logical locator.
    pub fn asset_manifest(mut self, manifest_path: impl Into<PathBuf>) -> Self {
        self.asset_mounts.push(AssetMount::Manifest {
            path: manifest_path.into(),
        });
        self
    }

    /// Register a native/package-dev asset directory under the default app bundle id.
    ///
    /// This convenience lane scans `dir` eagerly during builder assembly and exposes the files as
    /// logical bundle assets under `AssetBundleId::app(root_name)`. Prefer
    /// [`asset_manifest`](Self::asset_manifest) when tooling already emits an explicit manifest
    /// artifact that should be reviewed or packaged directly. Asset registrations preserve the
    /// builder call order, so later calls can intentionally override earlier ones for the same
    /// logical locator.
    pub fn asset_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.asset_mounts.push(AssetMount::Dir {
            bundle: AssetBundleId::app(self.root_name),
            dir: dir.into(),
        });
        self
    }

    /// Register compile-time/static app bundle entries on the builder path.
    ///
    /// This is the packaged/web/mobile-friendly lane for assets already owned by the Rust build
    /// (for example generated `include_bytes!` modules). Asset registrations preserve the builder
    /// call order, so later calls can intentionally override earlier ones for the same logical
    /// locator.
    pub fn asset_entries(mut self, entries: impl IntoIterator<Item = StaticAssetEntry>) -> Self {
        self.asset_mounts.push(AssetMount::BundleEntries {
            bundle: AssetBundleId::app(self.root_name),
            entries: entries.into_iter().collect(),
        });
        self
    }

    /// Register static bundle-scoped entries on the builder path under an explicit bundle id.
    ///
    /// This is useful when app startup needs to mount generated/package-owned bundle assets while
    /// still preserving builder call order.
    pub fn bundle_asset_entries(
        mut self,
        bundle: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) -> Self {
        self.asset_mounts.push(AssetMount::BundleEntries {
            bundle: bundle.into(),
            entries: entries.into_iter().collect(),
        });
        self
    }

    /// Register static embedded entries owned by a specific bundle or crate on the builder path.
    ///
    /// This keeps packaged embedded bytes on the builder/startup surface instead of falling back
    /// to ad-hoc setup hooks.
    pub fn embedded_asset_entries(
        mut self,
        owner: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) -> Self {
        self.asset_mounts.push(AssetMount::EmbeddedEntries {
            owner: owner.into(),
            entries: entries.into_iter().collect(),
        });
        self
    }

    /// Apply one explicit development-vs-packaged startup plan on the default app builder path.
    ///
    /// Use this when app/bootstrap code wants one named asset-publication decision instead of
    /// manually branching between `asset_dir(...)`, `asset_manifest(...)`, and packaged static
    /// entry registration at the call site. The lower-level methods remain available for custom
    /// ordered layering beyond this higher-level split.
    pub fn asset_startup(
        mut self,
        mode: crate::assets::AssetStartupMode,
        plan: crate::assets::AssetStartupPlan,
    ) -> Self {
        match plan.into_mounts(AssetBundleId::app(self.root_name), mode) {
            Ok(mounts) => self.asset_mounts.extend(mounts),
            Err(err) => self.asset_mounts.push(AssetMount::StartupPlanError(err)),
        }
        self
    }

    /// Enable the command palette (driver-handled command + UI) if available.
    ///
    /// This is intentionally opt-in in the `fret` facade.
    #[cfg(feature = "command-palette")]
    pub fn command_palette(mut self, enabled: bool) -> Self {
        self.command_palette = enabled;
        self
    }

    /// Run app-level setup during bootstrap.
    ///
    /// This is the canonical ecosystem integration seam for app-level add-ons such as command
    /// registration, theme/bootstrap setup, icon-pack app installers, optional recipe-crate
    /// globals, or reusable app integration bundles that implement [`InstallIntoApp`]. Prefer
    /// named installer functions, small tuples of installers, or named bundle types here. Keep
    /// inline one-off closures on [`UiAppBuilder::setup_with`](crate::UiAppBuilder::setup_with) so
    /// the default `.setup(...)` story stays stable and grep-friendly.
    pub fn setup<T>(mut self, setup: T) -> Self
    where
        T: InstallIntoApp + 'static,
    {
        self.setup_hooks
            .push(Box::new(move |app| setup.install_into_app(app)));
        self
    }

    /// Configure the main window (title + size).
    pub fn window(mut self, title: impl Into<String>, size: (f64, f64)) -> Self {
        self.main_window = Some((title.into(), size));
        self
    }

    /// Build a view-runtime app (`fret::view`) and return a runnable builder.
    ///
    /// This is the recommended authoring loop once `AppUi` adoption lands for the target area.
    pub fn view<V: crate::view::View>(
        self,
    ) -> Result<UiAppBuilder<crate::view::ViewWindowState<V>>> {
        self.view_with_hooks::<V>(|driver| driver)
    }

    /// Same as [`view`](Self::view), but keeps the `UiAppDriver` configuration seam available on
    /// the builder path.
    pub fn view_with_hooks<V: crate::view::View>(
        self,
        configure: fn(
            UiAppDriver<crate::view::ViewWindowState<V>>,
        ) -> UiAppDriver<crate::view::ViewWindowState<V>>,
    ) -> Result<UiAppBuilder<crate::view::ViewWindowState<V>>> {
        let FretApp {
            root_name,
            main_window,
            defaults,
            command_palette,
            asset_mounts,
            setup_hooks,
            install_hooks,
        } = self;

        let driver =
            fret_bootstrap::ui_app_driver::UiAppDriver::new(
                root_name,
                crate::view::view_init_window::<V>,
                crate::view::view_view::<V>,
            )
            .on_preferences(
                fret_bootstrap::ui_app_driver::default_on_preferences::<
                    crate::view::ViewWindowState<V>,
                >,
            );
        #[cfg(feature = "shadcn")]
        let driver = driver.on_global_changes_middleware(
            crate::shadcn_sync_theme_from_environment_on_global_changes::<
                crate::view::ViewWindowState<V>,
            >,
        );
        let mut driver = UiAppDriver::new(driver)
            .record_engine_frame(crate::view::view_record_engine_frame::<V>);
        driver = configure(driver);
        #[cfg(feature = "command-palette")]
        {
            if command_palette {
                driver = driver.command_palette(true);
            }
        }
        #[cfg(not(feature = "command-palette"))]
        let _ = command_palette;

        finish_builder(
            root_name,
            main_window,
            defaults,
            asset_mounts,
            setup_hooks,
            install_hooks,
            driver,
        )
    }

    pub(crate) fn install_services(
        mut self,
        install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices),
    ) -> Self {
        self.install_hooks.push(install);
        self
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn finish_builder<S: 'static>(
    root_name: &'static str,
    main_window: Option<(String, (f64, f64))>,
    defaults: Defaults,
    asset_mounts: Vec<AssetMount>,
    setup_hooks: Vec<AppSetupHook>,
    install_hooks: Vec<fn(&mut crate::app::App, &mut dyn fret_core::UiServices)>,
    driver: UiAppDriver<S>,
) -> Result<UiAppBuilder<S>> {
    let mut builder = fret_bootstrap::BootstrapBuilder::new(
        fret_app::App::new(),
        driver.into_inner().into_fn_driver(),
    );

    for f in setup_hooks {
        builder = builder.init_app(f);
    }
    for f in install_hooks {
        builder = builder.install(f);
    }

    let builder = crate::apply_desktop_defaults_with(builder, defaults)
        .map_err(crate::BootstrapError::from)?;
    let mut builder = UiAppBuilder::from_bootstrap(builder);
    builder = apply_main_window(root_name, main_window, builder);
    crate::apply_asset_mounts(builder, asset_mounts)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn apply_main_window<S: 'static>(
    root_name: &'static str,
    main_window: Option<(String, (f64, f64))>,
    builder: UiAppBuilder<S>,
) -> UiAppBuilder<S> {
    if let Some((title, size)) = main_window {
        return builder.with_main_window(title, size);
    }

    builder.with_main_window(root_name, (960.0, 720.0))
}

#[cfg(all(test, not(target_arch = "wasm32"), feature = "desktop"))]
mod tests {
    use super::*;
    use crate::assets::AssetRevision;

    #[test]
    fn asset_mounts_preserve_builder_call_order() {
        let app = FretApp::new("demo-app")
            .asset_dir("assets/dev")
            .asset_entries([StaticAssetEntry::new(
                "icons/search.svg",
                AssetRevision(1),
                br#"<svg></svg>"#,
            )])
            .asset_manifest("assets.manifest.json")
            .asset_dir("assets/override");

        assert_eq!(app.asset_mounts.len(), 4);
        match &app.asset_mounts[0] {
            AssetMount::Dir { bundle, dir } => {
                assert_eq!(bundle, &AssetBundleId::app("demo-app"));
                assert_eq!(dir, &PathBuf::from("assets/dev"));
            }
            other => panic!("expected dir mount first, got {other:?}"),
        }
        match &app.asset_mounts[1] {
            AssetMount::BundleEntries { bundle, entries } => {
                assert_eq!(bundle, &AssetBundleId::app("demo-app"));
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].key, "icons/search.svg");
                assert_eq!(entries[0].revision, AssetRevision(1));
            }
            other => panic!("expected bundle entries mount second, got {other:?}"),
        }
        match &app.asset_mounts[2] {
            AssetMount::Manifest { path } => {
                assert_eq!(path, &PathBuf::from("assets.manifest.json"));
            }
            other => panic!("expected manifest mount third, got {other:?}"),
        }
        match &app.asset_mounts[3] {
            AssetMount::Dir { bundle, dir } => {
                assert_eq!(bundle, &AssetBundleId::app("demo-app"));
                assert_eq!(dir, &PathBuf::from("assets/override"));
            }
            other => panic!("expected dir mount fourth, got {other:?}"),
        }
    }

    #[test]
    fn asset_startup_selects_development_lane_only() {
        let app = FretApp::new("demo-app").asset_startup(
            crate::assets::AssetStartupMode::Development,
            crate::assets::AssetStartupPlan::new()
                .development_dir("assets/dev")
                .development_manifest("assets.manifest.json")
                .packaged_entries([StaticAssetEntry::new(
                    "icons/search.svg",
                    AssetRevision(1),
                    br#"<svg></svg>"#,
                )]),
        );

        assert_eq!(app.asset_mounts.len(), 2);
        match &app.asset_mounts[0] {
            AssetMount::Dir { bundle, dir } => {
                assert_eq!(bundle, &AssetBundleId::app("demo-app"));
                assert_eq!(dir, &PathBuf::from("assets/dev"));
            }
            other => panic!("expected development dir mount first, got {other:?}"),
        }
        match &app.asset_mounts[1] {
            AssetMount::Manifest { path } => {
                assert_eq!(path, &PathBuf::from("assets.manifest.json"));
            }
            other => panic!("expected development manifest second, got {other:?}"),
        }
    }

    #[test]
    fn asset_startup_selects_packaged_lane_only() {
        let app = FretApp::new("demo-app").asset_startup(
            crate::assets::AssetStartupMode::Packaged,
            crate::assets::AssetStartupPlan::new()
                .development_dir("assets/dev")
                .packaged_entries([StaticAssetEntry::new(
                    "icons/search.svg",
                    AssetRevision(1),
                    br#"<svg></svg>"#,
                )])
                .packaged_bundle_entries(
                    AssetBundleId::package("demo-kit"),
                    [StaticAssetEntry::new(
                        "images/logo.png",
                        AssetRevision(2),
                        b"demo-kit",
                    )],
                ),
        );

        assert_eq!(app.asset_mounts.len(), 2);
        match &app.asset_mounts[0] {
            AssetMount::BundleEntries { bundle, entries } => {
                assert_eq!(bundle, &AssetBundleId::app("demo-app"));
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].key, "icons/search.svg");
            }
            other => panic!("expected app packaged entries first, got {other:?}"),
        }
        match &app.asset_mounts[1] {
            AssetMount::BundleEntries { bundle, entries } => {
                assert_eq!(bundle, &AssetBundleId::package("demo-kit"));
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].key, "images/logo.png");
            }
            other => panic!("expected package packaged entries second, got {other:?}"),
        }
    }
}

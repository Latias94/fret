//! `fret::FretApp` builder-chain entry points.
//!
//! This module provides an ergonomic, desktop-first entry surface (ecosystem-level) while
//! preserving the golden-path driver's hotpatch-friendly posture (function-pointer hooks).

use crate::{Defaults, Result, UiAppBuilder, UiAppDriver, integration::InstallIntoApp};

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
    Ok(builder)
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

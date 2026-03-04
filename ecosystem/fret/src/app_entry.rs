//! `fret::App` builder-chain entry points.
//!
//! This module provides an ergonomic, desktop-first entry surface (ecosystem-level) while
//! preserving the golden-path driver's hotpatch-friendly posture (function-pointer hooks).

use crate::{Defaults, Result, UiAppBuilder, ViewElements};

/// Builder-chain facade for creating and running a desktop-first Fret UI app.
///
/// Notes:
/// - This is an ecosystem-level convenience layer (not a kernel contract).
/// - The builder composes existing `fret` entry points (`fret::mvu` / `fret::app`) and applies
///   a default main window if none is configured.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct App {
    root_name: &'static str,
    main_window: Option<(String, (f64, f64))>,
    defaults: Defaults,
    command_palette: bool,
    install_app_hooks: Vec<fn(&mut fret_app::App)>,
    install_hooks: Vec<fn(&mut fret_app::App, &mut dyn fret_core::UiServices)>,
    register_icon_pack_hooks: Vec<fn(&mut crate::IconRegistry)>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl App {
    /// Create a new app builder with a stable root name.
    ///
    /// `root_name` is used by the golden-path driver for IDs, diagnostics, and dev tooling.
    pub fn new(root_name: &'static str) -> Self {
        Self {
            root_name,
            main_window: None,
            defaults: Defaults::default(),
            command_palette: false,
            install_app_hooks: Vec::new(),
            install_hooks: Vec::new(),
            register_icon_pack_hooks: Vec::new(),
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

    /// Install app-owned services/globals/commands during bootstrap.
    ///
    /// Prefer calling this before enabling config files so command defaults/keymap layering see
    /// the full command registry.
    pub fn install_app(mut self, install: fn(&mut fret_app::App)) -> Self {
        self.install_app_hooks.push(install);
        self
    }

    /// Install wiring that needs `UiServices` during bootstrap.
    pub fn install(
        mut self,
        install: fn(&mut fret_app::App, &mut dyn fret_core::UiServices),
    ) -> Self {
        self.install_hooks.push(install);
        self
    }

    /// Register one or more custom icon packs (runs during bootstrap).
    pub fn register_icon_pack(mut self, register: fn(&mut crate::IconRegistry)) -> Self {
        self.register_icon_pack_hooks.push(register);
        self
    }

    /// Configure the main window (title + size).
    pub fn window(mut self, title: impl Into<String>, size: (f64, f64)) -> Self {
        self.main_window = Some((title.into(), size));
        self
    }

    /// Build a UI app from `init_window` + `view` and return a runnable builder.
    pub fn ui<S: 'static>(
        self,
        init_window: fn(&mut fret_app::App, fret_core::AppWindowId) -> S,
        view: for<'a> fn(&mut fret_ui::ElementContext<'a, fret_app::App>, &mut S) -> ViewElements,
    ) -> Result<UiAppBuilder<S>> {
        let App {
            root_name,
            main_window,
            defaults,
            command_palette,
            install_app_hooks,
            install_hooks,
            register_icon_pack_hooks,
        } = self;

        fn configure_ui_driver<S>(d: crate::UiAppDriver<S>) -> crate::UiAppDriver<S> {
            d
        }

        #[cfg(feature = "command-palette")]
        fn configure_ui_driver_with_palette<S>(d: crate::UiAppDriver<S>) -> crate::UiAppDriver<S> {
            d.command_palette(true)
        }

        let configure: fn(crate::UiAppDriver<S>) -> crate::UiAppDriver<S> = {
            #[cfg(feature = "command-palette")]
            {
                if command_palette {
                    configure_ui_driver_with_palette::<S>
                } else {
                    configure_ui_driver::<S>
                }
            }
            #[cfg(not(feature = "command-palette"))]
            {
                let _ = command_palette;
                configure_ui_driver::<S>
            }
        };

        let mut builder =
            crate::ui_bootstrap_builder_with_hooks(root_name, init_window, view, configure);

        for f in install_app_hooks {
            builder = builder.install_app(f);
        }
        for f in install_hooks {
            builder = builder.install(f);
        }
        for f in register_icon_pack_hooks {
            builder = builder.register_icon_pack(f);
        }

        let builder = crate::apply_desktop_defaults_with(builder, defaults)
            .map_err(crate::BootstrapError::from)?;
        let mut builder = UiAppBuilder::from_bootstrap(builder);
        builder = apply_main_window(root_name, main_window, builder);
        Ok(builder)
    }

    /// Build a view-runtime app (`fret::view`) and return a runnable builder.
    ///
    /// This is the recommended authoring loop once `ViewCx` adoption lands for the target area.
    pub fn view<V: crate::view::View>(
        self,
    ) -> Result<UiAppBuilder<crate::view::ViewWindowState<V>>> {
        let App {
            root_name,
            main_window,
            defaults,
            command_palette,
            install_app_hooks,
            install_hooks,
            register_icon_pack_hooks,
        } = self;

        fn configure_view_driver<V: crate::view::View>(
            d: crate::UiAppDriver<crate::view::ViewWindowState<V>>,
        ) -> crate::UiAppDriver<crate::view::ViewWindowState<V>> {
            d.record_engine_frame(crate::view::view_record_engine_frame::<V>)
        }

        #[cfg(feature = "command-palette")]
        fn configure_view_driver_with_palette<V: crate::view::View>(
            d: crate::UiAppDriver<crate::view::ViewWindowState<V>>,
        ) -> crate::UiAppDriver<crate::view::ViewWindowState<V>> {
            d.record_engine_frame(crate::view::view_record_engine_frame::<V>)
                .command_palette(true)
        }

        let configure: fn(
            crate::UiAppDriver<crate::view::ViewWindowState<V>>,
        ) -> crate::UiAppDriver<crate::view::ViewWindowState<V>> = {
            #[cfg(feature = "command-palette")]
            {
                if command_palette {
                    configure_view_driver_with_palette::<V>
                } else {
                    configure_view_driver::<V>
                }
            }
            #[cfg(not(feature = "command-palette"))]
            {
                let _ = command_palette;
                configure_view_driver::<V>
            }
        };

        let mut builder = crate::ui_bootstrap_builder_with_hooks(
            root_name,
            crate::view::view_init_window::<V>,
            crate::view::view_view::<V>,
            configure,
        );

        for f in install_app_hooks {
            builder = builder.install_app(f);
        }
        for f in install_hooks {
            builder = builder.install(f);
        }
        for f in register_icon_pack_hooks {
            builder = builder.register_icon_pack(f);
        }

        let builder = crate::apply_desktop_defaults_with(builder, defaults)
            .map_err(crate::BootstrapError::from)?;
        let mut builder = UiAppBuilder::from_bootstrap(builder);
        builder = apply_main_window(root_name, main_window, builder);
        Ok(builder)
    }

    /// Convenience: build a view-runtime app and run it immediately.
    pub fn run_view<V: crate::view::View>(self) -> Result<()> {
        self.view::<V>()?.run()
    }

    /// Convenience: build a UI app and run it immediately.
    pub fn run_ui<S: 'static>(
        self,
        init_window: fn(&mut fret_app::App, fret_core::AppWindowId) -> S,
        view: for<'a> fn(&mut fret_ui::ElementContext<'a, fret_app::App>, &mut S) -> ViewElements,
    ) -> Result<()> {
        self.ui(init_window, view)?.run()
    }
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

//! Batteries-included desktop-first entry points for Fret.
//!
//! This crate is intentionally **ecosystem-level**:
//! - it composes `fret-bootstrap` (golden-path wiring) with a default component surface,
//! - it enables a practical desktop-first default stack,
//! - it remains optional: advanced users can depend on `fret` + `fret-bootstrap` directly.

#[cfg(all(feature = "icons-lucide", feature = "icons-radix"))]
compile_error!("`fret-kit` features `icons-lucide` and `icons-radix` are mutually exclusive.");

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use fret_app::App;

/// Re-export the default shadcn/ui surface as `shadcn`.
#[cfg(feature = "shadcn")]
pub use fret_ui_shadcn as shadcn;

/// Re-export workspace-shell building blocks (editor-grade chrome) as `workspace`.
#[cfg(feature = "workspace-shell")]
pub use fret_workspace as workspace;

/// Re-export the `IconRegistry` type for app code that wants to install a custom icon pack.
pub use fret_icons::IconRegistry;

/// Re-export `ViewElements` so app code can stay on the `fret-kit` surface.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub use fret_bootstrap::ui_app_driver::ViewElements;

#[cfg(feature = "workspace-shell")]
pub mod pending_shortcut_overlay;
pub mod workspace_menu;
#[cfg(feature = "workspace-shell")]
pub mod workspace_shell;

/// MVU-style authoring helpers (desktop builds).
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub mod mvu;

/// Interop helpers for embedding foreign UI as isolated surfaces (desktop builds).
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub mod interop;

/// Re-export the underlying `fret` facade (desktop builds).
#[cfg(feature = "desktop")]
pub use fret;

/// Common imports for application code using `fret-kit`.
///
/// Recommended: `use fret_kit::prelude::*;`
pub mod prelude {
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::interop::embedded_viewport::{
        EmbeddedViewportForeignMvuUiAppDriverExt, EmbeddedViewportForeignUiAppDriverExt,
        EmbeddedViewportMvuUiAppDriverExt, EmbeddedViewportUiAppDriverExt,
    };
    #[cfg(feature = "shadcn")]
    pub use crate::shadcn;
    #[cfg(feature = "shadcn")]
    pub use crate::shadcn::prelude::*;
    pub use crate::workspace_menu::{
        InWindowMenubarFocusHandle, MenubarFromRuntimeOptions, menubar_from_runtime,
        menubar_from_runtime_with_focus_handle,
    };

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::ViewElements;
    pub use fret_app::App;
    pub use fret_app::Effect;
    pub use fret_core::{AppWindowId, Event, Px, SemanticsRole, UiServices};
    pub use fret_runtime::CommandId;
    pub use fret_runtime::Model;
    pub use fret_ui::element::{
        AnyElement, AnyElementIterExt, Elements, HoverRegionProps, Length, SemanticsProps,
        TextProps,
    };
    pub use fret_ui::{ElementContext, Invalidation, Theme, ThemeSnapshot, UiTree};
    pub use fret_ui_kit::declarative::AnyElementSemanticsExt;
    pub use fret_ui_kit::declarative::ModelWatchExt;
    #[cfg(not(feature = "shadcn"))]
    pub use fret_ui_kit::{
        UiBuilder, UiExt, UiIntoElement, UiPatch, UiPatchTarget, UiSupportsChrome,
        UiSupportsLayout, ui,
    };

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::mvu::{KeyedMessageRouter, MessageRouter, Program as MvuProgram};

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::interop;

    #[cfg(feature = "workspace-shell")]
    pub use crate::workspace;
    #[cfg(feature = "workspace-shell")]
    pub use crate::workspace::layout::{WorkspaceLayout, WorkspaceLayoutV1};
    #[cfg(feature = "workspace-shell")]
    pub use crate::workspace::menu::{WorkspaceMenuCommands, workspace_default_menu_bar};
    #[cfg(feature = "workspace-shell")]
    pub use crate::workspace::tabs::{TabCycleMode, WorkspaceTabs, WorkspaceTabsV1};
    #[cfg(feature = "workspace-shell")]
    pub use crate::workspace::{
        WorkspaceFrame, WorkspaceStatusBar, WorkspaceTab, WorkspaceTabStrip, WorkspaceTopBar,
    };
    #[cfg(feature = "workspace-shell")]
    pub use crate::workspace_shell::{workspace_shell_model, workspace_shell_model_default_menu};
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Bootstrap(#[from] BootstrapError),
    #[error(transparent)]
    Runner(#[from] RunnerError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BootstrapError(#[from] fret_bootstrap::BootstrapError);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct RunnerError(#[from] fret_launch::RunnerError);

/// A `UiAppDriver` wrapper used by `fret-kit` to avoid exposing `fret-bootstrap` types in signatures.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct UiAppDriver<S> {
    inner: fret_bootstrap::ui_app_driver::UiAppDriver<S>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl<S> UiAppDriver<S> {
    fn new(inner: fret_bootstrap::ui_app_driver::UiAppDriver<S>) -> Self {
        Self { inner }
    }

    fn into_inner(self) -> fret_bootstrap::ui_app_driver::UiAppDriver<S> {
        self.inner
    }

    pub fn close_on_window_close_requested(mut self, enabled: bool) -> Self {
        self.inner = self.inner.close_on_window_close_requested(enabled);
        self
    }

    #[cfg(feature = "ui-assets")]
    pub fn drive_ui_assets(mut self, enabled: bool) -> Self {
        self.inner = self.inner.drive_ui_assets(enabled);
        self
    }

    pub fn on_event(
        mut self,
        f: fn(
            &mut App,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<App>,
            &mut S,
            &fret_core::Event,
        ),
    ) -> Self {
        self.inner = self.inner.on_event(f);
        self
    }

    pub fn on_command(
        mut self,
        f: fn(
            &mut App,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<App>,
            &mut S,
            &fret_runtime::CommandId,
        ),
    ) -> Self {
        self.inner = self.inner.on_command(f);
        self
    }

    pub fn on_preferences(
        mut self,
        f: fn(
            &mut App,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<App>,
            &mut S,
        ),
    ) -> Self {
        self.inner = self.inner.on_preferences(f);
        self
    }

    pub fn on_hot_reload_window(
        mut self,
        f: fn(
            &mut App,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<App>,
            &mut S,
        ),
    ) -> Self {
        self.inner = self.inner.on_hot_reload_window(f);
        self
    }

    pub fn on_model_changes(
        mut self,
        f: fn(
            &mut App,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<App>,
            &mut S,
            &[fret_app::ModelId],
        ),
    ) -> Self {
        self.inner = self.inner.on_model_changes(f);
        self
    }

    pub fn on_global_changes(
        mut self,
        f: fn(
            &mut App,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<App>,
            &mut S,
            &[std::any::TypeId],
        ),
    ) -> Self {
        self.inner = self.inner.on_global_changes(f);
        self
    }

    pub fn window_create_spec(
        mut self,
        f: fn(&mut App, &fret_app::CreateWindowRequest) -> Option<fret_launch::WindowCreateSpec>,
    ) -> Self {
        self.inner = self.inner.window_create_spec(f);
        self
    }

    pub fn window_created(
        mut self,
        f: fn(&mut App, &fret_app::CreateWindowRequest, fret_core::AppWindowId),
    ) -> Self {
        self.inner = self.inner.window_created(f);
        self
    }

    pub fn before_close_window(mut self, f: fn(&mut App, fret_core::AppWindowId) -> bool) -> Self {
        self.inner = self.inner.before_close_window(f);
        self
    }

    pub fn handle_global_command(
        mut self,
        f: fn(&mut App, &mut dyn fret_core::UiServices, fret_runtime::CommandId),
    ) -> Self {
        self.inner = self.inner.handle_global_command(f);
        self
    }

    pub fn viewport_input(mut self, f: fn(&mut App, fret_core::ViewportInputEvent)) -> Self {
        self.inner = self.inner.viewport_input(f);
        self
    }

    pub fn record_engine_frame(
        mut self,
        f: fn(
            &mut App,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<App>,
            &mut S,
            &crate::fret::render::WgpuContext,
            &mut crate::fret::render::Renderer,
            f32,
            fret_runtime::TickId,
            fret_runtime::FrameId,
        ) -> fret_launch::EngineFrameUpdate,
    ) -> Self {
        self.inner = self.inner.record_engine_frame(f);
        self
    }

    pub fn dock_op(mut self, f: fn(&mut App, fret_core::DockOp)) -> Self {
        self.inner = self.inner.dock_op(f);
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn command_palette(mut self, enabled: bool) -> Self {
        self.inner = self.inner.command_palette(enabled);
        self
    }
}

/// A `BootstrapBuilder` wrapper used by `fret-kit` to avoid exposing `fret-bootstrap` types in signatures.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct UiAppBuilder<S> {
    inner: fret_bootstrap::UiAppBootstrapBuilder<S>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl<S: 'static> UiAppBuilder<S> {
    fn new(inner: fret_bootstrap::UiAppBootstrapBuilder<S>) -> Self {
        Self { inner }
    }

    pub fn with_main_window(self, title: impl Into<String>, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_main_window(title, size),
        }
    }

    pub fn init_app(self, f: impl FnOnce(&mut App)) -> Self {
        Self {
            inner: self.inner.init_app(f),
        }
    }

    pub fn install_app(self, install: fn(&mut App)) -> Self {
        Self {
            inner: self.inner.install_app(install),
        }
    }

    pub fn install(self, install: fn(&mut App, &mut dyn fret_core::UiServices)) -> Self {
        Self {
            inner: self.inner.install(install),
        }
    }

    pub fn register_icon_pack(self, register: fn(&mut IconRegistry)) -> Self {
        Self {
            inner: self.inner.register_icon_pack(register),
        }
    }

    #[cfg(feature = "ui-assets")]
    pub fn with_ui_assets_budgets(
        self,
        image_budget_bytes: u64,
        image_max_ready_entries: usize,
        svg_budget_bytes: u64,
        svg_max_ready_entries: usize,
    ) -> Self {
        Self {
            inner: self.inner.with_ui_assets_budgets(
                image_budget_bytes,
                image_max_ready_entries,
                svg_budget_bytes,
                svg_max_ready_entries,
            ),
        }
    }

    pub fn on_gpu_ready(
        self,
        f: impl FnOnce(&mut App, &crate::fret::render::WgpuContext, &mut crate::fret::render::Renderer)
        + 'static,
    ) -> Self {
        Self {
            inner: self.inner.on_gpu_ready(f),
        }
    }

    pub fn run(self) -> Result<()> {
        self.inner.run().map_err(RunnerError::from)?;
        Ok(())
    }
}

/// Run a native desktop demo using the `winit + wgpu` stack.
///
/// This is a small convenience wrapper for examples that implement `WinitAppDriver` directly,
/// keeping "how to boot the app" consistent with the `fret-kit` golden path.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run_native_demo<D: fret_launch::WinitAppDriver + 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: App,
    driver: D,
) -> Result<()> {
    let builder = fret_bootstrap::BootstrapBuilder::new(app, driver).configure(move |c| {
        *c = config;
    });

    #[cfg(feature = "diagnostics")]
    let builder = builder.with_default_diagnostics();

    let builder = builder
        .with_default_config_files()
        .map_err(BootstrapError::from)?;

    #[cfg(feature = "icons-lucide")]
    let builder = builder.with_lucide_icons();

    #[cfg(feature = "icons-radix")]
    let builder = builder.with_radix_icons();

    #[cfg(feature = "preload-icon-svgs")]
    let builder = builder.preload_icon_svgs_on_gpu_ready();

    builder.run().map_err(RunnerError::from)?;
    Ok(())
}

/// Create a desktop-first UI app builder with conservative defaults applied.
///
/// Defaults (when the corresponding features are enabled):
/// - diagnostics (`diagnostics`)
/// - layered config files (`.fret/settings.json`, `.fret/keymap.json`, `.fret/menubar.json`)
/// - shadcn app integration (`shadcn`)
/// - icon pack installation + optional SVG preloading
/// - UI assets caches with default budgets (`ui-assets`)
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn app_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ViewElements,
    configure: fn(UiAppDriver<S>) -> UiAppDriver<S>,
) -> Result<UiAppBuilder<S>> {
    let driver = fret_bootstrap::ui_app_driver::UiAppDriver::new(root_name, init_window, view)
        .on_preferences(fret_bootstrap::ui_app_driver::default_on_preferences::<S>);
    #[cfg(feature = "shadcn")]
    let driver = driver
        .on_global_changes_middleware(shadcn_sync_theme_from_environment_on_global_changes::<S>);
    let driver = configure(UiAppDriver::new(driver)).into_inner();
    let builder = fret_bootstrap::BootstrapBuilder::new(App::new(), driver.into_fn_driver());

    #[cfg(feature = "router")]
    let builder = builder.install_app(|app| {
        fret_router_ui::register_router_commands(app.commands_mut());
    });

    #[cfg(feature = "diagnostics")]
    let builder = builder.with_default_diagnostics();

    let builder = builder
        .with_default_config_files()
        .map_err(BootstrapError::from)?;

    #[cfg(feature = "shadcn")]
    let builder = builder.install_app(fret_ui_shadcn::install_app);

    #[cfg(feature = "ui-assets")]
    let builder = builder.with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096);

    #[cfg(feature = "icons-lucide")]
    let builder = builder.with_lucide_icons();

    #[cfg(feature = "icons-radix")]
    let builder = builder.with_radix_icons();

    #[cfg(feature = "preload-icon-svgs")]
    let builder = builder.preload_icon_svgs_on_gpu_ready();

    Ok(UiAppBuilder::new(builder))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop", feature = "shadcn"))]
fn shadcn_sync_theme_from_environment_on_global_changes<S>(
    app: &mut App,
    window: fret_core::AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    _st: &mut S,
    changed: &[std::any::TypeId],
) {
    if !changed.contains(&std::any::TypeId::of::<fret_core::WindowMetricsService>()) {
        return;
    }
    let config = app
        .global::<fret_ui_shadcn::ShadcnInstallConfig>()
        .copied()
        .unwrap_or_default();
    let _ =
        fret_ui_shadcn::sync_theme_from_environment(app, window, config.base_color, config.scheme);
}

/// Same as [`app_with_hooks`], but without a driver configuration hook.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn app<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ViewElements,
) -> Result<UiAppBuilder<S>> {
    app_with_hooks(root_name, init_window, view, |d| d)
}

/// Run a desktop-first UI app using default window settings.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ViewElements,
    configure: fn(UiAppDriver<S>) -> UiAppDriver<S>,
) -> Result<()> {
    app_with_hooks(root_name, init_window, view, configure)?
        .with_main_window(root_name, (960.0, 720.0))
        .run()
}

/// Run a desktop-first UI app using default window settings.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ViewElements,
) -> Result<()> {
    run_with_hooks(root_name, init_window, view, |d| d)
}

#[cfg(all(
    test,
    not(target_arch = "wasm32"),
    feature = "desktop",
    feature = "shadcn"
))]
mod tests {
    use std::any::TypeId;

    use fret_core::{AppWindowId, ColorScheme, WindowMetricsService};
    use fret_ui::{Theme, UiTree};

    use super::App;

    #[test]
    fn shadcn_auto_theme_middleware_reacts_to_window_metrics() {
        let mut app = App::new();
        fret_ui_shadcn::install_app(&mut app);

        let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));
        app.with_global_mut(WindowMetricsService::default, |svc, _app| {
            svc.set_color_scheme(window, Some(ColorScheme::Dark));
        });

        let mut ui = UiTree::<App>::default();
        let mut state = ();

        let before_bg = Theme::global(&app).colors.surface_background;
        let before_rev = Theme::global(&app).revision();

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[],
        );

        assert_eq!(Theme::global(&app).revision(), before_rev);
        assert_eq!(Theme::global(&app).colors.surface_background, before_bg);

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[TypeId::of::<WindowMetricsService>()],
        );

        assert_ne!(Theme::global(&app).colors.surface_background, before_bg);
        let rev_after = Theme::global(&app).revision();

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[TypeId::of::<WindowMetricsService>()],
        );

        assert_eq!(Theme::global(&app).revision(), rev_after);
    }
}

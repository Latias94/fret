//! Native builder wrappers and facade-owned default wiring.

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use crate::{
    AssetManifestError, AssetMount, BootstrapError, Defaults, DesktopDefaultsStage, Error, Result,
    RunnerError,
};
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use fret_app::App as KernelApp;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use fret_framework as kernel;

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn map_bootstrap_asset_builder_error(err: fret_bootstrap::BootstrapError) -> Error {
    match err {
        fret_bootstrap::BootstrapError::AssetManifest(err) => {
            Error::AssetManifest(AssetManifestError::from(err))
        }
        fret_bootstrap::BootstrapError::AssetStartup(err) => Error::AssetStartup(err),
        other => Error::Bootstrap(BootstrapError::from(other)),
    }
}

/// A `UiAppDriver` wrapper used by `fret` to avoid exposing `fret-bootstrap` types in signatures.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct UiAppDriver<S> {
    inner: fret_bootstrap::ui_app_driver::UiAppDriver<S>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl<S> UiAppDriver<S> {
    pub(crate) fn new(inner: fret_bootstrap::ui_app_driver::UiAppDriver<S>) -> Self {
        Self { inner }
    }

    pub(crate) fn into_inner(self) -> fret_bootstrap::ui_app_driver::UiAppDriver<S> {
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
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &fret_core::Event,
        ),
    ) -> Self {
        self.inner = self.inner.on_event(f);
        self
    }

    pub fn on_app_event(
        mut self,
        f: fn(&mut KernelApp, fret_core::AppWindowId, &mut S, &fret_core::Event),
    ) -> Self {
        self.inner = self.inner.on_app_event(f);
        self
    }

    pub fn on_command_before_ui(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &fret_runtime::CommandId,
        ) -> bool,
    ) -> Self {
        self.inner = self.inner.on_command_before_ui(f);
        self
    }

    pub fn on_app_command_before_ui(
        mut self,
        f: fn(
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut S,
            &fret_runtime::CommandId,
            fret_bootstrap::ui_app_driver::UiAppCommandBeforeUiContext<'_>,
        ) -> bool,
    ) -> Self {
        self.inner = self.inner.on_app_command_before_ui(f);
        self
    }

    pub fn on_command(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
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
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
        ),
    ) -> Self {
        self.inner = self.inner.on_preferences(f);
        self
    }

    pub fn on_hot_reload_window(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
        ),
    ) -> Self {
        self.inner = self.inner.on_hot_reload_window(f);
        self
    }

    pub fn on_model_changes(
        mut self,
        f: fn(
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
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
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &[std::any::TypeId],
        ),
    ) -> Self {
        self.inner = self.inner.on_global_changes(f);
        self
    }

    pub fn on_app_global_changes(
        mut self,
        f: fn(&mut KernelApp, fret_core::AppWindowId, &mut S, &[std::any::TypeId]),
    ) -> Self {
        self.inner = self.inner.on_app_global_changes(f);
        self
    }

    pub fn window_create_spec(
        mut self,
        f: fn(
            &mut KernelApp,
            &fret_app::CreateWindowRequest,
        ) -> Option<fret_launch::WindowCreateSpec>,
    ) -> Self {
        self.inner = self.inner.window_create_spec(f);
        self
    }

    pub fn window_created(
        mut self,
        f: fn(&mut KernelApp, &fret_app::CreateWindowRequest, fret_core::AppWindowId),
    ) -> Self {
        self.inner = self.inner.window_created(f);
        self
    }

    pub fn before_close_window(
        mut self,
        f: fn(&mut KernelApp, fret_core::AppWindowId) -> bool,
    ) -> Self {
        self.inner = self.inner.before_close_window(f);
        self
    }

    pub fn handle_global_command(
        mut self,
        f: fn(&mut KernelApp, &mut dyn fret_core::UiServices, fret_runtime::CommandId),
    ) -> Self {
        self.inner = self.inner.handle_global_command(f);
        self
    }

    pub fn viewport_input(mut self, f: fn(&mut KernelApp, fret_core::ViewportInputEvent)) -> Self {
        self.inner = self.inner.viewport_input(f);
        self
    }

    pub fn record_engine_frame(
        mut self,
        f: fn(
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &kernel::render::WgpuContext,
            &mut kernel::render::Renderer,
            f32,
            fret_runtime::TickId,
            fret_runtime::FrameId,
        ) -> fret_launch::EngineFrameUpdate,
    ) -> Self {
        self.inner = self.inner.record_engine_frame(f);
        self
    }

    pub fn dock_op(mut self, f: fn(&mut KernelApp, fret_core::DockOp)) -> Self {
        self.inner = self.inner.dock_op(f);
        self
    }

    /// Observe ordered app-frame stages without taking ownership of retained-tree staging.
    pub fn on_frame_stage(
        mut self,
        f: fn(&mut KernelApp, fret_core::AppWindowId, &mut S, crate::app::UiAppFrameObservation),
    ) -> Self {
        self.inner = self.inner.on_frame_stage(f);
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn command_palette(mut self, enabled: bool) -> Self {
        self.inner = self.inner.command_palette(enabled);
        if enabled {
            self.inner = fret_bootstrap::with_shadcn_command_palette(self.inner);
        }
        self
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl<S: crate::app::UiAppFrameStageSink> UiAppDriver<S> {
    /// Record ordered app-frame stages into the app state.
    pub fn record_frame_stages(mut self) -> Self {
        self.inner = self.inner.record_frame_stages();
        self
    }
}

/// A `UiAppBuilder` wrapper used by `fret` to avoid exposing `fret-bootstrap` types in signatures.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct UiAppBuilder<S> {
    inner: fret_bootstrap::UiAppBootstrapBuilder<S>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl<S: 'static> UiAppBuilder<S> {
    pub(crate) fn from_bootstrap(inner: fret_bootstrap::UiAppBootstrapBuilder<S>) -> Self {
        Self { inner }
    }

    pub(crate) fn install_services(
        self,
        install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices),
    ) -> Self {
        Self {
            inner: self.inner.install(install),
        }
    }

    pub(crate) fn install_custom_effects(
        self,
        install: fn(&mut KernelApp, &mut dyn fret_core::CustomEffectService),
    ) -> Self {
        Self {
            inner: self.inner.install_custom_effects(install),
        }
    }

    pub(crate) fn on_gpu_ready(
        self,
        f: impl FnOnce(&mut KernelApp, &kernel::render::WgpuContext, &mut kernel::render::Renderer)
        + 'static,
    ) -> Self {
        Self {
            inner: self.inner.on_gpu_ready(f),
        }
    }

    pub fn with_command_default_keybindings(self) -> Self {
        Self {
            inner: self.inner.with_command_default_keybindings(),
        }
    }

    pub fn with_default_config_files(self) -> Result<Self> {
        Ok(Self {
            inner: self
                .inner
                .with_default_config_files()
                .map_err(BootstrapError::from)?,
        })
    }

    pub fn with_default_config_files_for_root(
        self,
        project_root: impl AsRef<std::path::Path>,
    ) -> Result<Self> {
        Ok(Self {
            inner: self
                .inner
                .with_default_config_files_for_root(project_root)
                .map_err(BootstrapError::from)?,
        })
    }

    pub fn with_main_window(self, title: impl Into<String>, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_main_window(title, size),
        }
    }

    pub fn with_main_window_min_size(self, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_main_window_min_size(size),
        }
    }

    pub fn with_main_window_max_size(self, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_main_window_max_size(size),
        }
    }

    pub fn with_main_window_resize_increments(self, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_main_window_resize_increments(size),
        }
    }

    pub fn with_main_window_position_logical(self, position: (i32, i32)) -> Self {
        Self {
            inner: self.inner.with_main_window_position_logical(position),
        }
    }

    pub fn with_main_window_position_physical(self, position: (i32, i32)) -> Self {
        Self {
            inner: self.inner.with_main_window_position_physical(position),
        }
    }

    pub fn with_main_window_resizable(self, resizable: bool) -> Self {
        Self {
            inner: self.inner.with_main_window_resizable(resizable),
        }
    }

    pub fn with_default_window(self, title: impl Into<String>, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_default_window(title, size),
        }
    }

    pub fn with_default_window_min_size(self, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_default_window_min_size(size),
        }
    }

    pub fn with_default_window_max_size(self, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_default_window_max_size(size),
        }
    }

    pub fn with_default_window_resize_increments(self, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_default_window_resize_increments(size),
        }
    }

    pub fn with_default_window_position_logical(self, position: (i32, i32)) -> Self {
        Self {
            inner: self.inner.with_default_window_position_logical(position),
        }
    }

    pub fn with_default_window_position_physical(self, position: (i32, i32)) -> Self {
        Self {
            inner: self.inner.with_default_window_position_physical(position),
        }
    }

    pub fn configure(self, f: impl FnOnce(&mut fret_launch::WinitRunnerConfig)) -> Self {
        Self {
            inner: self.inner.configure(f),
        }
    }

    /// Run one-off app setup inline on the builder path.
    ///
    /// Use this when the setup needs to capture runtime values or is intentionally local to this
    /// call site. Prefer [`setup`](Self::setup) with named installer functions, tuples, or named
    /// [`crate::integration::InstallIntoApp`] bundles for reusable/default app wiring.
    pub fn setup_with(self, f: impl FnOnce(&mut crate::app::App)) -> Self {
        Self {
            inner: self.inner.init_app(f),
        }
    }

    /// Run app setup through the stable installer/bundle seam.
    ///
    /// Prefer this for named installer functions, small app-local tuples, and reusable
    /// [`crate::integration::InstallIntoApp`] bundles. Keep inline closures on
    /// [`setup_with`](Self::setup_with) so the default `.setup(...)` story stays explicit.
    pub fn setup<T>(self, setup: T) -> Self
    where
        T: crate::integration::InstallIntoApp + 'static,
    {
        Self {
            inner: self.inner.init_app(move |app| setup.install_into_app(app)),
        }
    }

    /// Register static bundle-scoped entries on the builder path.
    ///
    /// This is the packaged/web/mobile-friendly lane for compile-time owned assets such as
    /// generated `include_bytes!` modules. Builder registrations preserve call order, so later
    /// calls can intentionally override earlier ones for the same logical locator.
    pub fn with_bundle_asset_entries(
        self,
        bundle: impl Into<crate::assets::AssetBundleId>,
        entries: impl IntoIterator<Item = crate::assets::StaticAssetEntry>,
    ) -> Self {
        let bundle = bundle.into();
        let entries = entries.into_iter().collect::<Vec<_>>();
        Self {
            inner: self.inner.init_app(move |app| {
                crate::assets::register_bundle_entries(app, bundle, entries);
            }),
        }
    }

    /// Register static embedded entries on the builder path.
    ///
    /// This keeps compile-time owned embedded bytes on the same ordered startup surface as other
    /// asset registrations instead of forcing callers back to ad-hoc setup hooks.
    pub fn with_embedded_asset_entries(
        self,
        owner: impl Into<crate::assets::AssetBundleId>,
        entries: impl IntoIterator<Item = crate::assets::StaticAssetEntry>,
    ) -> Self {
        let owner = owner.into();
        let entries = entries.into_iter().collect::<Vec<_>>();
        Self {
            inner: self.inner.init_app(move |app| {
                crate::assets::register_embedded_entries(app, owner, entries);
            }),
        }
    }

    /// Apply one explicit development-vs-packaged startup plan on the builder path.
    ///
    /// This higher-level surface keeps the current startup decision on one named value while still
    /// composing with the same ordered static-entry registrations as
    /// `with_bundle_asset_entries(...)` and `with_embedded_asset_entries(...)`.
    pub fn with_asset_startup(
        self,
        app_bundle: impl Into<crate::assets::AssetBundleId>,
        mode: crate::assets::AssetStartupMode,
        plan: crate::assets::AssetStartupPlan,
    ) -> Result<Self> {
        Ok(Self {
            inner: self
                .inner
                .with_asset_startup(app_bundle.into(), mode, plan)
                .map_err(map_bootstrap_asset_builder_error)?,
        })
    }

    /// Enable development asset reload polling for file-backed startup mounts.
    pub fn with_asset_reload_policy(self, policy: crate::assets::AssetReloadPolicy) -> Self {
        Self {
            inner: self.inner.with_asset_reload_policy(policy),
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

    #[cfg(feature = "preload-icon-svgs")]
    pub fn preload_icon_svgs_on_gpu_ready(self) -> Self {
        Self {
            inner: self.inner.preload_icon_svgs_on_gpu_ready(),
        }
    }

    #[cfg(feature = "diagnostics")]
    pub fn with_default_diagnostics(self) -> Self {
        Self {
            inner: self.inner.with_default_diagnostics(),
        }
    }

    pub fn run(self) -> Result<()> {
        self.inner.run().map_err(RunnerError::from)?;
        Ok(())
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn apply_asset_mount<S: 'static>(
    builder: UiAppBuilder<S>,
    mount: AssetMount,
) -> Result<UiAppBuilder<S>> {
    match mount {
        AssetMount::BundleEntries { bundle, entries } => {
            Ok(builder.with_bundle_asset_entries(bundle, entries))
        }
        AssetMount::EmbeddedEntries { owner, entries } => {
            Ok(builder.with_embedded_asset_entries(owner, entries))
        }
        AssetMount::Startup { bundle, mode, plan } => {
            builder.with_asset_startup(bundle, mode, plan)
        }
        AssetMount::ReloadPolicy { policy } => Ok(builder.with_asset_reload_policy(policy)),
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn apply_asset_mounts<S: 'static>(
    builder: UiAppBuilder<S>,
    mounts: Vec<AssetMount>,
) -> Result<UiAppBuilder<S>> {
    mounts.into_iter().try_fold(builder, apply_asset_mount)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn apply_desktop_defaults_with<D: fret_launch::WinitAppDriver + 'static>(
    builder: fret_bootstrap::BootstrapBuilder<D>,
    defaults: Defaults,
) -> std::result::Result<fret_bootstrap::BootstrapBuilder<D>, fret_bootstrap::BootstrapError> {
    apply_desktop_defaults_stage_with(builder, defaults, DesktopDefaultsStage::All)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn apply_desktop_defaults_stage_with<D: fret_launch::WinitAppDriver + 'static>(
    builder: fret_bootstrap::BootstrapBuilder<D>,
    defaults: Defaults,
    stage: DesktopDefaultsStage,
) -> std::result::Result<fret_bootstrap::BootstrapBuilder<D>, fret_bootstrap::BootstrapError> {
    #[cfg(feature = "shadcn")]
    let apply_base = matches!(
        stage,
        DesktopDefaultsStage::Base | DesktopDefaultsStage::All
    );
    let apply_runtime = matches!(
        stage,
        DesktopDefaultsStage::Runtime | DesktopDefaultsStage::All
    );

    #[cfg(feature = "shadcn")]
    let builder = if apply_base && defaults.shadcn {
        builder.install_app(fret_ui_shadcn::app::install)
    } else {
        builder
    };

    if !apply_runtime {
        return Ok(builder);
    }

    // Always ensure an i18n backend exists unless the app provides one.
    let builder = builder.init_app(fret_bootstrap::install_default_i18n_backend);
    let _ = defaults;

    #[cfg(feature = "diagnostics")]
    let builder = if defaults.diagnostics {
        builder.with_default_diagnostics()
    } else {
        builder
    };

    #[cfg(feature = "config-files")]
    let builder = if defaults.config_files {
        builder.with_default_config_files()?
    } else {
        builder.with_command_default_keybindings()
    };

    #[cfg(not(feature = "config-files"))]
    let builder = builder.with_command_default_keybindings();

    #[cfg(feature = "ui-assets")]
    let builder = if defaults.ui_assets {
        let (image_budget_bytes, image_max_ready_entries, svg_budget_bytes, svg_max_ready_entries) =
            defaults
                .ui_assets_budgets
                .unwrap_or((64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096));
        builder.with_ui_assets_budgets(
            image_budget_bytes,
            image_max_ready_entries,
            svg_budget_bytes,
            svg_max_ready_entries,
        )
    } else {
        builder
    };

    #[cfg(feature = "icons")]
    let builder = if defaults.icons {
        builder.with_lucide_icons()
    } else {
        builder
    };

    #[cfg(feature = "preload-icon-svgs")]
    let builder = if defaults.preload_icon_svgs {
        builder.preload_icon_svgs_on_gpu_ready()
    } else {
        builder
    };

    Ok(builder)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn apply_desktop_defaults<D: fret_launch::WinitAppDriver + 'static>(
    builder: fret_bootstrap::BootstrapBuilder<D>,
) -> std::result::Result<fret_bootstrap::BootstrapBuilder<D>, fret_bootstrap::BootstrapError> {
    apply_desktop_defaults_with(builder, Defaults::default())
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn run_native_builder<D: fret_launch::WinitAppDriver + 'static>(
    builder: fret_bootstrap::BootstrapBuilder<D>,
    config: fret_launch::WinitRunnerConfig,
) -> Result<()> {
    let builder = builder.configure(move |runner_config| *runner_config = config);
    let builder = apply_desktop_defaults(builder).map_err(BootstrapError::from)?;
    builder.run().map_err(RunnerError::from)?;
    Ok(())
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop", feature = "shadcn"))]
pub(crate) fn shadcn_sync_theme_from_environment_on_global_changes<S>(
    app: &mut KernelApp,
    window: fret_core::AppWindowId,
    _ui: &mut fret_ui::UiTree<KernelApp>,
    _st: &mut S,
    changed: &[std::any::TypeId],
) {
    if !changed.contains(&std::any::TypeId::of::<fret_core::WindowMetricsService>()) {
        return;
    }
    let Some(config) = app.global::<fret_ui_shadcn::app::InstallConfig>().copied() else {
        return;
    };

    #[cfg(feature = "imui")]
    {
        let _ = fret_ui_editor::theme::sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change(
            app,
            changed,
            |app| {
                let _ = fret_ui_shadcn::advanced::sync_theme_from_environment(
                    app,
                    window,
                    config.base_color,
                    config.scheme,
                );
            },
        );
    }

    #[cfg(not(feature = "imui"))]
    {
        let _ = fret_ui_shadcn::advanced::sync_theme_from_environment(
            app,
            window,
            config.base_color,
            config.scheme,
        );
    }
}

#[cfg(all(test, not(target_arch = "wasm32"), feature = "desktop"))]
mod tests;

#[cfg(all(
    test,
    not(target_arch = "wasm32"),
    feature = "desktop",
    feature = "shadcn"
))]
mod theme_tests;

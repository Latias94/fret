//! Advanced native driver and builder escape hatches.

use super::KernelApp;

pub use crate::{UiAppBuilder, UiAppDriver};
pub use fret_bootstrap::ui_app_driver::ViewElements;

/// Create a golden-path native UI app builder on the explicit advanced surface.
///
/// This mirrors `fret-bootstrap`'s `ui_app(...)` helper while keeping author-facing code on the
/// `fret::advanced` surface.
pub fn ui_app<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut KernelApp, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, KernelApp>, &mut S) -> ViewElements,
) -> crate::UiAppBuilder<S> {
    ui_app_with_hooks(root_name, init_window, view, |driver| driver)
}

/// Create a golden-path native UI app builder on the explicit advanced surface, preserving the
/// driver hook configuration seam.
pub fn ui_app_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut KernelApp, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, KernelApp>, &mut S) -> ViewElements,
    configure: fn(crate::UiAppDriver<S>) -> crate::UiAppDriver<S>,
) -> crate::UiAppBuilder<S> {
    let driver = fret_bootstrap::ui_app_driver::UiAppDriver::new(root_name, init_window, view);
    let driver = configure(crate::UiAppDriver::new(driver))
        .into_inner()
        .into_fn_driver();
    crate::UiAppBuilder::from_bootstrap(fret_bootstrap::BootstrapBuilder::new(
        KernelApp::new(),
        driver,
    ))
}

/// Run a native desktop app using the advanced `FnDriver` escape hatch.
///
/// This is the recommended low-level path when the app wants the `fret` bootstrap/defaults story
/// but needs runner-level customization without teaching `WinitAppDriver` as the primary model.
pub fn run_native_with_fn_driver<D: 'static, S: 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: KernelApp,
    driver_state: D,
    create_window_state: fn(&mut D, &mut KernelApp, fret_core::AppWindowId) -> S,
    handle_event: for<'d, 'cx, 'e> fn(
        &'d mut D,
        fret_launch::WinitEventContext<'cx, S>,
        &'e fret_core::Event,
    ),
    render: for<'d, 'cx> fn(&'d mut D, fret_launch::WinitRenderContext<'cx, S>),
) -> crate::Result<()> {
    run_native_with_fn_driver_with_hooks(
        config,
        app,
        driver_state,
        create_window_state,
        handle_event,
        render,
        |_hooks| {},
    )
}

/// Run a native desktop app using the advanced `FnDriver` escape hatch, preserving hook
/// configuration.
pub fn run_native_with_fn_driver_with_hooks<D: 'static, S: 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: KernelApp,
    driver_state: D,
    create_window_state: fn(&mut D, &mut KernelApp, fret_core::AppWindowId) -> S,
    handle_event: for<'d, 'cx, 'e> fn(
        &'d mut D,
        fret_launch::WinitEventContext<'cx, S>,
        &'e fret_core::Event,
    ),
    render: for<'d, 'cx> fn(&'d mut D, fret_launch::WinitRenderContext<'cx, S>),
    configure_hooks: impl FnOnce(&mut fret_launch::FnDriverHooks<D, S>),
) -> crate::Result<()> {
    let builder = fret_bootstrap::BootstrapBuilder::new_fn_with_hooks(
        app,
        driver_state,
        create_window_state,
        handle_event,
        render,
        configure_hooks,
    );

    crate::builder::run_native_builder(builder, config)
}

/// Run a native desktop app using a preconfigured advanced `FnDriver` instance.
pub fn run_native_with_configured_fn_driver<D: 'static, S: 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: KernelApp,
    driver: fret_launch::FnDriver<D, S>,
) -> crate::Result<()> {
    let builder = fret_bootstrap::BootstrapBuilder::new(app, driver);
    crate::builder::run_native_builder(builder, config)
}

/// Advanced builder hooks that intentionally stay off the default `FretApp` surface.
pub trait FretAppAdvancedExt: Sized {
    /// Install wiring that needs `UiServices` during bootstrap.
    fn install(self, install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices)) -> Self;
}

impl FretAppAdvancedExt for crate::FretApp {
    fn install(self, install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices)) -> Self {
        self.install_services(install)
    }
}

/// Advanced `UiAppBuilder` hooks that are intentionally excluded from the default app path.
pub trait UiAppBuilderAdvancedExt: Sized {
    /// Install wiring that needs `UiServices` during bootstrap.
    fn install(self, install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices)) -> Self;

    /// Install custom GPU effects at the renderer boundary (ADR 0299).
    ///
    /// Note: the callback receives the **kernel** app type (`fret_app::App`, re-exported here as
    /// `KernelApp`), not the `fret::FretApp` builder-chain facade.
    fn install_custom_effects(
        self,
        install: fn(&mut KernelApp, &mut dyn fret_core::CustomEffectService),
    ) -> Self;

    /// Hook GPU-ready setup on the explicit advanced surface.
    fn on_gpu_ready(
        self,
        f: impl FnOnce(
            &mut KernelApp,
            &super::kernel::render::WgpuContext,
            &mut super::kernel::render::Renderer,
        ) + 'static,
    ) -> Self;
}

impl<S: 'static> UiAppBuilderAdvancedExt for crate::UiAppBuilder<S> {
    fn install(self, install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices)) -> Self {
        crate::UiAppBuilder::install_services(self, install)
    }

    fn install_custom_effects(
        self,
        install: fn(&mut KernelApp, &mut dyn fret_core::CustomEffectService),
    ) -> Self {
        crate::UiAppBuilder::install_custom_effects(self, install)
    }

    fn on_gpu_ready(
        self,
        f: impl FnOnce(
            &mut KernelApp,
            &super::kernel::render::WgpuContext,
            &mut super::kernel::render::Renderer,
        ) + 'static,
    ) -> Self {
        crate::UiAppBuilder::on_gpu_ready(self, f)
    }
}

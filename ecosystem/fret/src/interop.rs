//! Interop helpers for embedding "foreign UI" as isolated surfaces.
//!
//! These helpers are intentionally ecosystem-level and designed to keep kernel contracts stable:
//! - foreign UI is embedded as a render target surface + optional input forwarding,
//! - focus/IME/a11y are not shared across runtimes (isolation boundary).

pub mod embedded_viewport;

/// Run a native desktop app using the retained compatibility-driver path.
///
/// Prefer `fret::FretApp` / `UiAppBuilder` for general applications and
/// `fret::run_native_with_fn_driver(...)` for new advanced integrations. This helper exists for
/// low-level integrations that still implement `fret_launch::WinitAppDriver` directly while
/// wanting the higher-level defaults/bootstrap story from `fret`.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run_native_with_compat_driver<D: fret_launch::WinitAppDriver + 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: crate::advanced::KernelApp,
    driver: D,
) -> crate::Result<()> {
    let builder = fret_bootstrap::BootstrapBuilder::new(app, driver).configure(move |c| {
        *c = config;
    });

    let builder = crate::apply_desktop_defaults(builder).map_err(crate::BootstrapError::from)?;

    builder.run().map_err(crate::RunnerError::from)?;
    Ok(())
}

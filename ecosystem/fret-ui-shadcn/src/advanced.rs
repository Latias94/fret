//! Explicit advanced integration helpers for `fret-ui-shadcn`.
//!
//! These hooks are intentionally separate from the default `app::*` surface because they require
//! environment services or renderer-backed `UiServices`.

use fret_core::{AppWindowId, ColorScheme, UiServices, WindowMetricsService};

use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ShadcnAutoThemeState {
    last_applied: Option<(ShadcnBaseColor, ShadcnColorScheme)>,
}

/// Applies the shadcn theme for the given `base_color` and an environment-derived light/dark
/// scheme.
///
/// This is an advanced ecosystem policy helper. Call it from app/runner integration (for example
/// `handle_global_changes`) when `WindowMetricsService` changes.
///
/// Returns the scheme that was selected.
pub fn sync_theme_from_environment(
    app: &mut fret_app::App,
    window: AppWindowId,
    base_color: ShadcnBaseColor,
    default_scheme_when_unknown: ShadcnColorScheme,
) -> ShadcnColorScheme {
    let desired = app
        .global::<WindowMetricsService>()
        .and_then(|svc| svc.color_scheme(window))
        .map(|scheme| match scheme {
            ColorScheme::Light => ShadcnColorScheme::Light,
            ColorScheme::Dark => ShadcnColorScheme::Dark,
        })
        .unwrap_or(default_scheme_when_unknown);

    let should_apply =
        app.with_global_mut_untracked(ShadcnAutoThemeState::default, |state, _app| {
            if state.last_applied == Some((base_color, desired)) {
                return false;
            }
            state.last_applied = Some((base_color, desired));
            true
        });

    if should_apply {
        crate::shadcn_themes::apply_shadcn_new_york(app, base_color, desired);
    }

    desired
}

/// Advanced bootstrap-style integration entrypoint.
///
/// This mirrors `BootstrapBuilder::install(...)` which receives both the app and a `UiServices`
/// handle, but remains outside the default app-facing surface.
pub fn install_with_ui_services(app: &mut fret_app::App, services: &mut dyn UiServices) {
    let _ = services;
    crate::app::install(app);
}

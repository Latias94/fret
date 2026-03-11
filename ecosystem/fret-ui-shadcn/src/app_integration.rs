use fret_core::{AppWindowId, ColorScheme, UiServices, WindowMetricsService};

use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Theme installation configuration stored in app globals.
///
/// This is used by the `app-integration` helpers to keep a stable source of truth for what theme
/// should be applied when the environment (e.g. OS light/dark setting) changes.
pub struct InstallConfig {
    /// The base color family (e.g. Slate, Zinc, ...).
    pub base_color: ShadcnBaseColor,
    /// The preferred color scheme when the environment is unknown.
    pub scheme: ShadcnColorScheme,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            base_color: ShadcnBaseColor::Slate,
            scheme: ShadcnColorScheme::Light,
        }
    }
}

/// Install the default shadcn app integration into an app.
pub fn install(app: &mut fret_app::App) {
    install_with(app, InstallConfig::default());
}

/// Installs the default shadcn theme into the app and stores the configuration in app globals.
pub fn install_with(app: &mut fret_app::App, config: InstallConfig) {
    crate::shadcn_themes::apply_shadcn_new_york(app, config.base_color, config.scheme);
    app.with_global_mut_untracked(InstallConfig::default, |stored, _app| {
        *stored = config;
    });
}

/// Convenience wrapper around [`install_with`].
pub fn install_with_theme(
    app: &mut fret_app::App,
    base_color: ShadcnBaseColor,
    scheme: ShadcnColorScheme,
) {
    install_with(app, InstallConfig { base_color, scheme });
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ShadcnAutoThemeState {
    last_applied: Option<(ShadcnBaseColor, ShadcnColorScheme)>,
}

/// Applies the shadcn theme for the given `base_color` and an environment-derived light/dark scheme.
///
/// This is an **ecosystem policy helper**. Call it from app/runner integration (e.g.
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
pub fn install_with_services(app: &mut fret_app::App, services: &mut dyn UiServices) {
    let _ = services;
    install(app);
}

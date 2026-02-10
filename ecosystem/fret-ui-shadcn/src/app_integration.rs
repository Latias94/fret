use fret_core::{AppWindowId, ColorScheme, UiServices, WindowMetricsService};

use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadcnInstallConfig {
    pub base_color: ShadcnBaseColor,
    pub scheme: ShadcnColorScheme,
}

impl Default for ShadcnInstallConfig {
    fn default() -> Self {
        Self {
            base_color: ShadcnBaseColor::Slate,
            scheme: ShadcnColorScheme::Light,
        }
    }
}

pub fn install_app(app: &mut fret_app::App) {
    install_app_with(app, ShadcnInstallConfig::default());
}

pub fn install_app_with(app: &mut fret_app::App, config: ShadcnInstallConfig) {
    crate::shadcn_themes::apply_shadcn_new_york_v4(app, config.base_color, config.scheme);
}

pub fn install_app_with_theme(
    app: &mut fret_app::App,
    base_color: ShadcnBaseColor,
    scheme: ShadcnColorScheme,
) {
    crate::shadcn_themes::apply_shadcn_new_york_v4(app, base_color, scheme);
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

    let should_apply = app.with_global_mut(ShadcnAutoThemeState::default, |state, _app| {
        if state.last_applied == Some((base_color, desired)) {
            return false;
        }
        state.last_applied = Some((base_color, desired));
        true
    });

    if should_apply {
        crate::shadcn_themes::apply_shadcn_new_york_v4(app, base_color, desired);
    }

    desired
}

pub fn install(app: &mut fret_app::App, services: &mut dyn UiServices) {
    let _ = services;
    install_app(app);
}

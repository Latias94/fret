use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Theme installation configuration stored in app globals.
///
/// This is used by the `app` helpers to keep a stable source of truth for what theme should be
/// applied when the environment (e.g. OS light/dark setting) changes.
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

use fret_core::UiServices;

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

pub fn install(app: &mut fret_app::App, services: &mut dyn UiServices) {
    let _ = services;
    install_app(app);
}

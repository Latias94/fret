const LIB_RS: &str = include_str!("lib.rs");
const APP_RS: &str = include_str!("app.rs");
const ADVANCED_RS: &str = include_str!("advanced.rs");
const README: &str = include_str!("../README.md");

#[test]
fn app_integration_stays_under_explicit_app_module() {
    assert!(README.contains("`fret_ui_shadcn::app::{install, install_with, ...}`"));
    assert!(LIB_RS.contains("pub mod app;"));
    assert!(APP_RS.contains("pub struct InstallConfig"));
    assert!(APP_RS.contains("pub fn install(app: &mut fret_app::App)"));
    assert!(APP_RS.contains("pub fn install_with("));
    assert!(APP_RS.contains("pub fn install_with_theme("));
    assert!(!APP_RS.contains("sync_theme_from_environment"));
    assert!(!APP_RS.contains("install_with_ui_services"));
    assert!(!README.contains("`fret_ui_shadcn::install_app(...)`"));
}

#[test]
fn curated_facade_keeps_app_theme_and_raw_seams_explicit() {
    assert!(README.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
    assert!(README.contains("let _button = shadcn::Button::new(\"Save\");"));
    assert!(!README.contains("recipes/components stay under `fret_ui_shadcn::*`"));

    assert!(LIB_RS.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
    assert!(LIB_RS.contains("crate root as a raw escape hatch"));
    assert!(LIB_RS.contains("pub mod facade {"));
    assert!(LIB_RS.contains("pub mod themes {"));
    assert!(LIB_RS.contains("pub mod raw {"));
    assert!(LIB_RS.contains(
        "pub use crate::app::{InstallConfig, install, install_with, install_with_theme};"
    ));
    assert!(LIB_RS.contains("pub use crate::shadcn_themes::{"));
    assert!(LIB_RS.contains("pub use crate::*;"));
    assert!(LIB_RS.contains("pub mod advanced;"));
    assert!(ADVANCED_RS.contains("pub fn sync_theme_from_environment("));
    assert!(ADVANCED_RS.contains("pub fn install_with_ui_services("));
}

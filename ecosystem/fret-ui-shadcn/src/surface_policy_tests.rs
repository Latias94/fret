const LIB_RS: &str = include_str!("lib.rs");
const README: &str = include_str!("../README.md");

#[test]
fn app_integration_stays_under_explicit_app_module() {
    assert!(README.contains("`fret_ui_shadcn::app::{install, install_with, ...}`"));
    assert!(LIB_RS.contains("pub mod app {"));
    assert!(LIB_RS.contains("InstallConfig, install, install_with, install_with_services"));
    assert!(!LIB_RS.contains("pub use app_integration::{"));
    assert!(!README.contains("`fret_ui_shadcn::install_app(...)`"));
}

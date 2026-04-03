const FRET_LIB_RS: &str = include_str!("../src/lib.rs");
const DOCS_README: &str = include_str!("../../../docs/README.md");
const ROADMAP: &str = include_str!("../../../docs/roadmap.md");
const AUTHORING_GOLDEN_PATH: &str = include_str!("../../../docs/authoring-golden-path-v2.md");
const FEARLESS_REFACTORING: &str = include_str!("../../../docs/fearless-refactoring.md");
const FIRST_HOUR: &str = include_str!("../../../docs/first-hour.md");
const TODO_APP_GOLDEN_PATH: &str = include_str!("../../../docs/examples/todo-app-golden-path.md");

fn app_prelude_slice() -> &'static str {
    let app_start = FRET_LIB_RS
        .find("pub mod app {")
        .expect("app module marker should exist");
    let component_start = FRET_LIB_RS
        .find("pub mod component {")
        .expect("component module marker should exist");
    &FRET_LIB_RS[app_start..component_start]
}

fn advanced_public_slice() -> &'static str {
    let advanced_start = FRET_LIB_RS
        .find("pub mod advanced {")
        .expect("advanced module marker should exist");
    let tests_start = FRET_LIB_RS
        .find("#[cfg(test)]")
        .unwrap_or(FRET_LIB_RS.len());
    &FRET_LIB_RS[advanced_start..tests_start]
}

#[test]
fn raw_state_hook_is_exposed_on_the_advanced_surface() {
    let advanced_slice = advanced_public_slice();
    assert!(!advanced_slice.contains("AppUiRawStateExt"));
    assert!(advanced_slice.contains("pub use crate::view::AppUiRawModelExt;"));
    assert!(!app_prelude_slice().contains("AppUiRawModelExt"));
}

#[test]
fn default_docs_keep_raw_state_as_an_explicit_advanced_seam() {
    assert!(!AUTHORING_GOLDEN_PATH.contains("AppUiRawStateExt"));
    assert!(AUTHORING_GOLDEN_PATH.contains("use fret::advanced::AppUiRawModelExt;"));
    assert!(AUTHORING_GOLDEN_PATH.contains("cx.raw_model::<T>()"));
    assert!(!FEARLESS_REFACTORING.contains("AppUiRawStateExt"));
    assert!(FEARLESS_REFACTORING.contains("use fret::advanced::AppUiRawModelExt;"));
    assert!(!FIRST_HOUR.contains("AppUiRawStateExt"));
    assert!(FIRST_HOUR.contains("use fret::advanced::AppUiRawModelExt;"));
    assert!(FIRST_HOUR.contains("cx.raw_model::<T>()"));
    assert!(!TODO_APP_GOLDEN_PATH.contains("AppUiRawStateExt"));
    assert!(TODO_APP_GOLDEN_PATH.contains("use fret::advanced::AppUiRawModelExt;"));
}

#[test]
fn docs_indices_use_the_current_raw_model_name() {
    assert!(DOCS_README.contains("`AppUiRawModelExt::raw_model::<T>()`"));
    assert!(!DOCS_README.contains("keep\n    `use_state` as the explicit raw-model seam"));
    assert!(ROADMAP.contains("`AppUiRawModelExt::raw_model::<T>()`"));
    assert!(!ROADMAP.contains("`use_state` as the advanced raw-model seam"));
}

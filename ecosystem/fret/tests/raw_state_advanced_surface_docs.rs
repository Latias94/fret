const FRET_LIB_RS: &str = include_str!("../src/lib.rs");
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

#[test]
fn raw_state_hook_is_exposed_on_the_advanced_surface() {
    let advanced_start = FRET_LIB_RS
        .find("pub mod advanced {")
        .expect("advanced module marker should exist");
    let advanced_slice = &FRET_LIB_RS[advanced_start..];
    assert!(advanced_slice.contains("pub use crate::view::AppUiRawStateExt;"));
    assert!(!app_prelude_slice().contains("AppUiRawStateExt"));
}

#[test]
fn default_docs_keep_raw_state_as_an_explicit_advanced_seam() {
    assert!(AUTHORING_GOLDEN_PATH.contains("use fret::advanced::AppUiRawStateExt;"));
    assert!(AUTHORING_GOLDEN_PATH.contains("cx.use_state::<T>()"));
    assert!(FEARLESS_REFACTORING.contains("use fret::advanced::AppUiRawStateExt;"));
    assert!(FIRST_HOUR.contains("use fret::advanced::AppUiRawStateExt;"));
    assert!(TODO_APP_GOLDEN_PATH.contains("use fret::advanced::AppUiRawStateExt;"));
}

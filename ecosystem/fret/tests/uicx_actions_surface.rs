const FRET_LIB_RS: &str = include_str!("../src/lib.rs");
const VIEW_RS: &str = include_str!("../src/view.rs");

fn app_module_slice() -> &'static str {
    let app_start = FRET_LIB_RS
        .find("pub mod app {")
        .expect("app module marker should exist");
    let component_start = FRET_LIB_RS
        .find("pub mod component {")
        .expect("component module marker should exist");
    &FRET_LIB_RS[app_start..component_start]
}

fn app_prelude_slice() -> &'static str {
    let app_slice = app_module_slice();
    let prelude_start = app_slice
        .find("pub mod prelude {")
        .expect("app prelude marker should exist");
    &app_slice[prelude_start..]
}

fn advanced_prelude_slice() -> &'static str {
    let advanced_start = FRET_LIB_RS
        .find("pub mod advanced {")
        .expect("advanced module marker should exist");
    let advanced_slice = &FRET_LIB_RS[advanced_start..];
    let prelude_start = advanced_slice
        .find("pub mod prelude {")
        .expect("advanced prelude marker should exist");
    &advanced_slice[prelude_start..]
}

#[test]
fn app_render_actions_ext_is_part_of_the_default_and_advanced_preludes() {
    assert!(VIEW_RS.contains("pub trait AppRenderActionsExt"));
    assert!(app_prelude_slice().contains("pub use crate::view::AppRenderActionsExt as _;"));
    assert!(advanced_prelude_slice().contains("pub use crate::view::AppRenderActionsExt as _;"));
    assert!(!app_prelude_slice().contains("pub use crate::view::UiCxActionsExt as _;"));
    assert!(!advanced_prelude_slice().contains("pub use crate::view::UiCxActionsExt as _;"));
}

#[test]
fn app_lane_exports_only_canonical_grouped_helpers() {
    let app_slice = app_module_slice();
    assert!(app_slice.contains("AppRenderActionsExt"));
    assert!(app_slice.contains("AppRenderDataExt"));
    assert!(!app_slice.contains("pub use crate::view::{UiCxActionsExt, UiCxDataExt};"));
    assert!(!app_slice.contains("pub use crate::view::QueryHandleReadLayoutExt;"));
    assert!(app_prelude_slice().contains("pub use crate::view::QueryHandleReadLayoutExt as _;"));
}

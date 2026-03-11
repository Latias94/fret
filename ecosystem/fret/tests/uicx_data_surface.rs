const FRET_LIB_RS: &str = include_str!("../src/lib.rs");
const VIEW_RS: &str = include_str!("../src/view.rs");
const ASYNC_PLAYGROUND_DEMO: &str =
    include_str!("../../../apps/fret-examples/src/async_playground_demo.rs");
const MARKDOWN_DEMO: &str = include_str!("../../../apps/fret-examples/src/markdown_demo.rs");

fn app_prelude_slice() -> &'static str {
    let app_start = FRET_LIB_RS
        .find("pub mod app {")
        .expect("app module marker should exist");
    let component_start = FRET_LIB_RS
        .find("pub mod component {")
        .expect("component module marker should exist");
    &FRET_LIB_RS[app_start..component_start]
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
fn uicx_data_ext_is_part_of_the_default_and_advanced_preludes() {
    assert!(VIEW_RS.contains("pub trait UiCxDataExt"));
    assert!(app_prelude_slice().contains("pub use crate::view::UiCxDataExt as _;"));
    assert!(advanced_prelude_slice().contains("pub use crate::view::UiCxDataExt as _;"));
}

#[test]
fn helper_heavy_examples_use_grouped_data_helpers() {
    assert!(ASYNC_PLAYGROUND_DEMO.contains("cx.data().query("));
    assert!(ASYNC_PLAYGROUND_DEMO.contains("cx.data().query_async("));
    assert!(!ASYNC_PLAYGROUND_DEMO.contains("cx.use_query("));
    assert!(!ASYNC_PLAYGROUND_DEMO.contains("cx.use_query_async("));

    assert!(MARKDOWN_DEMO.contains("cx.data().query("));
    assert!(!MARKDOWN_DEMO.contains("cx.use_query("));
}

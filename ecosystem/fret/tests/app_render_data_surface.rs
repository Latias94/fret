const FRET_LIB_RS: &str = include_str!("../src/lib.rs");
const VIEW_RS: &str = include_str!("../src/view.rs");
const ASYNC_PLAYGROUND_DEMO: &str =
    include_str!("../../../apps/fret-examples/src/async_playground_demo.rs");
const QUERY_DEMO: &str = include_str!("../../../apps/fret-examples/src/query_demo.rs");
const QUERY_ASYNC_TOKIO_DEMO: &str =
    include_str!("../../../apps/fret-examples/src/query_async_tokio_demo.rs");
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
fn app_render_data_ext_is_part_of_the_default_and_advanced_preludes() {
    assert!(VIEW_RS.contains("pub trait AppRenderDataExt"));
    assert!(app_prelude_slice().contains("pub use crate::view::AppRenderDataExt as _;"));
    assert!(advanced_prelude_slice().contains("pub use crate::view::AppRenderDataExt as _;"));
    assert!(!app_prelude_slice().contains("pub use crate::view::UiCxDataExt as _;"));
    assert!(!advanced_prelude_slice().contains("pub use crate::view::UiCxDataExt as _;"));
    assert!(!VIEW_RS.contains("pub use AppRenderDataExt as UiCxDataExt;"));
    assert!(!VIEW_RS.contains("pub use AppRenderData as UiCxData;"));
}

#[test]
fn helper_heavy_examples_use_grouped_data_helpers() {
    assert!(ASYNC_PLAYGROUND_DEMO.contains("cx.data().query("));
    assert!(ASYNC_PLAYGROUND_DEMO.contains("cx.data().selector_layout("));
    assert!(ASYNC_PLAYGROUND_DEMO.contains("cx.data().invalidate_query("));
    assert!(ASYNC_PLAYGROUND_DEMO.contains("cx.data().cancel_query("));
    assert!(ASYNC_PLAYGROUND_DEMO.contains("cx.data().invalidate_query_namespace("));
    assert!(ASYNC_PLAYGROUND_DEMO.contains("cx.data().query_snapshot_entry("));
    assert!(!ASYNC_PLAYGROUND_DEMO.contains("cx.use_query("));
    assert!(!ASYNC_PLAYGROUND_DEMO.contains("with_query_client("));

    assert!(QUERY_DEMO.contains("cx.data().invalidate_query("));
    assert!(QUERY_DEMO.contains("cx.data().invalidate_query_namespace("));
    assert!(!QUERY_DEMO.contains("with_query_client("));

    assert!(QUERY_ASYNC_TOKIO_DEMO.contains("cx.data().query_async("));
    assert!(QUERY_ASYNC_TOKIO_DEMO.contains("cx.data().invalidate_query("));
    assert!(QUERY_ASYNC_TOKIO_DEMO.contains("cx.data().invalidate_query_namespace("));
    assert!(!QUERY_ASYNC_TOKIO_DEMO.contains("cx.use_query_async("));
    assert!(!QUERY_ASYNC_TOKIO_DEMO.contains("with_query_client("));

    assert!(MARKDOWN_DEMO.contains("cx.data().query("));
    assert!(MARKDOWN_DEMO.contains("cx.data().invalidate_query_namespace("));
    assert!(!MARKDOWN_DEMO.contains("cx.use_query("));
    assert!(!MARKDOWN_DEMO.contains("cx.use_selector("));
    assert!(!MARKDOWN_DEMO.contains("with_query_client("));
}

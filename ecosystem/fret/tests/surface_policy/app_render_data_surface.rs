const APP_PRELUDE_RS: &str = include_str!("../../src/app/prelude.rs");
const ADVANCED_PRELUDE_RS: &str = include_str!("../../src/advanced/prelude.rs");
const VIEW_RS: &str = include_str!("../../src/view.rs");
const VIEW_DATA_RS: &str = include_str!("../../src/view/data.rs");
const VIEW_DATA_RENDER_RS: &str = include_str!("../../src/view/data/render.rs");
const ASYNC_PLAYGROUND_DEMO: &str =
    include_str!("../../../../apps/fret-examples/src/async_playground_demo.rs");
const QUERY_DEMO: &str = include_str!("../../../../apps/fret-examples/src/query_demo.rs");
const QUERY_ASYNC_TOKIO_DEMO: &str =
    include_str!("../../../../apps/fret-examples/src/query_async_tokio_demo.rs");
const MARKDOWN_DEMO: &str = include_str!("../../../../apps/fret-examples/src/markdown_demo.rs");

fn app_prelude_slice() -> &'static str {
    APP_PRELUDE_RS
}

fn advanced_prelude_slice() -> &'static str {
    ADVANCED_PRELUDE_RS
}

fn view_api_surface() -> String {
    let view_api = VIEW_RS
        .split("\n#[cfg(test)]\nmod tests")
        .next()
        .expect("view.rs test module marker should exist");
    format!("{view_api}\n{VIEW_DATA_RS}\n{VIEW_DATA_RENDER_RS}")
}

#[test]
fn app_render_data_ext_is_part_of_the_default_and_advanced_preludes() {
    let view_api = view_api_surface();
    assert!(view_api.contains("pub trait AppRenderDataExt"));
    assert!(app_prelude_slice().contains("pub use crate::view::AppRenderDataExt as _;"));
    assert!(advanced_prelude_slice().contains("pub use crate::view::AppRenderDataExt as _;"));
    assert!(!app_prelude_slice().contains("pub use crate::view::UiCxDataExt as _;"));
    assert!(!advanced_prelude_slice().contains("pub use crate::view::UiCxDataExt as _;"));
    assert!(!view_api.contains("pub use AppRenderDataExt as UiCxDataExt;"));
    assert!(!view_api.contains("pub use AppRenderData as UiCxData;"));
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

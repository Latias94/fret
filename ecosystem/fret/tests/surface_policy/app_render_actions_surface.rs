const APP_RS: &str = include_str!("../../src/app.rs");
const APP_PRELUDE_RS: &str = include_str!("../../src/app/prelude.rs");
const ADVANCED_PRELUDE_RS: &str = include_str!("../../src/advanced/prelude.rs");
const VIEW_RS: &str = include_str!("../../src/view.rs");
const VIEW_ACTIONS_RS: &str = include_str!("../../src/view/actions.rs");

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
    format!("{view_api}\n{VIEW_ACTIONS_RS}")
}

#[test]
fn app_render_actions_ext_is_part_of_the_default_and_advanced_preludes() {
    let view_api = view_api_surface();
    assert!(view_api.contains("pub trait AppRenderActionsExt"));
    assert!(app_prelude_slice().contains("pub use crate::view::AppRenderActionsExt as _;"));
    assert!(advanced_prelude_slice().contains("pub use crate::view::AppRenderActionsExt as _;"));
    assert!(!app_prelude_slice().contains("pub use crate::view::UiCxActionsExt as _;"));
    assert!(!advanced_prelude_slice().contains("pub use crate::view::UiCxActionsExt as _;"));
}

#[test]
fn app_lane_exports_only_canonical_grouped_helpers() {
    let app_slice = APP_RS;
    assert!(app_slice.contains("AppRenderActionsExt"));
    assert!(app_slice.contains("AppRenderDataExt"));
    assert!(!app_slice.contains("pub use crate::view::{UiCxActionsExt, UiCxDataExt};"));
    assert!(!app_slice.contains("pub use crate::view::QueryHandleReadLayoutExt;"));
    assert!(app_prelude_slice().contains("pub use crate::view::QueryHandleReadLayoutExt as _;"));
}

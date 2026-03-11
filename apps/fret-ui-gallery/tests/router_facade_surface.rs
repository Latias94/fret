const ROUTER_DRIVER: &str = include_str!("../src/driver/router.rs");
const RUNTIME_DRIVER: &str = include_str!("../src/driver/runtime_driver.rs");
const RENDER_FLOW: &str = include_str!("../src/driver/render_flow.rs");
const SPEC: &str = include_str!("../src/spec.rs");

#[test]
fn ui_gallery_router_code_prefers_fret_router_facade() {
    assert!(ROUTER_DRIVER.contains("use fret::router::{"));
    assert!(!ROUTER_DRIVER.contains("use fret_router::{"));
    assert!(!ROUTER_DRIVER.contains("fret_router::"));

    assert!(RUNTIME_DRIVER.contains("use fret::router::{NavigationAction, Router};"));
    assert!(!RUNTIME_DRIVER.contains("use fret_router::{"));
    assert!(!RUNTIME_DRIVER.contains("fret_router::"));

    assert!(RENDER_FLOW.contains("fret::router::NavigationAction::Replace"));
    assert!(!RENDER_FLOW.contains("fret_router::"));

    assert!(SPEC.contains("fret::router::core::web::current_location()"));
    assert!(SPEC.contains("fret::router::core::first_query_value_from_search_or_hash("));
    assert!(!SPEC.contains("fret_router::"));
}

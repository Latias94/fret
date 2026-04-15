const FRET_LIB_RS: &str = include_str!("../src/lib.rs");
const VIEW_RS: &str = include_str!("../src/view.rs");
const COOKBOOK_SCAFFOLD: &str = include_str!("../../../apps/fret-cookbook/src/scaffold.rs");

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

#[test]
fn app_lane_exports_explicit_render_authoring_capability_surface() {
    let app_slice = app_module_slice();
    let app_prelude = app_prelude_slice();

    assert!(app_slice.contains("pub use fret_ui::ElementContextAccess;"));
    assert!(app_prelude.contains("pub use fret_ui_kit::IntoUiElementInExt as _;"));
    assert!(VIEW_RS.contains(
        "impl<'cx, 'a, H: UiHost> fret_ui::ElementContextAccess<'a, H> for AppUi<'cx, 'a, H> {"
    ));
}

#[test]
fn cookbook_scaffold_uses_explicit_context_access_and_late_landing_helpers() {
    assert!(COOKBOOK_SCAFFOLD.contains("Cx: fret::app::RenderContextAccess<'a, App>"));
    assert!(COOKBOOK_SCAFFOLD.contains("let theme = cx.theme_snapshot();"));
    assert!(COOKBOOK_SCAFFOLD.contains(".into_element_in(cx)"));
    assert!(!COOKBOOK_SCAFFOLD.contains("&mut UiCx<'_>"));
    assert!(!COOKBOOK_SCAFFOLD.contains("surface.into_element(cx);"));
    assert!(!COOKBOOK_SCAFFOLD.contains("cx.elements().theme().snapshot()"));
}

const FRET_LIB_RS: &str = include_str!("../src/lib.rs");
const VIEW_RS: &str = include_str!("../src/view.rs");
const COOKBOOK_SCAFFOLD: &str = include_str!("../../../apps/fret-cookbook/src/scaffold.rs");
const DEFAULT_SNIPPET_GATE: &str =
    include_str!("../../../tools/gate_no_raw_app_context_in_default_teaching_snippets.py");
const PRE_RELEASE_PY: &str = include_str!("../../../tools/pre_release.py");

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

fn public_surface_slice() -> &'static str {
    let tests_start = FRET_LIB_RS
        .find("#[cfg(test)]")
        .unwrap_or(FRET_LIB_RS.len());
    &FRET_LIB_RS[..tests_start]
}

#[test]
fn app_lane_exports_explicit_render_authoring_capability_surface() {
    let public_surface = public_surface_slice();
    let app_slice = app_module_slice();
    let app_prelude = app_prelude_slice();

    assert!(app_slice.contains("pub use crate::view::{"));
    assert!(app_slice.contains("pub use crate::AppComponentCx;"));
    assert!(app_slice.contains("pub use crate::AppRenderCx;"));
    assert!(app_slice.contains("AppRenderContext"));
    assert!(app_slice.contains("pub use fret_ui::ElementContextAccess;"));
    assert!(app_prelude.contains("pub use crate::app::AppRenderCx;"));
    assert!(app_prelude.contains("pub use crate::app::AppRenderContext;"));
    assert!(app_prelude.contains("pub use fret_ui_kit::IntoUiElementInExt as _;"));
    assert!(app_prelude.contains("pub use crate::{AppUi, Ui, UiChild, WindowId};"));
    assert!(!app_prelude.contains("pub use crate::{AppUi, Ui, UiChild, UiCx, WindowId};"));
    assert!(
        VIEW_RS
            .contains("pub trait AppRenderContext<'a>: RenderContextAccess<'a, crate::app::App>")
    );
    assert!(VIEW_RS.contains(
        "impl<'cx, 'a, H: UiHost> fret_ui::ElementContextAccess<'a, H> for AppUi<'cx, 'a, H> {"
    ));
    assert!(VIEW_RS.contains(
        "impl<'cx, 'a, H: UiHost> fret_ui_kit::command::ElementCommandGatingExt for AppUi<'cx, 'a, H> {"
    ));
    assert!(VIEW_RS.contains(
        "pub fn request_animation_frame(&mut self) {\n        self.cx.request_animation_frame();\n    }"
    ));
    assert!(VIEW_RS.contains(
        "pub fn set_continuous_frames(&mut self, enabled: bool) {\n        fret_ui_kit::declarative::scheduling::set_continuous_frames(self.cx, enabled);\n    }"
    ));
    assert!(VIEW_RS.contains("pub fn layout_query_bounds("));
    assert!(VIEW_RS.contains("pub fn layout_query_region_with_id<I>("));
    assert!(VIEW_RS.contains("pub fn layout_query_region<I>("));
    assert!(VIEW_RS.contains(
        "let mut carried_action_handlers = Some(std::mem::take(&mut self.action_handlers));"
    ));
    assert!(
        public_surface
            .contains("pub type AppRenderCx<'a> = fret_ui::ElementContext<'a, crate::app::App>;")
    );
    assert!(
        public_surface.contains("pub type AppComponentCx<'a> = ComponentCx<'a, crate::app::App>;")
    );
    assert!(public_surface.contains("Canonical app-hosted component/snippet context alias."));
    assert!(public_surface.contains("Use this for first-party examples, gallery snippets"));
    assert!(!public_surface.contains("pub type UiCx<'a>"));
    assert!(!public_surface.contains("Deprecated compatibility raw app element context alias"));
}

#[test]
fn default_teaching_snippet_gate_tracks_app_component_context_surface() {
    assert!(DEFAULT_SNIPPET_GATE.contains("default teaching snippets use AppComponentCx"));
    assert!(DEFAULT_SNIPPET_GATE.contains("AppComponentCx<'_>"));
    assert!(
        !DEFAULT_SNIPPET_GATE
            .contains("must keep the default app-facing helper signature on UiCx<'_>")
    );
    assert!(
        PRE_RELEASE_PY.contains("Teaching surfaces policy (default snippets use AppComponentCx)")
    );
}

#[test]
fn cookbook_scaffold_uses_explicit_context_access_and_late_landing_helpers() {
    assert!(COOKBOOK_SCAFFOLD.contains("Cx: AppRenderContext<'a>"));
    assert!(COOKBOOK_SCAFFOLD.contains("let theme = cx.theme_snapshot();"));
    assert!(COOKBOOK_SCAFFOLD.contains(".into_element_in(cx)"));
    assert!(!COOKBOOK_SCAFFOLD.contains("&mut UiCx<'_>"));
    assert!(!COOKBOOK_SCAFFOLD.contains("surface.into_element(cx);"));
    assert!(!COOKBOOK_SCAFFOLD.contains("cx.elements().theme().snapshot()"));
}

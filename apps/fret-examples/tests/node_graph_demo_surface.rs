#[test]
fn node_graph_demo_uses_default_app_prelude() {
    let source = include_str!("../src/node_graph_demo.rs");

    for needle in [
        "use fret::app::prelude::*;",
        "use fret::style::{Color, DashPatternV1};",
        ".view::<NodeGraphDemoView>()?",
        "impl View for NodeGraphDemoView",
        "fn init(app: &mut App, _window: WindowId) -> Self",
        "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
        "node_graph_surface_in(cx, props).into()",
    ] {
        assert!(
            source.contains(needle),
            "node_graph_demo should stay on the default app-facing view surface; missing `{needle}`",
        );
    }

    for forbidden in [
        "advanced::prelude::*",
        "component::prelude::*",
        "use fret_core::",
        "fret_core::",
    ] {
        assert!(
            !source.contains(forbidden),
            "node_graph_demo should not reintroduce broad prelude imports: `{forbidden}`",
        );
    }
}

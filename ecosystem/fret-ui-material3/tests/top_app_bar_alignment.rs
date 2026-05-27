use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size, UiServices};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

#[test]
fn top_app_bar_exposes_toolbar_semantics_role() {
    use fret_ui_material3::{TopAppBar, TopAppBarVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(220.0)),
    );

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = TopAppBar::new("TopAppBar")
                    .variant(TopAppBarVariant::Small)
                    .a11y_label("Material 3 Top App Bar")
                    .test_id("top-app-bar")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let node = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("top-app-bar"))
        })
        .expect("expected top-app-bar in semantics snapshot");

    assert_eq!(
        node.role,
        SemanticsRole::Toolbar,
        "expected top app bar semantics role to be Toolbar",
    );
}

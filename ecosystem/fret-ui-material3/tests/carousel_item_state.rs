//! Material 3 carousel item semantics, sizing, and elevation regression tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, NodeId, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsRole, Size,
    UiServices,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{CarouselItem, CarouselItemVariant};

mod interaction_harness;
mod support;

use support::events::pointer_move;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(240.0)),
    )
}

fn item_content<H: fret_ui::UiHost>(
    cx: &mut fret_ui::elements::ElementContext<'_, H>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    cx.container(props, |cx| vec![cx.text("Carousel item")])
}

fn render_item(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    interactive: bool,
    disabled: bool,
    test_id: &'static str,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut item = CarouselItem::new()
            .variant(CarouselItemVariant::Standard)
            .width(Px(180.0))
            .height(Px(96.0))
            .disabled(disabled)
            .a11y_label("Material carousel item")
            .test_id(test_id);
        if interactive {
            item = item.on_activate(Arc::new(|_host, _cx, _reason| {}));
        }

        let item = item.into_element(cx, |cx| vec![item_content(cx)]);
        vec![with_padding(cx, Px(32.0), item)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn paint(ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices) -> Scene {
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds(), &mut scene, 1.0);
    scene
}

fn shadow_count(scene: &Scene) -> usize {
    scene
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::ShadowRRect { .. }))
        .count()
}

fn center_of(ui: &UiTree<TestHost>, node: NodeId) -> Point {
    let rect = ui
        .debug_node_visual_bounds(node)
        .expect("expected carousel item visual bounds");
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

#[test]
fn non_interactive_carousel_item_is_group_not_disabled_button() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_item(
        &mut ui,
        &mut app,
        &mut services,
        window,
        false,
        false,
        "m3-carousel-static",
    );

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot");
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("m3-carousel-static"))
        .expect("expected static carousel item semantics node");

    assert_eq!(node.role, SemanticsRole::Group);
    assert_eq!(node.label.as_deref(), Some("Material carousel item"));
    assert!(
        !node.flags.disabled,
        "static carousel items are not disabled controls"
    );
    assert!(
        !node.actions.invoke,
        "static carousel items should not expose an invoke action"
    );
}

#[test]
fn explicit_carousel_item_size_constrains_root_and_chrome() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_item(
        &mut ui,
        &mut app,
        &mut services,
        window,
        true,
        false,
        "m3-carousel-sized",
    );

    for test_id in ["m3-carousel-sized", "m3-carousel-sized.chrome"] {
        let node = semantics_node_id_by_test_id(&ui, test_id).expect("expected carousel item node");
        let rect = ui
            .debug_node_visual_bounds(node)
            .expect("expected carousel item bounds");
        assert!(
            (rect.size.width.0 - 180.0).abs() < 0.1,
            "expected {test_id} width to match explicit CarouselItem width"
        );
        assert!(
            (rect.size.height.0 - 96.0).abs() < 0.1,
            "expected {test_id} height to match explicit CarouselItem height"
        );
    }
}

#[test]
fn interactive_carousel_item_animates_hover_elevation() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_item(
        &mut ui,
        &mut app,
        &mut services,
        window,
        true,
        false,
        "m3-carousel-interactive",
    );

    let initial = paint(&mut ui, &mut app, &mut services);
    assert_eq!(
        shadow_count(&initial),
        0,
        "carousel item should start at Material level0 elevation"
    );

    let node = semantics_node_id_by_test_id(&ui, "m3-carousel-interactive")
        .expect("expected carousel item semantics node");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), center_of(&ui, node)),
    );

    app.advance_frame();
    render_item(
        &mut ui,
        &mut app,
        &mut services,
        window,
        true,
        false,
        "m3-carousel-interactive",
    );
    let first_hover = paint(&mut ui, &mut app, &mut services);
    assert_eq!(
        shadow_count(&first_hover),
        0,
        "carousel item hover elevation should animate instead of snapping on the first hover frame"
    );

    let mut hovered_shadow_count = 0;
    for _ in 0..12 {
        app.advance_frame();
        render_item(
            &mut ui,
            &mut app,
            &mut services,
            window,
            true,
            false,
            "m3-carousel-interactive",
        );
        let scene = paint(&mut ui, &mut app, &mut services);
        hovered_shadow_count = shadow_count(&scene);
        if hovered_shadow_count >= 2 {
            break;
        }
    }

    assert!(
        hovered_shadow_count >= 2,
        "hovered carousel item should animate toward Material level1 shadow"
    );
}

//! Material 3 card semantics, elevation, and state regression tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, NodeId, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsRole, Size,
    UiServices,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{Card, CardVariant};

mod support;

use support::events::pointer_move;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(220.0)),
    )
}

fn card_content<H: fret_ui::UiHost>(
    cx: &mut fret_ui::elements::ElementContext<'_, H>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Px(Px(180.0));
    props.layout.size.height = Length::Px(Px(72.0));
    cx.container(props, |cx| vec![cx.text("Card content")])
}

fn render_card(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    variant: CardVariant,
    interactive: bool,
    disabled: bool,
    test_id: &'static str,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut card = Card::new()
            .variant(variant)
            .disabled(disabled)
            .a11y_label("Material card")
            .test_id(test_id);
        if interactive {
            card = card.on_activate(Arc::new(|_host, _cx, _reason| {}));
        }

        let card = card.into_element(cx, |cx| vec![card_content(cx)]);
        vec![with_padding(cx, Px(32.0), card)]
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
        .expect("expected card visual bounds");
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

#[test]
fn non_interactive_card_is_group_not_disabled_button() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_card(
        &mut ui,
        &mut app,
        &mut services,
        window,
        CardVariant::Outlined,
        false,
        false,
        "m3-card-static",
    );

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot");
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("m3-card-static"))
        .expect("expected static card semantics node");

    assert_eq!(node.role, SemanticsRole::Group);
    assert_eq!(node.label.as_deref(), Some("Material card"));
    assert!(
        !node.flags.disabled,
        "static cards are not disabled controls"
    );
    assert!(
        !node.actions.invoke,
        "static cards should not expose an invoke action"
    );
}

#[test]
fn interactive_filled_card_animates_hover_elevation() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_card(
        &mut ui,
        &mut app,
        &mut services,
        window,
        CardVariant::Filled,
        true,
        false,
        "m3-card-filled",
    );

    let initial = paint(&mut ui, &mut app, &mut services);
    assert_eq!(
        shadow_count(&initial),
        0,
        "filled card should start at Material level0 elevation"
    );

    let node =
        semantics_node_id_by_test_id(&ui, "m3-card-filled").expect("expected card semantics node");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), center_of(&ui, node)),
    );

    app.advance_frame();
    render_card(
        &mut ui,
        &mut app,
        &mut services,
        window,
        CardVariant::Filled,
        true,
        false,
        "m3-card-filled",
    );
    let first_hover = paint(&mut ui, &mut app, &mut services);
    assert_eq!(
        shadow_count(&first_hover),
        0,
        "card hover elevation should animate instead of snapping on the first hover frame"
    );

    let mut hovered_shadow_count = 0;
    for _ in 0..12 {
        app.advance_frame();
        render_card(
            &mut ui,
            &mut app,
            &mut services,
            window,
            CardVariant::Filled,
            true,
            false,
            "m3-card-filled",
        );
        let scene = paint(&mut ui, &mut app, &mut services);
        hovered_shadow_count = shadow_count(&scene);
        if hovered_shadow_count >= 2 {
            break;
        }
    }

    assert!(
        hovered_shadow_count >= 2,
        "hovered filled card should animate toward Material level1 shadow"
    );
}

use fret_core::{
    AppWindowId, NodeId, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsRole, Size,
    UiServices,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{Button, ButtonVariant};

mod interaction_harness;
mod support;

use support::events::pointer_move;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    )
}

fn render_button(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    variant: ButtonVariant,
    disabled: bool,
    test_id: &'static str,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let button = Button::new("Action")
            .variant(variant)
            .disabled(disabled)
            .test_id(test_id)
            .into_element(cx);
        vec![with_padding(cx, Px(32.0), button)]
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
        .expect("expected button visual bounds");
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

#[test]
fn elevated_button_paints_material_default_elevation_shadow() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_button(
        &mut ui,
        &mut app,
        &mut services,
        window,
        ButtonVariant::Elevated,
        false,
        "m3-elevated-button",
    );

    let scene = paint(&mut ui, &mut app, &mut services);
    assert!(
        shadow_count(&scene) >= 2,
        "expected elevated button to paint Material key and ambient shadow layers"
    );
}

#[test]
fn filled_button_hover_animates_to_material_elevation_shadow() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_button(
        &mut ui,
        &mut app,
        &mut services,
        window,
        ButtonVariant::Filled,
        false,
        "m3-filled-button",
    );

    let initial = paint(&mut ui, &mut app, &mut services);
    assert_eq!(
        shadow_count(&initial),
        0,
        "filled button should start at Material level0 elevation"
    );

    let node = semantics_node_id_by_test_id(&ui, "m3-filled-button")
        .expect("expected filled button semantics node");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), center_of(&ui, node)),
    );

    let mut hovered_shadow_count = 0;
    for _ in 0..12 {
        app.advance_frame();
        render_button(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ButtonVariant::Filled,
            false,
            "m3-filled-button",
        );
        let scene = paint(&mut ui, &mut app, &mut services);
        hovered_shadow_count = shadow_count(&scene);
        if hovered_shadow_count >= 2 {
            break;
        }
    }

    assert!(
        hovered_shadow_count >= 2,
        "expected hovered filled button to animate toward Material level1 shadow"
    );
}

#[test]
fn button_semantics_expose_role_label_and_disabled_state() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_button(
        &mut ui,
        &mut app,
        &mut services,
        window,
        ButtonVariant::Outlined,
        true,
        "m3-disabled-button",
    );

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot");
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("m3-disabled-button"))
        .expect("expected disabled button semantics node");

    assert_eq!(node.role, SemanticsRole::Button);
    assert_eq!(node.label.as_deref(), Some("Action"));
    assert!(node.flags.disabled, "expected disabled semantics flag");
    assert!(
        !node.actions.invoke,
        "expected disabled button not to expose invoke action"
    );
}

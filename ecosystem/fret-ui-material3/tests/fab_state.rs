#![cfg(feature = "diagnostics")]

//! Material 3 FAB sizing, semantics, and elevation-motion regression tests.

use fret_core::{
    AppWindowId, NodeId, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsRole, Size,
    UiServices,
};
use fret_icons::ids;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{Fab, FabSize, FabVariant};

mod support;

use support::events::pointer_move;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    )
}

fn render_fab(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    fab: Fab,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let fab = fab.into_element(cx);
        vec![with_padding(cx, Px(32.0), fab)]
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

fn live_test_id_bounds(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .find_map(|m| {
            ui.debug_node_visual_bounds(m.node)
                .or_else(|| ui.debug_node_bounds(m.node))
        })
        .unwrap_or_else(|| panic!("expected live bounds for test_id {id}"))
}

fn semantics_bounds(ui: &UiTree<TestHost>, test_id: &str) -> Rect {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"));
    ui.debug_node_visual_bounds(node)
        .unwrap_or_else(|| panic!("expected visual bounds for test_id {test_id}"))
}

fn assert_size_close(bounds: Rect, width: f32, height: f32, context: &str) {
    let width_delta = (bounds.size.width.0 - width).abs();
    let height_delta = (bounds.size.height.0 - height).abs();
    assert!(
        width_delta <= 0.5 && height_delta <= 0.5,
        "{context}: expected {width}x{height}px, got {}x{}px",
        bounds.size.width.0,
        bounds.size.height.0
    );
}

fn assert_at_least_size(bounds: Rect, width: f32, height: f32, context: &str) {
    assert!(
        bounds.size.width.0 + 0.5 >= width && bounds.size.height.0 + 0.5 >= height,
        "{context}: expected at least {width}x{height}px, got {}x{}px",
        bounds.size.width.0,
        bounds.size.height.0
    );
}

fn center_of(ui: &UiTree<TestHost>, node: NodeId) -> Point {
    let rect = ui
        .debug_node_visual_bounds(node)
        .expect("expected FAB visual bounds");
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

fn shadow_signature(scene: &Scene) -> Vec<(f32, f32, f32, f32)> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::ShadowRRect {
                offset,
                spread,
                blur_radius,
                color,
                ..
            } => Some((offset.y.0, spread.0, blur_radius.0, color.a)),
            _ => None,
        })
        .collect()
}

#[test]
fn fab_semantics_expose_role_label_and_disabled_state() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_fab(
        &mut ui,
        &mut app,
        &mut services,
        window,
        Fab::new(ids::ui::PLUS)
            .a11y_label("Create item")
            .disabled(true)
            .test_id("m3-fab-disabled"),
    );

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot");
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("m3-fab-disabled"))
        .expect("expected disabled FAB semantics node");

    assert_eq!(node.role, SemanticsRole::Button);
    assert_eq!(node.label.as_deref(), Some("Create item"));
    assert!(node.flags.disabled, "expected disabled semantics flag");
    assert!(
        !node.actions.invoke,
        "expected disabled FAB not to expose invoke action"
    );
}

#[test]
fn icon_fab_sizes_use_material_chrome_inside_interactive_target() {
    let cases = [
        (
            "m3-fab-small",
            Fab::new(ids::ui::PLUS).size(FabSize::Small),
            48.0,
            40.0,
        ),
        ("m3-fab-regular", Fab::new(ids::ui::PLUS), 56.0, 56.0),
        (
            "m3-fab-medium",
            Fab::new(ids::ui::PLUS).size(FabSize::Medium),
            80.0,
            80.0,
        ),
        (
            "m3-fab-large",
            Fab::new(ids::ui::PLUS).size(FabSize::Large),
            96.0,
            96.0,
        ),
    ];

    for (test_id, fab, root_size, chrome_size) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        render_fab(
            &mut ui,
            &mut app,
            &mut services,
            window,
            fab.a11y_label("Create").test_id(test_id),
        );

        assert_size_close(
            semantics_bounds(&ui, test_id),
            root_size,
            root_size,
            &format!("{test_id} root"),
        );
        assert_size_close(
            live_test_id_bounds(&ui, &app, window, &format!("{test_id}.chrome")),
            chrome_size,
            chrome_size,
            &format!("{test_id} chrome"),
        );
    }
}

#[test]
fn extended_fab_sizes_use_material_height_and_min_width() {
    let cases = [
        (
            "m3-extended-fab-regular",
            Fab::new(ids::ui::PLUS),
            80.0,
            56.0,
        ),
        (
            "m3-extended-fab-small",
            Fab::new(ids::ui::PLUS).size(FabSize::Small),
            56.0,
            56.0,
        ),
        (
            "m3-extended-fab-medium",
            Fab::new(ids::ui::PLUS).size(FabSize::Medium),
            80.0,
            80.0,
        ),
        (
            "m3-extended-fab-large",
            Fab::new(ids::ui::PLUS).size(FabSize::Large),
            96.0,
            96.0,
        ),
    ];

    for (test_id, fab, min_width, height) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        render_fab(
            &mut ui,
            &mut app,
            &mut services,
            window,
            fab.label("Go").a11y_label("Go").test_id(test_id),
        );

        assert_at_least_size(
            live_test_id_bounds(&ui, &app, window, &format!("{test_id}.chrome")),
            min_width,
            height,
            &format!("{test_id} chrome"),
        );
    }
}

#[test]
fn primary_fab_hover_elevation_animates_without_first_frame_jump() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        render_fab(
            ui,
            app,
            services,
            window,
            Fab::new(ids::ui::PLUS)
                .variant(FabVariant::Primary)
                .a11y_label("Create")
                .test_id("m3-fab-primary"),
        );
    };

    render(&mut ui, &mut app, &mut services);

    let initial_signature = shadow_signature(&paint(&mut ui, &mut app, &mut services));
    assert!(
        !initial_signature.is_empty(),
        "expected primary FAB to paint Material default elevation"
    );

    let node =
        semantics_node_id_by_test_id(&ui, "m3-fab-primary").expect("expected FAB semantics node");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), center_of(&ui, node)),
    );

    app.advance_frame();
    render(&mut ui, &mut app, &mut services);
    let first_hover_signature = shadow_signature(&paint(&mut ui, &mut app, &mut services));
    assert_eq!(
        first_hover_signature, initial_signature,
        "FAB hover elevation should animate instead of snapping on the first hover frame"
    );

    let mut settled_signature = first_hover_signature;
    for _ in 0..12 {
        app.advance_frame();
        render(&mut ui, &mut app, &mut services);
        settled_signature = shadow_signature(&paint(&mut ui, &mut app, &mut services));
        if settled_signature != initial_signature {
            break;
        }
    }

    assert_ne!(
        settled_signature, initial_signature,
        "hovered FAB should animate toward the Material hover elevation shadow"
    );
}

#[test]
fn lowered_primary_fab_uses_lowered_material_elevation() {
    let signature_for = |fab: Fab, test_id: &'static str| {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        render_fab(
            &mut ui,
            &mut app,
            &mut services,
            window,
            fab.variant(FabVariant::Primary)
                .a11y_label("Create")
                .test_id(test_id),
        );

        shadow_signature(&paint(&mut ui, &mut app, &mut services))
    };

    let normal = signature_for(Fab::new(ids::ui::PLUS), "m3-fab-primary-normal");
    let lowered = signature_for(
        Fab::new(ids::ui::PLUS).lowered(true),
        "m3-fab-primary-lowered",
    );

    assert!(
        !normal.is_empty() && !lowered.is_empty(),
        "expected both normal and lowered primary FABs to paint Material shadows"
    );
    assert_ne!(
        lowered, normal,
        "lowered primary FAB should use the lowered elevation token path"
    );
}

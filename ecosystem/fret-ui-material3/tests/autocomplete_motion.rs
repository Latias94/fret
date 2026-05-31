//! Fixed-frame motion regression tests for Material 3 autocomplete-family fields.

use std::sync::Arc;

use fret_core::{
    AppWindowId, KeyCode, NodeId, Point, PointerId, Px, Rect, Scene, SceneOp, Size, UiServices,
};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{Autocomplete, AutocompleteItem, ExposedDropdown};

mod support;

use support::events::{key_down, key_up, pointer_down, pointer_up};
use support::goldens::run_overlay_frame_with_scene_scaled;
use support::host::{FakeUiServices, TestHost};
use support::layout::semantics_node_id_by_test_id;
use support::theme::apply_material_theme;

fn autocomplete_items() -> Arc<[AutocompleteItem]> {
    vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
    ]
    .into()
}

fn center_for_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Point {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node for {test_id}"));
    center_for_node(ui, node)
}

fn center_for_node(ui: &UiTree<TestHost>, node: NodeId) -> Point {
    let bounds = ui
        .debug_node_visual_bounds(node)
        .or_else(|| ui.debug_node_bounds(node))
        .expect("expected node bounds");
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

fn scene_has_intermediate_rotation(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.b.abs() > 0.01 || transform.c.abs() > 0.01
        )
    })
}

fn scene_has_half_turn_rotation(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.a < -0.9 && transform.d < -0.9
        )
    })
}

fn scene_has_intermediate_overlay_motion(scene: &Scene) -> bool {
    let has_alpha = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushOpacity { opacity } if *opacity > 0.01 && *opacity < 0.99
        )
    });
    let has_scale = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.b.abs() < 0.001
                    && transform.c.abs() < 0.001
                    && transform.a > 0.8
                    && transform.a < 1.0
                    && transform.d > 0.8
                    && transform.d < 1.0
        )
    });
    has_alpha && has_scale
}

fn assert_popover_open(ui: &UiTree<TestHost>, app: &mut TestHost, window: AppWindowId) {
    let stack = OverlayController::stack_snapshot_for_window(ui, app, window);
    assert!(
        stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Popover && entry.open),
        "expected autocomplete-family popover to be open"
    );
}

#[test]
fn autocomplete_popup_and_chevron_animate_on_open_close_frames() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );
    let query = app.models_mut().insert(String::new());
    let items = autocomplete_items();

    let query_model = query.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let query_model = query_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    Autocomplete::new(query_model)
                        .items(items)
                        .trailing_dropdown_icon(true)
                        .a11y_label("Autocomplete")
                        .placeholder("Search")
                        .test_id("m3-autocomplete")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let click_at = center_for_test_id(&ui, "m3-autocomplete.trailing-icon");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    let first_open_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    assert_popover_open(&ui, &mut app, window);
    assert!(
        scene_has_intermediate_rotation(&first_open_scene),
        "expected Autocomplete chevron to rotate on the first open frame"
    );
    assert!(
        scene_has_intermediate_overlay_motion(&first_open_scene),
        "expected Autocomplete popup to fade and scale on the first open frame"
    );

    let mut settled_scene = first_open_scene;
    for _ in 0..64 {
        settled_scene = run_overlay_frame_with_scene_scaled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            1.0,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }
    assert!(
        scene_has_half_turn_rotation(&settled_scene),
        "expected open Autocomplete chevron to settle at a half-turn rotation"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));
    let first_close_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    assert!(
        scene_has_intermediate_rotation(&first_close_scene),
        "expected Autocomplete chevron to rotate on the first close frame"
    );
    assert!(
        scene_has_intermediate_overlay_motion(&first_close_scene),
        "expected Autocomplete popup to fade and scale on the first close frame"
    );
}

#[test]
fn exposed_dropdown_popup_and_chevron_animate_on_open_close_frames() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );
    let selected = app.models_mut().insert(None::<Arc<str>>);
    let items = autocomplete_items();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    ExposedDropdown::new(selected_model)
                        .items(items)
                        .a11y_label("Exposed dropdown")
                        .placeholder("Pick")
                        .test_id("m3-exposed-dropdown")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let click_at = center_for_test_id(&ui, "m3-exposed-dropdown.trailing-icon");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    let first_open_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    assert_popover_open(&ui, &mut app, window);
    assert!(
        scene_has_intermediate_rotation(&first_open_scene),
        "expected ExposedDropdown chevron to rotate on the first open frame"
    );
    assert!(
        scene_has_intermediate_overlay_motion(&first_open_scene),
        "expected ExposedDropdown popup to fade and scale on the first open frame"
    );

    let mut settled_scene = first_open_scene;
    for _ in 0..64 {
        settled_scene = run_overlay_frame_with_scene_scaled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            1.0,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }
    assert!(
        scene_has_half_turn_rotation(&settled_scene),
        "expected open ExposedDropdown chevron to settle at a half-turn rotation"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));
    let first_close_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    assert!(
        scene_has_intermediate_rotation(&first_close_scene),
        "expected ExposedDropdown chevron to rotate on the first close frame"
    );
    assert!(
        scene_has_intermediate_overlay_motion(&first_close_scene),
        "expected ExposedDropdown popup to fade and scale on the first close frame"
    );
}

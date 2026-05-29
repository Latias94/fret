use fret_core::{AppWindowId, Point, Px, Rect, Scene, SceneOp, SemanticsRole, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{Button, Dialog, DialogAction};

mod interaction_harness;
mod support;

use support::goldens::run_overlay_frame_with_scene_scaled;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(420.0)),
    )
}

fn scene_has_intermediate_opacity(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushOpacity { opacity } if *opacity > 0.01 && *opacity < 0.99
        )
    })
}

fn scene_has_dialog_rise_scale(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.b.abs() < 0.001
                    && transform.c.abs() < 0.001
                    && transform.a > 0.88
                    && transform.a < 1.0
                    && transform.d > 0.88
                    && transform.d < 1.0
                    && transform.ty > 1.0
        )
    })
}

fn visual_bounds_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Rect {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"));
    ui.debug_node_visual_bounds(node)
        .or_else(|| ui.debug_node_bounds(node))
        .unwrap_or_else(|| panic!("expected visual bounds for test_id {test_id}"))
}

fn assert_px_close(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.5,
        "{context}: expected {expected}px, got {actual}px (delta {delta}px)"
    );
}

fn render_open_dialog_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    open: fret_runtime::Model<bool>,
    capture_semantics: bool,
) -> Scene {
    run_overlay_frame_with_scene_scaled(
        ui,
        app,
        services,
        window,
        bounds(),
        1.0,
        capture_semantics,
        move |ui, app, services| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
                let dialog = Dialog::new(open.clone())
                    .headline("Discard changes?")
                    .supporting_text("Unsaved edits will be lost.")
                    .actions(vec![
                        DialogAction::new("Cancel").test_id("m3-dialog-action-cancel"),
                        DialogAction::new("Discard").test_id("m3-dialog-action-confirm"),
                    ])
                    .test_id("m3-dialog")
                    .into_element(
                        cx,
                        |cx| {
                            let trigger = Button::new("Open dialog")
                                .test_id("m3-dialog-trigger")
                                .into_element(cx);
                            with_padding(cx, Px(24.0), trigger)
                        },
                        |_cx| Vec::new(),
                    );
                vec![dialog]
            })
        },
    )
}

#[test]
fn dialog_panel_exposes_material_parts_relations_and_spacing() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let open = app.models_mut().insert(true);

    for _ in 0..64 {
        render_open_dialog_frame(&mut ui, &mut app, &mut services, window, open.clone(), true);
    }

    let panel = visual_bounds_by_test_id(&ui, "m3-dialog.panel");
    let headline = visual_bounds_by_test_id(&ui, "m3-dialog.headline");
    let supporting = visual_bounds_by_test_id(&ui, "m3-dialog.supporting-text");
    let cancel = visual_bounds_by_test_id(&ui, "m3-dialog-action-cancel.chrome");

    assert_px_close(panel.size.width.0, 560.0, "settled dialog panel width");
    assert_px_close(panel.origin.x.0, 40.0, "settled dialog panel x");
    assert_px_close(
        headline.origin.x.0 - panel.origin.x.0,
        24.0,
        "headline content inset",
    );
    assert!(
        supporting.origin.y.0 - (headline.origin.y.0 + headline.size.height.0) >= 15.5,
        "expected 16dp title-to-supporting spacing; headline={headline:?}, supporting={supporting:?}"
    );
    assert!(
        cancel.origin.y.0 - (supporting.origin.y.0 + supporting.size.height.0) >= 23.5,
        "expected 24dp supporting-to-actions spacing; supporting={supporting:?}, cancel={cancel:?}"
    );

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot");
    let panel_node = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("m3-dialog.panel"))
        .expect("expected dialog panel semantics");
    let headline_node = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("m3-dialog.headline"))
        .expect("expected dialog headline semantics");
    let supporting_node = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("m3-dialog.supporting-text"))
        .expect("expected dialog supporting-text semantics");

    assert_eq!(panel_node.role, SemanticsRole::Dialog);
    assert!(
        panel_node.labelled_by.contains(&headline_node.id),
        "expected dialog panel to be labelled by headline"
    );
    assert!(
        panel_node.described_by.contains(&supporting_node.id),
        "expected dialog panel to be described by supporting text"
    );
}

#[test]
fn dialog_scrim_and_panel_animate_on_open_close_frames() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let open = app.models_mut().insert(false);

    render_open_dialog_frame(&mut ui, &mut app, &mut services, window, open.clone(), true);

    let _ = app.models_mut().update(&open, |v| *v = true);
    let first_open_scene =
        render_open_dialog_frame(&mut ui, &mut app, &mut services, window, open.clone(), true);
    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Modal && entry.open),
        "expected dialog modal overlay to be open"
    );
    assert!(
        scene_has_intermediate_opacity(&first_open_scene),
        "expected Dialog panel/scrim to fade on the first open frame"
    );
    assert!(
        scene_has_dialog_rise_scale(&first_open_scene),
        "expected Dialog panel to rise and scale on the first open frame"
    );

    for _ in 0..64 {
        render_open_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            open.clone(),
            false,
        );
    }

    let _ = app.models_mut().update(&open, |v| *v = false);
    let mut close_has_opacity = false;
    let mut close_has_rise_scale = false;
    for _ in 0..8 {
        let scene = render_open_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            open.clone(),
            false,
        );
        close_has_opacity |= scene_has_intermediate_opacity(&scene);
        close_has_rise_scale |= scene_has_dialog_rise_scale(&scene);
    }

    assert!(
        close_has_opacity,
        "expected Dialog panel/scrim to fade during fixed close frames"
    );
    assert!(
        close_has_rise_scale,
        "expected Dialog panel to rise/settle through modal motion during fixed close frames"
    );
}

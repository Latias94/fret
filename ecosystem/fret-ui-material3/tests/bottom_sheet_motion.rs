//! Fixed-frame motion and semantics regression tests for Material 3 bottom sheets.

use fret_core::{AppWindowId, Point, Px, Rect, Scene, SceneOp, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{Button, ButtonVariant, ModalBottomSheet};

mod support;

use support::goldens::run_overlay_frame_with_scene_scaled;
use support::host::{FakeUiServices, TestHost};
use support::theme::apply_material_theme;

fn largest_vertical_slide_ty(scene: &Scene) -> Option<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::PushTransform { transform }
                if transform.b.abs() < 0.001
                    && transform.c.abs() < 0.001
                    && (transform.a - 1.0).abs() < 0.001
                    && (transform.d - 1.0).abs() < 0.001
                    && transform.tx.abs() < 0.001
                    && transform.ty > 1.0 =>
            {
                Some(transform.ty)
            }
            _ => None,
        })
        .max_by(|a, b| a.total_cmp(b))
}

fn scene_has_intermediate_opacity(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushOpacity { opacity } if *opacity > 0.01 && *opacity < 0.99
        )
    })
}

fn assert_modal_open(ui: &UiTree<TestHost>, app: &mut TestHost, window: AppWindowId) {
    let stack = OverlayController::stack_snapshot_for_window(ui, app, window);
    assert!(
        stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Modal && entry.open),
        "expected bottom sheet modal overlay to be open"
    );
}

#[test]
fn modal_bottom_sheet_slides_from_own_height_without_panel_fade() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(520.0)),
    );
    let open = app.models_mut().insert(false);

    let open_model = open.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let open = open_model.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let sheet = ModalBottomSheet::new(open).test_id("m3-bottom-sheet-motion");
                vec![sheet.into_element(
                    cx,
                    |cx| cx.text("Underlay"),
                    |cx| {
                        vec![
                            cx.text("Modal bottom sheet"),
                            Button::new("Close")
                                .variant(ButtonVariant::Filled)
                                .test_id("m3-bottom-sheet-motion-close")
                                .into_element(cx),
                        ]
                    },
                )]
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

    let _ = app.models_mut().update(&open, |v| *v = true);
    let first_open_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    assert_modal_open(&ui, &mut app, window);

    let ty = largest_vertical_slide_ty(&first_open_scene)
        .expect("expected bottom sheet to slide with a vertical render transform");
    assert!(
        ty < bounds.size.height.0 * 0.75,
        "expected the hidden anchor to be one sheet height below the expanded anchor, got ty={ty}"
    );
    assert!(
        !scene_has_intermediate_opacity(&first_open_scene),
        "Compose Material3 animates bottom sheet offset plus scrim alpha; the sheet panel itself should not fade"
    );

    for _ in 0..64 {
        run_overlay_frame_with_scene_scaled(
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

    let _ = app.models_mut().update(&open, |v| *v = false);
    let mut close_slide_ty = None::<f32>;
    let mut close_has_panel_fade = false;
    for _ in 0..8 {
        let scene = run_overlay_frame_with_scene_scaled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            1.0,
            false,
            |ui, app, services| render(ui, app, services),
        );
        close_slide_ty = close_slide_ty.or_else(|| largest_vertical_slide_ty(&scene));
        close_has_panel_fade |= scene_has_intermediate_opacity(&scene);
    }

    let close_ty = close_slide_ty.expect("expected bottom sheet close to use vertical transform");
    assert!(
        close_ty < bounds.size.height.0 * 0.75,
        "expected close transform to keep using the sheet-height anchor, got ty={close_ty}"
    );
    assert!(
        !close_has_panel_fade,
        "bottom sheet panel should remain offset-animated without panel fade during close"
    );
}

#[test]
fn modal_bottom_sheet_exposes_dialog_scrim_and_drag_handle_semantics() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(520.0)),
    );
    let open = app.models_mut().insert(true);

    let open_model = open.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let open = open_model.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let sheet = ModalBottomSheet::new(open)
                    .open_duration_ms(Some(1))
                    .close_duration_ms(Some(1))
                    .test_id("m3-bottom-sheet-semantics");
                vec![sheet.into_element(
                    cx,
                    |cx| cx.text("Underlay"),
                    |cx| {
                        vec![
                            cx.text("Modal bottom sheet"),
                            Button::new("Action")
                                .variant(ButtonVariant::Filled)
                                .test_id("m3-bottom-sheet-semantics-action")
                                .into_element(cx),
                        ]
                    },
                )]
            })
        };

    for _ in 0..8 {
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
        if ui.semantics_snapshot().is_some_and(|snapshot| {
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("m3-bottom-sheet-semantics.sheet"))
        }) {
            break;
        }
    }

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected bottom sheet semantics snapshot");
    let by_test_id = |test_id: &str| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
            .unwrap_or_else(|| panic!("expected semantics node with test_id {test_id}"))
    };

    let sheet = by_test_id("m3-bottom-sheet-semantics.sheet");
    assert_eq!(sheet.role, fret_core::SemanticsRole::Dialog);
    assert_eq!(sheet.label.as_deref(), Some("Bottom sheet"));

    let scrim = by_test_id("m3-bottom-sheet-semantics.scrim");
    assert_eq!(scrim.label.as_deref(), Some("Close sheet"));

    let drag_handle = by_test_id("m3-bottom-sheet-semantics.sheet.drag-handle");
    assert_eq!(drag_handle.label.as_deref(), Some("Drag handle"));
}

//! Fixed-frame motion regression tests for Material 3 time picker dialogs.

use fret_core::{AppWindowId, Point, Px, Rect, Scene, SceneOp, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{TimePickerDialog, TimePickerDisplayMode};
use time::Time;

mod interaction_harness;
mod support;

use support::goldens::run_overlay_frame_with_scene_scaled;
use support::host::{FakeUiServices, TestHost};
use support::theme::apply_material_theme;

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

fn assert_modal_open(ui: &UiTree<TestHost>, app: &mut TestHost, window: AppWindowId) {
    let stack = OverlayController::stack_snapshot_for_window(ui, app, window);
    assert!(
        stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Modal && entry.open),
        "expected time picker modal overlay to be open"
    );
}

fn assert_modal_motion_for_mode(initial_mode: TimePickerDisplayMode, test_id: &'static str) {
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
    let selected = app
        .models_mut()
        .insert(Time::from_hms(10, 30, 0).expect("time"));

    let open_model = open.clone();
    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let open = open_model.clone();
            let selected = selected_model.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dialog = TimePickerDialog::new(open, selected)
                    .initial_display_mode(initial_mode)
                    .test_id(test_id);
                vec![dialog.into_element(cx, |cx| cx.text("Underlay"))]
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
    assert!(
        scene_has_intermediate_opacity(&first_open_scene),
        "expected TimePicker modal panel to fade on the first open frame ({initial_mode:?})"
    );
    assert!(
        scene_has_dialog_rise_scale(&first_open_scene),
        "expected TimePicker modal panel to rise and scale on the first open frame ({initial_mode:?})"
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
    let mut close_has_opacity = false;
    let mut close_has_rise_scale = false;
    let first_close_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    close_has_opacity |= scene_has_intermediate_opacity(&first_close_scene);
    close_has_rise_scale |= scene_has_dialog_rise_scale(&first_close_scene);

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
        close_has_opacity |= scene_has_intermediate_opacity(&scene);
        close_has_rise_scale |= scene_has_dialog_rise_scale(&scene);
    }

    assert!(
        close_has_opacity,
        "expected TimePicker modal panel to fade during fixed close frames ({initial_mode:?})"
    );
    assert!(
        close_has_rise_scale,
        "expected TimePicker modal panel to rise/settle through dialog motion during fixed close frames ({initial_mode:?})"
    );
}

#[test]
fn time_picker_modal_scrim_and_panel_animate_on_open_close_frames() {
    assert_modal_motion_for_mode(TimePickerDisplayMode::Dial, "m3-time-picker-modal");
    assert_modal_motion_for_mode(TimePickerDisplayMode::Input, "m3-time-picker-modal-input");
}

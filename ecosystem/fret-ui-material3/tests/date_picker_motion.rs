//! Fixed-frame motion regression tests for Material 3 date picker dialogs.

use fret_core::{AppWindowId, Point, Px, Rect, Scene, SceneOp, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_kit::headless::calendar::CalendarMonth;
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::DatePickerDialog;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use time::{Date, Month};

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
        "expected date picker modal overlay to be open"
    );
}

#[test]
fn date_picker_modal_scrim_and_panel_animate_on_open_close_frames() {
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
    let today = Date::from_calendar_date(2026, Month::May, 15).expect("date");
    let open = app.models_mut().insert(false);
    let month = app.models_mut().insert(CalendarMonth::from_date(today));
    let selected = app.models_mut().insert(None::<Date>);

    let open_model = open.clone();
    let month_model = month.clone();
    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let open = open_model.clone();
            let month = month_model.clone();
            let selected = selected_model.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dialog = DatePickerDialog::new(open, month, selected)
                    .today(Some(today))
                    .test_id("m3-date-picker-modal");
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
        "expected DatePicker modal panel to fade on the first open frame"
    );
    assert!(
        scene_has_dialog_rise_scale(&first_open_scene),
        "expected DatePicker modal panel to rise and scale on the first open frame"
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
        "expected DatePicker modal panel to fade during fixed close frames"
    );
    assert!(
        close_has_rise_scale,
        "expected DatePicker modal panel to rise/settle through dialog motion during fixed close frames"
    );
}

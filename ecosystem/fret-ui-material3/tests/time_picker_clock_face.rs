//! Layout and pointer regression tests for Material 3 time picker clock faces.

use std::f32::consts::PI;

use fret_core::{AppWindowId, Point, PointerId, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
use time::Time;

mod support;

use support::events::{pointer_down, pointer_up};
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn angle_from_top(center: Point, point: Point) -> f32 {
    let dx = point.x.0 - center.x.0;
    let dy = point.y.0 - center.y.0;
    (dy.atan2(dx) + PI * 0.5).rem_euclid(2.0 * PI)
}

fn angular_delta(a: f32, b: f32) -> f32 {
    let mut diff = (a - b).abs();
    while diff > PI {
        diff = (2.0 * PI - diff).abs();
    }
    diff
}

fn distance(a: Point, b: Point) -> f32 {
    let dx = a.x.0 - b.x.0;
    let dy = a.y.0 - b.y.0;
    (dx * dx + dy * dy).sqrt()
}

fn node_center(ui: &UiTree<TestHost>, test_id: &str) -> Point {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node `{test_id}`"));
    let bounds = ui
        .debug_node_visual_bounds(node)
        .unwrap_or_else(|| panic!("expected visual bounds for `{test_id}`"));
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

fn dial_center(ui: &UiTree<TestHost>, dial_test_id: &str) -> (Point, f32) {
    let dial = semantics_node_id_by_test_id(ui, dial_test_id)
        .unwrap_or_else(|| panic!("expected semantics node `{dial_test_id}`"));
    let bounds = ui
        .debug_node_visual_bounds(dial)
        .unwrap_or_else(|| panic!("expected visual bounds for `{dial_test_id}`"));
    (
        Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
            Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
        ),
        bounds.size.width.0.min(bounds.size.height.0),
    )
}

#[test]
fn time_picker_24h_clock_face_uses_inner_and_outer_hour_rings() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );
    let time = app
        .models_mut()
        .insert(Time::from_hms(13, 30, 0).expect("time"));

    let time_model = time.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_model.clone())
                    .is_24h(true)
                    .display_mode(TimePickerDisplayMode::Dial)
                    .test_id("time-picker-24h")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (center, _dial_size) = dial_center(&ui, "time-picker-24h.clock-dial");
    let outer_01 = node_center(&ui, "time-picker-24h.clock-dial.hour.01");
    let inner_13 = node_center(&ui, "time-picker-24h.clock-dial.hour.13");

    assert!(
        angular_delta(
            angle_from_top(center, outer_01),
            angle_from_top(center, inner_13)
        ) < 0.08,
        "expected 01 and 13 to share the same clock angle in 24h mode"
    );
    assert!(
        distance(center, inner_13) + 20.0 < distance(center, outer_01),
        "expected 13 to be on an inner hour ring while 01 stays on the outer ring"
    );
}

#[test]
fn time_picker_24h_inner_ring_pointer_selects_pm_hour() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );
    let time = app
        .models_mut()
        .insert(Time::from_hms(1, 0, 0).expect("time"));

    let time_model = time.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_model.clone())
                    .is_24h(true)
                    .display_mode(TimePickerDisplayMode::Dial)
                    .test_id("time-picker-24h")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (center, dial_size) = dial_center(&ui, "time-picker-24h.clock-dial");
    let inner_radius = dial_size * (69.0 / 256.0);
    let hour_13_angle = -PI * 0.5 + (2.0 * PI / 12.0);
    let press_at = Point::new(
        Px(center.x.0 + inner_radius * hour_13_angle.cos()),
        Px(center.y.0 + inner_radius * hour_13_angle.sin()),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));

    let after = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after.hour(),
        13,
        "expected pressing the 24h inner ring at 13 to select 13:00"
    );
}

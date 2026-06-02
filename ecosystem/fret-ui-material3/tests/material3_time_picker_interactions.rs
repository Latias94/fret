use fret_core::{
    AppWindowId, Event, KeyCode, NodeId, Point, PointerId, Px, Rect, SemanticsInvalid,
    SemanticsLive, Size, UiServices,
};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::events::{
    drain_zero_delay_timer_tokens, key_down, key_up, pointer_down, pointer_move, pointer_up,
};
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn semantics_invalid_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Option<SemanticsInvalid> {
    ui.semantics_snapshot().and_then(|snapshot| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
            .and_then(|node| node.flags.invalid)
    })
}

fn semantics_label_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Option<String> {
    ui.semantics_snapshot().and_then(|snapshot| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
            .and_then(|node| node.label.clone())
    })
}

fn semantics_live_by_test_id(
    ui: &UiTree<TestHost>,
    test_id: &str,
) -> Option<(SemanticsLive, bool)> {
    ui.semantics_snapshot().and_then(|snapshot| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
            .and_then(|node| node.flags.live.map(|live| (live, node.flags.live_atomic)))
    })
}

// TimePicker interaction regressions.

#[test]
fn time_picker_clock_dial_drag_updates_time() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

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

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .display_mode(TimePickerDisplayMode::Dial)
                    .test_id("time-picker-docked")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let dial: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("time-picker-docked.clock-dial") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected time picker clock dial node in semantics snapshot");

    let dial_bounds = ui
        .debug_node_visual_bounds(dial)
        .expect("expected dial bounds");

    let center = Point::new(
        Px(dial_bounds.origin.x.0 + dial_bounds.size.width.0 * 0.5),
        Px(dial_bounds.origin.y.0 + dial_bounds.size.height.0 * 0.5),
    );
    let r = dial_bounds.size.width.0.min(dial_bounds.size.height.0) * 0.45;

    let start_at = Point::new(center.x, Px(center.y.0 - r));
    let drag_to = Point::new(Px(center.x.0 + r), center.y);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), start_at),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), drag_to),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), drag_to));

    let after = app.models().get_cloned(&time).unwrap_or(selected_time);
    assert_ne!(
        after, selected_time,
        "expected dial drag to update the time model"
    );
}

#[test]
fn time_picker_selector_keyboard_arrows_step_time() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

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

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .display_mode(TimePickerDisplayMode::Dial)
                    .test_id("time-picker-docked")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hour_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("time-picker-docked.hour-selector") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected hour selector node in semantics snapshot");

    ui.set_focus(Some(hour_node));
    assert_eq!(
        ui.focus(),
        Some(hour_node),
        "expected focus to be set to the hour input node"
    );
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let after_hour = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_hour,
        Time::from_hms(10, 41, 0).expect("valid time"),
        "expected ArrowUp on hour selector to step +1 hour",
    );

    let minute_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("time-picker-docked.minute-selector") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected minute selector node in semantics snapshot");

    ui.set_focus(Some(minute_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let after_minute = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_minute,
        Time::from_hms(10, 40, 0).expect("valid time"),
        "expected ArrowDown on minute selector to step -1 minute",
    );
}

#[test]
fn time_picker_time_input_replaces_and_auto_advances_hour() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

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

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .display_mode(TimePickerDisplayMode::Input)
                    .test_id("time-picker-docked-input")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hour_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("time-picker-docked-input.input.hour"))
                    .then_some(node.id)
            })
        })
        .expect("expected time-picker-docked-input.input.hour in semantics snapshot");
    let minute_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("time-picker-docked-input.input.minute"))
                    .then_some(node.id)
            })
        })
        .expect("expected time-picker-docked-input.input.minute in semantics snapshot");

    ui.set_focus(Some(hour_node));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("1".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after_first = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_first,
        Time::from_hms(1, 41, 0).expect("valid time"),
        "expected first digit to replace the existing hour",
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("2".to_string()));

    for token in drain_zero_delay_timer_tokens(&mut app, window) {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
    }

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after_second = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_second,
        Time::from_hms(0, 41, 0).expect("valid time"),
        "expected second digit to complete a two-digit hour (12 AM -> 00h in 24h time)",
    );
    assert_eq!(
        ui.focus(),
        Some(minute_node),
        "expected entering a two-digit hour to auto-advance focus to minutes",
    );
}

#[test]
fn time_picker_time_input_rejects_invalid_values_and_recovers() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

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

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .is_24h(true)
                    .display_mode(TimePickerDisplayMode::Input)
                    .test_id("time-picker-docked-input")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hour_node = semantics_node_id_by_test_id(&ui, "time-picker-docked-input.input.hour")
        .expect("expected time input hour field in semantics snapshot");
    ui.set_focus(Some(hour_node));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("2".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_cloned(&time).expect("time model exists"),
        Time::from_hms(2, 41, 0).expect("valid time"),
        "first valid hour digit should still update the committed time",
    );
    assert_eq!(
        semantics_invalid_by_test_id(&ui, "time-picker-docked-input.input.hour"),
        None,
        "single valid hour digit should not expose invalid semantics",
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit7));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit7));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("7".to_string()));

    for token in drain_zero_delay_timer_tokens(&mut app, window) {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
    }

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_cloned(&time).expect("time model exists"),
        Time::from_hms(2, 41, 0).expect("valid time"),
        "invalid 24h hour input must not clamp or overwrite the committed time",
    );
    assert_eq!(
        semantics_invalid_by_test_id(&ui, "time-picker-docked-input.input.hour"),
        Some(SemanticsInvalid::True),
        "invalid hour input should expose aria-invalid semantics",
    );
    assert_eq!(
        semantics_label_by_test_id(&ui, "time-picker-docked-input.input.hour.supporting-text"),
        Some(String::from("Hour must be 0-23")),
        "invalid hour input should expose Material supporting error text",
    );
    assert_eq!(
        semantics_live_by_test_id(&ui, "time-picker-docked-input.input.hour.supporting-text"),
        Some((SemanticsLive::Polite, true)),
        "supporting error text should be a polite atomic live region",
    );

    let hour_node = semantics_node_id_by_test_id(&ui, "time-picker-docked-input.input.hour")
        .expect("expected time input hour field after invalid input");
    ui.set_focus(Some(hour_node));
    for _ in 0..2 {
        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Backspace));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Backspace));
    }
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("1".to_string()));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("2".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_cloned(&time).expect("time model exists"),
        Time::from_hms(12, 41, 0).expect("valid time"),
        "recovered valid hour input should update the committed time",
    );
    assert_eq!(
        semantics_invalid_by_test_id(&ui, "time-picker-docked-input.input.hour"),
        None,
        "valid recovery should clear invalid semantics",
    );
    assert_eq!(
        semantics_label_by_test_id(&ui, "time-picker-docked-input.input.hour.supporting-text"),
        Some(String::from("Hour")),
        "valid recovery should restore supporting text",
    );
}

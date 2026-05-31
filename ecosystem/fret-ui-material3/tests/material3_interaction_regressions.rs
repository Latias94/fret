use std::sync::Arc;

use fret_core::{
    AppWindowId, Event, KeyCode, NodeId, Point, PointerId, Px, Rect, SemanticsInvalid,
    SemanticsLive, SemanticsRole, Size, UiServices,
};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::events::{
    drain_zero_delay_timer_tokens, key_down, key_up, pointer_down, pointer_move, pointer_up,
};
use support::goldens::run_overlay_frame_scaled;
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
// Historical Material3 interaction regressions live here so radio_alignment.rs stays
// focused on Radio-owned geometry, ripple, and pressed-scene behavior.

#[test]
fn text_input_text_input_event_updates_model() {
    use fret_ui::element::TextInputProps;

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

    let model = app.models_mut().insert(String::new());
    let model_for_render = model.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut props = TextInputProps::new(model_for_render.clone());
                props.layout.size.width = fret_ui::element::Length::Px(Px(200.0));
                props.layout.size.height = fret_ui::element::Length::Px(Px(40.0));
                props.a11y_label = Some(Arc::<str>::from("input"));
                props.test_id = Some(Arc::<str>::from("plain-text-input"));
                let input = cx.text_input(props);
                vec![with_padding(cx, Px(24.0), input)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("plain-text-input")).then_some(node.id)
            })
        })
        .expect("expected plain-text-input in semantics snapshot");

    ui.set_focus(Some(input_node));
    assert_eq!(
        ui.focus(),
        Some(input_node),
        "expected focus to be set to the input node",
    );

    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("a".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let value = app.models().get_cloned(&model).expect("model exists");
    assert_eq!(value, "a", "expected text input event to update the model");
}

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

#[test]
fn material3_autocomplete_semantics_v1() {
    use fret_core::SemanticsRole;
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let selected_value = app
        .models_mut()
        .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .selected_value(selected_value.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    // Frame 1: build stable input id + bounds.
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");

    ui.set_focus(Some(input_node));

    // Frame 2: focus visible to the widget.
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    // Open via keyboard.
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    // Frame 3/4: overlay created, then relationships stabilize (controls/active-descendant).
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after ArrowDown"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node");
    assert_eq!(input.role, SemanticsRole::ComboBox);
    assert!(
        input.flags.expanded,
        "combobox input should report expanded=true while open"
    );

    let list = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete.listbox"))
        .expect("listbox node");
    assert!(
        input.controls.contains(&list.id),
        "combobox input should control the listbox"
    );
    assert!(
        list.labelled_by.contains(&input.id),
        "listbox should be labelled by the combobox input"
    );

    let active = input
        .active_descendant
        .expect("active_descendant should be set");
    let active_node = snap
        .nodes
        .iter()
        .find(|n| n.id == active)
        .expect("active_descendant should reference a node in the snapshot");
    assert_eq!(active_node.role, SemanticsRole::ListBoxOption);

    let beta = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete.option.beta"))
        .expect("expected beta option node");
    assert!(beta.flags.selected, "expected beta to be marked selected");

    // Typing still works while the overlay is open.
    ui.set_focus(Some(input.id));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("a".to_string()),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node after typing");
    assert_eq!(input.value.as_deref(), Some("a"));
}

#[test]
fn material3_autocomplete_filters_items_by_query_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");
    ui.set_focus(Some(input_node));

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("ga".to_string()),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after typing"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| { n.test_id.as_deref() == Some("material3-autocomplete.option.gamma") }),
        "expected gamma option after typing 'ga'"
    );
    assert!(
        !snap
            .nodes
            .iter()
            .any(|n| { n.test_id.as_deref() == Some("material3-autocomplete.option.alpha") }),
        "expected alpha option to be filtered out after typing 'ga'"
    );
}

#[test]
fn material3_autocomplete_enter_commits_and_does_not_reopen_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let selected_value = app.models_mut().insert(None::<Arc<str>>);
    let selected_value_for_render = selected_value.clone();
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .selected_value(selected_value_for_render.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");
    ui.set_focus(Some(input_node));

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after ArrowDown"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Enter));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Enter));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        !stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to remain closed after Enter commit"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node after Enter");
    assert_eq!(input.value.as_deref(), Some("Alpha"));

    let selected = app.models_mut().get_cloned(&selected_value).unwrap_or(None);
    assert_eq!(
        selected.as_deref(),
        Some("alpha"),
        "expected selected_value model to be committed on Enter"
    );
}

#[test]
fn material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1() {
    use fret_ui::element::{FlexProps, Length};
    use fret_ui_material3::{AutocompleteItem, ExposedDropdown, TextField, TextFieldVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let selected_value = app
        .models_mut()
        .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
    let query = app.models_mut().insert(String::new());
    let query_for_render = query.clone();
    let other = app.models_mut().insert(String::new());

    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let exposed = ExposedDropdown::new(selected_value.clone())
                    .query(query_for_render.clone())
                    .items(items.clone())
                    .a11y_label("exposed dropdown")
                    .test_id("material3-exposed-dropdown")
                    .into_element(cx);

                let other = TextField::new(other.clone())
                    .variant(TextFieldVariant::Outlined)
                    .label("Other")
                    .test_id("other-field")
                    .into_element(cx);

                let mut column = FlexProps::default();
                column.direction = fret_core::Axis::Vertical;
                column.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                column.layout.size.width = Length::Fill;

                let content = cx.flex(column, |_cx| vec![exposed, other]);
                vec![with_padding(cx, Px(24.0), content)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "Beta",
        "expected query to synchronize from the committed selection while blurred"
    );

    let (input_node, other_node): (NodeId, NodeId) = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            let input = snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-exposed-dropdown")).then_some(node.id)
            })?;
            let other = snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("other-field")).then_some(node.id)
            })?;
            Some((input, other))
        })
        .expect("expected input and other nodes in semantics snapshot");

    ui.set_focus(Some(input_node));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let _ = app.models_mut().update(&query, |v| *v = "ga".to_string());
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "ga",
        "expected query to remain editable while focused"
    );

    ui.set_focus(Some(other_node));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "Beta",
        "expected query to revert to the committed selection label on blur"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown"))
        .expect("combobox input node after blur");
    assert_eq!(input.value.as_deref(), Some("Beta"));
}

#[test]
fn material3_exposed_dropdown_trailing_icon_toggles_overlay_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{AutocompleteItem, ExposedDropdown};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let selected_value = app.models_mut().insert(None::<Arc<str>>);
    let query = app.models_mut().insert(String::new());
    let query_for_render = query.clone();

    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let exposed = ExposedDropdown::new(selected_value.clone())
                    .query(query_for_render.clone())
                    .items(items.clone())
                    .a11y_label("exposed dropdown")
                    .test_id("material3-exposed-dropdown")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), exposed)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let icon_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-exposed-dropdown.trailing-icon"))
                    .then_some(node.id)
            })
        })
        .expect("expected trailing icon node in semantics snapshot");

    let icon_bounds = ui
        .debug_node_visual_bounds(icon_node)
        .expect("expected trailing icon bounds");
    let click_at = Point::new(
        Px(icon_bounds.origin.x.0 + icon_bounds.size.width.0 * 0.5),
        Px(icon_bounds.origin.y.0 + icon_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected popover overlay to be open after clicking the trailing icon"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown"))
        .expect("expected exposed dropdown input node");
    assert_eq!(input.role, SemanticsRole::ComboBox);
    assert!(
        input.flags.expanded,
        "exposed dropdown input should report expanded=true while open"
    );

    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown.listbox"))
        .expect("expected exposed dropdown listbox node");
    assert_eq!(listbox.role, SemanticsRole::ListBox);
    assert!(
        input.controls.contains(&listbox.id),
        "exposed dropdown input should control its listbox"
    );
    assert!(
        listbox.labelled_by.contains(&input.id),
        "exposed dropdown listbox should be labelled by its input"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        !stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected popover overlay to be closed after clicking the trailing icon again"
    );
}

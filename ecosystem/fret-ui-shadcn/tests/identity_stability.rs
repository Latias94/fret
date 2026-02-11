use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::{ElementContext, ElementRuntime, GlobalElementId};
use fret_ui_kit::UiExt as _;
use fret_ui_shadcn::Button;

fn frame_button_ids(
    app: &mut App,
    runtime: &mut ElementRuntime,
    insert_prefix: bool,
) -> Vec<GlobalElementId> {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(400.0)),
    );

    let mut cx =
        ElementContext::new_for_root_name(app, runtime, window, bounds, "identity-stability");

    let mut ids = Vec::new();

    if insert_prefix {
        let x = Button::new("X").into_element(&mut cx);
        ids.push(x.id);
    }

    let a = Button::new("A").into_element(&mut cx);
    ids.push(a.id);

    let b = Button::new("B").into_element(&mut cx);
    ids.push(b.id);

    ids
}

fn frame_button_ids_via_ui_builder_terminal(
    app: &mut App,
    runtime: &mut ElementRuntime,
    insert_prefix: bool,
) -> Vec<GlobalElementId> {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(400.0)),
    );

    let mut cx = ElementContext::new_for_root_name(app, runtime, window, bounds, "ui");

    let mut ids = Vec::new();

    if insert_prefix {
        let x = Button::new("X").ui().into_element(&mut cx);
        ids.push(x.id);
    }

    let a = Button::new("A").ui().into_element(&mut cx);
    ids.push(a.id);

    let b = Button::new("B").ui().into_element(&mut cx);
    ids.push(b.id);

    ids
}

#[test]
fn sibling_insertion_does_not_shift_button_ids() {
    // This is a regression test for identity stability: inserting a new sibling before existing
    // siblings must not cause `GlobalElementId` churn for the existing elements.
    //
    // In practice, this avoids "hover/focus/state sticks to the wrong control" bugs when a toolbar
    // or form conditionally inserts controls.
    let mut app = App::new();
    let mut runtime = ElementRuntime::new();

    let ids_frame_1 = frame_button_ids(&mut app, &mut runtime, false);

    // Simulate a later frame by advancing app-owned clocks.
    app.set_frame_id(fret_runtime::FrameId(app.frame_id().0.saturating_add(1)));
    app.set_tick_id(fret_runtime::TickId(app.tick_id().0.saturating_add(1)));

    let ids_frame_2 = frame_button_ids(&mut app, &mut runtime, true);

    assert_eq!(ids_frame_1.len(), 2);
    assert_eq!(ids_frame_2.len(), 3);

    // Frame 2 inserts X at index 0, so A and B shift right in the returned list. Their IDs must
    // remain stable across frames.
    assert_eq!(
        ids_frame_1[0], ids_frame_2[1],
        "A id shifted after insertion"
    );
    assert_eq!(
        ids_frame_1[1], ids_frame_2[2],
        "B id shifted after insertion"
    );
}

#[test]
fn sibling_insertion_does_not_shift_button_ids_via_ui_builder_terminal() {
    let mut app = App::new();
    let mut runtime = ElementRuntime::new();

    let ids_frame_1 = frame_button_ids_via_ui_builder_terminal(&mut app, &mut runtime, false);

    app.set_frame_id(fret_runtime::FrameId(app.frame_id().0.saturating_add(1)));
    app.set_tick_id(fret_runtime::TickId(app.tick_id().0.saturating_add(1)));

    let ids_frame_2 = frame_button_ids_via_ui_builder_terminal(&mut app, &mut runtime, true);

    assert_eq!(ids_frame_1.len(), 2);
    assert_eq!(ids_frame_2.len(), 3);
    assert_eq!(ids_frame_1[0], ids_frame_2[1]);
    assert_eq!(ids_frame_1[1], ids_frame_2[2]);
}

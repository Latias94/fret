use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct TooltipObserverFixture {
    name: String,
    open: bool,
    interactive: bool,
    trigger: bool,
    has_dismiss_request: bool,
    has_pointer_move: bool,
    expect_wants_pointer_down_outside_events: bool,
    expect_wants_pointer_move_events: bool,
}

fn run_tooltip_observer_fixture(name: &str) {
    let fixtures: Vec<TooltipObserverFixture> =
        serde_json::from_str(include_str!("fixtures/tooltip_observers.json"))
            .expect("tooltip observer fixtures");
    let fx = fixtures
        .iter()
        .find(|fx| fx.name == name)
        .unwrap_or_else(|| panic!("missing tooltip observer fixture: {name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let base_open = app.models_mut().insert(false);
    let open = app.models_mut().insert(fx.open);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    let trigger =
        render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, base_open);

    begin_frame(&mut app, window);

    let on_dismiss_request: Option<fret_ui::action::OnDismissRequest> = fx
        .has_dismiss_request
        .then_some(Arc::new(|_host, _cx, _req| {}));
    let on_pointer_move: Option<fret_ui::action::OnDismissiblePointerMove> = fx
        .has_pointer_move
        .then_some(Arc::new(|_host, _cx, _move| false));

    // Install a tooltip layer (possibly closing: present=true, open=false).
    let tooltip_id = GlobalElementId(0x44);
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: tooltip_id,
            root_name: tooltip_root_name(tooltip_id),
            interactive: fx.interactive,
            trigger: fx.trigger.then_some(trigger),
            open,
            present: true,
            on_dismiss_request,
            on_pointer_move,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays
                .tooltips
                .get(&(window, tooltip_id))
                .map(|p| p.layer)
        })
        .expect("tooltip layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("tooltip debug layer info");

    assert!(info.visible, "case={}", fx.name);
    assert!(!info.blocks_underlay_input, "case={}", fx.name);
    assert!(!info.hit_testable, "case={}", fx.name);
    assert_eq!(
        info.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None,
        "case={}",
        fx.name
    );
    assert_eq!(
        info.wants_pointer_down_outside_events, fx.expect_wants_pointer_down_outside_events,
        "case={}",
        fx.name
    );
    assert_eq!(
        info.wants_pointer_move_events, fx.expect_wants_pointer_move_events,
        "case={}",
        fx.name
    );
}

#[test]
fn tooltip_is_pointer_transparent_and_does_not_request_observers_while_closing() {
    run_tooltip_observer_fixture("closing_open_false_even_if_interactive");
}

#[test]
fn tooltip_does_not_request_observers_by_default() {
    run_tooltip_observer_fixture("default_no_observers");
}

#[test]
fn tooltip_does_not_request_observers_while_closing() {
    run_tooltip_observer_fixture("closing_handlers_do_not_install_observers");
}

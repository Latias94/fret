use super::state::WindowOverlays;
use super::*;

use crate::declarative::action_hooks::ActionHooksExt;
use fret_app::App;
use fret_core::AppWindowId;
use fret_core::{PathCommand, SvgId, SvgService};
use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
use fret_core::{
    Point, Px, Rect, TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle,
};
use fret_runtime::Model;
use fret_ui::UiTree;
use fret_ui::element::{
    ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;

#[derive(Default)]
struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _text: &str,
        _style: &TextStyle,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: fret_core::Size::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

fn render_base_with_trigger(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
) -> GlobalElementId {
    begin_frame(app, window);

    let mut trigger_id: Option<GlobalElementId> = None;
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
        vec![cx.pressable_with_id(
            PressableProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(80.0));
                    layout.size.height = Length::Px(Px(32.0));
                    layout
                },
                ..Default::default()
            },
            |cx, _st, id| {
                cx.pressable_toggle_bool(&open);
                trigger_id = Some(id);
                vec![cx.container(ContainerProps::default(), |_| Vec::new())]
            },
        )]
    });
    ui.set_root(root);
    trigger_id.expect("trigger id")
}

fn render_base_with_trigger_and_underlay(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
    underlay_clicked: Model<bool>,
) -> (GlobalElementId, GlobalElementId) {
    begin_frame(app, window);

    let mut trigger_id: Option<GlobalElementId> = None;
    let mut underlay_id: Option<GlobalElementId> = None;

    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |cx| {
                vec![
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                LayoutStyle {
                                    position: PositionStyle::Absolute,
                                    inset: InsetStyle {
                                        left: Some(Px(0.0)),
                                        top: Some(Px(0.0)),
                                        ..Default::default()
                                    },
                                    size: SizeStyle {
                                        width: Length::Px(Px(80.0)),
                                        height: Length::Px(Px(32.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            cx.pressable_toggle_bool(&open);
                            trigger_id = Some(id);
                            Vec::new()
                        },
                    ),
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                LayoutStyle {
                                    position: PositionStyle::Absolute,
                                    inset: InsetStyle {
                                        left: Some(Px(0.0)),
                                        top: Some(Px(120.0)),
                                        ..Default::default()
                                    },
                                    size: SizeStyle {
                                        width: Length::Px(Px(160.0)),
                                        height: Length::Px(Px(32.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            cx.pressable_toggle_bool(&underlay_clicked);
                            underlay_id = Some(id);
                            Vec::new()
                        },
                    ),
                ]
            },
        )]
    });
    ui.set_root(root);

    (
        trigger_id.expect("trigger id"),
        underlay_id.expect("underlay id"),
    )
}

#[test]
fn dismissible_popover_closes_on_outside_press() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request and render a dismissible popover.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: vec![],
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.captured(), None);

    // Pointer down outside should close (observer pass).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(250.0), Px(180.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn dismissible_popover_does_not_close_on_inside_press() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request and render a dismissible popover with a non-pressable child so
    // the pointer-down bubbles to the root in the normal dispatch path.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    let root_name = popover_root_name(trigger);
    let children = fret_ui::elements::with_element_cx(&mut app, window, bounds, &root_name, |cx| {
        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: fret_ui::element::InsetStyle {
                        top: Some(Px(40.0)),
                        left: Some(Px(40.0)),
                        ..Default::default()
                    },
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(Px(120.0)),
                        height: Length::Px(Px(80.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_| Vec::new(),
        )]
    });

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name,
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.captured(), None);

    // Pointer down inside the popover content should not close it.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(50.0), Px(50.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn dismissible_popover_does_not_close_on_outside_press_in_branch_subtree() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer with trigger + underlay pressable.
    let (_trigger, underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on the trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request a dismissible popover, with a branch pointing at the underlay subtree.
    begin_frame(&mut app, window);
    let (trigger, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: vec![underlay],
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Clicking the underlay subtree should remain click-through and should NOT dismiss.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(true));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));

    // Third frame: keep requesting the overlay; it should still be open.
    begin_frame(&mut app, window);
    let (trigger, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: vec![underlay],
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn dismissible_popover_treats_trigger_as_implicit_branch() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer with trigger + underlay pressable.
    let (_trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on the trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request a dismissible popover with no explicit branches.
    begin_frame(&mut app, window);
    let (trigger, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Clicking the trigger should close via toggle, and should NOT re-open due to outside-press
    // observer dismissal running before the trigger activation.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn dismissible_popover_closes_on_focus_change_outside() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer, open via trigger click.
    let (_trigger, underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request the popover and render it.
    begin_frame(&mut app, window);
    let (trigger, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Simulate a focus change outside of the overlay subtree (e.g. Tab navigation).
    let underlay_node =
        fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay node");
    ui.set_focus(Some(underlay_node));

    // Third frame: focus-outside should dismiss.
    begin_frame(&mut app, window);
    let (trigger, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn dismissible_popover_does_not_close_on_focus_change_to_trigger() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer, open via trigger click.
    let (_trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request the popover and render it.
    begin_frame(&mut app, window);
    let (trigger, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Simulate a focus change to the trigger element (branch subtree).
    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));

    // Third frame: focus is outside the overlay root but inside the trigger branch, so it should
    // remain open.
    begin_frame(&mut app, window);
    let (trigger, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn modal_blocks_underlay_click_and_closes_on_escape() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer contains a pressable that increments underlay_clicks.
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&underlay_clicked);
                    vec![]
                },
            )]
        },
    );
    ui.set_root(base);

    // Install modal layer.
    begin_frame(&mut app, window);
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| vec![],
            )]
        });
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0xabc),
            root_name: modal_root_name(GlobalElementId(0xabc)),
            trigger: None,
            open: open.clone(),
            present: true,
            initial_focus: None,
            children: modal_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Clicking underlay area should not reach base (modal barrier blocks underlay input).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));

    // Escape should close via DismissibleLayer.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn modal_can_remain_present_while_still_blocking_underlay_during_close_animation() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);
    let overlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer contains a full-size pressable we expect NOT to receive the click.
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&underlay_clicked);
                    vec![]
                },
            )]
        },
    );
    ui.set_root(base);

    // Install a modal layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let modal_id = GlobalElementId(0xbeef);
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: false,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&overlay_clicked);
                    vec![]
                },
            )]
        });
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: None,
            open,
            present: true,
            initial_focus: None,
            children: modal_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
    assert_eq!(app.models().get_copied(&overlay_clicked), Some(true));
}

#[test]
fn non_modal_overlay_can_remain_present_while_pointer_transparent_during_close_animation() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);
    let overlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer contains a full-size pressable we expect to receive the click.
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&underlay_clicked);
                    vec![]
                },
            )]
        },
    );
    ui.set_root(base);

    // Install a non-modal layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let trigger = GlobalElementId(0xdead);
    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "popover-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: false,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&overlay_clicked);
                    vec![]
                },
            )]
        });

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open,
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
    assert_eq!(app.models().get_copied(&overlay_clicked), Some(false));
}

#[test]
fn non_modal_overlay_does_not_request_outside_press_observer_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    // Install a non-modal layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let trigger = GlobalElementId(0xdead);
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open,
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.popovers.get(&(window, trigger)).map(|p| p.layer)
        })
        .expect("popover layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("popover debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert!(!info.wants_pointer_down_outside_events);
}

#[test]
fn non_modal_overlay_restores_focus_when_focus_is_missing_on_unmount() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: request and render a dismissible popover (open=true).
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.set_focus(None);

    // Third frame: do not request the popover (present=false / unmounted), and expect focus to
    // be restored to the trigger since focus is missing.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn non_modal_overlay_does_not_restore_focus_when_focus_moves_to_underlay_on_unmount() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let (trigger, underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request and render a dismissible popover.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "overlay-children", |cx| {
            vec![cx.container(
                ContainerProps {
                    layout: {
                        LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(Px(0.0)),
                                top: Some(Px(40.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(Px(200.0)),
                                height: Length::Px(Px(40.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    },
                    ..Default::default()
                },
                |_| Vec::new(),
            )]
        });
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Click underlay while popover is open: outside press is click-through, so focus should move
    // to the underlay target (and the popover should close).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));

    let underlay_node =
        fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay node");
    assert_eq!(ui.focus(), Some(underlay_node));

    // Third frame: unmount the popover. Focus restoration must not override the new underlay focus.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(underlay_node));
}

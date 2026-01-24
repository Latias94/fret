use super::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct PhaseLog {
    entries: Arc<Mutex<Vec<(fret_runtime::InputDispatchPhase, &'static str)>>>,
}

impl PhaseLog {
    fn push(&self, phase: fret_runtime::InputDispatchPhase, name: &'static str) {
        self.entries.lock().unwrap().push((phase, name));
    }

    fn take(&self) -> Vec<(fret_runtime::InputDispatchPhase, &'static str)> {
        std::mem::take(&mut *self.entries.lock().unwrap())
    }
}

struct PhaseLogWidget {
    name: &'static str,
    log: PhaseLog,
    stop_in_capture: bool,
}

impl PhaseLogWidget {
    fn new(name: &'static str, log: PhaseLog) -> Self {
        Self {
            name,
            log,
            stop_in_capture: false,
        }
    }
}

impl<H: UiHost> Widget<H> for PhaseLogWidget {
    fn event_capture(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::Pointer(
                PointerEvent::Down { .. }
                | PointerEvent::Up { .. }
                | PointerEvent::Move { .. }
                | PointerEvent::Wheel { .. }
                | PointerEvent::PinchGesture { .. },
            )
            | Event::PointerCancel(_)
            | Event::KeyDown { .. }
            | Event::KeyUp { .. } => {}
            _ => return,
        }

        self.log.push(cx.input_ctx.dispatch_phase, self.name);
        if self.stop_in_capture {
            cx.stop_propagation();
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::Pointer(
                PointerEvent::Down { .. }
                | PointerEvent::Up { .. }
                | PointerEvent::Move { .. }
                | PointerEvent::Wheel { .. }
                | PointerEvent::PinchGesture { .. },
            )
            | Event::PointerCancel(_)
            | Event::KeyDown { .. }
            | Event::KeyUp { .. } => {}
            _ => return,
        }
        self.log.push(cx.input_ctx.dispatch_phase, self.name);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            cx.paint(child, cx.bounds);
        }
        let _ = cx.scene;
    }
}

fn setup_ui(
    log: PhaseLog,
    root_stop_in_capture: bool,
) -> (
    crate::test_host::TestHost,
    UiTree<crate::test_host::TestHost>,
    FakeUiServices,
    NodeId,
    NodeId,
) {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut root_widget = PhaseLogWidget::new("root", log.clone());
    root_widget.stop_in_capture = root_stop_in_capture;
    let root = ui.create_node(root_widget);
    let child = ui.create_node(PhaseLogWidget::new("child", log));
    ui.set_root(root);
    ui.set_children(root, vec![child]);

    let mut services = FakeUiServices;
    ui.layout_all(
        &mut app,
        &mut services,
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ),
        1.0,
    );

    (app, ui, services, root, child)
}

#[test]
fn pointer_down_dispatches_capture_then_bubble() {
    let log = PhaseLog {
        entries: Arc::new(Mutex::new(Vec::new())),
    };
    let (mut app, mut ui, mut services, _root, _child) = setup_ui(log.clone(), false);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        log.take(),
        vec![
            (fret_runtime::InputDispatchPhase::Capture, "root"),
            (fret_runtime::InputDispatchPhase::Capture, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "root"),
        ]
    );
}

#[test]
fn stop_propagation_in_capture_skips_bubble() {
    let log = PhaseLog {
        entries: Arc::new(Mutex::new(Vec::new())),
    };
    let (mut app, mut ui, mut services, _root, _child) = setup_ui(log.clone(), true);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        log.take(),
        vec![(fret_runtime::InputDispatchPhase::Capture, "root")]
    );
}

#[test]
fn key_down_dispatches_capture_then_bubble() {
    let log = PhaseLog {
        entries: Arc::new(Mutex::new(Vec::new())),
    };
    let (mut app, mut ui, mut services, _root, child) = setup_ui(log.clone(), false);
    ui.set_focus(Some(child));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::KeyK,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        log.take(),
        vec![
            (fret_runtime::InputDispatchPhase::Capture, "root"),
            (fret_runtime::InputDispatchPhase::Capture, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "root"),
        ]
    );
}

#[test]
fn key_up_dispatches_capture_then_bubble() {
    let log = PhaseLog {
        entries: Arc::new(Mutex::new(Vec::new())),
    };
    let (mut app, mut ui, mut services, _root, child) = setup_ui(log.clone(), false);
    ui.set_focus(Some(child));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyUp {
            key: fret_core::KeyCode::KeyK,
            modifiers: fret_core::Modifiers::default(),
        },
    );

    assert_eq!(
        log.take(),
        vec![
            (fret_runtime::InputDispatchPhase::Capture, "root"),
            (fret_runtime::InputDispatchPhase::Capture, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "root"),
        ]
    );
}

#[test]
fn pointer_up_dispatches_capture_then_bubble() {
    let log = PhaseLog {
        entries: Arc::new(Mutex::new(Vec::new())),
    };
    let (mut app, mut ui, mut services, _root, _child) = setup_ui(log.clone(), false);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        log.take(),
        vec![
            (fret_runtime::InputDispatchPhase::Capture, "root"),
            (fret_runtime::InputDispatchPhase::Capture, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "root"),
        ]
    );
}

#[test]
fn pointer_wheel_dispatches_capture_then_bubble() {
    let log = PhaseLog {
        entries: Arc::new(Mutex::new(Vec::new())),
    };
    let (mut app, mut ui, mut services, _root, _child) = setup_ui(log.clone(), false);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: Point::new(Px(0.0), Px(-8.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        log.take(),
        vec![
            (fret_runtime::InputDispatchPhase::Capture, "root"),
            (fret_runtime::InputDispatchPhase::Capture, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "root"),
        ]
    );
}

#[test]
fn pointer_move_with_buttons_dispatches_capture_then_bubble() {
    let log = PhaseLog {
        entries: Arc::new(Mutex::new(Vec::new())),
    };
    let (mut app, mut ui, mut services, _root, _child) = setup_ui(log.clone(), false);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons {
                left: true,
                right: false,
                middle: false,
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        log.take(),
        vec![
            (fret_runtime::InputDispatchPhase::Capture, "root"),
            (fret_runtime::InputDispatchPhase::Capture, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "root"),
        ]
    );
}

#[test]
fn pointer_move_without_buttons_skips_capture() {
    let log = PhaseLog {
        entries: Arc::new(Mutex::new(Vec::new())),
    };
    let (mut app, mut ui, mut services, _root, _child) = setup_ui(log.clone(), false);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        log.take(),
        vec![
            (fret_runtime::InputDispatchPhase::Bubble, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "root"),
        ]
    );
}

#[test]
fn pointer_cancel_without_position_dispatches_capture_then_bubble_via_focus_path() {
    let log = PhaseLog {
        entries: Arc::new(Mutex::new(Vec::new())),
    };
    let (mut app, mut ui, mut services, _root, child) = setup_ui(log.clone(), false);
    ui.set_focus(Some(child));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: None,
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );

    assert_eq!(
        log.take(),
        vec![
            (fret_runtime::InputDispatchPhase::Capture, "root"),
            (fret_runtime::InputDispatchPhase::Capture, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "child"),
            (fret_runtime::InputDispatchPhase::Bubble, "root"),
        ]
    );
}

use super::*;

use fret_runtime::{
    CommandMeta, CommandScope, InputDispatchPhase, WindowCommandActionAvailabilityService,
    WindowMenuBarFocusService,
};

#[derive(Debug, Default)]
struct AvailabilityLeaf;

impl<H: UiHost> Widget<H> for AvailabilityLeaf {
    fn is_focusable(&self) -> bool {
        true
    }

    fn command_availability(
        &self,
        _cx: &mut crate::widget::CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> crate::widget::CommandAvailability {
        match command.as_str() {
            "test.available" => crate::widget::CommandAvailability::Available,
            "test.blocked" => crate::widget::CommandAvailability::Blocked,
            _ => crate::widget::CommandAvailability::NotHandled,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[derive(Debug, Default)]
struct FocusableLeaf;

impl<H: UiHost> Widget<H> for FocusableLeaf {
    fn is_focusable(&self) -> bool {
        true
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

fn widget_command_meta(title: &str) -> CommandMeta {
    CommandMeta::new(title).with_scope(CommandScope::Widget)
}

fn publish_snapshot(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
    window: AppWindowId,
) {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    let input_ctx = InputContext {
        platform: Platform::current(),
        caps,
        ui_has_modal: false,
        window_arbitration: None,
        focus_is_text_input: ui.focus_is_text_input(app),
        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
        edit_can_undo: true,
        edit_can_redo: true,
        dispatch_phase: InputDispatchPhase::Bubble,
    };

    ui.publish_window_command_action_availability_snapshot(app, &input_ctx);

    assert!(
        app.global::<WindowCommandActionAvailabilityService>()
            .and_then(|svc| svc.snapshot(window))
            .is_some()
    );
}

#[test]
fn action_availability_snapshot_skips_unhandled_commands() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );
    app.register_command(
        CommandId::from("test.blocked"),
        widget_command_meta("Blocked"),
    );
    app.register_command(
        CommandId::from("test.unhandled"),
        widget_command_meta("Unhandled"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack::default());
    let leaf = ui.create_node(AvailabilityLeaf::default());
    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.blocked")),
        Some(false)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.unhandled")),
        None
    );
}

#[test]
fn action_availability_snapshot_publishes_focus_traversal_gating() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("focus.next"),
        widget_command_meta("Focus Next"),
    );
    app.register_command(
        CommandId::from("focus.previous"),
        widget_command_meta("Focus Previous"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack::default());
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("focus.next")),
        Some(false)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("focus.previous")),
        Some(false)
    );

    let leaf = ui.create_node(FocusableLeaf::default());
    ui.add_child(root, leaf);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("focus.next")),
        Some(true)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("focus.previous")),
        Some(true)
    );
}

#[test]
fn action_availability_snapshot_publishes_focus_menu_bar_gating() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("focus.menu_bar"),
        widget_command_meta("Focus Menu Bar"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack::default());
    let leaf = ui.create_node(FocusableLeaf::default());
    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("focus.menu_bar")),
        Some(false)
    );

    let mut focus_svc = WindowMenuBarFocusService::default();
    focus_svc.set_present(window, true);
    app.set_global(focus_svc);

    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("focus.menu_bar")),
        Some(true)
    );
}

#[derive(Debug, Default)]
struct FocusOnPointerDownAvailable;

impl<H: UiHost> Widget<H> for FocusOnPointerDownAvailable {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if matches!(event, Event::Pointer(fret_core::PointerEvent::Down { .. })) {
            cx.request_focus(cx.node);
            cx.stop_propagation();
        }
    }

    fn command_availability(
        &self,
        _cx: &mut crate::widget::CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> crate::widget::CommandAvailability {
        match command.as_str() {
            "test.available" => crate::widget::CommandAvailability::Available,
            _ => crate::widget::CommandAvailability::NotHandled,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn dispatch_event_publishes_action_availability_snapshot() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack::default());
    let leaf = ui.create_node(FocusOnPointerDownAvailable::default());
    ui.set_root(root);
    ui.add_child(root, leaf);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
}

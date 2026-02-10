#![cfg(any(target_os = "windows", target_os = "linux"))]

use super::*;

use fret_runtime::{CommandMeta, CommandScope, WindowMenuBarFocusService};

#[test]
fn alt_key_up_emits_focus_menu_bar_when_present() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("focus.menu_bar"),
        CommandMeta::new("Focus Menu Bar").with_scope(CommandScope::Widget),
    );

    let mut focus_svc = WindowMenuBarFocusService::default();
    focus_svc.set_present(window, true);
    app.set_global(focus_svc);

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::AltLeft,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyUp {
            key: KeyCode::AltLeft,
            modifiers: fret_core::Modifiers::default(),
        },
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Command {
                window: Some(w),
                command
            } if *w == window && command.as_str() == "focus.menu_bar"
        )),
        "expected an Effect::Command for focus.menu_bar"
    );
}

#[test]
fn alt_key_up_does_not_emit_when_canceled_by_other_key() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("focus.menu_bar"),
        CommandMeta::new("Focus Menu Bar").with_scope(CommandScope::Widget),
    );

    let mut focus_svc = WindowMenuBarFocusService::default();
    focus_svc.set_present(window, true);
    app.set_global(focus_svc);

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::AltLeft,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::KeyA,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyUp {
            key: KeyCode::AltLeft,
            modifiers: fret_core::Modifiers::default(),
        },
    );

    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|e| matches!(e, Effect::Command { .. })),
        "expected no command effects when Alt was canceled"
    );
}

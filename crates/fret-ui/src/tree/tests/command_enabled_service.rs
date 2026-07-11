use super::*;

use crate::action::{
    ActionCx, ActivateReason, UiActionHost, UiActionHostAdapter,
    record_pending_action_payload_if_enabled,
};
use crate::elements::GlobalElementId;
use fret_core::{AppWindowId, Event, KeyCode, Modifiers, Point, Px, Rect, Size};
use fret_runtime::keymap::Binding;
use fret_runtime::{
    CommandId, CommandMeta, CommandScope, Keymap, KeymapService, PlatformCapabilities,
    PlatformFilter, WindowCommandActionAvailabilityService, WindowMenuBarFocusService,
};
use std::collections::HashMap;

#[test]
fn disabled_ui_action_does_not_leave_pending_source_or_payload() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let command = CommandId::from("workspace.dirty_close.discard");
    app.with_global_mut(
        fret_runtime::WindowCommandEnabledService::default,
        |service, _app| service.set_enabled(window, command.clone(), false),
    );

    let action_cx = ActionCx {
        window,
        target: GlobalElementId(42),
    };
    let mut host = UiActionHostAdapter { app: &mut app };
    host.record_pending_command_dispatch_source(action_cx, &command, ActivateReason::Pointer);
    assert!(!record_pending_action_payload_if_enabled(
        &mut app,
        action_cx,
        &command,
        Box::new(7u32),
    ));

    let source = app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |service, app| service.consume(window, app.tick_id(), &command),
    );
    let payload = app.with_global_mut(
        fret_runtime::WindowPendingActionPayloadService::default,
        |service, app| service.consume(window, app.tick_id(), &command),
    );

    assert_eq!(source, None);
    assert!(payload.is_none());
}

#[cfg(feature = "diagnostics")]
#[test]
fn disabled_shortcut_effect_does_not_leave_pending_source() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let command = CommandId::from("workspace.dirty_close.discard");
    app.with_global_mut(
        fret_runtime::WindowCommandEnabledService::default,
        |service, _app| service.set_enabled(window, command.clone(), false),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.push_shortcut_command_effect(&mut app, command.clone());

    assert!(app.take_effects().is_empty());
    let source = app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |service, app| service.consume(window, app.tick_id(), &command),
    );
    assert_eq!(source, None);
}

#[test]
fn shortcut_dispatch_respects_window_command_enabled_service() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let command = CommandId::from("test.command");

    let mut keymap = Keymap::empty();
    keymap.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![fret_runtime::KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        )],
        when: None,
        command: Some(command.clone()),
    });
    app.set_global(KeymapService { keymap });

    app.with_global_mut(
        fret_runtime::WindowCommandEnabledService::default,
        |svc, _app| {
            svc.set_enabled(window, command.clone(), false);
        },
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::KeyP,
            modifiers: Modifiers {
                ctrl: true,
                ..Default::default()
            },
            repeat: false,
        },
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().all(
            |e| !matches!(e, fret_runtime::Effect::Command { command: c, .. } if c == &command)
        ),
        "disabled command should not dispatch via shortcuts"
    );
}

#[cfg(feature = "diagnostics")]
#[test]
fn unhandled_shortcut_effect_preserves_dispatch_source_for_app_driver() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let command = CommandId::from("test.unhandled_shortcut");
    let mut keymap = Keymap::empty();
    keymap.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![fret_runtime::KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        )],
        when: None,
        command: Some(command.clone()),
    });
    app.set_global(KeymapService { keymap });

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    ui.layout_in(
        &mut app,
        &mut services,
        root,
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ),
        1.0,
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::KeyP,
            modifiers: Modifiers {
                ctrl: true,
                ..Default::default()
            },
            repeat: false,
        },
    );

    assert!(app.take_effects().iter().any(
        |effect| matches!(effect, fret_runtime::Effect::Command { command: actual, .. } if actual == &command)
    ));
    let ui_decision = app
        .global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
        .expect("UI dispatch should record the unhandled attempt")
        .snapshot_since(window, 0, 10)
        .into_iter()
        .find(|decision| decision.command == command)
        .expect("expected an unhandled UI dispatch decision");
    assert!(!ui_decision.handled);
    assert_eq!(
        ui_decision.source.kind,
        fret_runtime::CommandDispatchSourceKindV1::Shortcut
    );

    let pending_source = app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |service, app| service.consume(window, app.tick_id(), &command),
    );
    assert_eq!(
        pending_source.map(|source| source.kind),
        Some(fret_runtime::CommandDispatchSourceKindV1::Shortcut),
        "the app driver must receive the same shortcut source after UI fallback"
    );
}

#[test]
fn shortcut_dispatch_respects_window_command_action_availability_snapshot() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let command = CommandId::from("test.command");

    app.register_command(
        command.clone(),
        CommandMeta::new("Test").with_scope(CommandScope::Widget),
    );

    let mut keymap = Keymap::empty();
    keymap.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![fret_runtime::KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        )],
        when: None,
        command: Some(command.clone()),
    });
    app.set_global(KeymapService { keymap });

    app.with_global_mut(
        WindowCommandActionAvailabilityService::default,
        |svc, _app| {
            let mut availability: HashMap<CommandId, bool> = HashMap::new();
            availability.insert(command.clone(), false);
            svc.set_snapshot(window, availability);
        },
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::KeyP,
            modifiers: Modifiers {
                ctrl: true,
                ..Default::default()
            },
            repeat: false,
        },
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().all(
            |e| !matches!(e, fret_runtime::Effect::Command { command: c, .. } if c == &command)
        ),
        "unavailable widget-scope command should not dispatch via shortcuts"
    );
}

#[test]
fn focus_menu_bar_shortcut_dispatches_when_menu_bar_focus_service_is_present() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let command = CommandId::from("focus.menu_bar");

    app.register_command(
        command.clone(),
        CommandMeta::new("Focus Menu Bar").with_scope(CommandScope::Widget),
    );

    let mut keymap = Keymap::empty();
    keymap.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![fret_runtime::KeyChord::new(
            KeyCode::F10,
            Modifiers::default(),
        )],
        when: None,
        command: Some(command.clone()),
    });
    app.set_global(KeymapService { keymap });

    let mut focus_svc = WindowMenuBarFocusService::default();
    focus_svc.set_present(window, true);
    app.set_global(focus_svc);

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::F10,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(
            |e| matches!(e, fret_runtime::Effect::Command { command: c, .. } if c == &command)
        ),
        "focus.menu_bar should dispatch via shortcut when menu bar focus is present"
    );
}

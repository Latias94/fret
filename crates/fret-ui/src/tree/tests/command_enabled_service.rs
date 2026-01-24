use super::*;

use fret_core::{AppWindowId, Event, KeyCode, Modifiers, Point, Px, Rect, Size};
use fret_runtime::keymap::Binding;
use fret_runtime::{
    CommandId, CommandMeta, CommandScope, Keymap, KeymapService, PlatformCapabilities,
    PlatformFilter, WindowCommandActionAvailabilityService,
};
use std::collections::HashMap;

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

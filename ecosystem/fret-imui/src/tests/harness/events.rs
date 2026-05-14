use super::*;

pub(crate) fn click_at(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
) {
    click_at_with_modifiers(ui, app, services, at, Modifiers::default());
}

pub(crate) fn click_at_with_modifiers(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
    modifiers: Modifiers,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Left,
            modifiers,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Left,
            modifiers,
            is_click: true,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
}

pub(crate) fn double_click_at(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 2,
            pointer_type: PointerType::Mouse,
        }),
    );
}

pub(crate) fn right_click_at(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
}

pub(crate) fn pointer_move_at(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
    buttons: MouseButtons,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: PointerId(0),
            position: at,
            buttons,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
}

pub(crate) fn key_down(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    key: KeyCode,
    modifiers: Modifiers,
) {
    key_down_with_repeat(ui, app, services, key, modifiers, false);
}

pub(crate) fn key_down_with_repeat(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    key: KeyCode,
    modifiers: Modifiers,
    repeat: bool,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyDown {
            key,
            modifiers,
            repeat,
        },
    );
}

pub(crate) fn ctrl_modifiers() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Default::default()
    }
}

pub(crate) fn ctrl_shortcut(key: KeyCode) -> KeyChord {
    KeyChord::new(key, ctrl_modifiers())
}

pub(crate) fn key_down_ctrl(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    key: KeyCode,
) {
    key_down(ui, app, services, key, ctrl_modifiers());
}

pub(crate) fn key_down_ctrl_repeat(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    key: KeyCode,
) {
    key_down_with_repeat(ui, app, services, key, ctrl_modifiers(), true);
}

pub(crate) fn text_input_event(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    text: &str,
) {
    ui.dispatch_event(app, services, &Event::TextInput(text.to_string()));
}

pub(crate) fn pointer_down_at(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
}

pub(crate) fn pointer_up_at(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
) {
    pointer_up_at_with_is_click(ui, app, services, at, true);
}

pub(crate) fn pointer_up_at_with_is_click(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
    is_click: bool,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
}

pub(crate) fn dispatch_all_timers(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
) -> usize {
    let mut pending: Vec<TimerToken> = Vec::new();
    for effect in &app.effects {
        if let Effect::SetTimer { token, repeat, .. } = effect
            && repeat.is_none()
        {
            pending.push(*token);
        }
    }
    app.effects
        .retain(|effect| !matches!(effect, Effect::SetTimer { repeat, .. } if repeat.is_none()));

    let dispatched = pending.len();
    for token in pending {
        ui.dispatch_event(app, services, &Event::Timer { token });
    }
    dispatched
}

pub(crate) fn pending_nonrepeating_timer_tokens(app: &TestHost) -> Vec<TimerToken> {
    let mut pending: Vec<TimerToken> = Vec::new();
    for effect in &app.effects {
        if let Effect::SetTimer { token, repeat, .. } = effect
            && repeat.is_none()
        {
            pending.push(*token);
        }
    }
    pending
}

pub(crate) fn dispatch_timer_tokens(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    tokens: &[TimerToken],
) -> usize {
    let mut dispatched = 0usize;
    for token in tokens {
        let token = *token;
        let mut removed = false;
        app.effects.retain(|effect| {
            let is_match = matches!(
                effect,
                Effect::SetTimer { token: t, repeat, .. } if *t == token && repeat.is_none()
            );
            if is_match {
                removed = true;
            }
            !is_match
        });
        if removed {
            dispatched += 1;
            ui.dispatch_event(app, services, &Event::Timer { token });
        }
    }
    dispatched
}

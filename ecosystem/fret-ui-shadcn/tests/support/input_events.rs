#![allow(dead_code)]

use fret_app::App;
use fret_core::{
    Event, KeyCode, Modifiers, MouseButton, Point, PointerEvent, PointerId, PointerType,
};
use fret_ui::tree::UiTree;

pub(crate) fn dispatch_key_press(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyDown {
            key,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        app,
        services,
        &Event::KeyUp {
            key,
            modifiers: Modifiers::default(),
        },
    );
}

pub(crate) fn click_at(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    position: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: PointerId(0),
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: PointerId(0),
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

pub(crate) fn right_click_at(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    position: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: PointerId(0),
            position,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: PointerId(0),
            position,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

pub(crate) fn dispatch_text_input(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    text: impl Into<String>,
) {
    ui.dispatch_event(app, services, &Event::TextInput(text.into()));
}

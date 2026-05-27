use fret_core::{
    AppWindowId, Event, KeyCode, Modifiers, MouseButton, MouseButtons, Point, PointerEvent,
    PointerId, PointerType,
};
use fret_runtime::Effect;

use super::host::TestHost;

pub(crate) fn pointer_down(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Down {
        pointer_id,
        position,
        button: MouseButton::Left,
        modifiers: Modifiers::default(),
        click_count: 1,
        pointer_type: PointerType::Mouse,
    })
}

pub(crate) fn pointer_move(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_type: PointerType::Mouse,
    })
}

pub(crate) fn pointer_move_touch(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_type: PointerType::Touch,
    })
}

pub(crate) fn pointer_up(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Up {
        pointer_id,
        position,
        button: MouseButton::Left,
        modifiers: Modifiers::default(),
        is_click: true,
        click_count: 1,
        pointer_type: PointerType::Mouse,
    })
}

pub(crate) fn key_down(key: KeyCode) -> Event {
    Event::KeyDown {
        key,
        modifiers: Modifiers::default(),
        repeat: false,
    }
}

pub(crate) fn key_up(key: KeyCode) -> Event {
    Event::KeyUp {
        key,
        modifiers: Modifiers::default(),
    }
}

pub(crate) fn drain_zero_delay_timer_tokens(
    app: &mut TestHost,
    window: AppWindowId,
) -> Vec<fret_runtime::TimerToken> {
    let mut out: Vec<fret_runtime::TimerToken> = Vec::new();
    app.effects.retain(|effect| match effect {
        Effect::SetTimer {
            window: Some(w),
            token,
            after,
            repeat: None,
        } if *w == window && after.as_millis() == 0 => {
            out.push(*token);
            false
        }
        _ => true,
    });
    out
}

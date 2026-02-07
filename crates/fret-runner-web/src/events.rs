use std::collections::HashMap;

use fret_core::{
    Event, KeyCode, Modifiers, MouseButton, MouseButtons, PointerCancelEvent, PointerCancelReason,
    PointerEvent, PointerId, PointerType,
    geometry::{Point, Px},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebPointerEventKind {
    Down,
    Move,
    Up,
    Wheel,
    Leave,
    Cancel,
}

#[derive(Debug, Default, Clone)]
pub struct WebInputState {
    pointers: HashMap<PointerId, PointerButtonsAndClicks>,
}

#[derive(Debug, Default, Clone, Copy)]
struct PointerButtonsAndClicks {
    buttons: MouseButtons,
    click: ClickTracker,
}

impl WebInputState {
    pub fn handle_pointer_event(
        &mut self,
        kind: WebPointerEventKind,
        event: &web_sys::PointerEvent,
    ) -> Option<Event> {
        let pointer_id = map_pointer_id(event);
        let pointer_type = map_pointer_type(event);
        let modifiers = map_modifiers_from_pointer(event);
        let position = map_pointer_position(event);
        let buttons = map_buttons(event);

        let state = self
            .pointers
            .entry(pointer_id)
            .or_insert_with(PointerButtonsAndClicks::default);
        state.buttons = buttons;
        state.click.update_move(position);

        match kind {
            WebPointerEventKind::Move => Some(Event::Pointer(PointerEvent::Move {
                pointer_id,
                position,
                buttons,
                modifiers,
                pointer_type,
            })),
            WebPointerEventKind::Down => {
                let button = map_button(event);
                let click_count = state.click.begin_press(button, position);
                Some(Event::Pointer(PointerEvent::Down {
                    pointer_id,
                    position,
                    button,
                    modifiers,
                    click_count,
                    pointer_type,
                }))
            }
            WebPointerEventKind::Up => {
                let button = map_button(event);
                let (click_count, is_click) = state.click.end_press(button, position);
                Some(Event::Pointer(PointerEvent::Up {
                    pointer_id,
                    position,
                    button,
                    modifiers,
                    is_click,
                    click_count,
                    pointer_type,
                }))
            }
            WebPointerEventKind::Wheel => None,
            WebPointerEventKind::Leave => Some(Event::PointerCancel(PointerCancelEvent {
                pointer_id,
                position: Some(position),
                buttons,
                modifiers,
                pointer_type,
                reason: PointerCancelReason::LeftWindow,
            })),
            WebPointerEventKind::Cancel => Some(Event::PointerCancel(PointerCancelEvent {
                pointer_id,
                position: Some(position),
                buttons,
                modifiers,
                pointer_type,
                reason: PointerCancelReason::LeftWindow,
            })),
        }
    }

    pub fn handle_wheel_event(
        &mut self,
        pointer_id: PointerId,
        pointer_type: PointerType,
        event: &web_sys::WheelEvent,
    ) -> Event {
        let modifiers = map_modifiers_from_wheel(event);
        let position = map_wheel_position(event);
        let delta = map_wheel_delta(event);

        Event::Pointer(PointerEvent::Wheel {
            pointer_id,
            position,
            delta,
            modifiers,
            pointer_type,
        })
    }
}

pub fn map_keyboard_event(event: &web_sys::KeyboardEvent, is_down: bool) -> Option<Event> {
    let key: KeyCode = event.code().parse().ok()?;
    let modifiers = map_modifiers_from_keyboard(event);
    if is_down {
        Some(Event::KeyDown {
            key,
            modifiers,
            repeat: event.repeat(),
        })
    } else {
        Some(Event::KeyUp { key, modifiers })
    }
}

fn map_pointer_id(event: &web_sys::PointerEvent) -> PointerId {
    // Spec: PointerEvent.pointerId is a signed long; treat negatives as 0.
    PointerId(event.pointer_id().max(0) as u64)
}

fn map_pointer_type(event: &web_sys::PointerEvent) -> PointerType {
    match event.pointer_type().as_str() {
        "mouse" => PointerType::Mouse,
        "touch" => PointerType::Touch,
        "pen" => PointerType::Pen,
        _ => PointerType::Unknown,
    }
}

fn map_pointer_position(event: &web_sys::PointerEvent) -> Point {
    // `offsetX/Y` is relative to the event target's padding edge.
    // Treat this as our logical coordinate space (CSS px).
    Point::new(Px(event.offset_x() as f32), Px(event.offset_y() as f32))
}

fn map_wheel_position(event: &web_sys::WheelEvent) -> Point {
    Point::new(Px(event.offset_x() as f32), Px(event.offset_y() as f32))
}

fn map_wheel_delta(event: &web_sys::WheelEvent) -> Point {
    // WheelEvent delta units can be lines/pages; treat values as pixels for now.
    Point::new(Px(event.delta_x() as f32), Px(event.delta_y() as f32))
}

fn map_buttons(event: &web_sys::PointerEvent) -> MouseButtons {
    let b = event.buttons();
    MouseButtons {
        left: (b & 1) != 0,
        right: (b & 2) != 0,
        middle: (b & 4) != 0,
    }
}

fn map_button(event: &web_sys::PointerEvent) -> MouseButton {
    match event.button() {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        3 => MouseButton::Back,
        4 => MouseButton::Forward,
        other => MouseButton::Other(other as u16),
    }
}

fn map_modifiers_from_keyboard(event: &web_sys::KeyboardEvent) -> Modifiers {
    Modifiers {
        shift: event.shift_key(),
        ctrl: event.ctrl_key(),
        alt: event.alt_key(),
        alt_gr: event.get_modifier_state("AltGraph"),
        meta: event.meta_key(),
    }
}

fn map_modifiers_from_pointer(event: &web_sys::PointerEvent) -> Modifiers {
    Modifiers {
        shift: event.shift_key(),
        ctrl: event.ctrl_key(),
        alt: event.alt_key(),
        alt_gr: event.get_modifier_state("AltGraph"),
        meta: event.meta_key(),
    }
}

fn map_modifiers_from_wheel(event: &web_sys::WheelEvent) -> Modifiers {
    Modifiers {
        shift: event.shift_key(),
        ctrl: event.ctrl_key(),
        alt: event.alt_key(),
        alt_gr: event.get_modifier_state("AltGraph"),
        meta: event.meta_key(),
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ClickState {
    last_time: Option<fret_core::time::Instant>,
    last_pos: Point,
    count: u8,
}

#[derive(Debug, Default, Clone, Copy)]
struct PressState {
    start_pos: Point,
    click_count: u8,
    moved: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct ClickTracker {
    left: ClickState,
    right: ClickState,
    middle: ClickState,
    back: ClickState,
    forward: ClickState,
    press_left: Option<PressState>,
    press_right: Option<PressState>,
    press_middle: Option<PressState>,
    press_back: Option<PressState>,
    press_forward: Option<PressState>,
}

impl ClickTracker {
    const CLICK_SLOP_PX: f32 = 6.0;
    const MULTI_CLICK_MAX_DELAY: std::time::Duration = std::time::Duration::from_millis(500);

    fn begin_press(&mut self, button: MouseButton, pos: Point) -> u8 {
        if matches!(button, MouseButton::Other(_)) {
            return 1;
        }
        let now = fret_core::time::Instant::now();
        let (state, press) = self.state_for_button_mut(button);
        let count = match state.last_time {
            Some(t)
                if now.duration_since(t) <= Self::MULTI_CLICK_MAX_DELAY
                    && distance_px(pos, state.last_pos) <= Self::CLICK_SLOP_PX =>
            {
                state.count.saturating_add(1).max(2)
            }
            _ => 1,
        };

        *press = Some(PressState {
            start_pos: pos,
            click_count: count,
            moved: false,
        });
        count
    }

    fn update_move(&mut self, pos: Point) {
        for press in [
            &mut self.press_left,
            &mut self.press_right,
            &mut self.press_middle,
            &mut self.press_back,
            &mut self.press_forward,
        ] {
            let Some(st) = press.as_mut() else {
                continue;
            };
            if st.moved {
                continue;
            }
            if distance_px(pos, st.start_pos) > Self::CLICK_SLOP_PX {
                st.moved = true;
            }
        }
    }

    fn end_press(&mut self, button: MouseButton, pos: Point) -> (u8, bool) {
        if matches!(button, MouseButton::Other(_)) {
            return (1, true);
        }
        let now = fret_core::time::Instant::now();
        let (state, press) = self.state_for_button_mut(button);
        let Some(press_state) = press.take() else {
            return (1, false);
        };

        let is_click =
            !press_state.moved && distance_px(pos, press_state.start_pos) <= Self::CLICK_SLOP_PX;
        if is_click {
            state.last_time = Some(now);
            state.last_pos = pos;
            state.count = press_state.click_count;
        }

        (press_state.click_count.max(1), is_click)
    }

    fn state_for_button_mut(
        &mut self,
        button: MouseButton,
    ) -> (&mut ClickState, &mut Option<PressState>) {
        match button {
            MouseButton::Left => (&mut self.left, &mut self.press_left),
            MouseButton::Right => (&mut self.right, &mut self.press_right),
            MouseButton::Middle => (&mut self.middle, &mut self.press_middle),
            MouseButton::Back => (&mut self.back, &mut self.press_back),
            MouseButton::Forward => (&mut self.forward, &mut self.press_forward),
            MouseButton::Other(_) => (&mut self.left, &mut self.press_left),
        }
    }
}

fn distance_px(a: Point, b: Point) -> f32 {
    let dx = a.x.0 - b.x.0;
    let dy = a.y.0 - b.y.0;
    (dx * dx + dy * dy).sqrt()
}

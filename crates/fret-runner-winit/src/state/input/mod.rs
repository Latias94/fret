use fret_core::time::Instant;
use fret_core::{
    Event, Modifiers, MouseButton, MouseButtons, Point, PointerCancelEvent, PointerCancelReason,
    PointerEvent, PointerId, Px,
};
use std::collections::HashMap;
use std::time::Duration;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};
use winit::event::{ElementState, KeyEvent, PointerSource, WindowEvent};
use winit::keyboard::ModifiersState;

use crate::mapping::{
    WheelConfig, is_alt_gr_key, map_modifiers, map_mouse_button, map_physical_key,
    map_pointer_button, map_pointer_id_from_button_source, map_pointer_id_from_pointer_kind,
    map_pointer_id_from_pointer_source, map_pointer_kind, map_pointer_type,
    map_pointer_type_from_pointer_source, map_wheel_delta, sanitize_text_input, set_mouse_buttons,
};

#[derive(Debug, Default, Clone)]
pub struct WinitInputState {
    pub cursor_pos: Point,
    pub cursor_pos_physical: Option<PhysicalPosition<f64>>,
    /// Mouse button state for the primary mouse pointer (`PointerId(0)`).
    ///
    /// This is a compatibility view used by higher-level runner glue. Multi-pointer state is
    /// tracked in `pointers`.
    pub pressed_buttons: MouseButtons,
    pub modifiers: Modifiers,
    pub raw_modifiers: ModifiersState,
    pub alt_gr_down: bool,
    pub last_pointer_type: fret_core::PointerType,
    pointers: HashMap<PointerId, PointerState>,
}

#[derive(Debug, Default, Clone, Copy)]
struct PointerState {
    buttons: MouseButtons,
    click: ClickTracker,
}

#[derive(Debug, Default, Clone, Copy)]
struct ClickState {
    last_time: Option<Instant>,
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
    const MULTI_CLICK_MAX_DELAY: Duration = Duration::from_millis(500);

    fn begin_press(&mut self, button: MouseButton, pos: Point) -> u8 {
        if matches!(button, MouseButton::Other(_)) {
            return 1;
        }
        let now = Instant::now();
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
        let now = Instant::now();
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

impl WinitInputState {
    fn pointer_state_mut(&mut self, pointer_id: PointerId) -> &mut PointerState {
        let state = self.pointers.entry(pointer_id).or_default();
        if pointer_id == PointerId(0) {
            state.buttons = self.pressed_buttons;
        }
        state
    }

    fn pointer_buttons(&self, pointer_id: PointerId) -> MouseButtons {
        if pointer_id == PointerId(0) {
            return self.pressed_buttons;
        }
        self.pointers
            .get(&pointer_id)
            .map(|state| state.buttons)
            .unwrap_or_default()
    }

    pub fn handle_window_event(
        &mut self,
        window_scale_factor: f64,
        event: &WindowEvent,
        out: &mut Vec<Event>,
    ) {
        self.handle_window_event_with_config(
            window_scale_factor,
            event,
            WheelConfig::default(),
            out,
        );
    }

    pub fn handle_window_event_with_config(
        &mut self,
        window_scale_factor: f64,
        event: &WindowEvent,
        wheel: WheelConfig,
        out: &mut Vec<Event>,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                out.push(Event::WindowCloseRequested);
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.raw_modifiers = mods.state();
                self.modifiers = map_modifiers(self.raw_modifiers, self.alt_gr_down);
            }
            WindowEvent::Moved(position) => {
                let logical = position.to_logical::<f32>(window_scale_factor);
                out.push(Event::WindowMoved(fret_core::WindowLogicalPosition {
                    x: logical.x.round() as i32,
                    y: logical.y.round() as i32,
                }));
            }
            WindowEvent::Ime(ime) => {
                let mapped = match ime {
                    winit::event::Ime::Enabled => fret_core::ImeEvent::Enabled,
                    winit::event::Ime::Disabled => fret_core::ImeEvent::Disabled,
                    winit::event::Ime::Commit(text) => fret_core::ImeEvent::Commit(text.clone()),
                    winit::event::Ime::Preedit(text, cursor) => fret_core::ImeEvent::Preedit {
                        text: text.clone(),
                        cursor: *cursor,
                    },
                    winit::event::Ime::DeleteSurrounding {
                        before_bytes,
                        after_bytes,
                    } => fret_core::ImeEvent::DeleteSurrounding {
                        before_bytes: *before_bytes,
                        after_bytes: *after_bytes,
                    },
                };
                out.push(Event::Ime(mapped));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key_event(event, out);
            }
            WindowEvent::PointerMoved {
                device_id,
                position,
                source,
                ..
            } => {
                // Some platforms/reporters can emit `PointerMoved` with `PointerSource::Unknown`
                // even for the primary mouse cursor. If we treat that as a distinct pointer id,
                // higher-level UI (which often assumes `PointerId(0)` for mouse) will see a
                // mismatch between button events (mouse) and move events (unknown), breaking
                // hover/drag interactions (e.g. docking float-zone hover or floating window drag).
                //
                // Be conservative: map `Unknown` pointer-moves to the primary mouse pointer.
                let (pointer_id, pointer_type) = if matches!(source, PointerSource::Unknown) {
                    (PointerId(0), self.last_pointer_type)
                } else {
                    (
                        map_pointer_id_from_pointer_source(*device_id, source),
                        map_pointer_type_from_pointer_source(source),
                    )
                };

                let logical: LogicalPosition<f32> = position.to_logical(window_scale_factor);
                let pos = Point::new(Px(logical.x), Px(logical.y));

                if pointer_id == PointerId(0) {
                    self.cursor_pos_physical = Some(*position);
                    self.cursor_pos = pos;
                    self.last_pointer_type = pointer_type;
                }

                self.pointer_state_mut(pointer_id).click.update_move(pos);
                if std::env::var_os("FRET_WINIT_POINTER_DEBUG").is_some_and(|v| !v.is_empty())
                    && (pointer_id != PointerId(0) || matches!(source, PointerSource::Unknown))
                {
                    tracing::info!(
                        pointer_id = ?pointer_id,
                        pointer_type = ?pointer_type,
                        source = ?source,
                        device_id = ?device_id,
                        pos = ?pos,
                        buttons = ?self.pointer_buttons(pointer_id),
                        modifiers = ?self.modifiers,
                        "winit pointer move"
                    );
                }
                out.push(Event::Pointer(PointerEvent::Move {
                    pointer_id,
                    position: pos,
                    buttons: self.pointer_buttons(pointer_id),
                    modifiers: self.modifiers,
                    pointer_type,
                }));
            }
            WindowEvent::PointerLeft {
                device_id,
                position,
                kind,
                ..
            } => {
                let pos = position.map(|position| {
                    let logical: LogicalPosition<f32> = position.to_logical(window_scale_factor);
                    Point::new(Px(logical.x), Px(logical.y))
                });
                let pointer_type = map_pointer_kind(*kind);
                let pointer_id = map_pointer_id_from_pointer_kind(*device_id, *kind);
                if pointer_id == PointerId(0) {
                    if let Some(pos) = pos {
                        self.cursor_pos = pos;
                    }
                    self.last_pointer_type = pointer_type;
                }

                let buttons = self.pointer_buttons(pointer_id);
                out.push(Event::PointerCancel(PointerCancelEvent {
                    pointer_id,
                    position: pos,
                    buttons,
                    modifiers: self.modifiers,
                    pointer_type,
                    reason: PointerCancelReason::LeftWindow,
                }));

                // `PointerLeft` may arrive without a matching button release (e.g. touch tracking
                // canceled by the OS). Reset runner-side state to avoid stuck buttons/click counts.
                self.pointers.remove(&pointer_id);
                if pointer_id == PointerId(0) {
                    self.pressed_buttons = MouseButtons::default();
                }
            }
            WindowEvent::PointerButton {
                state,
                device_id,
                position,
                button,
                ..
            } => {
                let pointer_id = map_pointer_id_from_button_source(*device_id, button);
                let logical: LogicalPosition<f32> = position.to_logical(window_scale_factor);
                let pos = Point::new(Px(logical.x), Px(logical.y));

                if pointer_id == PointerId(0) {
                    self.cursor_pos_physical = Some(*position);
                    self.cursor_pos = pos;
                }

                let pointer_type = map_pointer_type(button);
                if pointer_id == PointerId(0) {
                    self.last_pointer_type = pointer_type;
                }

                let Some(winit_button) = map_pointer_button(button) else {
                    return;
                };
                let pressed = matches!(state, ElementState::Pressed);
                let mapped_button = map_mouse_button(winit_button);

                if std::env::var_os("FRET_WINIT_POINTER_DEBUG").is_some_and(|v| !v.is_empty())
                    && pointer_id != PointerId(0)
                {
                    tracing::info!(
                        pointer_id = ?pointer_id,
                        pointer_type = ?pointer_type,
                        button = ?mapped_button,
                        pressed = pressed,
                        device_id = ?device_id,
                        "winit pointer button"
                    );
                }

                let (evt, buttons_now) = {
                    let pointer_state = self.pointer_state_mut(pointer_id);
                    set_mouse_buttons(&mut pointer_state.buttons, winit_button, pressed);
                    let buttons_now = pointer_state.buttons;

                    let evt = if pressed {
                        let click_count = pointer_state.click.begin_press(mapped_button, pos);
                        PointerEvent::Down {
                            pointer_id,
                            position: pos,
                            button: mapped_button,
                            modifiers: self.modifiers,
                            click_count,
                            pointer_type,
                        }
                    } else {
                        let (click_count, is_click) =
                            pointer_state.click.end_press(mapped_button, pos);
                        PointerEvent::Up {
                            pointer_id,
                            position: pos,
                            button: mapped_button,
                            modifiers: self.modifiers,
                            is_click,
                            click_count,
                            pointer_type,
                        }
                    };

                    (evt, buttons_now)
                };
                if pointer_id == PointerId(0) {
                    self.pressed_buttons = buttons_now;
                }
                out.push(Event::Pointer(evt));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = map_wheel_delta(*delta, window_scale_factor, wheel);
                out.push(Event::Pointer(PointerEvent::Wheel {
                    pointer_id: PointerId(0),
                    position: self.cursor_pos,
                    delta: scroll,
                    modifiers: self.modifiers,
                    pointer_type: fret_core::PointerType::Mouse,
                }));
            }
            WindowEvent::PinchGesture { delta, .. } => {
                if delta.is_nan() {
                    return;
                }
                out.push(Event::Pointer(PointerEvent::PinchGesture {
                    pointer_id: PointerId(0),
                    position: self.cursor_pos,
                    delta: *delta as f32,
                    modifiers: self.modifiers,
                    pointer_type: fret_core::PointerType::Mouse,
                }));
            }
            WindowEvent::SurfaceResized(size) => {
                let logical: LogicalSize<f32> = size.to_logical(window_scale_factor);
                out.push(Event::WindowResized {
                    width: Px(logical.width),
                    height: Px(logical.height),
                });
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                surface_size_writer: _,
            } => {
                out.push(Event::WindowScaleFactorChanged(*scale_factor as f32));
            }
            _ => {}
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_web_cursor_updates(
        &mut self,
        window_scale_factor: f64,
        cursor_offset_px: Option<(f32, f32)>,
        out: &mut Vec<Event>,
    ) {
        let Some((x, y)) = cursor_offset_px else {
            self.cursor_pos_physical = None;
            return;
        };

        let changed =
            (self.cursor_pos.x.0 - x).abs() > 0.001 || (self.cursor_pos.y.0 - y).abs() > 0.001;

        self.cursor_pos = Point::new(Px(x), Px(y));
        self.cursor_pos_physical = Some(PhysicalPosition::new(
            x as f64 * window_scale_factor,
            y as f64 * window_scale_factor,
        ));

        if changed {
            out.push(Event::Pointer(PointerEvent::Move {
                pointer_id: PointerId(0),
                position: self.cursor_pos,
                buttons: self.pointer_buttons(PointerId(0)),
                modifiers: self.modifiers,
                pointer_type: self.last_pointer_type,
            }));
        }
    }

    fn handle_key_event(&mut self, event: &KeyEvent, out: &mut Vec<Event>) {
        let repeat = event.repeat;
        let key = map_physical_key(event.physical_key);

        // Track AltGr: on many layouts it is implemented as (Ctrl+Alt). We follow the desktop runner
        // and explicitly model it to avoid "Ctrl+Alt" shortcuts firing while typing.
        if is_alt_gr_key(&event.logical_key) {
            self.alt_gr_down = matches!(event.state, ElementState::Pressed);
            self.modifiers = map_modifiers(self.raw_modifiers, self.alt_gr_down);
        }

        match event.state {
            ElementState::Pressed => {
                out.push(Event::KeyDown {
                    key,
                    modifiers: self.modifiers,
                    repeat,
                });
                if let Some(text) = event.text.as_ref().and_then(|t| sanitize_text_input(t)) {
                    out.push(Event::TextInput(text));
                }
            }
            ElementState::Released => out.push(Event::KeyUp {
                key,
                modifiers: self.modifiers,
            }),
        }
    }
}

#[cfg(test)]
mod click_tracker_tests;

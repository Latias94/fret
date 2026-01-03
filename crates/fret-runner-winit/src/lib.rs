use fret_core::{Event, KeyCode, Modifiers, MouseButton, MouseButtons, Point, PointerEvent, Px};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};
use winit::event::{
    ButtonSource, ElementState, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta,
    WindowEvent,
};
use winit::keyboard::{Key, ModifiersState, NamedKey};

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWeb;

pub mod accessibility;

#[derive(Debug, Default, Clone)]
pub struct WinitPlatform {
    pub input: WinitInputState,
    pub wheel: WheelConfig,
}

impl WinitPlatform {
    pub fn handle_window_event(
        &mut self,
        window_scale_factor: f64,
        event: &WindowEvent,
        out: &mut Vec<Event>,
    ) {
        self.input
            .handle_window_event_with_config(window_scale_factor, event, self.wheel, out);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerError {
    message: String,
}

impl RunnerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RunnerError {}

#[cfg(target_arch = "wasm32")]
pub fn canvas_by_id(id: &str) -> Result<web_sys::HtmlCanvasElement, RunnerError> {
    use wasm_bindgen::JsCast as _;

    let window = web_sys::window().ok_or_else(|| RunnerError::new("window is not available"))?;
    let document = window
        .document()
        .ok_or_else(|| RunnerError::new("document is not available"))?;
    let el = document
        .get_element_by_id(id)
        .ok_or_else(|| RunnerError::new("canvas element not found"))?;
    el.dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| RunnerError::new("element is not a canvas"))
}

#[cfg(target_arch = "wasm32")]
pub struct WebCursorListener {
    _on_move: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>,
    _on_leave: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>,
}

#[cfg(target_arch = "wasm32")]
mod web_cursor {
    use std::cell::Cell;

    use wasm_bindgen::JsCast as _;
    use wasm_bindgen::prelude::wasm_bindgen;

    thread_local! {
        static LAST_POS: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
    }

    pub(super) fn set(pos: Option<(f32, f32)>) {
        LAST_POS.with(|cell| cell.set(pos));
    }

    pub(super) fn get() -> Option<(f32, f32)> {
        LAST_POS.with(|cell| cell.get())
    }

    pub(super) fn pointer_offset(event: &web_sys::PointerEvent) -> (f32, f32) {
        #[wasm_bindgen]
        extern "C" {
            type PointerEventExt;

            #[wasm_bindgen(method, getter, js_name = offsetX)]
            fn offset_x(this: &PointerEventExt) -> f64;

            #[wasm_bindgen(method, getter, js_name = offsetY)]
            fn offset_y(this: &PointerEventExt) -> f64;
        }

        let event: &PointerEventExt = event.unchecked_ref();
        (event.offset_x() as f32, event.offset_y() as f32)
    }
}

#[cfg(target_arch = "wasm32")]
pub fn install_web_cursor_listener(
    window: &dyn winit::window::Window,
    wake: impl Fn() + 'static,
) -> Result<WebCursorListener, RunnerError> {
    use wasm_bindgen::JsCast as _;

    let Some(canvas) = window.canvas() else {
        return Err(RunnerError::new("winit window has no canvas"));
    };
    let canvas: web_sys::HtmlCanvasElement = canvas.clone();

    let wake = Rc::new(wake);
    let wake_move = wake.clone();
    let on_move =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
            let (x, y) = web_cursor::pointer_offset(&event);
            web_cursor::set(Some((x, y)));
            wake_move();
        }) as Box<dyn FnMut(web_sys::PointerEvent)>);

    let wake_leave = wake.clone();
    let on_leave =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::PointerEvent| {
            web_cursor::set(None);
            wake_leave();
        }) as Box<dyn FnMut(web_sys::PointerEvent)>);

    canvas
        .add_event_listener_with_callback("pointermove", on_move.as_ref().unchecked_ref())
        .map_err(|_| RunnerError::new("failed to add pointermove listener"))?;

    canvas
        .add_event_listener_with_callback("pointerleave", on_leave.as_ref().unchecked_ref())
        .map_err(|_| RunnerError::new("failed to add pointerleave listener"))?;

    Ok(WebCursorListener {
        _on_move: on_move,
        _on_leave: on_leave,
    })
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WinitInputState {
    pub cursor_pos: Point,
    pub cursor_pos_physical: Option<PhysicalPosition<f64>>,
    pub pressed_buttons: MouseButtons,
    pub modifiers: Modifiers,
    pub raw_modifiers: ModifiersState,
    pub alt_gr_down: bool,
}

impl WinitInputState {
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
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key_event(event, out);
            }
            WindowEvent::PointerMoved { position, .. } => {
                self.cursor_pos_physical = Some(*position);
                let logical: LogicalPosition<f32> = position.to_logical(window_scale_factor);
                self.cursor_pos = Point::new(Px(logical.x), Px(logical.y));
                out.push(Event::Pointer(PointerEvent::Move {
                    position: self.cursor_pos,
                    buttons: self.pressed_buttons,
                    modifiers: self.modifiers,
                }));
            }
            WindowEvent::PointerButton {
                state,
                position,
                button,
                ..
            } => {
                self.cursor_pos_physical = Some(*position);
                let logical: LogicalPosition<f32> = position.to_logical(window_scale_factor);
                self.cursor_pos = Point::new(Px(logical.x), Px(logical.y));

                let Some(winit_button) = map_pointer_button(button) else {
                    return;
                };
                let pressed = matches!(state, ElementState::Pressed);
                set_mouse_buttons(&mut self.pressed_buttons, winit_button, pressed);

                let evt = if pressed {
                    PointerEvent::Down {
                        position: self.cursor_pos,
                        button: map_mouse_button(winit_button),
                        modifiers: self.modifiers,
                    }
                } else {
                    PointerEvent::Up {
                        position: self.cursor_pos,
                        button: map_mouse_button(winit_button),
                        modifiers: self.modifiers,
                    }
                };
                out.push(Event::Pointer(evt));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = map_wheel_delta(*delta, window_scale_factor, wheel);
                out.push(Event::Pointer(PointerEvent::Wheel {
                    position: self.cursor_pos,
                    delta: scroll,
                    modifiers: self.modifiers,
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
    pub fn poll_web_cursor_updates(&mut self, window_scale_factor: f64, out: &mut Vec<Event>) {
        let Some((x, y)) = web_cursor::get() else {
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
                position: self.cursor_pos,
                buttons: self.pressed_buttons,
                modifiers: self.modifiers,
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

#[derive(Debug, Clone, Copy)]
pub struct WheelConfig {
    pub line_delta_px: f32,
    pub pixel_delta_scale: f32,
}

impl Default for WheelConfig {
    fn default() -> Self {
        Self {
            line_delta_px: 16.0,
            pixel_delta_scale: 1.0,
        }
    }
}

pub fn map_cursor_icon(icon: fret_core::CursorIcon) -> winit::cursor::CursorIcon {
    match icon {
        fret_core::CursorIcon::Default => winit::cursor::CursorIcon::Default,
        fret_core::CursorIcon::Pointer => winit::cursor::CursorIcon::Pointer,
        fret_core::CursorIcon::Text => winit::cursor::CursorIcon::Text,
        fret_core::CursorIcon::ColResize => winit::cursor::CursorIcon::ColResize,
        fret_core::CursorIcon::RowResize => winit::cursor::CursorIcon::RowResize,
    }
}

pub fn sanitize_text_input(text: &str) -> Option<String> {
    // Contract: `Event::TextInput` represents committed insertion text and must not include
    // control characters. Keys like Backspace/Enter/Tab must be handled via `KeyDown` + commands.
    //
    // Some platform stacks report control keys in `KeyboardInput.text` (e.g. backspace on macOS).
    let filtered: String = text.chars().filter(|ch| !ch.is_control()).collect();
    if filtered.is_empty() {
        None
    } else {
        Some(filtered)
    }
}

pub fn map_modifiers(state: ModifiersState, alt_gr_down: bool) -> Modifiers {
    let mut mods = Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        alt_gr: alt_gr_down,
        meta: state.meta_key(),
    };

    if mods.alt_gr {
        mods.ctrl = false;
        mods.alt = false;
    }

    mods
}

pub fn map_mouse_button(button: WinitMouseButton) -> MouseButton {
    match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Back,
        WinitMouseButton::Forward => MouseButton::Forward,
        other => MouseButton::Other(other as u16),
    }
}

pub fn map_pointer_button(button: &ButtonSource) -> Option<WinitMouseButton> {
    match button {
        ButtonSource::Mouse(mouse) => Some(*mouse),
        ButtonSource::Touch { .. } => Some(WinitMouseButton::Left),
        ButtonSource::TabletTool { .. } => Some(WinitMouseButton::Left),
        ButtonSource::Unknown(_) => None,
    }
}

pub fn set_mouse_buttons(buttons: &mut MouseButtons, button: WinitMouseButton, pressed: bool) {
    match button {
        WinitMouseButton::Left => buttons.left = pressed,
        WinitMouseButton::Right => buttons.right = pressed,
        WinitMouseButton::Middle => buttons.middle = pressed,
        _ => {}
    }
}

pub fn map_wheel_delta(delta: MouseScrollDelta, scale_factor: f64, config: WheelConfig) -> Point {
    // `fret-core` wheel delta follows winit semantics: positive y means wheel up.
    match delta {
        MouseScrollDelta::LineDelta(dx, dy) => {
            Point::new(Px(dx * config.line_delta_px), Px(dy * config.line_delta_px))
        }
        MouseScrollDelta::PixelDelta(physical) => {
            let logical: LogicalPosition<f32> = physical.to_logical(scale_factor);
            Point::new(
                Px(logical.x * config.pixel_delta_scale),
                Px(logical.y * config.pixel_delta_scale),
            )
        }
    }
}

pub fn is_alt_gr_key(key: &Key) -> bool {
    matches!(key, Key::Named(NamedKey::AltGraph))
}

pub fn map_physical_key(key: winit::keyboard::PhysicalKey) -> KeyCode {
    use winit::keyboard::KeyCode as WinitKeyCode;

    let winit::keyboard::PhysicalKey::Code(code) = key else {
        return KeyCode::Unknown;
    };

    match code {
        WinitKeyCode::Escape => KeyCode::Escape,
        WinitKeyCode::Enter => KeyCode::Enter,
        WinitKeyCode::Tab => KeyCode::Tab,
        WinitKeyCode::Backspace => KeyCode::Backspace,
        WinitKeyCode::Space => KeyCode::Space,

        WinitKeyCode::ArrowUp => KeyCode::ArrowUp,
        WinitKeyCode::ArrowDown => KeyCode::ArrowDown,
        WinitKeyCode::ArrowLeft => KeyCode::ArrowLeft,
        WinitKeyCode::ArrowRight => KeyCode::ArrowRight,

        WinitKeyCode::Home => KeyCode::Home,
        WinitKeyCode::End => KeyCode::End,
        WinitKeyCode::PageUp => KeyCode::PageUp,
        WinitKeyCode::PageDown => KeyCode::PageDown,
        WinitKeyCode::Insert => KeyCode::Insert,
        WinitKeyCode::Delete => KeyCode::Delete,

        WinitKeyCode::CapsLock => KeyCode::CapsLock,

        WinitKeyCode::ShiftLeft => KeyCode::ShiftLeft,
        WinitKeyCode::ShiftRight => KeyCode::ShiftRight,
        WinitKeyCode::ControlLeft => KeyCode::ControlLeft,
        WinitKeyCode::ControlRight => KeyCode::ControlRight,
        WinitKeyCode::AltLeft => KeyCode::AltLeft,
        WinitKeyCode::AltRight => KeyCode::AltRight,
        WinitKeyCode::MetaLeft => KeyCode::SuperLeft,
        WinitKeyCode::MetaRight => KeyCode::SuperRight,

        WinitKeyCode::Digit0 => KeyCode::Digit0,
        WinitKeyCode::Digit1 => KeyCode::Digit1,
        WinitKeyCode::Digit2 => KeyCode::Digit2,
        WinitKeyCode::Digit3 => KeyCode::Digit3,
        WinitKeyCode::Digit4 => KeyCode::Digit4,
        WinitKeyCode::Digit5 => KeyCode::Digit5,
        WinitKeyCode::Digit6 => KeyCode::Digit6,
        WinitKeyCode::Digit7 => KeyCode::Digit7,
        WinitKeyCode::Digit8 => KeyCode::Digit8,
        WinitKeyCode::Digit9 => KeyCode::Digit9,

        WinitKeyCode::KeyA => KeyCode::KeyA,
        WinitKeyCode::KeyB => KeyCode::KeyB,
        WinitKeyCode::KeyC => KeyCode::KeyC,
        WinitKeyCode::KeyD => KeyCode::KeyD,
        WinitKeyCode::KeyE => KeyCode::KeyE,
        WinitKeyCode::KeyF => KeyCode::KeyF,
        WinitKeyCode::KeyG => KeyCode::KeyG,
        WinitKeyCode::KeyH => KeyCode::KeyH,
        WinitKeyCode::KeyI => KeyCode::KeyI,
        WinitKeyCode::KeyJ => KeyCode::KeyJ,
        WinitKeyCode::KeyK => KeyCode::KeyK,
        WinitKeyCode::KeyL => KeyCode::KeyL,
        WinitKeyCode::KeyM => KeyCode::KeyM,
        WinitKeyCode::KeyN => KeyCode::KeyN,
        WinitKeyCode::KeyO => KeyCode::KeyO,
        WinitKeyCode::KeyP => KeyCode::KeyP,
        WinitKeyCode::KeyQ => KeyCode::KeyQ,
        WinitKeyCode::KeyR => KeyCode::KeyR,
        WinitKeyCode::KeyS => KeyCode::KeyS,
        WinitKeyCode::KeyT => KeyCode::KeyT,
        WinitKeyCode::KeyU => KeyCode::KeyU,
        WinitKeyCode::KeyV => KeyCode::KeyV,
        WinitKeyCode::KeyW => KeyCode::KeyW,
        WinitKeyCode::KeyX => KeyCode::KeyX,
        WinitKeyCode::KeyY => KeyCode::KeyY,
        WinitKeyCode::KeyZ => KeyCode::KeyZ,

        WinitKeyCode::Minus => KeyCode::Minus,
        WinitKeyCode::Equal => KeyCode::Equal,
        WinitKeyCode::BracketLeft => KeyCode::BracketLeft,
        WinitKeyCode::BracketRight => KeyCode::BracketRight,
        WinitKeyCode::Backslash => KeyCode::Backslash,
        WinitKeyCode::Semicolon => KeyCode::Semicolon,
        WinitKeyCode::Quote => KeyCode::Quote,
        WinitKeyCode::Backquote => KeyCode::Backquote,
        WinitKeyCode::Comma => KeyCode::Comma,
        WinitKeyCode::Period => KeyCode::Period,
        WinitKeyCode::Slash => KeyCode::Slash,

        WinitKeyCode::F1 => KeyCode::F1,
        WinitKeyCode::F2 => KeyCode::F2,
        WinitKeyCode::F3 => KeyCode::F3,
        WinitKeyCode::F4 => KeyCode::F4,
        WinitKeyCode::F5 => KeyCode::F5,
        WinitKeyCode::F6 => KeyCode::F6,
        WinitKeyCode::F7 => KeyCode::F7,
        WinitKeyCode::F8 => KeyCode::F8,
        WinitKeyCode::F9 => KeyCode::F9,
        WinitKeyCode::F10 => KeyCode::F10,
        WinitKeyCode::F11 => KeyCode::F11,
        WinitKeyCode::F12 => KeyCode::F12,

        WinitKeyCode::Numpad0 => KeyCode::Numpad0,
        WinitKeyCode::Numpad1 => KeyCode::Numpad1,
        WinitKeyCode::Numpad2 => KeyCode::Numpad2,
        WinitKeyCode::Numpad3 => KeyCode::Numpad3,
        WinitKeyCode::Numpad4 => KeyCode::Numpad4,
        WinitKeyCode::Numpad5 => KeyCode::Numpad5,
        WinitKeyCode::Numpad6 => KeyCode::Numpad6,
        WinitKeyCode::Numpad7 => KeyCode::Numpad7,
        WinitKeyCode::Numpad8 => KeyCode::Numpad8,
        WinitKeyCode::Numpad9 => KeyCode::Numpad9,
        WinitKeyCode::NumpadAdd => KeyCode::NumpadAdd,
        WinitKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
        WinitKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
        WinitKeyCode::NumpadDivide => KeyCode::NumpadDivide,
        WinitKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
        WinitKeyCode::NumpadEnter => KeyCode::NumpadEnter,

        _ => KeyCode::Unknown,
    }
}

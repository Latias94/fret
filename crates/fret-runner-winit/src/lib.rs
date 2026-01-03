use fret_core::{Event, KeyCode, Modifiers, MouseButton, MouseButtons, Point, PointerEvent, Px};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};
use winit::event::{
    ElementState, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
};
use winit::keyboard::{Key, ModifiersState, NamedKey};

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

#[derive(Debug, Default, Clone, Copy)]
pub struct WinitInputState {
    pub cursor_pos: Point,
    pub cursor_pos_physical: Option<PhysicalPosition<f64>>,
    pub pressed_buttons: MouseButtons,
    pub modifiers: Modifiers,
    pub alt_gr_down: bool,
}

impl WinitInputState {
    pub fn handle_window_event(
        &mut self,
        window_scale_factor: f64,
        event: &WindowEvent,
        out: &mut Vec<Event>,
    ) {
        match event {
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = map_modifiers(mods.state(), self.alt_gr_down);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key_event(event, out);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos_physical = Some(*position);
                let logical: LogicalPosition<f32> = position.to_logical(window_scale_factor);
                self.cursor_pos = Point::new(Px(logical.x), Px(logical.y));
                out.push(Event::Pointer(PointerEvent::Move {
                    position: self.cursor_pos,
                    buttons: self.pressed_buttons,
                    modifiers: self.modifiers,
                }));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let winit_button = *button;
                let Some(button) = map_mouse_button(winit_button) else {
                    return;
                };
                let pressed = matches!(state, ElementState::Pressed);
                set_mouse_buttons(&mut self.pressed_buttons, winit_button, pressed);
                let evt = if pressed {
                    PointerEvent::Down {
                        position: self.cursor_pos,
                        button,
                        modifiers: self.modifiers,
                    }
                } else {
                    PointerEvent::Up {
                        position: self.cursor_pos,
                        button,
                        modifiers: self.modifiers,
                    }
                };
                out.push(Event::Pointer(evt));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = map_wheel_delta(*delta, window_scale_factor);
                out.push(Event::Pointer(PointerEvent::Wheel {
                    position: self.cursor_pos,
                    delta: scroll,
                    modifiers: self.modifiers,
                }));
            }
            WindowEvent::Resized(size) => {
                let logical: LogicalSize<f32> = size.to_logical(window_scale_factor);
                out.push(Event::WindowResized {
                    width: Px(logical.width),
                    height: Px(logical.height),
                });
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer,
            } => {
                out.push(Event::WindowScaleFactorChanged(*scale_factor as f32));
                let _ = inner_size_writer;
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, event: &KeyEvent, out: &mut Vec<Event>) {
        let repeat = event.repeat;
        let key = map_physical_key(event.physical_key);

        // Track AltGr: on many layouts it is implemented as (Ctrl+Alt). We follow the desktop runner
        // and explicitly model it to avoid "Ctrl+Alt" shortcuts firing while typing.
        if is_alt_gr_key(&event.logical_key) {
            self.alt_gr_down = matches!(event.state, ElementState::Pressed);
        }

        match event.state {
            ElementState::Pressed => out.push(Event::KeyDown {
                key,
                modifiers: self.modifiers,
                repeat,
            }),
            ElementState::Released => out.push(Event::KeyUp {
                key,
                modifiers: self.modifiers,
            }),
        }
    }
}

fn map_modifiers(state: ModifiersState, alt_gr_down: bool) -> Modifiers {
    let mut mods = Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        alt_gr: alt_gr_down,
        meta: state.super_key(),
    };

    if mods.alt_gr {
        mods.ctrl = false;
        mods.alt = false;
    }

    mods
}

fn map_mouse_button(button: WinitMouseButton) -> Option<MouseButton> {
    Some(match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Back,
        WinitMouseButton::Forward => MouseButton::Forward,
        WinitMouseButton::Other(v) => MouseButton::Other(v),
    })
}

fn set_mouse_buttons(buttons: &mut MouseButtons, button: WinitMouseButton, pressed: bool) {
    match button {
        WinitMouseButton::Left => buttons.left = pressed,
        WinitMouseButton::Right => buttons.right = pressed,
        WinitMouseButton::Middle => buttons.middle = pressed,
        WinitMouseButton::Back | WinitMouseButton::Forward | WinitMouseButton::Other(_) => {}
    }
}

fn map_wheel_delta(delta: MouseScrollDelta, scale_factor: f64) -> Point {
    // `fret-core` wheel delta follows winit semantics: positive y means wheel up.
    match delta {
        MouseScrollDelta::LineDelta(dx, dy) => Point::new(Px(dx * 16.0), Px(dy * 16.0)),
        MouseScrollDelta::PixelDelta(physical) => {
            let logical: LogicalPosition<f32> = physical.to_logical(scale_factor);
            Point::new(Px(logical.x), Px(logical.y))
        }
    }
}

fn is_alt_gr_key(key: &Key) -> bool {
    matches!(key, Key::Named(NamedKey::AltGraph))
}

fn map_physical_key(key: winit::keyboard::PhysicalKey) -> KeyCode {
    // For now, prefer token-based mapping to keep this crate small. The desktop runner has a full
    // match table; we can move it here once `fret-runner-winit` becomes the only winit front-end.
    let winit::keyboard::PhysicalKey::Code(code) = key else {
        return KeyCode::Unknown;
    };
    KeyCode::from_token(&format!("{code:?}")).unwrap_or(KeyCode::Unknown)
}

use fret_core::{
    CursorIcon, Event, ExternalDragFile, ExternalDragFiles, ExternalDropToken, KeyCode, Modifiers,
    MouseButton, MouseButtons, Point, PointerEvent, Px, Rect,
};
use std::time::{Duration, Instant};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};
use winit::event::{
    ButtonSource, ElementState, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta,
    WindowEvent,
};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWeb;

pub mod accessibility;

#[cfg(windows)]
pub mod windows_ime;

pub mod window_registry;

pub fn map_physical_position_to_point(
    window_scale_factor: f64,
    position: PhysicalPosition<f64>,
) -> Point {
    let logical: LogicalPosition<f32> = position.to_logical(window_scale_factor);
    Point::new(Px(logical.x), Px(logical.y))
}

pub fn map_optional_physical_position_to_point(
    window_scale_factor: f64,
    position: Option<PhysicalPosition<f64>>,
    fallback: Point,
) -> Point {
    position
        .map(|position| map_physical_position_to_point(window_scale_factor, position))
        .unwrap_or(fallback)
}

pub fn external_drag_files(
    token: ExternalDropToken,
    paths: &[std::path::PathBuf],
) -> ExternalDragFiles {
    let files = paths
        .iter()
        .map(|p| ExternalDragFile {
            name: p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.to_string_lossy().to_string()),
        })
        .collect();
    ExternalDragFiles { token, files }
}

#[derive(Debug, Default, Clone)]
pub struct WinitPlatform {
    pub input: WinitInputState,
    pub wheel: WheelConfig,
    pub window: WinitWindowState,
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

    pub fn set_ime_allowed(&mut self, enabled: bool) -> bool {
        self.window.set_ime_allowed(enabled)
    }

    pub fn set_ime_cursor_area(&mut self, rect: Rect) -> bool {
        self.window.set_ime_cursor_area(rect)
    }

    pub fn ime_cursor_area(&self) -> Option<Rect> {
        self.window.ime_cursor_area()
    }

    pub fn set_cursor_icon(&mut self, icon: CursorIcon) -> bool {
        self.window.set_cursor_icon(icon)
    }

    /// Applies any pending window-side state (IME/cursor) before drawing a frame.
    ///
    /// This mirrors the backend split pattern in Dear ImGui (`prepare_frame`).
    pub fn prepare_frame(&mut self, window: &dyn Window) {
        self.window.prepare_frame(window);
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
    canvas: web_sys::HtmlCanvasElement,
    on_move: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>,
    on_leave: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for WebCursorListener {
    fn drop(&mut self) {
        use wasm_bindgen::JsCast as _;

        let _ = self.canvas.remove_event_listener_with_callback(
            "pointermove",
            self.on_move.as_ref().unchecked_ref(),
        );
        let _ = self.canvas.remove_event_listener_with_callback(
            "pointerleave",
            self.on_leave.as_ref().unchecked_ref(),
        );
    }
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
        canvas,
        on_move,
        on_leave,
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
    pub last_pointer_type: fret_core::PointerType,
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

    fn end_press(&mut self, button: MouseButton, pos: Point) -> u8 {
        if matches!(button, MouseButton::Other(_)) {
            return 1;
        }
        let now = Instant::now();
        let (state, press) = self.state_for_button_mut(button);
        let Some(press_state) = press.take() else {
            return 1;
        };

        if !press_state.moved && distance_px(pos, press_state.start_pos) <= Self::CLICK_SLOP_PX {
            state.last_time = Some(now);
            state.last_pos = pos;
            state.count = press_state.click_count;
        }

        press_state.click_count.max(1)
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

#[derive(Debug, Default, Clone)]
pub struct WinitWindowState {
    ime_allowed: bool,
    ime_cursor_area: Option<Rect>,
    cursor_icon: CursorIcon,
    pending: WinitWindowPendingState,
}

#[derive(Debug, Default, Clone, Copy)]
struct WinitWindowPendingState {
    ime_allowed: Option<bool>,
    ime_cursor_area: Option<Rect>,
    cursor_icon: Option<CursorIcon>,
}

impl WinitWindowState {
    pub fn set_ime_allowed(&mut self, enabled: bool) -> bool {
        if self.ime_allowed == enabled {
            return false;
        }
        self.ime_allowed = enabled;
        self.pending.ime_allowed = Some(enabled);
        true
    }

    pub fn set_ime_cursor_area(&mut self, rect: Rect) -> bool {
        if self.ime_cursor_area == Some(rect) {
            return false;
        }
        self.ime_cursor_area = Some(rect);
        self.pending.ime_cursor_area = Some(rect);
        true
    }

    pub fn ime_cursor_area(&self) -> Option<Rect> {
        self.ime_cursor_area
    }

    pub fn set_cursor_icon(&mut self, icon: CursorIcon) -> bool {
        if self.cursor_icon == icon {
            return false;
        }
        self.cursor_icon = icon;
        self.pending.cursor_icon = Some(icon);
        true
    }

    pub fn prepare_frame(&mut self, window: &dyn Window) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let pending_cursor_area = self.pending.ime_cursor_area.take();
            if let Some(rect) = pending_cursor_area {
                #[cfg(windows)]
                {
                    windows_ime::set_ime_cursor_area(window, rect);
                }

                #[cfg(not(windows))]
                {
                    let request_data = winit::window::ImeRequestData::default().with_cursor_area(
                        winit::dpi::LogicalPosition::new(rect.origin.x.0, rect.origin.y.0).into(),
                        winit::dpi::LogicalSize::new(rect.size.width.0, rect.size.height.0).into(),
                    );
                    let _ =
                        window.request_ime_update(winit::window::ImeRequest::Update(request_data));
                }
            }

            if let Some(enabled) = self.pending.ime_allowed.take() {
                if enabled {
                    let rect = self.ime_cursor_area.unwrap_or_else(|| Rect {
                        origin: fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                        size: fret_core::Size::new(fret_core::Px(1.0), fret_core::Px(1.0)),
                    });

                    let request_data = winit::window::ImeRequestData::default().with_cursor_area(
                        winit::dpi::LogicalPosition::new(rect.origin.x.0, rect.origin.y.0).into(),
                        winit::dpi::LogicalSize::new(rect.size.width.0, rect.size.height.0).into(),
                    );

                    let caps = winit::window::ImeCapabilities::new().with_cursor_area();
                    if let Some(enable) = winit::window::ImeEnableRequest::new(caps, request_data) {
                        let _ =
                            window.request_ime_update(winit::window::ImeRequest::Enable(enable));
                    }
                } else {
                    let _ = window.request_ime_update(winit::window::ImeRequest::Disable);
                }
            }
        }

        if let Some(icon) = self.pending.cursor_icon.take() {
            window.set_cursor(winit::cursor::Cursor::Icon(map_cursor_icon(icon)));
        }
    }
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
                    winit::event::Ime::DeleteSurrounding { .. } => return,
                };
                out.push(Event::Ime(mapped));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key_event(event, out);
            }
            WindowEvent::PointerMoved { position, .. } => {
                self.cursor_pos_physical = Some(*position);
                let logical: LogicalPosition<f32> = position.to_logical(window_scale_factor);
                self.cursor_pos = Point::new(Px(logical.x), Px(logical.y));
                self.click.update_move(self.cursor_pos);
                out.push(Event::Pointer(PointerEvent::Move {
                    position: self.cursor_pos,
                    buttons: self.pressed_buttons,
                    modifiers: self.modifiers,
                    pointer_type: self.last_pointer_type,
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

                let pointer_type = map_pointer_type(button);
                self.last_pointer_type = pointer_type;

                let Some(winit_button) = map_pointer_button(button) else {
                    return;
                };
                let pressed = matches!(state, ElementState::Pressed);
                set_mouse_buttons(&mut self.pressed_buttons, winit_button, pressed);

                let mapped_button = map_mouse_button(winit_button);

                let evt = if pressed {
                    let click_count = self.click.begin_press(mapped_button, self.cursor_pos);
                    PointerEvent::Down {
                        position: self.cursor_pos,
                        button: mapped_button,
                        modifiers: self.modifiers,
                        click_count,
                        pointer_type,
                    }
                } else {
                    let click_count = self.click.end_press(mapped_button, self.cursor_pos);
                    PointerEvent::Up {
                        position: self.cursor_pos,
                        button: mapped_button,
                        modifiers: self.modifiers,
                        click_count,
                        pointer_type,
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

pub fn map_pointer_type(button: &ButtonSource) -> fret_core::PointerType {
    match button {
        ButtonSource::Mouse(_) => fret_core::PointerType::Mouse,
        ButtonSource::Touch { .. } => fret_core::PointerType::Touch,
        ButtonSource::TabletTool { .. } => fret_core::PointerType::Pen,
        ButtonSource::Unknown(_) => fret_core::PointerType::Unknown,
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
    match key {
        winit::keyboard::PhysicalKey::Code(code) => code,
        winit::keyboard::PhysicalKey::Unidentified(_) => KeyCode::Unidentified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_key_code_roundtrips() {
        assert_eq!(
            map_physical_key(winit::keyboard::PhysicalKey::Code(
                winit::keyboard::KeyCode::KeyA
            )),
            KeyCode::KeyA
        );
    }

    #[test]
    fn physical_key_unidentified_maps_to_unidentified() {
        assert_eq!(
            map_physical_key(winit::keyboard::PhysicalKey::Unidentified(
                winit::keyboard::NativeKeyCode::Unidentified
            )),
            KeyCode::Unidentified
        );
    }
}

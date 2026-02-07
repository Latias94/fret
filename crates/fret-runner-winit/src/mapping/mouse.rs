use fret_core::{MouseButton, MouseButtons};
use winit::event::MouseButton as WinitMouseButton;

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

pub fn set_mouse_buttons(buttons: &mut MouseButtons, button: WinitMouseButton, pressed: bool) {
    match button {
        WinitMouseButton::Left => buttons.left = pressed,
        WinitMouseButton::Right => buttons.right = pressed,
        WinitMouseButton::Middle => buttons.middle = pressed,
        _ => {}
    }
}

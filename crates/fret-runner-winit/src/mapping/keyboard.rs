use fret_core::KeyCode;
use winit::keyboard::{Key, NamedKey};

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

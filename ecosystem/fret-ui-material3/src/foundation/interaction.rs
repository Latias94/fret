#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressableInteraction {
    Hovered,
    Focused,
    Pressed,
}

pub fn pressable_interaction(
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> Option<PressableInteraction> {
    if pressed {
        return Some(PressableInteraction::Pressed);
    }
    if focused {
        return Some(PressableInteraction::Focused);
    }
    if hovered {
        return Some(PressableInteraction::Hovered);
    }
    None
}

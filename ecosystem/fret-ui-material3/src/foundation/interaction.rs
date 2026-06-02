#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressableInteraction {
    Hovered,
    Focused,
    Pressed,
}

impl PressableInteraction {
    pub(crate) const fn token_state(self) -> &'static str {
        match self {
            Self::Hovered => "hover",
            Self::Focused => "focus",
            Self::Pressed => "pressed",
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressable_interaction_token_states_match_material_suffixes() {
        assert_eq!(PressableInteraction::Hovered.token_state(), "hover");
        assert_eq!(PressableInteraction::Focused.token_state(), "focus");
        assert_eq!(PressableInteraction::Pressed.token_state(), "pressed");
    }
}

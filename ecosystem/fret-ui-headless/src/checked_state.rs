/// Tri-state checked value for controls like checkboxes.
///
/// Radix models this as `boolean | 'indeterminate'`. In Fret we keep it Rust-native while
/// preserving the same user-facing outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckedState {
    #[default]
    Unchecked,
    Checked,
    Indeterminate,
}

impl CheckedState {
    pub fn is_on(self) -> bool {
        matches!(self, Self::Checked | Self::Indeterminate)
    }

    pub fn is_checked(self) -> bool {
        matches!(self, Self::Checked)
    }

    pub fn is_indeterminate(self) -> bool {
        matches!(self, Self::Indeterminate)
    }

    /// Toggle behavior matching Radix CheckboxTrigger:
    /// - `indeterminate` -> `checked`
    /// - otherwise boolean invert
    pub fn toggle(self) -> Self {
        match self {
            Self::Indeterminate => Self::Checked,
            Self::Checked => Self::Unchecked,
            Self::Unchecked => Self::Checked,
        }
    }

    /// Maps tri-state into Fret's current semantics flag surface.
    ///
    /// `None` represents the indeterminate/mixed state.
    pub fn to_semantics_checked(self) -> Option<bool> {
        match self {
            Self::Unchecked => Some(false),
            Self::Checked => Some(true),
            Self::Indeterminate => None,
        }
    }

    pub fn to_semantics_checked_state(self) -> Option<fret_core::SemanticsCheckedState> {
        match self {
            Self::Unchecked => Some(fret_core::SemanticsCheckedState::False),
            Self::Checked => Some(fret_core::SemanticsCheckedState::True),
            Self::Indeterminate => Some(fret_core::SemanticsCheckedState::Mixed),
        }
    }
}

impl From<bool> for CheckedState {
    fn from(value: bool) -> Self {
        if value {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }
}

impl From<CheckedState> for bool {
    fn from(value: CheckedState) -> Self {
        value.is_checked()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_matches_radix_outcomes() {
        assert_eq!(CheckedState::Unchecked.toggle(), CheckedState::Checked);
        assert_eq!(CheckedState::Checked.toggle(), CheckedState::Unchecked);
        assert_eq!(CheckedState::Indeterminate.toggle(), CheckedState::Checked);
    }

    #[test]
    fn semantics_mapping_uses_none_for_indeterminate() {
        assert_eq!(CheckedState::Unchecked.to_semantics_checked(), Some(false));
        assert_eq!(CheckedState::Checked.to_semantics_checked(), Some(true));
        assert_eq!(CheckedState::Indeterminate.to_semantics_checked(), None);
    }

    #[test]
    fn semantics_checked_state_maps_indeterminate_to_mixed() {
        assert_eq!(
            CheckedState::Unchecked.to_semantics_checked_state(),
            Some(fret_core::SemanticsCheckedState::False)
        );
        assert_eq!(
            CheckedState::Checked.to_semantics_checked_state(),
            Some(fret_core::SemanticsCheckedState::True)
        );
        assert_eq!(
            CheckedState::Indeterminate.to_semantics_checked_state(),
            Some(fret_core::SemanticsCheckedState::Mixed)
        );
    }
}

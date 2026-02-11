use std::ops::{BitOr, BitOrAssign};

use fret_ui::element::PressableState;
use fret_ui::{ElementContext, UiHost};
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum WidgetState {
    Disabled = 1 << 0,
    Hovered = 1 << 1,
    Active = 1 << 2,
    Focused = 1 << 3,
    FocusVisible = 1 << 4,
    Selected = 1 << 5,
    Open = 1 << 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WidgetStates(u16);

impl WidgetStates {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const DISABLED: Self = Self(WidgetState::Disabled as u16);
    pub const HOVERED: Self = Self(WidgetState::Hovered as u16);
    pub const ACTIVE: Self = Self(WidgetState::Active as u16);
    pub const FOCUSED: Self = Self(WidgetState::Focused as u16);
    pub const FOCUS_VISIBLE: Self = Self(WidgetState::FocusVisible as u16);
    pub const SELECTED: Self = Self(WidgetState::Selected as u16);
    pub const OPEN: Self = Self(WidgetState::Open as u16);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn has(self, state: WidgetState) -> bool {
        self.contains(Self::from(state))
    }

    pub fn insert(&mut self, state: WidgetState) {
        self.0 |= state as u16;
    }

    pub fn remove(&mut self, state: WidgetState) {
        self.0 &= !(state as u16);
    }

    pub fn set(&mut self, state: WidgetState, enabled: bool) {
        if enabled {
            self.insert(state);
        } else {
            self.remove(state);
        }
    }

    pub fn from_pressable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        st: PressableState,
        enabled: bool,
    ) -> Self {
        let mut states = Self::empty();
        states.set(WidgetState::Disabled, !enabled);
        states.set(WidgetState::Hovered, enabled && st.hovered);
        states.set(WidgetState::Active, enabled && st.pressed);
        states.set(WidgetState::Focused, enabled && st.focused);
        states.set(
            WidgetState::FocusVisible,
            enabled
                && st.focused
                && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window)),
        );
        states
    }
}

impl From<WidgetState> for WidgetStates {
    fn from(value: WidgetState) -> Self {
        Self(value as u16)
    }
}

impl BitOr for WidgetStates {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for WidgetStates {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone)]
pub struct WidgetStateProperty<T> {
    default: T,
    overrides: SmallVec<[(WidgetStates, T); 4]>,
}

impl<T> WidgetStateProperty<T> {
    pub fn new(default: T) -> Self {
        Self {
            default,
            overrides: SmallVec::new(),
        }
    }

    /// Adds an override that applies when `states` is a subset of the current widget state.
    ///
    /// Precedence rule: the **last** matching override wins.
    pub fn when(mut self, states: WidgetStates, value: T) -> Self {
        self.overrides.push((states, value));
        self
    }

    pub fn resolve(&self, states: WidgetStates) -> &T {
        for (required, value) in self.overrides.iter().rev() {
            if states.contains(*required) {
                return value;
            }
        }
        &self.default
    }
}

/// Applies ADR 0220 "nullable per-state override" semantics for a single slot.
///
/// If the override is present and resolves to `Some(value)` for the current `states`, that value is
/// returned. Otherwise the provided `default` is returned.
pub fn resolve_slot<T: Clone>(
    overrides: Option<&WidgetStateProperty<Option<T>>>,
    default: T,
    states: WidgetStates,
) -> T {
    if let Some(overrides) = overrides {
        if let Some(value) = overrides.resolve(states).clone() {
            return value;
        }
    }
    default
}

/// Merges ADR 0220 slot overrides with shallow, right-biased precedence.
pub fn merge_slot<T>(dst: &mut Option<T>, src: Option<T>) {
    if src.is_some() {
        *dst = src;
    }
}

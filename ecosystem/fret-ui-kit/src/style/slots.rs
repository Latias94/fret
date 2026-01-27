use super::{WidgetStateProperty, WidgetStates};

/// ADR 1159 override slot shape.
///
/// - outer `Option`: whether a slot is overridden at all
/// - inner `Option<T>`: per-state "no override" to fall back to defaults for that state
pub type OverrideSlot<T> = Option<WidgetStateProperty<Option<T>>>;

/// Resolves an ADR 1159-style override slot for a non-optional value.
///
/// - `overrides`: `None` means "slot not overridden".
/// - `overrides.resolve(states) == None` means "no override for this state; fall back to defaults".
pub fn resolve_override_slot<T: Clone>(
    overrides: Option<&WidgetStateProperty<Option<T>>>,
    defaults: &WidgetStateProperty<T>,
    states: WidgetStates,
) -> T {
    match overrides {
        Some(overrides) => overrides
            .resolve(states)
            .as_ref()
            .cloned()
            .unwrap_or_else(|| defaults.resolve(states).clone()),
        None => defaults.resolve(states).clone(),
    }
}

/// Resolves an ADR 1159-style override slot for a computed default.
///
/// Use this when defaults are already computed for the current `states`, rather than stored in a
/// `WidgetStateProperty`.
pub fn resolve_override_slot_with<T, U>(
    overrides: Option<&WidgetStateProperty<Option<T>>>,
    states: WidgetStates,
    map: impl FnOnce(&T) -> U,
    fallback: impl FnOnce() -> U,
) -> U {
    match overrides.and_then(|slot| slot.resolve(states).as_ref()) {
        Some(value) => map(value),
        None => fallback(),
    }
}

/// Resolves an ADR 1159-style override slot for an optional value.
///
/// This variant is useful when the default style itself can be absent (e.g. "no background").
pub fn resolve_override_slot_opt<T: Clone>(
    overrides: Option<&WidgetStateProperty<Option<T>>>,
    defaults: &WidgetStateProperty<Option<T>>,
    states: WidgetStates,
) -> Option<T> {
    match overrides {
        Some(overrides) => overrides
            .resolve(states)
            .as_ref()
            .cloned()
            .or_else(|| defaults.resolve(states).clone()),
        None => defaults.resolve(states).clone(),
    }
}

/// Resolves an ADR 1159-style override slot for a computed optional default.
pub fn resolve_override_slot_opt_with<T, U>(
    overrides: Option<&WidgetStateProperty<Option<T>>>,
    states: WidgetStates,
    map: impl FnOnce(&T) -> U,
    fallback: impl FnOnce() -> Option<U>,
) -> Option<U> {
    match overrides.and_then(|slot| slot.resolve(states).as_ref()) {
        Some(value) => Some(map(value)),
        None => fallback(),
    }
}

/// Right-biased merge for an optional override slot (ADR 1159).
pub fn merge_override_slot<T>(base: Option<T>, other: Option<T>) -> Option<T> {
    if other.is_some() { other } else { base }
}

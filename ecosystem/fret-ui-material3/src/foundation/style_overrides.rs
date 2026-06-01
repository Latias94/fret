//! Shared ADR 0220 style override helpers for Material3 recipes.
//!
//! Public style structs remain owned by each recipe. This Module keeps the merge Implementation
//! local so the right-biased override semantics cannot drift between recipes.

use fret_ui_kit::OverrideSlot;

pub(crate) fn merge_override_slot_into<T>(base: &mut OverrideSlot<T>, other: OverrideSlot<T>) {
    if other.is_some() {
        *base = other;
    }
}

macro_rules! merge_style_override_slots {
    ($base:expr, $other:ident, [$($field:ident),+ $(,)?]) => {{
        let mut out = $base;
        $(
            $crate::foundation::style_overrides::merge_override_slot_into(
                &mut out.$field,
                $other.$field,
            );
        )+
        out
    }};
}

pub(crate) use merge_style_override_slots;

#[cfg(test)]
mod tests {
    use fret_ui_kit::WidgetStateProperty;

    use super::*;

    #[test]
    fn merge_override_slot_into_is_right_biased() {
        let mut base = Some(WidgetStateProperty::new(Some(1)));
        merge_override_slot_into(&mut base, None);
        assert_eq!(*base.as_ref().unwrap().resolve(Default::default()), Some(1));

        merge_override_slot_into(&mut base, Some(WidgetStateProperty::new(Some(2))));
        assert_eq!(*base.as_ref().unwrap().resolve(Default::default()), Some(2));
    }

    #[test]
    fn merge_override_slot_into_keeps_explicit_nullable_override() {
        let mut base = Some(WidgetStateProperty::new(Some(1)));
        merge_override_slot_into(&mut base, Some(WidgetStateProperty::new(None)));
        assert_eq!(*base.as_ref().unwrap().resolve(Default::default()), None);
    }
}

use super::*;

fn modifiers() -> fret_core::Modifiers {
    fret_core::Modifiers::default()
}

#[test]
fn allow_modifier_shortcut_requires_primary_modifier() {
    assert!(!allow_modifier_shortcut(modifiers()));
    assert!(allow_modifier_shortcut(fret_core::Modifiers {
        ctrl: true,
        ..modifiers()
    }));
    assert!(allow_modifier_shortcut(fret_core::Modifiers {
        meta: true,
        ..modifiers()
    }));
}

use super::*;

fn modifiers() -> fret_core::Modifiers {
    fret_core::Modifiers::default()
}

#[test]
fn allow_plain_tab_navigation_requires_unmodified_tab() {
    assert!(allow_plain_tab_navigation(
        false,
        fret_core::KeyCode::Tab,
        fret_core::Modifiers {
            shift: true,
            ..modifiers()
        }
    ));
    assert!(!allow_plain_tab_navigation(
        true,
        fret_core::KeyCode::Tab,
        modifiers()
    ));
    assert!(!allow_plain_tab_navigation(
        false,
        fret_core::KeyCode::KeyA,
        modifiers()
    ));
    assert!(!allow_plain_tab_navigation(
        false,
        fret_core::KeyCode::Tab,
        fret_core::Modifiers {
            ctrl: true,
            ..modifiers()
        }
    ));
    assert!(!allow_plain_tab_navigation(
        false,
        fret_core::KeyCode::Tab,
        fret_core::Modifiers {
            alt: true,
            ..modifiers()
        }
    ));
    assert!(!allow_plain_tab_navigation(
        false,
        fret_core::KeyCode::Tab,
        fret_core::Modifiers {
            alt_gr: true,
            ..modifiers()
        }
    ));
}

#[test]
fn allow_arrow_nudging_rejects_non_arrow_or_chorded_keys() {
    assert!(allow_arrow_nudging(
        fret_core::KeyCode::ArrowLeft,
        modifiers()
    ));
    assert!(!allow_arrow_nudging(fret_core::KeyCode::KeyA, modifiers()));
    assert!(!allow_arrow_nudging(
        fret_core::KeyCode::ArrowLeft,
        fret_core::Modifiers {
            meta: true,
            ..modifiers()
        }
    ));
    assert!(!allow_arrow_nudging(
        fret_core::KeyCode::ArrowLeft,
        fret_core::Modifiers {
            alt: true,
            ..modifiers()
        }
    ));
}

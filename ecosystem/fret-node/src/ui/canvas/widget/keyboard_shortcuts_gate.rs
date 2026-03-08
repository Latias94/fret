use crate::io::NodeGraphDeleteKey;

pub(super) fn allow_modifier_shortcut(modifiers: fret_core::Modifiers) -> bool {
    modifiers.ctrl || modifiers.meta
}

pub(super) fn allow_plain_tab_navigation(
    disable_keyboard_a11y: bool,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    !disable_keyboard_a11y
        && key == fret_core::KeyCode::Tab
        && !modifiers.ctrl
        && !modifiers.meta
        && !modifiers.alt
        && !modifiers.alt_gr
}

pub(super) fn allow_arrow_nudging(
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::keyboard_shortcuts_map::is_arrow_key(key)
        && !modifiers.ctrl
        && !modifiers.meta
        && !modifiers.alt
        && !modifiers.alt_gr
}

pub(super) fn matches_delete_shortcut(
    delete_key: NodeGraphDeleteKey,
    key: fret_core::KeyCode,
) -> bool {
    delete_key.matches(key)
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn matches_delete_shortcut_uses_configured_binding() {
        assert!(matches_delete_shortcut(
            NodeGraphDeleteKey::BackspaceOrDelete,
            fret_core::KeyCode::Backspace,
        ));
        assert!(matches_delete_shortcut(
            NodeGraphDeleteKey::BackspaceOrDelete,
            fret_core::KeyCode::Delete,
        ));
        assert!(!matches_delete_shortcut(
            NodeGraphDeleteKey::Backspace,
            fret_core::KeyCode::Delete,
        ));
    }
}

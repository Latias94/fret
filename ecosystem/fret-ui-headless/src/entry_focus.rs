//! Input-modality gated entry-focus selection for menu/select-like overlays.
//!
//! Radix-style menu and select surfaces share the same high-level behavior:
//! keyboard opens may focus an entry, while pointer opens should focus the content container and
//! suppress automatic entry focus. This module owns that pure target-selection policy; runtime
//! layers are responsible for observing the current input modality and applying the returned target.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryFocusOpenModality {
    Pointer,
    Keyboard,
}

impl EntryFocusOpenModality {
    pub fn from_is_keyboard(is_keyboard: bool) -> Self {
        if is_keyboard {
            Self::Keyboard
        } else {
            Self::Pointer
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardEntryFallback {
    /// Keyboard opens do not fall back to content focus when no entry target exists.
    None,
    /// Keyboard opens fall back to the content target when no entry target exists.
    PointerContent,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EntryFocusTargets<T> {
    pub keyboard_entry_focus: Option<T>,
    pub pointer_content_focus: Option<T>,
}

impl<T> EntryFocusTargets<T> {
    pub fn new() -> Self {
        Self {
            keyboard_entry_focus: None,
            pointer_content_focus: None,
        }
    }

    pub fn keyboard_entry_focus(mut self, focus: Option<T>) -> Self {
        self.keyboard_entry_focus = focus;
        self
    }

    pub fn pointer_content_focus(mut self, focus: Option<T>) -> Self {
        self.pointer_content_focus = focus;
        self
    }

    pub fn resolve(
        self,
        modality: EntryFocusOpenModality,
        keyboard_fallback: KeyboardEntryFallback,
    ) -> Option<T> {
        match modality {
            EntryFocusOpenModality::Pointer => self.pointer_content_focus,
            EntryFocusOpenModality::Keyboard => match keyboard_fallback {
                KeyboardEntryFallback::None => self.keyboard_entry_focus,
                KeyboardEntryFallback::PointerContent => {
                    self.keyboard_entry_focus.or(self.pointer_content_focus)
                }
            },
        }
    }

    pub fn resolve_menu(self, modality: EntryFocusOpenModality) -> Option<T> {
        self.resolve(modality, KeyboardEntryFallback::None)
    }

    pub fn resolve_select(self, modality: EntryFocusOpenModality) -> Option<T> {
        self.resolve(modality, KeyboardEntryFallback::PointerContent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_open_prefers_content_focus() {
        let targets = EntryFocusTargets::new()
            .pointer_content_focus(Some("content"))
            .keyboard_entry_focus(Some("entry"));

        assert_eq!(
            targets.resolve_menu(EntryFocusOpenModality::Pointer),
            Some("content")
        );
        assert_eq!(
            targets.resolve_select(EntryFocusOpenModality::Pointer),
            Some("content")
        );
    }

    #[test]
    fn menu_keyboard_open_does_not_fallback_to_content() {
        let targets = EntryFocusTargets::new().pointer_content_focus(Some("content"));

        assert_eq!(targets.resolve_menu(EntryFocusOpenModality::Keyboard), None);
    }

    #[test]
    fn select_keyboard_open_falls_back_to_content() {
        let targets = EntryFocusTargets::new().pointer_content_focus(Some("content"));

        assert_eq!(
            targets.resolve_select(EntryFocusOpenModality::Keyboard),
            Some("content")
        );
    }

    #[test]
    fn keyboard_open_prefers_entry_when_available() {
        let targets = EntryFocusTargets::new()
            .pointer_content_focus(Some("content"))
            .keyboard_entry_focus(Some("entry"));

        assert_eq!(
            targets.resolve_menu(EntryFocusOpenModality::Keyboard),
            Some("entry")
        );
        assert_eq!(
            targets.resolve_select(EntryFocusOpenModality::Keyboard),
            Some("entry")
        );
    }
}

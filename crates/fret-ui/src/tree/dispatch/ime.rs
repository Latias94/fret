use super::*;

impl<H: UiHost> UiTree<H> {
    pub(super) fn update_ime_composing_for_event(
        &mut self,
        focus_is_text_input: bool,
        event: &Event,
    ) {
        if !focus_is_text_input {
            self.ime_composing = false;
            return;
        }

        let Event::Ime(ime) = event else {
            return;
        };

        match ime {
            fret_core::ImeEvent::Preedit { text, cursor } => {
                self.ime_composing = crate::text_edit::ime::is_composing(text, *cursor);
            }
            fret_core::ImeEvent::Commit(_) | fret_core::ImeEvent::Disabled => {
                self.ime_composing = false;
            }
            fret_core::ImeEvent::DeleteSurrounding { .. } => {}
            fret_core::ImeEvent::Enabled => {}
        }
    }
}

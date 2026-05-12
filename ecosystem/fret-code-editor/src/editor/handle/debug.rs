use super::*;

impl CodeEditorHandle {
    pub fn set_preedit_debug(&self, text: impl Into<String>, cursor: Option<(usize, usize)>) {
        let text = text.into();
        let mut st = self.state.borrow_mut();
        let preedit = (!text.is_empty()).then_some(PreeditState { text, cursor });
        st.set_preedit(preedit);
        st.caret_preferred_x = None;
    }

    /// Debug-only IME composition helper: start/advance composition by replacing the current
    /// selection in the platform-facing composed view (UTF-16) without mutating the base buffer.
    ///
    /// This exists to support diag scripts that need to exercise the same path as the
    /// `TextInputRegion` platform replace hooks (selection-replacing composition + cancel).
    pub fn debug_platform_set_marked_text_for_selection(&self, text: &str) {
        let mut st = self.state.borrow_mut();

        let (value, selection, _composition) = a11y::a11y_composed_text_window(&mut st, 0);
        let Some((anchor, focus)) = selection else {
            return;
        };
        let (sel_lo, sel_hi) = if anchor <= focus {
            (anchor, focus)
        } else {
            (focus, anchor)
        };

        let start_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value.as_str(),
            sel_lo as usize,
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let end_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value.as_str(),
            sel_hi as usize,
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let range = fret_runtime::Utf16Range::new(start_utf16, end_utf16);

        let text_len_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            text,
            text.len(),
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let marked =
            fret_runtime::Utf16Range::new(start_utf16, start_utf16.saturating_add(text_len_utf16));

        let _ = platform_replace_and_mark_text_in_range_utf16(
            &mut st,
            0,
            value.as_str(),
            range,
            text,
            Some(marked),
            None,
        );
    }

    /// Debug-only IME composition helper: cancel/unmark composition best-effort.
    ///
    /// This sends an empty composition update (`text=""` with `marked=Some(_)`), which our editor
    /// treats as cancel/unmark and restores the selection captured at composition start.
    pub fn debug_platform_cancel_marked_text(&self) {
        let mut st = self.state.borrow_mut();

        let (value, selection, _composition) = a11y::a11y_composed_text_window(&mut st, 0);
        let Some((anchor, focus)) = selection else {
            return;
        };
        let (sel_lo, sel_hi) = if anchor <= focus {
            (anchor, focus)
        } else {
            (focus, anchor)
        };

        let start_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value.as_str(),
            sel_lo as usize,
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let end_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value.as_str(),
            sel_hi as usize,
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let range = fret_runtime::Utf16Range::new(start_utf16, end_utf16);

        let marked = fret_runtime::Utf16Range::new(start_utf16, start_utf16);
        let _ = platform_replace_and_mark_text_in_range_utf16(
            &mut st,
            0,
            value.as_str(),
            range,
            "",
            Some(marked),
            None,
        );
    }

    pub fn allow_decorations_under_inline_preedit(&self) -> bool {
        self.state.borrow().allow_decorations_under_inline_preedit
    }

    pub fn set_allow_decorations_under_inline_preedit(&self, allowed: bool) {
        self.state
            .borrow_mut()
            .set_allow_decorations_under_inline_preedit(allowed);
    }

    pub fn compose_inline_preedit(&self) -> bool {
        self.state.borrow().compose_inline_preedit
    }

    pub fn set_compose_inline_preedit(&self, enabled: bool) {
        self.state.borrow_mut().set_compose_inline_preedit(enabled);
    }

    pub fn diag_decorated_line_text(&self, line: usize) -> Option<String> {
        let mut st = self.state.borrow_mut();
        if st.preedit.is_some() {
            return None;
        }
        let row = st.display_map.line_first_display_row(line);
        let (_, text, _, _, _) = paint::cached_row_text_with_range(&mut st, row, 64);
        Some(text.as_ref().to_string())
    }
}

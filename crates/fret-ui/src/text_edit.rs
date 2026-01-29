use fret_core::ImeEvent;
use fret_runtime::TickId;

pub(crate) mod normalize {
    pub(crate) fn newlines_to_lf(text: &str) -> String {
        text.replace("\r\n", "\n").replace('\r', "\n")
    }
}

pub(crate) mod clipboard {
    use super::normalize;

    pub(crate) fn normalize_single_line(text: &str) -> Option<String> {
        if text.is_empty() {
            return None;
        }

        Some(text.replace(['\n', '\r'], " "))
    }

    pub(crate) fn normalize_multiline(text: &str) -> Option<String> {
        if text.is_empty() {
            return None;
        }

        let normalized = if text.contains('\r') {
            normalize::newlines_to_lf(text)
        } else {
            text.to_string()
        };

        Some(normalized)
    }
}

pub(crate) mod utf8 {
    use unicode_segmentation::UnicodeSegmentation;

    use fret_runtime::TextBoundaryMode;

    pub(crate) fn clamp_to_char_boundary(text: &str, idx: usize) -> usize {
        if idx >= text.len() {
            return text.len();
        }
        if text.is_char_boundary(idx) {
            return idx;
        }
        let mut i = idx;
        while i > 0 && !text.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    pub(crate) fn prev_char_boundary(text: &str, idx: usize) -> usize {
        let idx = clamp_to_char_boundary(text, idx);
        if idx == 0 {
            return 0;
        }
        let slice = &text[..idx];
        slice.char_indices().last().map(|(i, _)| i).unwrap_or(0)
    }

    pub(crate) fn next_char_boundary(text: &str, idx: usize) -> usize {
        let idx = clamp_to_char_boundary(text, idx);
        if idx >= text.len() {
            return text.len();
        }
        let ch = text[idx..].chars().next().unwrap();
        idx + ch.len_utf8()
    }

    pub(crate) fn is_identifier_char(ch: char) -> bool {
        ch == '_' || unicode_ident::is_xid_continue(ch)
    }

    fn char_at(text: &str, idx: usize) -> Option<char> {
        let idx = clamp_to_char_boundary(text, idx);
        text.get(idx..)?.chars().next()
    }

    fn is_unicode_word_char(text: &str, idx: usize) -> bool {
        let idx = clamp_to_char_boundary(text, idx);
        text.unicode_word_indices()
            .any(|(start, word)| (start..start + word.len()).contains(&idx))
    }

    fn unicode_word_range_at(text: &str, idx: usize) -> Option<(usize, usize)> {
        let idx = clamp_to_char_boundary(text, idx);
        for (start, word) in text.unicode_word_indices() {
            let end = start + word.len();
            if (start..end).contains(&idx) {
                return Some((start, end));
            }
        }
        None
    }

    fn identifier_range_at(text: &str, idx: usize) -> Option<(usize, usize)> {
        let idx = clamp_to_char_boundary(text, idx);
        let ch = char_at(text, idx)?;
        if !is_identifier_char(ch) {
            return None;
        }

        let mut start = idx;
        while start > 0 {
            let prev = prev_char_boundary(text, start);
            let prev_ch = char_at(text, prev).unwrap_or(' ');
            if !is_identifier_char(prev_ch) {
                break;
            }
            start = prev;
        }

        let mut end = next_char_boundary(text, idx);
        while end < text.len() {
            let next_ch = char_at(text, end).unwrap_or(' ');
            if !is_identifier_char(next_ch) {
                break;
            }
            end = next_char_boundary(text, end);
        }

        Some((start, end))
    }

    pub(crate) fn select_word_range(
        text: &str,
        idx: usize,
        mode: TextBoundaryMode,
    ) -> (usize, usize) {
        if text.is_empty() {
            return (0, 0);
        }

        let mut idx = clamp_to_char_boundary(text, idx).min(text.len());
        if idx >= text.len() {
            idx = prev_char_boundary(text, idx);
        }

        // Prefer selecting the previous word when clicking just after it.
        if char_at(text, idx).is_some_and(|c| c.is_whitespace()) && idx > 0 {
            let prev = prev_char_boundary(text, idx);
            let prev_is_word = match mode {
                TextBoundaryMode::UnicodeWord => is_unicode_word_char(text, prev),
                TextBoundaryMode::Identifier => char_at(text, prev).is_some_and(is_identifier_char),
            };
            if prev_is_word {
                idx = prev;
            }
        }

        let Some(ch) = char_at(text, idx) else {
            return (0, 0);
        };

        if ch.is_whitespace() {
            let mut start = idx;
            while start > 0 {
                let prev = prev_char_boundary(text, start);
                if char_at(text, prev).is_some_and(|c| c.is_whitespace()) {
                    start = prev;
                } else {
                    break;
                }
            }
            let mut end = next_char_boundary(text, idx);
            while end < text.len() {
                if char_at(text, end).is_some_and(|c| c.is_whitespace()) {
                    end = next_char_boundary(text, end);
                } else {
                    break;
                }
            }
            return (start, end);
        }

        match mode {
            TextBoundaryMode::UnicodeWord => {
                unicode_word_range_at(text, idx).unwrap_or((idx, next_char_boundary(text, idx)))
            }
            TextBoundaryMode::Identifier => {
                identifier_range_at(text, idx).unwrap_or((idx, next_char_boundary(text, idx)))
            }
        }
    }

    pub(crate) fn select_line_range(text: &str, idx: usize) -> (usize, usize) {
        if text.is_empty() {
            return (0, 0);
        }

        let idx = clamp_to_char_boundary(text, idx).min(text.len());
        let start = text[..idx]
            .rfind('\n')
            .map(|i| (i + 1).min(text.len()))
            .unwrap_or(0);
        let end = text[idx..]
            .find('\n')
            .map(|i| (idx + i + 1).min(text.len()))
            .unwrap_or(text.len());
        (start, end)
    }

    pub(crate) fn move_word_left(text: &str, idx: usize, mode: TextBoundaryMode) -> usize {
        let mut i = prev_char_boundary(text, idx);
        while i > 0 {
            let prev = prev_char_boundary(text, i);
            let ch = text[prev..i].chars().next().unwrap_or(' ');
            if !ch.is_whitespace() {
                break;
            }
            i = prev;
        }

        if i == 0 {
            return 0;
        }

        match mode {
            TextBoundaryMode::UnicodeWord => unicode_word_range_at(text, i)
                .map(|(start, _)| start)
                .unwrap_or(i),
            TextBoundaryMode::Identifier => identifier_range_at(text, i)
                .map(|(start, _)| start)
                .unwrap_or(i),
        }
    }

    pub(crate) fn move_word_right(text: &str, idx: usize, mode: TextBoundaryMode) -> usize {
        let mut i = next_char_boundary(text, idx);
        while i < text.len() {
            let next = next_char_boundary(text, i);
            let ch = text[i..next].chars().next().unwrap_or(' ');
            if !ch.is_whitespace() {
                break;
            }
            i = next;
        }

        if i >= text.len() {
            return text.len();
        }

        match mode {
            TextBoundaryMode::UnicodeWord => unicode_word_range_at(text, i)
                .map(|(_, end)| end)
                .unwrap_or(i),
            TextBoundaryMode::Identifier => identifier_range_at(text, i)
                .map(|(_, end)| end)
                .unwrap_or(i),
        }
    }
}

pub(crate) mod buffer {
    pub(crate) fn selection_range(selection_anchor: usize, caret: usize) -> (usize, usize) {
        let a = selection_anchor.min(caret);
        let b = selection_anchor.max(caret);
        (a, b)
    }

    pub(crate) fn has_selection(selection_anchor: usize, caret: usize) -> bool {
        selection_anchor != caret
    }

    pub(crate) fn replace_selection(
        text: &mut String,
        caret: &mut usize,
        selection_anchor: &mut usize,
        insert: &str,
    ) {
        let (a, b) = selection_range(*selection_anchor, *caret);
        if a != b {
            text.replace_range(a..b, insert);
            *caret = a + insert.len();
            *selection_anchor = *caret;
        } else {
            text.insert_str(*caret, insert);
            *caret += insert.len();
            *selection_anchor = *caret;
        }
    }

    pub(crate) fn delete_selection_if_any(
        text: &mut String,
        caret: &mut usize,
        selection_anchor: &mut usize,
    ) -> bool {
        let (a, b) = selection_range(*selection_anchor, *caret);
        if a == b {
            return false;
        }
        text.replace_range(a..b, "");
        *caret = a;
        *selection_anchor = *caret;
        true
    }
}

pub(crate) mod state {
    use super::{buffer, ime, utf8};
    use fret_runtime::TextBoundaryMode;

    pub(crate) struct TextEditState<'a> {
        text: &'a mut String,
        caret: &'a mut usize,
        selection_anchor: &'a mut usize,
        preedit: &'a mut String,
        preedit_cursor: &'a mut Option<(usize, usize)>,
        ime_replace_range: &'a mut Option<(usize, usize)>,
        boundary_mode: TextBoundaryMode,
    }

    impl<'a> TextEditState<'a> {
        pub(crate) fn new(
            text: &'a mut String,
            caret: &'a mut usize,
            selection_anchor: &'a mut usize,
            preedit: &'a mut String,
            preedit_cursor: &'a mut Option<(usize, usize)>,
            ime_replace_range: &'a mut Option<(usize, usize)>,
        ) -> Self {
            Self {
                text,
                caret,
                selection_anchor,
                preedit,
                preedit_cursor,
                ime_replace_range,
                boundary_mode: TextBoundaryMode::UnicodeWord,
            }
        }

        pub(crate) fn set_boundary_mode(&mut self, mode: TextBoundaryMode) {
            self.boundary_mode = mode;
        }

        fn clamp_indexes(&mut self) {
            *self.caret = utf8::clamp_to_char_boundary(self.text, *self.caret);
            *self.selection_anchor =
                utf8::clamp_to_char_boundary(self.text, *self.selection_anchor);
        }

        pub(crate) fn clamp_caret_and_anchor_to_char_boundary(&mut self) {
            self.clamp_indexes();
        }

        pub(crate) fn selection_range(&self) -> (usize, usize) {
            buffer::selection_range(*self.selection_anchor, *self.caret)
        }

        pub(crate) fn selected_text_owned(&self) -> Option<String> {
            let (a, b) = self.selection_range();
            if a == b {
                return None;
            }
            Some(self.text.get(a..b)?.to_string())
        }

        pub(crate) fn select_all(&mut self) -> bool {
            self.clamp_indexes();

            let next_anchor = 0;
            let next_caret = self.text.len();
            if *self.selection_anchor == next_anchor && *self.caret == next_caret {
                return false;
            }

            *self.selection_anchor = next_anchor;
            *self.caret = next_caret;
            true
        }

        pub(crate) fn set_selection_char_clamped(&mut self, selection_anchor: usize, caret: usize) {
            *self.selection_anchor = selection_anchor;
            *self.caret = caret;
            self.clamp_indexes();
        }

        pub(crate) fn move_home(&mut self, extend_selection: bool) -> bool {
            self.clamp_indexes();
            self.move_caret_to(0, extend_selection)
        }

        pub(crate) fn move_end(&mut self, extend_selection: bool) -> bool {
            self.clamp_indexes();
            self.move_caret_to(self.text.len(), extend_selection)
        }

        pub(crate) fn move_left(&mut self, extend_selection: bool) -> bool {
            self.clamp_indexes();
            let next = utf8::prev_char_boundary(self.text, *self.caret);
            self.move_caret_to(next, extend_selection)
        }

        pub(crate) fn move_right(&mut self, extend_selection: bool) -> bool {
            self.clamp_indexes();
            let next = utf8::next_char_boundary(self.text, *self.caret);
            self.move_caret_to(next, extend_selection)
        }

        pub(crate) fn move_word_left(&mut self, extend_selection: bool) -> bool {
            self.clamp_indexes();
            let next = utf8::move_word_left(self.text, *self.caret, self.boundary_mode);
            self.move_caret_to(next, extend_selection)
        }

        pub(crate) fn move_word_right(&mut self, extend_selection: bool) -> bool {
            self.clamp_indexes();
            let next = utf8::move_word_right(self.text, *self.caret, self.boundary_mode);
            self.move_caret_to(next, extend_selection)
        }

        pub(crate) fn has_selection(&self) -> bool {
            buffer::has_selection(*self.selection_anchor, *self.caret)
        }

        pub(crate) fn clear_ime_composition(&mut self) {
            ime::clear_state(self.preedit, self.preedit_cursor, self.ime_replace_range);
        }

        fn move_caret_to(&mut self, next: usize, extend_selection: bool) -> bool {
            let had_selection = self.has_selection();

            if next == *self.caret {
                if !extend_selection && had_selection {
                    *self.selection_anchor = *self.caret;
                    return true;
                }
                return false;
            }

            *self.caret = next;
            if !extend_selection {
                *self.selection_anchor = *self.caret;
            }
            true
        }

        pub(crate) fn replace_selection(&mut self, insert: &str) -> bool {
            self.clamp_indexes();

            if insert.is_empty() && !self.has_selection() {
                return false;
            }

            buffer::replace_selection(self.text, self.caret, self.selection_anchor, insert);
            self.clear_ime_composition();
            true
        }

        pub(crate) fn delete_selection_if_any(&mut self) -> bool {
            self.clamp_indexes();
            if !buffer::delete_selection_if_any(self.text, self.caret, self.selection_anchor) {
                return false;
            }
            self.clear_ime_composition();
            true
        }

        pub(crate) fn delete_backward_char(&mut self) -> bool {
            self.clamp_indexes();

            if self.delete_selection_if_any() {
                return true;
            }
            if *self.caret == 0 {
                return false;
            }
            let prev = utf8::prev_char_boundary(self.text, *self.caret);
            self.text.replace_range(prev..*self.caret, "");
            *self.caret = prev;
            *self.selection_anchor = *self.caret;
            self.clear_ime_composition();
            true
        }

        pub(crate) fn delete_forward_char(&mut self) -> bool {
            self.clamp_indexes();

            if self.delete_selection_if_any() {
                return true;
            }
            if *self.caret >= self.text.len() {
                return false;
            }
            let next = utf8::next_char_boundary(self.text, *self.caret);
            self.text.replace_range(*self.caret..next, "");
            *self.selection_anchor = *self.caret;
            self.clear_ime_composition();
            true
        }

        pub(crate) fn delete_word_backward(&mut self) -> bool {
            self.clamp_indexes();

            if self.delete_selection_if_any() {
                return true;
            }
            if *self.caret == 0 {
                return false;
            }
            let prev = utf8::move_word_left(self.text, *self.caret, self.boundary_mode);
            self.text.replace_range(prev..*self.caret, "");
            *self.caret = prev;
            *self.selection_anchor = *self.caret;
            self.clear_ime_composition();
            true
        }

        pub(crate) fn delete_word_forward(&mut self) -> bool {
            self.clamp_indexes();

            if self.delete_selection_if_any() {
                return true;
            }
            if *self.caret >= self.text.len() {
                return false;
            }
            let next = utf8::move_word_right(self.text, *self.caret, self.boundary_mode);
            self.text.replace_range(*self.caret..next, "");
            *self.selection_anchor = *self.caret;
            self.clear_ime_composition();
            true
        }
    }
}

pub(crate) mod commands {
    use super::clipboard;
    use super::state::TextEditState;
    use fret_runtime::TextBoundaryMode;

    #[derive(Debug, Default, Clone, Copy)]
    pub(crate) struct Outcome {
        pub(crate) handled: bool,
        pub(crate) invalidate_paint: bool,
        pub(crate) invalidate_layout: bool,
    }

    #[derive(Debug, Default, Clone, Copy)]
    pub(crate) struct SingleLineUiDelta {
        pub(crate) handled: bool,
        pub(crate) invalidate_paint: bool,
        pub(crate) invalidate_layout: bool,
        pub(crate) release_text_blobs: bool,
        pub(crate) request_redraw: bool,
    }

    #[derive(Debug, Default, Clone, Copy)]
    pub(crate) struct MultilineUiDelta {
        pub(crate) handled: bool,
        pub(crate) invalidate_paint: bool,
        pub(crate) invalidate_layout: bool,
        pub(crate) clear_preedit: bool,
        pub(crate) text_dirty: bool,
        pub(crate) reset_affinity: bool,
        pub(crate) ensure_caret_visible: bool,
    }

    impl Outcome {
        fn paint(changed: bool) -> Self {
            Self {
                handled: true,
                invalidate_paint: changed,
                invalidate_layout: false,
            }
        }

        fn layout(changed: bool) -> Self {
            Self {
                handled: true,
                invalidate_paint: false,
                invalidate_layout: changed,
            }
        }

        fn noop_handled() -> Self {
            Self {
                handled: true,
                invalidate_paint: false,
                invalidate_layout: false,
            }
        }
    }

    pub(crate) fn singleline_ui_delta(command: &str, outcome: Outcome) -> SingleLineUiDelta {
        let is_navigation = command.starts_with("text.move")
            || command.starts_with("text.select")
            || command == "text.select_all";

        let invalidate_layout = outcome.invalidate_layout;
        let invalidate_paint = outcome.invalidate_paint || is_navigation;

        SingleLineUiDelta {
            handled: outcome.handled,
            invalidate_layout,
            invalidate_paint,
            release_text_blobs: invalidate_layout,
            request_redraw: invalidate_layout || invalidate_paint,
        }
    }

    pub(crate) fn multiline_ui_delta(command: &str, outcome: Outcome) -> MultilineUiDelta {
        let is_navigation = command.starts_with("text.move")
            || command.starts_with("text.select")
            || command == "text.select_all";

        let invalidate_layout = outcome.invalidate_layout;
        let invalidate_paint = outcome.invalidate_paint || is_navigation;

        MultilineUiDelta {
            handled: outcome.handled,
            invalidate_layout,
            invalidate_paint,
            clear_preedit: invalidate_layout,
            text_dirty: invalidate_layout,
            reset_affinity: invalidate_layout || invalidate_paint,
            ensure_caret_visible: invalidate_layout || invalidate_paint,
        }
    }

    pub(crate) fn apply_basic(
        edit: &mut TextEditState<'_>,
        command: &str,
        is_ime_composing: bool,
        boundary_mode: TextBoundaryMode,
    ) -> Outcome {
        edit.set_boundary_mode(boundary_mode);
        match command {
            "text.select_all" => Outcome::paint(edit.select_all()),
            "text.move_left" => Outcome::paint(edit.move_left(false)),
            "text.move_right" => Outcome::paint(edit.move_right(false)),
            "text.move_word_left" => Outcome::paint(edit.move_word_left(false)),
            "text.move_word_right" => Outcome::paint(edit.move_word_right(false)),
            "text.move_home" => Outcome::paint(edit.move_home(false)),
            "text.move_end" => Outcome::paint(edit.move_end(false)),
            "text.select_left" => Outcome::paint(edit.move_left(true)),
            "text.select_right" => Outcome::paint(edit.move_right(true)),
            "text.select_word_left" => Outcome::paint(edit.move_word_left(true)),
            "text.select_word_right" => Outcome::paint(edit.move_word_right(true)),
            "text.select_home" => Outcome::paint(edit.move_home(true)),
            "text.select_end" => Outcome::paint(edit.move_end(true)),
            "text.delete_backward" => {
                if is_ime_composing {
                    Outcome::noop_handled()
                } else {
                    Outcome::layout(edit.delete_backward_char())
                }
            }
            "text.delete_forward" => {
                if is_ime_composing {
                    Outcome::noop_handled()
                } else {
                    Outcome::layout(edit.delete_forward_char())
                }
            }
            "text.delete_word_backward" => {
                if is_ime_composing {
                    Outcome::noop_handled()
                } else {
                    Outcome::layout(edit.delete_word_backward())
                }
            }
            "text.delete_word_forward" => {
                if is_ime_composing {
                    Outcome::noop_handled()
                } else {
                    Outcome::layout(edit.delete_word_forward())
                }
            }
            _ => Outcome::default(),
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum ClipboardTextPolicy {
        SingleLine,
        Multiline,
    }

    pub(crate) fn apply_clipboard_text(
        edit: &mut TextEditState<'_>,
        policy: ClipboardTextPolicy,
        text: &str,
    ) -> Outcome {
        let normalized = match policy {
            ClipboardTextPolicy::SingleLine => clipboard::normalize_single_line(text),
            ClipboardTextPolicy::Multiline => clipboard::normalize_multiline(text),
        };

        let Some(normalized) = normalized else {
            return Outcome::noop_handled();
        };

        Outcome::layout(edit.replace_selection(&normalized))
    }

    #[derive(Debug, Clone)]
    pub(crate) enum ClipboardRequest {
        GetText,
        SetText { text: String },
    }

    #[derive(Debug, Default, Clone)]
    pub(crate) struct ClipboardOutcome {
        pub(crate) outcome: Outcome,
        pub(crate) request: Option<ClipboardRequest>,
    }

    pub(crate) fn apply_clipboard(
        edit: &mut TextEditState<'_>,
        command: &str,
        window_available: bool,
    ) -> ClipboardOutcome {
        match command {
            "text.copy" => ClipboardOutcome {
                outcome: Outcome::noop_handled(),
                request: edit
                    .selected_text_owned()
                    .map(|text| ClipboardRequest::SetText { text }),
            },
            "text.cut" => {
                let request = edit
                    .selected_text_owned()
                    .map(|text| ClipboardRequest::SetText { text });
                let changed = edit.delete_selection_if_any();
                ClipboardOutcome {
                    outcome: Outcome::layout(changed),
                    request,
                }
            }
            "text.paste" => ClipboardOutcome {
                outcome: Outcome::noop_handled(),
                request: window_available.then_some(ClipboardRequest::GetText),
            },
            _ => ClipboardOutcome::default(),
        }
    }
}

pub(crate) mod ime {
    use super::buffer;
    use super::{ImeEvent, TickId};

    #[derive(Debug, Default, Clone)]
    pub(crate) struct Deduper {
        last_text_input_tick: Option<TickId>,
        last_text_input_text: Option<String>,
        last_ime_commit_tick: Option<TickId>,
        last_ime_commit_text: Option<String>,
    }

    impl Deduper {
        pub(crate) fn record_text_input(&mut self, tick: TickId, text: &str) {
            self.last_text_input_tick = Some(tick);
            self.last_text_input_text = Some(text.to_string());
        }

        pub(crate) fn ignore_text_input_after_ime_commit(&self, tick: TickId, text: &str) -> bool {
            self.last_ime_commit_tick == Some(tick)
                && self.last_ime_commit_text.as_deref() == Some(text)
        }

        fn last_text_input(&self) -> (Option<TickId>, Option<&str>) {
            (
                self.last_text_input_tick,
                self.last_text_input_text.as_deref(),
            )
        }

        fn record_ime_commit(&mut self, tick: TickId, text: &str) {
            self.last_ime_commit_tick = Some(tick);
            self.last_ime_commit_text = Some(text.to_string());
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum ApplyResult {
        Noop,
        Cleared,
        PreeditUpdated { starting: bool },
        CommitApplied,
        CommitDuplicate,
    }

    pub(crate) fn is_composing(preedit: &str, preedit_cursor: Option<(usize, usize)>) -> bool {
        !preedit.is_empty() || preedit_cursor.is_some()
    }

    pub(crate) fn preedit_cursor_end(
        preedit: &str,
        preedit_cursor: Option<(usize, usize)>,
    ) -> usize {
        preedit_cursor
            .map(|(_, end)| end.min(preedit.len()))
            .unwrap_or(preedit.len())
    }

    pub(crate) fn clear_state(
        preedit: &mut String,
        preedit_cursor: &mut Option<(usize, usize)>,
        ime_replace_range: &mut Option<(usize, usize)>,
    ) {
        preedit.clear();
        *preedit_cursor = None;
        *ime_replace_range = None;
    }

    pub(crate) fn compose_text_at_caret(text: &str, caret: usize, insert: &str) -> Option<String> {
        let prefix = text.get(..caret)?;
        let suffix = text.get(caret..)?;
        Some(format!("{prefix}{insert}{suffix}"))
    }

    pub(crate) fn caret_display_index(
        caret: usize,
        preedit: &str,
        preedit_cursor: Option<(usize, usize)>,
    ) -> usize {
        caret + preedit_cursor_end(preedit, preedit_cursor)
    }

    pub(crate) fn base_to_display_index(
        caret: usize,
        preedit_len: usize,
        base_index: usize,
    ) -> usize {
        if preedit_len == 0 || base_index <= caret {
            base_index
        } else {
            base_index + preedit_len
        }
    }

    pub(crate) fn display_to_base_index(
        caret: usize,
        preedit_len: usize,
        display_index: usize,
    ) -> usize {
        if preedit_len == 0 {
            return display_index;
        }

        let anchor = caret;
        if display_index <= anchor {
            display_index
        } else if display_index >= anchor + preedit_len {
            display_index - preedit_len
        } else {
            anchor
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn apply_event(
        ime: &ImeEvent,
        tick: TickId,
        normalize_newlines: bool,
        deduper: &mut Deduper,
        text: &mut String,
        caret: &mut usize,
        selection_anchor: &mut usize,
        preedit: &mut String,
        preedit_cursor: &mut Option<(usize, usize)>,
        ime_replace_range: &mut Option<(usize, usize)>,
    ) -> ApplyResult {
        match ime {
            ImeEvent::Enabled => ApplyResult::Noop,
            ImeEvent::Disabled => {
                clear_state(preedit, preedit_cursor, ime_replace_range);
                ApplyResult::Cleared
            }
            ImeEvent::Commit(text_in) => {
                let committed = if normalize_newlines && text_in.contains('\r') {
                    super::normalize::newlines_to_lf(text_in)
                } else {
                    text_in.clone()
                };

                let (last_text_input_tick, last_text_input_text) = deduper.last_text_input();
                if last_text_input_tick == Some(tick)
                    && last_text_input_text == Some(committed.as_str())
                {
                    clear_state(preedit, preedit_cursor, ime_replace_range);
                    return ApplyResult::CommitDuplicate;
                }

                deduper.record_ime_commit(tick, committed.as_str());

                if let Some((start, end)) = ime_replace_range.take() {
                    *selection_anchor = start;
                    *caret = end;
                }

                buffer::replace_selection(text, caret, selection_anchor, &committed);
                clear_state(preedit, preedit_cursor, ime_replace_range);
                ApplyResult::CommitApplied
            }
            ImeEvent::Preedit { text: next, cursor } => {
                if next.is_empty() && cursor.is_none() {
                    clear_state(preedit, preedit_cursor, ime_replace_range);
                    return ApplyResult::Cleared;
                }

                let starting = !is_composing(preedit, *preedit_cursor);
                if starting {
                    let (start, end) = buffer::selection_range(*selection_anchor, *caret);
                    if start != end {
                        *ime_replace_range = Some((start, end));
                        *caret = start;
                        *selection_anchor = start;
                    } else {
                        *ime_replace_range = None;
                    }
                }

                *preedit = next.clone();
                *preedit_cursor = *cursor;
                ApplyResult::PreeditUpdated { starting }
            }
        }
    }
}

#[cfg(test)]
mod word_boundary_tests {
    use super::utf8;
    use fret_runtime::TextBoundaryMode;

    #[test]
    fn unicode_word_and_identifier_boundaries_differ_on_apostrophes() {
        let text = "can't";

        assert_eq!(
            utf8::move_word_right(text, 0, TextBoundaryMode::UnicodeWord),
            text.len()
        );
        assert_eq!(
            utf8::move_word_right(text, 0, TextBoundaryMode::Identifier),
            3
        );

        assert_eq!(
            utf8::move_word_left(text, text.len(), TextBoundaryMode::UnicodeWord),
            0
        );
        assert_eq!(
            utf8::move_word_left(text, text.len(), TextBoundaryMode::Identifier),
            4
        );
    }

    #[test]
    fn select_word_range_respects_boundary_mode() {
        let text = "can't";
        assert_eq!(
            utf8::select_word_range(text, 1, TextBoundaryMode::UnicodeWord),
            (0, text.len())
        );
        assert_eq!(
            utf8::select_word_range(text, 1, TextBoundaryMode::Identifier),
            (0, 3)
        );
    }

    #[test]
    fn identifier_selects_full_identifier_with_underscore_and_digits() {
        let text = "foo_bar99 baz";
        let idx = text.find('b').expect("expected identifier character");
        assert_eq!(
            utf8::select_word_range(text, idx, TextBoundaryMode::Identifier),
            (0, "foo_bar99".len())
        );
    }

    #[test]
    fn select_word_range_prefers_previous_word_when_clicking_whitespace_after_word() {
        let text = "hello world";
        let idx = text.find(' ').expect("expected whitespace");
        assert_eq!(
            utf8::select_word_range(text, idx, TextBoundaryMode::UnicodeWord),
            (0, "hello".len())
        );
        assert_eq!(
            utf8::select_word_range(text, idx, TextBoundaryMode::Identifier),
            (0, "hello".len())
        );
    }

    #[test]
    fn select_word_range_selects_whitespace_runs() {
        let text = "a   b";
        assert_eq!(
            utf8::select_word_range(text, 2, TextBoundaryMode::UnicodeWord),
            (1, 4)
        );
        assert_eq!(
            utf8::select_word_range(text, 2, TextBoundaryMode::Identifier),
            (1, 4)
        );
    }

    #[test]
    fn identifier_treats_unicode_identifier_chars_as_word_members() {
        let text = "变量名_foo";
        let idx = text
            .char_indices()
            .nth(1)
            .expect("expected a second char")
            .0;
        assert_eq!(
            utf8::select_word_range(text, idx, TextBoundaryMode::Identifier),
            (0, text.len())
        );
    }
}

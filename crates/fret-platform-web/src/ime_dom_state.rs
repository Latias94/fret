#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

//! Small, testable state machine for DOM event disposition in the wasm IME bridge.
//!
//! This module helps avoid “command executes + DOM inserts text” double-inserts by tracking
//! whether the next DOM `input` event should be suppressed (e.g. after shortcut handling or
//! immediately after `compositionend`).
//!
//! Event ordering note:
//!
//! Some browsers can still emit an `input` event even when:
//! - we handled an insertion via `beforeinput` + `prevent_default()`, or
//! - we suppressed a DOM mutation via `beforeinput` because the edit was already handled via a
//!   command path or `compositionend`.
//!
//! In those cases we must ignore the follow-up `input` once to avoid double-inserting text.
//!
//! Normative behavior is defined by ADR 0180: Web IME and Text Input Bridge (wasm, v1).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomControlIntent {
    DeleteBackward,
    DeleteForward,
    Enter,
}

pub fn control_intent_for_beforeinput_type(input_type: &str) -> Option<DomControlIntent> {
    match input_type {
        "deleteContentBackward" => Some(DomControlIntent::DeleteBackward),
        "deleteContentForward" => Some(DomControlIntent::DeleteForward),
        "insertLineBreak" | "insertParagraph" => Some(DomControlIntent::Enter),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomInputDisposition {
    IgnoreComposing,
    IgnoreSuppressed,
    Process,
}

#[derive(Default, Clone)]
pub struct WebImeDomState {
    composing: bool,
    suppress_next_input: bool,
    ignore_next_input: bool,
    pending_control_intent: Option<DomControlIntent>,
}

impl fmt::Debug for WebImeDomState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebImeDomState")
            .field("composing", &self.composing)
            .field("suppress_next_input", &self.suppress_next_input)
            .field("ignore_next_input", &self.ignore_next_input)
            .field("pending_control_intent", &self.pending_control_intent)
            .finish()
    }
}

impl WebImeDomState {
    pub fn composing(&self) -> bool {
        self.composing
    }

    pub fn suppress_next_input(&self) -> bool {
        self.suppress_next_input
    }

    pub fn on_composition_start(&mut self) {
        self.composing = true;
    }

    pub fn on_composition_update(&mut self) {
        self.composing = true;
    }

    pub fn on_composition_end(&mut self) {
        self.composing = false;
        self.suppress_next_input = true;
    }

    pub fn on_shortcut_suppressed(&mut self) {
        self.suppress_next_input = true;
    }

    pub fn on_beforeinput_handled(&mut self) {
        self.ignore_next_input = true;
    }

    pub fn on_control_keydown(&mut self, intent: DomControlIntent) {
        self.pending_control_intent = Some(intent);
    }

    pub fn on_control_keyup(&mut self, intent: DomControlIntent) {
        if self.pending_control_intent == Some(intent) {
            self.pending_control_intent = None;
        }
    }

    pub fn suppress_matching_control_beforeinput(&mut self, intent: DomControlIntent) -> bool {
        if self.pending_control_intent == Some(intent) {
            self.pending_control_intent = None;
            true
        } else {
            false
        }
    }

    pub fn on_ime_disabled(&mut self) {
        self.composing = false;
        self.suppress_next_input = false;
        self.ignore_next_input = false;
        self.pending_control_intent = None;
    }

    pub fn beforeinput_disposition(&mut self, dom_is_composing: bool) -> DomInputDisposition {
        if self.composing || dom_is_composing {
            return DomInputDisposition::IgnoreComposing;
        }
        if self.suppress_next_input {
            self.suppress_next_input = false;
            return DomInputDisposition::IgnoreSuppressed;
        }
        DomInputDisposition::Process
    }

    pub fn input_disposition(&mut self) -> DomInputDisposition {
        if self.composing {
            return DomInputDisposition::IgnoreComposing;
        }
        if self.ignore_next_input {
            self.ignore_next_input = false;
            return DomInputDisposition::IgnoreSuppressed;
        }
        if self.suppress_next_input {
            self.suppress_next_input = false;
            return DomInputDisposition::IgnoreSuppressed;
        }
        DomInputDisposition::Process
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_sequence_suppresses_commit_followups() {
        let mut st = WebImeDomState::default();

        st.on_composition_start();
        assert_eq!(
            st.beforeinput_disposition(true),
            DomInputDisposition::IgnoreComposing
        );

        st.on_composition_end();
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::IgnoreSuppressed
        );
        st.on_beforeinput_handled();
        assert_eq!(
            st.input_disposition(),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(st.input_disposition(), DomInputDisposition::Process);
    }

    #[test]
    fn shortcut_suppression_then_composing_beforeinput_does_not_prevent_ignoring_followup_input() {
        let mut st = WebImeDomState::default();

        st.on_shortcut_suppressed();
        assert_eq!(
            st.beforeinput_disposition(true),
            DomInputDisposition::IgnoreComposing
        );
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::IgnoreSuppressed
        );
        st.on_beforeinput_handled();
        assert_eq!(
            st.input_disposition(),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(st.input_disposition(), DomInputDisposition::Process);
    }

    #[test]
    fn beforeinput_processed_insert_suppresses_followup_input() {
        let mut st = WebImeDomState::default();
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::Process
        );
        st.on_beforeinput_handled();
        assert_eq!(
            st.input_disposition(),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(st.input_disposition(), DomInputDisposition::Process);
    }

    #[test]
    fn beforeinput_suppression_marks_followup_input_ignored() {
        let mut st = WebImeDomState::default();
        st.on_shortcut_suppressed();
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::IgnoreSuppressed
        );
        st.on_beforeinput_handled();
        assert_eq!(
            st.input_disposition(),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(st.input_disposition(), DomInputDisposition::Process);
    }

    #[test]
    fn composition_end_beforeinput_suppression_marks_followup_input_ignored() {
        let mut st = WebImeDomState::default();
        st.on_composition_start();
        st.on_composition_end();
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::IgnoreSuppressed
        );
        st.on_beforeinput_handled();
        assert_eq!(
            st.input_disposition(),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(st.input_disposition(), DomInputDisposition::Process);
    }

    #[test]
    fn shortcut_suppression_consumes_one_input_turn() {
        let mut st = WebImeDomState::default();
        st.on_shortcut_suppressed();
        assert_eq!(
            st.input_disposition(),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(st.input_disposition(), DomInputDisposition::Process);
    }

    #[test]
    fn shortcut_suppression_consumes_one_beforeinput_turn() {
        let mut st = WebImeDomState::default();
        st.on_shortcut_suppressed();
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::Process
        );
    }

    #[test]
    fn composition_end_suppresses_followup_input() {
        let mut st = WebImeDomState::default();
        st.on_composition_start();
        assert_eq!(st.input_disposition(), DomInputDisposition::IgnoreComposing);

        st.on_composition_end();
        assert_eq!(
            st.input_disposition(),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(st.input_disposition(), DomInputDisposition::Process);
    }

    #[test]
    fn composition_end_suppresses_followup_beforeinput() {
        let mut st = WebImeDomState::default();
        st.on_composition_start();
        assert_eq!(
            st.beforeinput_disposition(true),
            DomInputDisposition::IgnoreComposing
        );

        st.on_composition_end();
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::Process
        );
    }

    #[test]
    fn composing_does_not_consume_suppression() {
        let mut st = WebImeDomState::default();
        st.on_shortcut_suppressed();
        st.on_composition_start();
        assert_eq!(st.input_disposition(), DomInputDisposition::IgnoreComposing);

        st.on_composition_end();
        assert_eq!(
            st.input_disposition(),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(st.input_disposition(), DomInputDisposition::Process);
    }

    #[test]
    fn beforeinput_ignores_dom_is_composing_without_consuming_suppression() {
        let mut st = WebImeDomState::default();
        st.on_shortcut_suppressed();
        assert_eq!(
            st.beforeinput_disposition(true),
            DomInputDisposition::IgnoreComposing
        );
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::IgnoreSuppressed
        );
        assert_eq!(
            st.beforeinput_disposition(false),
            DomInputDisposition::Process
        );
    }

    #[test]
    fn matching_control_beforeinput_is_suppressed_once_after_keydown() {
        let mut st = WebImeDomState::default();
        st.on_control_keydown(DomControlIntent::DeleteBackward);
        assert!(st.suppress_matching_control_beforeinput(DomControlIntent::DeleteBackward));
        assert!(!st.suppress_matching_control_beforeinput(DomControlIntent::DeleteBackward));
    }

    #[test]
    fn control_keyup_clears_pending_control_intent() {
        let mut st = WebImeDomState::default();
        st.on_control_keydown(DomControlIntent::DeleteForward);
        st.on_control_keyup(DomControlIntent::DeleteForward);
        assert!(!st.suppress_matching_control_beforeinput(DomControlIntent::DeleteForward));
    }

    #[test]
    fn enter_beforeinput_is_suppressed_once_after_keydown() {
        let mut st = WebImeDomState::default();
        st.on_control_keydown(DomControlIntent::Enter);
        assert!(st.suppress_matching_control_beforeinput(DomControlIntent::Enter));
        assert!(!st.suppress_matching_control_beforeinput(DomControlIntent::Enter));
    }

    #[test]
    fn beforeinput_control_intent_mapping_covers_delete_and_enter_types() {
        assert_eq!(
            control_intent_for_beforeinput_type("deleteContentBackward"),
            Some(DomControlIntent::DeleteBackward)
        );
        assert_eq!(
            control_intent_for_beforeinput_type("deleteContentForward"),
            Some(DomControlIntent::DeleteForward)
        );
        assert_eq!(
            control_intent_for_beforeinput_type("insertLineBreak"),
            Some(DomControlIntent::Enter)
        );
        assert_eq!(
            control_intent_for_beforeinput_type("insertParagraph"),
            Some(DomControlIntent::Enter)
        );
        assert_eq!(control_intent_for_beforeinput_type("insertText"), None);
    }
}

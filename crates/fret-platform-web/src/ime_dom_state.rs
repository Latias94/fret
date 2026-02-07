#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

//! Small, testable state machine for DOM event disposition in the wasm IME bridge.
//!
//! This module helps avoid “command executes + DOM inserts text” double-inserts by tracking
//! whether the next DOM `input` event should be suppressed (e.g. after shortcut handling or
//! immediately after `compositionend`).
//!
//! Normative behavior is defined by ADR 0195: Web IME and Text Input Bridge (wasm, v1).

use std::fmt;

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
}

impl fmt::Debug for WebImeDomState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebImeDomState")
            .field("composing", &self.composing)
            .field("suppress_next_input", &self.suppress_next_input)
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

    pub fn on_ime_disabled(&mut self) {
        self.composing = false;
        self.suppress_next_input = false;
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
}

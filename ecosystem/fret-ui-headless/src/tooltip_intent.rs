use fret_core::PointerType;

/// Deterministic tooltip trigger intent gates used by shadcn/Radix recipes.
///
/// This concentrates the "suppress hover/focus reopen" policy so overlay recipes stay wiring-only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TooltipTriggerIntentGates {
    pub has_pointer_move_opened: bool,
    pub suppress_hover_open: bool,
    pub suppress_focus_open: bool,
}

impl TooltipTriggerIntentGates {
    pub fn trigger_hovered(self, hovered: bool) -> bool {
        hovered && self.has_pointer_move_opened && !self.suppress_hover_open
    }

    pub fn trigger_focused(self, focused: bool) -> bool {
        focused && !self.suppress_focus_open
    }

    /// Applies a hover-leave edge: it resets the "must see pointermove before opening" gate and any
    /// hover suppression.
    pub fn on_left_hover(mut self, left_hover: bool) -> Self {
        if left_hover && (self.has_pointer_move_opened || self.suppress_hover_open) {
            self.has_pointer_move_opened = false;
            self.suppress_hover_open = false;
        }
        self
    }

    /// Clears focus suppression once focus is no longer on the trigger.
    pub fn on_focus_changed(mut self, focused: bool) -> Self {
        if !focused && self.suppress_focus_open {
            self.suppress_focus_open = false;
        }
        self
    }

    /// Handles a close request and returns the updated gates plus whether the close flag should be
    /// cleared.
    pub fn on_close_requested(mut self, close_requested: bool, focused: bool) -> (Self, bool) {
        if !close_requested {
            return (self, false);
        }

        // Radix-like behavior: if a tooltip was opened via pointermove, closing it should suppress
        // immediate hover reopen until we leave and re-enter.
        if self.has_pointer_move_opened && !self.suppress_hover_open {
            self.suppress_hover_open = true;
        }

        // Closing (via outside interaction, activate, escape) should suppress focus-driven reopen
        // for the current focus session.
        if focused && !self.suppress_focus_open {
            self.suppress_focus_open = true;
        }

        (self, true)
    }

    /// Applies a non-touch pointer down on the trigger.
    ///
    /// Returns `(updated_gates, request_close_now)`.
    pub fn on_pointer_down(mut self, pointer_type: PointerType) -> (Self, bool) {
        let request_close = pointer_type != PointerType::Touch;
        self.suppress_focus_open = true;
        if self.has_pointer_move_opened {
            self.suppress_hover_open = true;
        }
        (self, request_close)
    }

    /// Applies an "activate" intent on the trigger (e.g. keyboard activation).
    ///
    /// Returns `(updated_gates, request_close_now)`.
    pub fn on_activate(mut self) -> (Self, bool) {
        self.suppress_focus_open = true;
        (self, true)
    }

    /// Applies an Escape key dismissal intent on the trigger.
    ///
    /// Returns `(updated_gates, request_close_now)`.
    pub fn on_escape(mut self) -> (Self, bool) {
        self.suppress_focus_open = true;
        (self, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_hover_resets_hover_gates() {
        let st = TooltipTriggerIntentGates {
            has_pointer_move_opened: true,
            suppress_hover_open: true,
            suppress_focus_open: false,
        };
        let out = st.on_left_hover(true);
        assert_eq!(
            out,
            TooltipTriggerIntentGates {
                has_pointer_move_opened: false,
                suppress_hover_open: false,
                suppress_focus_open: false,
            }
        );
    }

    #[test]
    fn blur_clears_focus_suppression() {
        let st = TooltipTriggerIntentGates {
            has_pointer_move_opened: false,
            suppress_hover_open: false,
            suppress_focus_open: true,
        };
        assert_eq!(
            st.on_focus_changed(false),
            TooltipTriggerIntentGates {
                suppress_focus_open: false,
                ..st
            }
        );
    }

    #[test]
    fn close_request_suppresses_focus_and_optionally_hover() {
        let st = TooltipTriggerIntentGates {
            has_pointer_move_opened: true,
            suppress_hover_open: false,
            suppress_focus_open: false,
        };
        let (out, clear) = st.on_close_requested(true, true);
        assert!(clear);
        assert!(out.suppress_focus_open);
        assert!(out.suppress_hover_open);
        assert!(out.has_pointer_move_opened);
    }

    #[test]
    fn pointer_down_suppresses_focus_and_hover_when_pointer_opened() {
        let st = TooltipTriggerIntentGates {
            has_pointer_move_opened: true,
            suppress_hover_open: false,
            suppress_focus_open: false,
        };
        let (out, request_close) = st.on_pointer_down(PointerType::Mouse);
        assert!(request_close);
        assert!(out.suppress_focus_open);
        assert!(out.suppress_hover_open);
    }

    #[test]
    fn touch_pointer_down_does_not_request_close() {
        let st = TooltipTriggerIntentGates::default();
        let (_out, request_close) = st.on_pointer_down(PointerType::Touch);
        assert!(!request_close);
    }
}

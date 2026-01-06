//! Keyboard policy helpers (Radix-aligned outcomes).
//!
//! Radix primitives sometimes consume certain keys to match WAI-ARIA expectations, for example:
//! - Checkbox and RadioGroup prevent `Enter` from activating items (Space activates instead).
//!
//! This module provides tiny, reusable key hook handlers so recipes can stay thin.

use std::sync::Arc;

use fret_core::KeyCode;

/// Returns a key hook handler that consumes Enter/NumpadEnter key presses.
pub fn consume_enter_key_handler() -> fret_ui::action::OnKeyDown {
    Arc::new(|_host, _cx, down| matches!(down.key, KeyCode::Enter | KeyCode::NumpadEnter))
}

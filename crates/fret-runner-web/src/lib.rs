//! Web/wasm runner glue for Fret.
//!
//! On `wasm32`, this crate re-exports `fret-platform-web` services used by `fret-runtime::Effect`s
//! and provides DOM-adjacent adapters (cursor, input event mapping, RAF/timers). It intentionally
//! keeps non-wasm builds explicit via a stub module.
//!
//! For module ownership and “where should this go?” guidance, see `crates/fret-runner-web/README.md`.
//!
//! Long-term direction: a dedicated DOM adapter for IME/keyboard fidelity (see ADR 0089/0092).

#[cfg(target_arch = "wasm32")]
pub use fret_platform_web::*;

#[cfg(target_arch = "wasm32")]
mod cursor;
#[cfg(target_arch = "wasm32")]
mod events;
#[cfg(target_arch = "wasm32")]
mod raf;

#[cfg(target_arch = "wasm32")]
pub use cursor::{
    RunnerError, WebCursorListener, canvas_by_id, install_canvas_cursor_listener,
    last_cursor_offset_px,
};
#[cfg(target_arch = "wasm32")]
pub use events::{WebInputState, WebPointerEventKind, map_keyboard_event};
#[cfg(target_arch = "wasm32")]
pub use raf::{cancel_animation_frame, request_animation_frame, set_timeout_ms};

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn runner_error_is_actionable_on_non_wasm_targets() {
        let err = RunnerError;
        assert_eq!(
            err.to_string(),
            "fret-runner-web is only available on wasm32"
        );

        fn assert_error(_err: &dyn std::error::Error) {}
        assert_error(&err);
    }
}

//! Web platform adapter utilities.
//!
//! This crate is intended to host **web-specific** glue that should not live in:
//! - `fret-platform-web` (platform I/O backend for Effects),
//! - `fret-runner-winit` (winit-specific event mapping).
//!
//! Long-term direction: a dedicated DOM adapter for IME/keyboard fidelity (ADR 0091/0093).

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

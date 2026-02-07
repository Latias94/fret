//! Winit-based runner glue for Fret.
//!
//! This crate focuses on translating winit input/window events into `fret-core` events and
//! maintaining a small amount of window-side state (cursor/IME) at the platform boundary.
//!
//! For module ownership and “where should this go?” guidance, see
//! `crates/fret-runner-winit/README.md`.

pub mod accessibility;

#[cfg(windows)]
pub mod windows_ime;

pub mod window_registry;

mod error;
mod external_drag;
mod mapping;
mod state;

#[cfg(target_arch = "wasm32")]
mod web_cursor;

pub use error::RunnerError;
pub use external_drag::external_drag_files;

pub use mapping::{
    WheelConfig, is_alt_gr_key, map_cursor_icon, map_modifiers, map_mouse_button,
    map_optional_physical_position_to_point, map_physical_key, map_physical_position_to_point,
    map_pointer_button, map_pointer_id_from_button_source, map_pointer_id_from_pointer_kind,
    map_pointer_id_from_pointer_source, map_pointer_kind, map_pointer_type,
    map_pointer_type_from_pointer_source, map_wheel_delta, sanitize_text_input, set_mouse_buttons,
};

pub use state::input::WinitInputState;
pub use state::{WinitPlatform, WinitWindowState};

#[cfg(target_arch = "wasm32")]
pub use web_cursor::{WebCursorListener, canvas_by_id, install_web_cursor_listener};

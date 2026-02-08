//! Winit runner glue for Fret.
//!
//! This crate maps winit window/input events into portable `fret-core` events and hosts
//! platform-specific integrations (IME, accessibility) that should not leak into core crates.

pub mod accessibility;

#[cfg(windows)]
pub mod windows_ime;

pub mod window_registry;

mod error;
mod external_drag;
mod mapping;
mod state;

pub use external_drag::external_drag_files;
pub use mapping::{
    WheelConfig, is_alt_gr_key, map_cursor_icon, map_modifiers, map_mouse_button,
    map_optional_physical_position_to_point, map_physical_key, map_physical_position_to_point,
    map_pointer_button, map_pointer_id_from_button_source, map_pointer_id_from_pointer_kind,
    map_pointer_id_from_pointer_source, map_pointer_kind, map_pointer_type,
    map_pointer_type_from_pointer_source, map_wheel_delta, sanitize_text_input, set_mouse_buttons,
};
pub use state::{WinitInputState, WinitPlatform, WinitWindowState};

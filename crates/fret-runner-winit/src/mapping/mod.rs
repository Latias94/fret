mod cursor;
mod keyboard;
mod modifiers;
mod mouse;
mod pointer;
mod position;
mod wheel;

pub use cursor::map_cursor_icon;
pub use keyboard::{is_alt_gr_key, map_physical_key, sanitize_text_input};
pub use modifiers::map_modifiers;
pub use mouse::{map_mouse_button, set_mouse_buttons};
pub use pointer::{
    map_pointer_button, map_pointer_id_from_button_source, map_pointer_id_from_pointer_kind,
    map_pointer_id_from_pointer_source, map_pointer_kind, map_pointer_type,
    map_pointer_type_from_pointer_source,
};
pub use position::{map_optional_physical_position_to_point, map_physical_position_to_point};
pub use wheel::{WheelConfig, map_wheel_delta};

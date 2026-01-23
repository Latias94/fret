//! Material Design 3 (and Expressive) component surface for Fret.
//!
//! This crate targets **visual + interaction outcome alignment** with the Material 3 design
//! system, while keeping `crates/fret-ui` focused on mechanisms (not Material-specific policy).

#![forbid(unsafe_code)]

pub mod button;
pub mod checkbox;
pub mod icon_button;
pub mod interaction;
pub mod menu;
pub mod motion;
pub mod radio;
pub mod switch;
pub mod tabs;
pub mod text_field;
pub mod theme;
pub mod tokens;

pub use button::{Button, ButtonVariant};
pub use checkbox::Checkbox;
pub use icon_button::{IconButton, IconButtonSize, IconButtonVariant};
pub use menu::{Menu, MenuEntry, MenuItem};
pub use radio::{Radio, RadioGroup, RadioGroupItem, RadioGroupOrientation};
pub use switch::Switch;
pub use tabs::{TabItem, Tabs};
pub use text_field::{TextField, TextFieldVariant};

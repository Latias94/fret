//! Editor primitives (surfaces, density helpers, chrome building blocks).

pub mod density;
pub mod edit_session;
pub(crate) mod icons;
pub(crate) mod input_group;
pub(crate) mod inspector_layout;
pub mod numeric_text_entry;
pub(crate) mod style;
pub mod tokens;
pub(crate) mod visuals;

pub(crate) mod chrome;

pub mod drag_value_core;
pub mod numeric_format;

pub use density::EditorDensity;
pub use drag_value_core::{
    DragValueCore, DragValueCoreOptions, DragValueCoreResponse, DragValueScalar,
};
pub use edit_session::{EditSession, EditSessionOutcome};
pub use numeric_format::{percent_0_1_format, percent_0_1_parse};
pub use numeric_text_entry::NumericInputSelectionBehavior;
pub use tokens::EditorTokenKeys;

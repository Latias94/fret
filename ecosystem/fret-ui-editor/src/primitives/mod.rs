//! Editor primitives (surfaces, density helpers, chrome building blocks).

pub mod density;
pub mod edit_session;
pub(crate) mod icons;
pub(crate) mod input_group;
pub(crate) mod inspector_layout;
pub mod numeric_text_entry;
pub(crate) mod popup_list;
pub(crate) mod popup_surface;
pub mod readout;
pub(crate) mod style;
pub mod text_entry;
pub mod tokens;
pub(crate) mod visuals;

pub(crate) mod chrome;
pub(crate) mod colors;

pub mod drag_value_core;
pub mod numeric_format;
pub mod numeric_value;

pub use density::EditorDensity;
pub use drag_value_core::{
    DragValueCore, DragValueCoreOptions, DragValueCoreResponse, DragValueScalar,
};
pub use edit_session::{EditSession, EditSessionOutcome};
pub use numeric_format::{
    NumericPresentation, NumericTextAffixes, affixed_number_format, affixed_number_parse,
    degrees_format, degrees_parse, fixed_decimals_format, percent_0_1_format, percent_0_1_parse,
    plain_number_parse,
};
pub use numeric_text_entry::NumericInputSelectionBehavior;
pub use numeric_value::{NumericValueConstraints, constrain_numeric_value};
pub use readout::{EditorCompactReadoutStyle, compact_readout_text_px};
pub use text_entry::{EditorTextCancelBehavior, EditorTextSelectionBehavior};
pub use tokens::EditorTokenKeys;

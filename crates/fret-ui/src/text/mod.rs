//! Text subsystem modules for the UI runtime.
//!
//! These are retained/widget-like editing engines (IME/caret/selection) that need hard-to-change
//! behavior contracts and platform-facing hooks. UI policy should remain in ecosystem crates.

#[allow(dead_code)]
pub(crate) mod area;
pub(crate) mod edit;
#[allow(dead_code)]
pub(crate) mod input;
pub(crate) mod input_style;
pub(crate) mod props;
pub(crate) mod surface;

pub use area::TextAreaStyle;
pub use input_style::TextInputStyle;

//! Input-family chrome helpers built on shared theme/style resolution.
//!
//! This module intentionally stays in `recipes`:
//! - chrome/token resolution stays here,
//! - text-input default styling stays here,
//! - and app-facing wrappers compose these building blocks without owning policy.

mod chrome;
mod refinement;
mod text_style;

pub use chrome::{
    InputTokenKeys, ResolvedInputChrome, input_chrome_container_props, resolve_input_chrome,
};
pub use refinement::input_base_refinement;
pub use text_style::default_text_input_style;

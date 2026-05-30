//! Immediate-mode text input and textarea helpers.

mod focus;
mod input;
mod policy_commands;
mod style;
mod textarea;

use input::text_model_changed_for;
pub(super) use input::{
    InputTextAssistiveSemantics, input_text_model_element_with_options_and_semantics,
    input_text_model_with_options,
};
pub(in crate::imui) use textarea::textarea_model_with_options;

#[cfg(test)]
mod tests;

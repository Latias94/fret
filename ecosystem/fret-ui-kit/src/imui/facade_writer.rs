//! Immediate-mode facade writer glue.

use super::*;
use std::any::Any;

mod basic_items;
mod basic_surface;
mod boolean_wrappers;
mod button_actions;
mod button_surface;
mod container_methods;
mod container_surface;
mod container_wrappers;
mod disclosure;
mod disclosure_surface;
mod facade_core;
mod floating_popup;
mod floating_surface;
mod image_items;
mod menu_items;
mod menu_selection_surface;
mod model_surface;
mod scope_methods;
mod scope_surface;
mod selection_combo;
mod text_models;
mod trait_ext;
mod value_models;

pub use facade_core::ImUiFacade;
pub use trait_ext::UiWriterImUiFacadeExt;

#[cfg(test)]
mod tests;

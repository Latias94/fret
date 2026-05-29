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
mod value_models;

pub use facade_core::ImUiFacade;

/// Immediate-mode facade helpers for any authoring frontend that implements `UiWriter`.
///
/// This is intentionally a small convenience layer. It aims to feel closer to egui/imgui while
/// still compiling down to Fret's declarative element tree and delegating complex policy to
/// higher-level components.
pub trait UiWriterImUiFacadeExt<H: UiHost>: UiWriter<H> {
    scope_surface::scope_surface_methods!();

    basic_surface::basic_surface_methods!();

    container_surface::layout_surface_methods!();

    container_surface::menu_tab_surface_methods!();

    container_surface::collection_surface_methods!();

    container_surface::region_surface_methods!();

    floating_surface::floating_popup_surface_methods!();

    disclosure_surface::disclosure_surface_methods!();

    floating_surface::tooltip_drag_surface_methods!();

    menu_selection_surface::menu_item_surface_methods!();

    menu_selection_surface::menu_family_surface_methods!();

    menu_selection_surface::selection_combo_surface_methods!();

    menu_selection_surface::context_popup_surface_methods!();

    button_surface::button_surface_methods!();

    model_surface::boolean_model_surface_methods!();

    model_surface::value_combo_model_surface_methods!();

    model_surface::text_model_surface_methods!();

    floating_surface::window_surface_methods!();
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterImUiFacadeExt<H> for W {}

#[cfg(test)]
mod tests;

//! Immediate-mode facade writer glue.

use super::*;
use std::any::Any;

mod basic_items;
mod boolean_wrappers;
mod button_actions;
mod button_surface;
mod container_methods;
mod container_surface;
mod container_wrappers;
mod disclosure;
mod facade_core;
mod floating_popup;
mod floating_surface;
mod image_items;
mod menu_items;
mod menu_selection_surface;
mod model_surface;
mod scope_methods;
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
    fn push_id<K: Hash, R>(
        &mut self,
        key: K,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>) -> R,
    ) -> R {
        scope_methods::push_id(self, key, f)
    }

    /// Disable all `imui`-facade interactions within the closure and dim visuals (ImGui-style
    /// `BeginDisabled/EndDisabled`).
    ///
    /// Notes:
    /// - This helper is scoped to the closure (Rust-friendly) rather than a manual begin/end pair.
    /// - Nested disabled scopes do not multiply opacity; only the outermost disabled scope applies
    ///   the visual dimming.
    /// - The disabled alpha multiplier is controlled by theme number
    ///   `component.imui.disabled_alpha` (default `0.60`).
    fn disabled_scope(
        &mut self,
        disabled: bool,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        scope_methods::disabled_scope(self, disabled, f);
    }

    fn text(&mut self, text: impl Into<Arc<str>>) {
        basic_items::text(self, text.into());
    }

    fn text_wrapped(&mut self, text: impl Into<Arc<str>>) {
        basic_items::text_wrapped(self, text.into());
    }

    fn bullet_text(&mut self, text: impl Into<Arc<str>>) {
        self.bullet_text_with_options(text, BulletTextOptions::default());
    }

    fn bullet_text_with_options(&mut self, text: impl Into<Arc<str>>, options: BulletTextOptions) {
        basic_items::bullet_text_with_options(self, text.into(), options);
    }

    fn debug_draw<K: Hash>(
        &mut self,
        id: K,
        draw: impl FnOnce(&mut ImUiDebugDrawList),
    ) -> DebugDrawResponse {
        self.debug_draw_with_options(id, DebugDrawOptions::default(), draw)
    }

    fn debug_draw_with_options<K: Hash>(
        &mut self,
        id: K,
        options: DebugDrawOptions,
        draw: impl FnOnce(&mut ImUiDebugDrawList),
    ) -> DebugDrawResponse {
        basic_items::debug_draw_with_options(self, id, options, draw)
    }

    fn separator(&mut self) {
        basic_items::separator(self);
    }

    fn separator_text(&mut self, label: impl Into<Arc<str>>) {
        self.separator_text_with_options(label, SeparatorTextOptions::default());
    }

    fn separator_text_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: SeparatorTextOptions,
    ) {
        basic_items::separator_text_with_options(self, label.into(), options);
    }

    container_surface::container_surface_methods!();

    floating_surface::floating_popup_surface_methods!();

    /// Build a generic immediate collapsing header with explicit stable identity.
    ///
    /// `id` must be stable and semantic across frames. Do not derive identity from the visible
    /// label alone; prefer domain keys such as `"scene.sections.rendering"`.
    fn collapsing_header(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.collapsing_header_with_options(id, label, CollapsingHeaderOptions::default(), f)
    }

    fn collapsing_header_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: CollapsingHeaderOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        disclosure_controls::collapsing_header_with_options(self, id, label.into(), options, f)
    }

    /// Build a generic immediate tree node with explicit stable identity and explicit depth.
    ///
    /// Fret does not emulate ImGui's implicit ID/indent stack here. Child nodes should use their
    /// own stable ids (for example `"scene/root/camera"`) and set `TreeNodeOptions::level`
    /// explicitly instead of inventing `"##suffix"` tricks.
    fn tree_node(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.tree_node_with_options(id, label, TreeNodeOptions::default(), f)
    }

    fn tree_node_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: TreeNodeOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        disclosure_controls::tree_node_with_options(self, id, label.into(), options, f)
    }

    floating_surface::tooltip_drag_surface_methods!();

    menu_selection_surface::menu_selection_surface_methods!();

    button_surface::button_surface_methods!();

    model_surface::model_surface_methods!();

    floating_surface::window_surface_methods!();
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterImUiFacadeExt<H> for W {}

#[cfg(test)]
mod tests;

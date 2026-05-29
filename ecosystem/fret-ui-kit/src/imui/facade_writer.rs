//! Immediate-mode facade writer glue.

use super::*;
use std::any::Any;

mod basic_items;
mod boolean_wrappers;
mod button_actions;
mod button_surface;
mod container_methods;
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

    /// Explicit vertical item-flow convenience for ImGui ports.
    ///
    /// This does not add an implicit layout cursor. It is a scoped vertical group whose default
    /// gap reads `component.imui.item_spacing_y_px` (fallback `4px`).
    fn items(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        container_methods::items(self, None, f);
    }

    fn items_with_options(
        &mut self,
        options: ItemFlowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::items_with_options(self, None, options, f);
    }

    /// Explicit horizontal same-line group for ImGui ports.
    ///
    /// This intentionally scopes "same line" to the closure instead of reaching backward to a
    /// previous item. The default gap reads `component.imui.item_spacing_x_px` (fallback `8px`).
    fn same_line(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        container_methods::same_line(self, None, f);
    }

    fn same_line_with_options(
        &mut self,
        options: SameLineOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::same_line_with_options(self, None, options, f);
    }

    fn dummy(&mut self, size: Size) {
        container_methods::dummy(self, size);
    }

    fn dummy_with_options(&mut self, size: Size, options: DummyOptions) {
        container_methods::dummy_with_options(self, size, options);
    }

    fn spacing(&mut self) {
        container_methods::spacing(self);
    }

    fn spacing_with_options(&mut self, options: SpacingOptions) {
        container_methods::spacing_with_options(self, options);
    }

    fn indent(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        container_methods::indent(self, None, f);
    }

    fn indent_with_options(
        &mut self,
        options: IndentOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::indent_with_options(self, None, options, f);
    }

    fn horizontal(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        container_methods::horizontal(self, None, f);
    }

    fn horizontal_with_options(
        &mut self,
        options: HorizontalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::horizontal_with_options(self, None, options, f);
    }

    fn menu_bar(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        container_methods::menu_bar(self, None, f);
    }

    fn menu_bar_with_options(
        &mut self,
        options: MenuBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::menu_bar_with_options(self, None, options, f);
    }

    fn tab_bar(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        container_methods::tab_bar(self, None, id, f)
    }

    fn tab_bar_with_options(
        &mut self,
        id: &str,
        options: TabBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        container_methods::tab_bar_with_options(self, None, id, options, f)
    }

    fn vertical(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        container_methods::vertical(self, None, f);
    }

    fn vertical_with_options(
        &mut self,
        options: VerticalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::vertical_with_options(self, None, options, f);
    }

    fn list_box(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::list_box(self, None, id, label, f);
    }

    fn list_box_with_options(
        &mut self,
        id: &str,
        options: ListBoxOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::list_box_with_options(self, None, id, options, f);
    }

    fn grid(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        container_methods::grid(self, None, f);
    }

    fn grid_with_options(
        &mut self,
        options: GridOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::grid_with_options(self, None, options, f);
    }

    fn table(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) -> TableResponse {
        container_methods::table(self, None, id, columns, f)
    }

    fn table_with_options(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        options: TableOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) -> TableResponse {
        container_methods::table_with_options(self, None, id, columns, options, f)
    }

    fn virtual_list<K, R>(&mut self, id: &str, len: usize, key_at: K, row: R) -> VirtualListResponse
    where
        K: FnMut(usize) -> fret_ui::ItemKey,
        R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
    {
        container_methods::virtual_list(self, None, id, len, key_at, row)
    }

    fn virtual_list_with_options<K, R>(
        &mut self,
        id: &str,
        len: usize,
        options: VirtualListOptions,
        key_at: K,
        row: R,
    ) -> VirtualListResponse
    where
        K: FnMut(usize) -> fret_ui::ItemKey,
        R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
    {
        container_methods::virtual_list_with_options(self, None, id, len, options, key_at, row)
    }

    fn scroll(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        container_methods::scroll(self, None, f);
    }

    fn scroll_with_options(
        &mut self,
        options: ScrollOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        container_methods::scroll_with_options(self, None, options, f);
    }

    fn child_region(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        container_methods::child_region(self, None, id, f)
    }

    fn child_region_with_options(
        &mut self,
        id: &str,
        options: ChildRegionOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        container_methods::child_region_with_options(self, None, id, options, f)
    }

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

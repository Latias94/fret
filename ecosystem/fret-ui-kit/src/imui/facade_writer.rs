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

    /// Render a window-scoped floating window layer that manages z-order (bring-to-front).
    ///
    /// Notes:
    /// - This is an opt-in container; a plain `floating_area(...)` / `window(...)` call
    ///   sequence keeps call-order z.
    /// - Call this late in the parent tree to ensure the layer paints above base content.
    fn floating_layer(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        floating_popup::floating_layer(self, id, f);
    }

    /// Render a minimal in-window floating area primitive.
    ///
    /// This is the lowest-level building block for ImGui-like floating surfaces in-window:
    ///
    /// - always in-window (not an OS window / viewport),
    /// - position is stored as element-local state under the area id scope,
    /// - movement is driven by a caller-provided drag surface (via `floating_area_drag_surface(...)`),
    /// - optional z-order activation when nested inside `floating_layer(...)`.
    ///
    /// Notes:
    /// - `id` must be stable across frames (mirrors Dear ImGui's "name is the id" rule).
    fn floating_area(
        &mut self,
        id: &str,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
    ) -> FloatingAreaResponse {
        self.floating_area_with_options(id, initial_position, FloatingAreaOptions::default(), f)
    }

    fn floating_area_with_options(
        &mut self,
        id: &str,
        initial_position: Point,
        options: FloatingAreaOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
    ) -> FloatingAreaResponse {
        floating_popup::floating_area_with_options(self, id, initial_position, options, f)
    }

    /// Build a drag surface that moves a floating area (ImGui-style).
    ///
    /// The returned element should be placed as part of the area content (e.g. a title bar).
    fn floating_area_drag_surface(
        &mut self,
        area: FloatingAreaContext,
        props: PointerRegionProps,
        setup: impl FnOnce(&mut ElementContext<'_, H>, GlobalElementId),
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> AnyElement {
        floating_popup::floating_area_drag_surface(self, area, props, setup, f)
    }

    /// Returns the internal open model for a named popup scope.
    ///
    /// This is intended to support ImGui-like `OpenPopup` / `BeginPopup` splits without forcing
    /// callers to allocate a dedicated `Model<bool>` per popup.
    fn popup_open_model(&mut self, id: &str) -> fret_runtime::Model<bool> {
        floating_popup::popup_open_model(self, id)
    }

    /// Drops all internal state for a named popup scope.
    ///
    /// This is primarily intended for ephemeral/dynamic scopes where the id space could grow
    /// without bound (e.g. popups keyed by user-generated strings). Dropping a scope will close the
    /// popup (if open) and release the internal models if no other references exist.
    fn drop_popup_scope(&mut self, id: &str) {
        floating_popup::drop_popup_scope(self, id);
    }

    fn open_popup(&mut self, id: &str) {
        floating_popup::open_popup(self, id);
    }

    fn open_popup_at(&mut self, id: &str, anchor: fret_core::Rect) {
        floating_popup::open_popup_at(self, id, anchor);
    }

    fn close_popup(&mut self, id: &str) {
        floating_popup::close_popup(self, id);
    }

    fn begin_popup_menu(
        &mut self,
        id: &str,
        trigger: Option<GlobalElementId>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        self.begin_popup_menu_with_options(id, trigger, PopupMenuOptions::default(), f)
    }

    fn begin_popup_menu_with_options(
        &mut self,
        id: &str,
        trigger: Option<GlobalElementId>,
        options: PopupMenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        floating_popup::begin_popup_menu_with_options(self, id, trigger, options, f)
    }

    fn begin_popup_modal(
        &mut self,
        id: &str,
        trigger: Option<GlobalElementId>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        self.begin_popup_modal_with_options(id, trigger, PopupModalOptions::default(), f)
    }

    fn begin_popup_modal_with_options(
        &mut self,
        id: &str,
        trigger: Option<GlobalElementId>,
        options: PopupModalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        floating_popup::begin_popup_modal_with_options(self, id, trigger, options, f)
    }

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

    fn tooltip_text(&mut self, id: &str, trigger: ResponseExt, text: impl Into<Arc<str>>) -> bool {
        self.tooltip_text_with_options(id, trigger, text, TooltipOptions::default())
    }

    fn tooltip_text_with_options(
        &mut self,
        id: &str,
        trigger: ResponseExt,
        text: impl Into<Arc<str>>,
        options: TooltipOptions,
    ) -> bool {
        floating_popup::tooltip_text_with_options(self, id, trigger, text, options)
    }

    fn tooltip(
        &mut self,
        id: &str,
        trigger: ResponseExt,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        self.tooltip_with_options(id, trigger, TooltipOptions::default(), f)
    }

    fn tooltip_with_options(
        &mut self,
        id: &str,
        trigger: ResponseExt,
        options: TooltipOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        floating_popup::tooltip_with_options(self, id, trigger, options, f)
    }

    /// Publish a typed payload for the trigger's existing pressable drag gesture.
    ///
    /// Notes:
    /// - This follows Fret's response-driven authoring style instead of cloning Dear ImGui's
    ///   begin/end drag-drop grammar.
    /// - The payload is stored in a model-backed immediate store keyed by the active drag session,
    ///   because object-safe pointer action hooks do not create typed `DragSession` payloads
    ///   directly.
    fn drag_source<T: std::any::Any>(
        &mut self,
        trigger: ResponseExt,
        payload: T,
    ) -> DragSourceResponse {
        self.drag_source_with_options(trigger, payload, DragSourceOptions::default())
    }

    fn drag_source_with_options<T: std::any::Any>(
        &mut self,
        trigger: ResponseExt,
        payload: T,
        options: DragSourceOptions,
    ) -> DragSourceResponse {
        floating_popup::drag_source_with_options(self, trigger, payload, options)
    }

    /// Resolve a typed drop target against the trigger's existing pressable surface.
    ///
    /// Preview state is reported while a compatible payload hovers the target. Delivery is
    /// reported exactly once on the next render after pointer release over the target.
    fn drop_target<T: std::any::Any>(&mut self, trigger: ResponseExt) -> DropTargetResponse<T> {
        self.drop_target_with_options(trigger, DropTargetOptions::default())
    }

    fn drop_target_with_options<T: std::any::Any>(
        &mut self,
        trigger: ResponseExt,
        options: DropTargetOptions,
    ) -> DropTargetResponse<T> {
        floating_popup::drop_target_with_options(self, trigger, options)
    }

    menu_selection_surface::menu_selection_surface_methods!();

    button_surface::button_surface_methods!();

    model_surface::model_surface_methods!();

    /// Render an in-window floating window.
    ///
    /// Scope:
    /// - in-window (not an OS window / viewport),
    /// - draggable via the title bar,
    /// - position is stored as element-local state under the window id scope,
    /// - `floating_layer(...)` owns bring-to-front ordering and hit-test order,
    /// - `WindowOptions` / `FloatingWindowOptions` own close, resize, collapse, focus-on-click,
    ///   activate-on-click, and no-inputs / pointer-pass-through policy.
    ///
    /// Notes:
    /// - `id` must be stable across frames (mirrors Dear ImGui's "window name is the id" rule).
    /// - OS-window tear-out and multi-viewport behavior are docking/runner concerns, not this
    ///   in-window helper.
    fn window(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        floating_popup::window(self, id, title, initial_position, f)
    }

    /// Render a floating window with explicit state and behavior options.
    fn window_with_options(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        options: WindowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        floating_popup::window_with_options(self, id, title, initial_position, options, f)
    }
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterImUiFacadeExt<H> for W {}

#[cfg(test)]
mod tests;

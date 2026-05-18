//! Immediate-mode facade writer glue.

use super::*;
use std::any::Any;

mod boolean_wrappers;
mod button_actions;
mod container_wrappers;
mod disclosure;
mod floating_popup;
mod menu_items;
mod selection_combo;
mod text_models;
mod value_models;

/// A minimal `UiWriter` implementation used by facade container helpers (e.g. floating windows).
///
/// This mirrors the `fret-imui::ImUi` pattern without depending on the `fret-imui` crate.
pub struct ImUiFacade<'cx, 'a, H: UiHost> {
    pub(super) cx: &'cx mut ElementContext<'a, H>,
    pub(super) out: &'cx mut Vec<AnyElement>,
    pub(super) build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
}

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    fn record_focusable(&mut self, id: Option<GlobalElementId>, enabled: bool) {
        if !enabled {
            return;
        }
        let Some(id) = id else {
            return;
        };
        let Some(st) = self.build_focus.as_ref() else {
            return;
        };
        if st.get().is_none() {
            st.set(Some(id));
        }
    }

    pub fn cx_mut(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
    }

    pub fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }

    pub fn id<K: Hash>(
        &mut self,
        key: K,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let out = &mut *self.out;
        let build_focus = self.build_focus.clone();
        self.cx.keyed(key, |cx| {
            prepare_imui_runtime_for_frame(cx);
            let mut ui = ImUiFacade {
                cx,
                out,
                build_focus,
            };
            f(&mut ui);
        });
    }

    pub fn push_id<K: Hash>(
        &mut self,
        key: K,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.id(key, f);
    }

    pub fn for_each_keyed<I, K, T>(
        &mut self,
        items: I,
        mut f: impl FnMut(&mut ImUiFacade<'_, '_, H>, &K, T),
    ) where
        I: IntoIterator<Item = (K, T)>,
        K: Hash,
    {
        let f = &mut f;
        for (key, item) in items {
            self.id(&key, |ui| f(ui, &key, item));
        }
    }

    /// Disable all `imui`-facade interactions within the closure and dim visuals (ImGui-style
    /// `BeginDisabled/EndDisabled`).
    ///
    /// Notes:
    /// - This is scoped to the closure (Rust-friendly) rather than a manual begin/end pair.
    /// - The disabled alpha multiplier is controlled by theme number
    ///   `component.imui.disabled_alpha` (default `0.60`).
    pub fn disabled_scope(
        &mut self,
        disabled: bool,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        if !disabled {
            f(self);
            return;
        }

        let was_disabled = self.with_cx_mut(|cx| imui_is_disabled(cx));
        if was_disabled {
            f(self);
            return;
        }

        let build_focus = self.build_focus.clone();
        let element = self.with_cx_mut(|cx| {
            let depth = disabled_scope_depth_for(cx);
            let _guard = DisabledScopeGuard::push(depth);
            let alpha = disabled_alpha_for(cx);
            cx.pointer_region(PointerRegionProps::default(), |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _acx, _down| true));
                cx.pointer_region_on_pointer_up(Arc::new(|_host, _acx, _up| true));
                vec![cx.opacity(alpha, |cx| {
                    vec![cx.focus_traversal_gate(false, |cx| {
                        prepare_imui_runtime_for_frame(cx);
                        let mut out = Vec::new();
                        let mut ui = ImUiFacade {
                            cx,
                            out: &mut out,
                            build_focus,
                        };
                        f(&mut ui);
                        out
                    })]
                })]
            })
        });
        self.add(element);
    }
}

impl<'cx, 'a, H: UiHost> UiWriter<H> for ImUiFacade<'cx, 'a, H> {
    fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
        f(self.cx)
    }

    fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }
}

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
        let mut result = None;
        let elements = self.with_cx_mut(|cx| {
            cx.keyed(key, |cx| {
                prepare_imui_runtime_for_frame(cx);
                let mut out = Vec::new();
                let mut ui = ImUiFacade {
                    cx,
                    out: &mut out,
                    build_focus: None,
                };
                result = Some(f(&mut ui));
                out
            })
        });
        self.extend(elements);
        result.expect("imui push_id closure should produce a result")
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
        if !disabled {
            let elements = self.with_cx_mut(|cx| {
                prepare_imui_runtime_for_frame(cx);
                let mut out = Vec::new();
                let mut ui = ImUiFacade {
                    cx,
                    out: &mut out,
                    build_focus: None,
                };
                f(&mut ui);
                out
            });
            self.extend(elements);
            return;
        }

        enum Built {
            Inline(Vec<AnyElement>),
            Wrapped(Box<AnyElement>),
        }

        let built = self.with_cx_mut(|cx| {
            let depth = disabled_scope_depth_for(cx);
            let was_disabled = depth.get() > 0;
            let _guard = DisabledScopeGuard::push(depth);

            let build_children = |cx: &mut ElementContext<'_, H>| {
                prepare_imui_runtime_for_frame(cx);
                let mut out = Vec::new();
                let mut ui = ImUiFacade {
                    cx,
                    out: &mut out,
                    build_focus: None,
                };
                f(&mut ui);
                out
            };

            if was_disabled {
                Built::Inline(build_children(cx))
            } else {
                let alpha = disabled_alpha_for(cx);
                Built::Wrapped(Box::new(cx.pointer_region(
                    PointerRegionProps::default(),
                    |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(|_host, _acx, _down| true));
                        cx.pointer_region_on_pointer_up(Arc::new(|_host, _acx, _up| true));
                        vec![cx.opacity(alpha, |cx| {
                            vec![cx.focus_traversal_gate(false, |cx| build_children(cx))]
                        })]
                    },
                )))
            }
        });

        match built {
            Built::Inline(elements) => self.extend(elements),
            Built::Wrapped(element) => self.add(*element),
        }
    }

    fn text(&mut self, text: impl Into<Arc<str>>) {
        let text = text.into();
        let element =
            self.with_cx_mut(|cx| crate::declarative::text::text_section_chrome_label(cx, text));
        self.add(element);
    }

    fn text_wrapped(&mut self, text: impl Into<Arc<str>>) {
        let text = text.into();
        let element =
            self.with_cx_mut(|cx| crate::declarative::text::text_compact_paragraph(cx, text));
        self.add(element);
    }

    fn bullet_text(&mut self, text: impl Into<Arc<str>>) {
        self.bullet_text_with_options(text, BulletTextOptions::default());
    }

    fn bullet_text_with_options(&mut self, text: impl Into<Arc<str>>, options: BulletTextOptions) {
        bullet_text_controls::bullet_text_with_options(self, text.into(), options);
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
        debug_draw_controls::debug_draw_with_options(self, id, options, draw)
    }

    fn separator(&mut self) {
        let element = self.with_cx_mut(|cx| {
            let mut props = fret_ui::element::ContainerProps::default();
            let theme = fret_ui::Theme::global(&*cx.app);
            props.background = Some(theme.color_token("border"));
            props.layout.size.width = fret_ui::element::Length::Fill;
            props.layout.size.height = fret_ui::element::Length::Px(fret_core::Px(1.0));
            cx.container(props, |_| Vec::new())
        });
        self.add(element);
    }

    fn separator_text(&mut self, label: impl Into<Arc<str>>) {
        self.separator_text_with_options(label, SeparatorTextOptions::default());
    }

    fn separator_text_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: SeparatorTextOptions,
    ) {
        separator_text_controls::separator_text_with_options(self, label.into(), options);
    }

    fn horizontal(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.horizontal_with_options(HorizontalOptions::default(), f);
    }

    fn horizontal_with_options(
        &mut self,
        options: HorizontalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element = self.with_cx_mut(|cx| horizontal_container_element(cx, None, options, f));
        self.add(element);
    }

    fn menu_bar(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.menu_bar_with_options(MenuBarOptions::default(), f);
    }

    fn menu_bar_with_options(
        &mut self,
        options: MenuBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element =
            self.with_cx_mut(|cx| menu_family_controls::menu_bar_element(cx, None, options, f));
        self.add(element);
    }

    fn tab_bar(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        self.tab_bar_with_options(id, TabBarOptions::default(), f)
    }

    fn tab_bar_with_options(
        &mut self,
        id: &str,
        options: TabBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        let (element, response) =
            self.with_cx_mut(|cx| tab_family_controls::tab_bar_element(cx, id, None, options, f));
        self.add(element);
        response
    }

    fn vertical(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.vertical_with_options(VerticalOptions::default(), f);
    }

    fn vertical_with_options(
        &mut self,
        options: VerticalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element = self.with_cx_mut(|cx| vertical_container_element(cx, None, options, f));
        self.add(element);
    }

    fn grid(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.grid_with_options(GridOptions::default(), f);
    }

    fn grid_with_options(
        &mut self,
        options: GridOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element = self.with_cx_mut(|cx| grid_container_element(cx, None, options, f));
        self.add(element);
    }

    fn table(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) -> TableResponse {
        self.table_with_options(id, columns, TableOptions::default(), f)
    }

    fn table_with_options(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        options: TableOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) -> TableResponse {
        let (element, response) =
            self.with_cx_mut(|cx| table_controls::table_element(cx, id, columns, None, options, f));
        self.add(element);
        response
    }

    fn virtual_list<K, R>(&mut self, id: &str, len: usize, key_at: K, row: R) -> VirtualListResponse
    where
        K: FnMut(usize) -> fret_ui::ItemKey,
        R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
    {
        self.virtual_list_with_options(id, len, VirtualListOptions::default(), key_at, row)
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
        let (element, response) = self.with_cx_mut(|cx| {
            virtual_list_controls::virtual_list_element(cx, id, len, None, options, key_at, row)
        });
        self.add(element);
        response
    }

    fn scroll(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.scroll_with_options(ScrollOptions::default(), f);
    }

    fn scroll_with_options(
        &mut self,
        options: ScrollOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element = self.with_cx_mut(|cx| scroll_container_element(cx, None, options, f));
        self.add(element);
    }

    fn child_region(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        self.child_region_with_options(id, ChildRegionOptions::default(), f)
    }

    fn child_region_with_options(
        &mut self,
        id: &str,
        options: ChildRegionOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        let (element, response) =
            self.with_cx_mut(|cx| child_region::child_region_element(cx, id, None, options, f));
        self.add(element);
        response
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

    fn menu_separator(&mut self) {
        self.separator();
    }

    fn menu_item(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        self.menu_item_with_options(label, MenuItemOptions::default())
    }

    fn menu_item_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        menu_controls::menu_item_with_options(self, label.into(), options)
    }

    fn menu_item_checkbox_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        menu_controls::menu_item_checkbox_with_options(self, label.into(), checked, options)
    }

    fn menu_item_radio_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        menu_controls::menu_item_radio_with_options(self, label.into(), checked, options)
    }

    fn menu_item_action(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
    ) -> ResponseExt {
        self.menu_item_action_with_options(label, action, MenuItemOptions::default())
    }

    fn menu_item_action_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        menu_controls::menu_item_action_with_options(self, label.into(), action.into(), options)
    }

    fn menu_item_command(&mut self, command: impl Into<CommandId>) -> ResponseExt {
        self.menu_item_command_with_options(command, MenuItemOptions::default())
    }

    fn menu_item_command_with_options(
        &mut self,
        command: impl Into<CommandId>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let command = command.into();
        let presentation =
            self.with_cx_mut(|cx| crate::command::command_presentation_for_window(cx, &command));

        let mut options = options;
        options.enabled = options.enabled && presentation.enabled;
        if options.shortcut.is_none() {
            options.shortcut = presentation.shortcut;
        }

        menu_controls::menu_item_action_with_options(self, presentation.label, command, options)
    }

    fn begin_menu(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.begin_menu_with_options(id, label, BeginMenuOptions::default(), f)
    }

    fn begin_menu_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: BeginMenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        menu_family_controls::begin_menu_with_options(self, id, label.into(), options, f)
    }

    fn begin_submenu(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.begin_submenu_with_options(id, label, BeginSubmenuOptions::default(), f)
    }

    fn begin_submenu_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: BeginSubmenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        menu_family_controls::begin_submenu_with_options(self, id, label.into(), options, f)
    }

    fn selectable(&mut self, label: impl Into<Arc<str>>, selected: bool) -> ResponseExt {
        self.selectable_with_options(
            label,
            SelectableOptions {
                selected,
                ..Default::default()
            },
        )
    }

    fn selectable_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: SelectableOptions,
    ) -> ResponseExt {
        selectable_controls::selectable_with_options(self, label.into(), options)
    }

    fn multi_selectable<K: Clone + PartialEq + 'static>(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<ImUiMultiSelectState<K>>,
        all_keys: &[K],
        key: K,
    ) -> ResponseExt {
        self.multi_selectable_with_options(
            label,
            model,
            all_keys,
            key,
            SelectableOptions::default(),
        )
    }

    fn multi_selectable_with_options<K: Clone + PartialEq + 'static>(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<ImUiMultiSelectState<K>>,
        all_keys: &[K],
        key: K,
        options: SelectableOptions,
    ) -> ResponseExt {
        multi_select::multi_selectable_with_options(
            self,
            label.into(),
            model,
            all_keys,
            key,
            options,
        )
    }

    fn combo(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        preview: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ComboResponse {
        self.combo_with_options(id, label, preview, ComboOptions::default(), f)
    }

    fn combo_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        preview: impl Into<Arc<str>>,
        options: ComboOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ComboResponse {
        combo_controls::combo_with_options(self, id, label.into(), preview.into(), options, f)
    }

    fn begin_popup_context_menu(
        &mut self,
        id: &str,
        trigger: ResponseExt,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        self.begin_popup_context_menu_with_options(id, trigger, PopupMenuOptions::default(), f)
    }

    fn begin_popup_context_menu_with_options(
        &mut self,
        id: &str,
        trigger: ResponseExt,
        options: PopupMenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        floating_popup::begin_popup_context_menu_with_options(self, id, trigger, options, f)
    }

    fn button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        self.button_with_options(label, ButtonOptions::default())
    }

    fn small_button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        self.small_button_with_options(label, ButtonOptions::default())
    }

    fn small_button_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: ButtonOptions,
    ) -> ResponseExt {
        button_controls::small_button_with_options(self, label.into(), options)
    }

    fn arrow_button(&mut self, id: &str, direction: ButtonArrowDirection) -> ResponseExt {
        self.arrow_button_with_options(id, direction, ButtonOptions::default())
    }

    fn arrow_button_with_options(
        &mut self,
        id: &str,
        direction: ButtonArrowDirection,
        options: ButtonOptions,
    ) -> ResponseExt {
        button_controls::arrow_button_with_options(self, id, direction, options)
    }

    fn invisible_button(&mut self, id: &str, size: Size) -> ResponseExt {
        self.invisible_button_with_options(id, size, ButtonOptions::default())
    }

    fn invisible_button_with_options(
        &mut self,
        id: &str,
        size: Size,
        options: ButtonOptions,
    ) -> ResponseExt {
        button_controls::invisible_button_with_options(self, id, size, options)
    }

    fn image_item(&mut self, id: &str, image: fret_core::ImageId, size: Size) -> ResponseExt {
        self.image_item_with_options(id, image, size, ImageItemOptions::default())
    }

    fn image_item_with_options(
        &mut self,
        id: &str,
        image: fret_core::ImageId,
        size: Size,
        options: ImageItemOptions,
    ) -> ResponseExt {
        image_item_controls::image_item_with_options(self, id, image, size, options)
    }

    fn image_button(&mut self, id: &str, image: fret_core::ImageId, size: Size) -> ResponseExt {
        self.image_button_with_options(id, image, size, ImageItemOptions::button())
    }

    fn image_button_with_options(
        &mut self,
        id: &str,
        image: fret_core::ImageId,
        size: Size,
        mut options: ImageItemOptions,
    ) -> ResponseExt {
        let was_plain_image_options = matches!(options.variant, ImageItemVariant::Image);
        options.variant = ImageItemVariant::Button;
        if was_plain_image_options {
            options.focusable = true;
        }
        image_item_controls::image_item_with_options(self, id, image, size, options)
    }

    fn button_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: ButtonOptions,
    ) -> ResponseExt {
        button_controls::button_with_options(self, label.into(), options)
    }

    fn action_button(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
    ) -> ResponseExt {
        self.action_button_with_options(label, action, ButtonOptions::default())
    }

    fn action_button_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        options: ButtonOptions,
    ) -> ResponseExt {
        button_controls::action_button_with_options(self, label.into(), action.into(), options)
    }

    fn action_payload_button<T>(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        payload: T,
    ) -> ResponseExt
    where
        T: Any + Clone + Send + Sync + 'static,
    {
        self.action_payload_button_with_options(label, action, payload, ButtonOptions::default())
    }

    fn action_payload_button_with_options<T>(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        payload: T,
        options: ButtonOptions,
    ) -> ResponseExt
    where
        T: Any + Clone + Send + Sync + 'static,
    {
        button_controls::action_payload_button_with_options(
            self,
            label.into(),
            action.into(),
            payload,
            options,
        )
    }

    fn button_command(&mut self, command: impl Into<CommandId>) -> ResponseExt {
        self.button_command_with_options(command, ButtonOptions::default())
    }

    fn button_command_with_options(
        &mut self,
        command: impl Into<CommandId>,
        options: ButtonOptions,
    ) -> ResponseExt {
        let command = command.into();
        let presentation =
            self.with_cx_mut(|cx| crate::command::command_presentation_for_window(cx, &command));

        let mut options = options;
        options.enabled = options.enabled && presentation.enabled;

        button_controls::action_button_with_options(self, presentation.label, command, options)
    }

    fn checkbox_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        boolean_controls::checkbox_model(self, label.into(), model)
    }

    fn checkbox_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: CheckboxOptions,
    ) -> ResponseExt {
        boolean_controls::checkbox_model_with_options(self, label.into(), model, options)
    }

    fn radio(&mut self, label: impl Into<Arc<str>>, selected: bool) -> ResponseExt {
        self.radio_with_options(label, selected, RadioOptions::default())
    }

    fn radio_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        selected: bool,
        options: RadioOptions,
    ) -> ResponseExt {
        boolean_controls::radio_with_options(self, label.into(), selected, options)
    }

    fn switch_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.switch_model_with_options(label, model, SwitchOptions::default())
    }

    fn switch_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: SwitchOptions,
    ) -> ResponseExt {
        boolean_controls::switch_model_with_options(self, label.into(), model, options)
    }

    fn slider_f32_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
    ) -> ResponseExt {
        self.slider_f32_model_with_options(label, model, SliderOptions::default())
    }

    fn slider_f32_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
        options: SliderOptions,
    ) -> ResponseExt {
        slider_controls::slider_f32_model_with_options(self, label.into(), model, options)
    }

    fn combo_model(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
    ) -> ResponseExt {
        self.combo_model_with_options(id, label, model, items, ComboModelOptions::default())
    }

    fn combo_model_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
        options: ComboModelOptions,
    ) -> ResponseExt {
        combo_model_controls::combo_model_with_options(
            self,
            id,
            label.into(),
            model,
            items,
            options,
        )
    }

    fn input_text_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.input_text_model_with_options(model, InputTextOptions::default())
    }

    fn input_text_model_with_options(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: InputTextOptions,
    ) -> ResponseExt {
        text_controls::input_text_model_with_options(self, model, options)
    }

    fn input_text_completion_model(
        &mut self,
        id: &str,
        model: &fret_runtime::Model<String>,
        candidates: &[Arc<str>],
    ) -> InputTextPickerResponse {
        self.input_text_completion_model_with_options(
            id,
            model,
            candidates,
            InputTextPickerOptions::default(),
        )
    }

    fn input_text_completion_model_with_options(
        &mut self,
        id: &str,
        model: &fret_runtime::Model<String>,
        candidates: &[Arc<str>],
        options: InputTextPickerOptions,
    ) -> InputTextPickerResponse {
        text_picker_controls::input_text_completion_model_with_options(
            self, id, model, candidates, options,
        )
    }

    fn input_text_history_model(
        &mut self,
        id: &str,
        model: &fret_runtime::Model<String>,
        history: &[Arc<str>],
    ) -> InputTextPickerResponse {
        self.input_text_history_model_with_options(
            id,
            model,
            history,
            InputTextPickerOptions::default(),
        )
    }

    fn input_text_history_model_with_options(
        &mut self,
        id: &str,
        model: &fret_runtime::Model<String>,
        history: &[Arc<str>],
        options: InputTextPickerOptions,
    ) -> InputTextPickerResponse {
        text_picker_controls::input_text_history_model_with_options(
            self, id, model, history, options,
        )
    }

    fn textarea_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.textarea_model_with_options(model, TextAreaOptions::default())
    }

    fn textarea_model_with_options(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: TextAreaOptions,
    ) -> ResponseExt {
        text_controls::textarea_model_with_options(self, model, options)
    }

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
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Px, Rect, TextOverflow, TextWrap};
    use fret_ui::element::{ElementKind, Length};
    use fret_ui::elements;

    struct TestWriter<'cx, 'a, H: UiHost> {
        cx: &'cx mut ElementContext<'a, H>,
        out: &'cx mut Vec<AnyElement>,
    }

    impl<'cx, 'a, H: UiHost> UiWriter<H> for TestWriter<'cx, 'a, H> {
        fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
            f(self.cx)
        }

        fn add(&mut self, element: AnyElement) {
            self.out.push(element);
        }
    }

    #[test]
    fn imui_text_item_is_single_line_and_shrinkable() {
        let mut app = App::new();

        elements::with_element_cx(
            &mut app,
            AppWindowId::default(),
            Rect::default(),
            "imui-text-item",
            |cx| {
                let mut out = Vec::new();
                let mut ui = TestWriter { cx, out: &mut out };

                ui.text("Long editor status text that should not wrap inside a dense row");

                assert_eq!(out.len(), 1);
                let ElementKind::Text(props) = &out[0].kind else {
                    panic!("expected imui text item to produce a Text element");
                };

                assert_eq!(props.layout.flex.shrink, 1.0);
                assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
                assert_eq!(props.wrap, TextWrap::None);
                assert_eq!(props.overflow, TextOverflow::Ellipsis);
                assert!(out[0].inherited_text_style.is_some());
            },
        );
    }

    #[test]
    fn imui_text_wrapped_is_explicit_wrapping_text() {
        let mut app = App::new();

        elements::with_element_cx(
            &mut app,
            AppWindowId::default(),
            Rect::default(),
            "imui-text-wrapped",
            |cx| {
                let mut out = Vec::new();
                let mut ui = TestWriter { cx, out: &mut out };

                ui.text_wrapped("Long explanatory text can opt into wrapping explicitly");

                assert_eq!(out.len(), 1);
                let ElementKind::Text(props) = &out[0].kind else {
                    panic!("expected imui wrapped text item to produce a Text element");
                };

                assert_eq!(props.layout.size.width, Length::Fill);
                assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
                assert_eq!(props.layout.flex.grow, 1.0);
                assert_eq!(props.layout.flex.shrink, 1.0);
                assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
                assert_eq!(props.wrap, TextWrap::Word);
                assert_eq!(props.overflow, TextOverflow::Clip);
                assert!(out[0].inherited_text_style.is_some());
            },
        );
    }
}

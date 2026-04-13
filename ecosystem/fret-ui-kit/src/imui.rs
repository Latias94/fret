//! Optional integration helpers for immediate-mode authoring frontends.
//!
//! This module lives in `fret-ui-kit` (not `fret-imui`) to preserve dependency direction:
//!
//! - `fret-imui` stays policy-light and depends only on `fret-ui` (+ `fret-authoring` contract).
//! - `fret-ui-kit` can optionally provide bridges that allow `UiBuilder<T>` patch vocabulary to be
//!   used from immediate-style control flow.

#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::too_many_arguments)]

use std::cell::Cell;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use fret_authoring::UiWriter;
use fret_core::{Point, Px, Rect, Size};
use fret_interaction::dpi;
use fret_runtime::{ActionId, CommandId};
use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::IntoUiElement;

pub mod adapters;
mod boolean_controls;
mod button_controls;
mod child_region;
mod combo_controls;
mod combo_model_controls;
mod containers;
mod disclosure_controls;
mod drag_drop;
mod floating_surface;
mod floating_window;
mod floating_window_on_area;
mod interaction_runtime;
mod menu_controls;
mod menu_family_controls;
mod multi_select;
mod options;
mod popup_overlay;
mod popup_store;
mod response;
mod selectable_controls;
mod separator_text_controls;
mod slider_controls;
mod tab_family_controls;
mod table_controls;
mod text_controls;
mod tooltip_overlay;
mod virtual_list_controls;

use containers::{
    grid_container_element, horizontal_container_element, scroll_container_element,
    vertical_container_element,
};
use floating_surface::{
    FloatWindowResizeHandle, FloatWindowState, FloatingAreaState, FloatingWindowChromeResponse,
    KEY_FLOAT_WINDOW_ACTIVATE, KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED, OnFloatingAreaLeftDoubleClick,
    float_layer_bring_to_front_if_activated, float_window_drag_kind_for_element,
    float_window_resize_kind_for_element, floating_area_drag_surface_element,
};
use floating_surface::{floating_area_element, floating_layer_element};
pub use fret_ui::element::{VirtualListKeyCacheMode, VirtualListMeasureMode};
pub use fret_ui::scroll::VirtualListScrollHandle;
use interaction_runtime::{
    DisabledScopeGuard, active_item_model_for_window, clear_active_item_on_left_pointer_up,
    context_menu_anchor_model_for, disabled_alpha_for, disabled_scope_depth_for,
    drag_kind_for_element, drag_threshold_for, finish_pointer_region_drag,
    finish_pressable_drag_on_pointer_up, float_window_collapsed_model_for,
    handle_pointer_region_drag_move_with_threshold, handle_pressable_drag_move_with_threshold,
    hover_blocked_by_active_item_for, imui_is_disabled, install_hover_query_hooks_for_pressable,
    lifecycle_session_model_for, long_press_signal_model_for,
    mark_active_item_on_left_pointer_down, mark_lifecycle_activated_on_left_pointer_down,
    mark_lifecycle_deactivated_on_left_pointer_up, mark_lifecycle_edit,
    mark_lifecycle_instant_if_inactive, pointer_click_modifiers_model_for,
    populate_pressable_drag_response, populate_response_lifecycle_from_active_state,
    populate_response_lifecycle_transients, prepare_pointer_region_drag_on_left_down,
    prepare_pressable_drag_on_pointer_down, sanitize_response_for_enabled,
};
pub use multi_select::{ImUiMultiSelectState, multi_select_use_model};
pub use options::{
    BeginMenuOptions, BeginSubmenuOptions, ButtonOptions, CheckboxOptions, ChildRegionOptions,
    CollapsingHeaderOptions, ComboModelOptions, ComboOptions, DragSourceOptions, DropTargetOptions,
    GridOptions, HorizontalOptions, InputTextOptions, MenuBarOptions, MenuItemOptions,
    PopupMenuOptions, PopupModalOptions, ScrollOptions, SelectableOptions, SeparatorTextOptions,
    SliderOptions, SwitchOptions, TabBarOptions, TabItemOptions, TableColumn, TableColumnWidth,
    TableOptions, TableRowOptions, TextAreaOptions, TooltipOptions, TreeNodeOptions,
    VerticalOptions, VirtualListOptions,
};
use popup_store::{drop_popup_scope_for_id, with_popup_store_for_id};
pub use response::{
    ComboResponse, DisclosureResponse, DragResponse, DragSourceResponse, DropTargetResponse,
    FloatingAreaResponse, FloatingWindowResponse, ImUiHoveredFlags, ResponseExt, TabBarResponse,
    TabTriggerResponse, VirtualListResponse,
};
pub use tab_family_controls::ImUiTabBar;
pub use table_controls::{ImUiTable, ImUiTableRow};

/// Extension trait bridging `fret-ui-kit` authoring (`UiBuilder<T>`) into an immediate-mode output.
pub trait UiWriterUiKitExt<H: UiHost>: UiWriter<H> {
    /// Render a `UiBuilder<T>` (or other supported authoring value) into the current output list.
    #[track_caller]
    fn add_ui<B>(&mut self, value: B)
    where
        B: IntoUiElement<H>,
    {
        let element = self.with_cx_mut(|cx| IntoUiElement::into_element(value, cx));
        self.add(element);
    }
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterUiKitExt<H> for W {}

const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0usize;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
        i += 1;
    }
    hash
}

const KEY_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.clicked.v1");
const KEY_CHANGED: u64 = fnv1a64(b"fret-ui-kit.imui.changed.v1");
const KEY_SECONDARY_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.secondary_clicked.v1");
const KEY_DOUBLE_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.double_clicked.v1");
const KEY_LONG_PRESSED: u64 = fnv1a64(b"fret-ui-kit.imui.long_pressed.v1");
const KEY_CONTEXT_MENU_REQUESTED: u64 = fnv1a64(b"fret-ui-kit.imui.context_menu_requested.v1");
const KEY_POINTER_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.pointer_clicked.v1");
const KEY_DRAG_STARTED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_started.v1");
const KEY_DRAG_STOPPED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_stopped.v1");
const KEY_ACTIVATED: u64 = fnv1a64(b"fret-ui-kit.imui.activated.v1");
const KEY_DEACTIVATED: u64 = fnv1a64(b"fret-ui-kit.imui.deactivated.v1");
const KEY_DEACTIVATED_AFTER_EDIT: u64 = fnv1a64(b"fret-ui-kit.imui.deactivated_after_edit.v1");
const KEY_HOVER_STATIONARY_MET: u64 = fnv1a64(b"fret-ui-kit.imui.hover.stationary_met.v1");
const KEY_HOVER_DELAY_SHORT_MET: u64 = fnv1a64(b"fret-ui-kit.imui.hover.delay_short_met.v1");
const KEY_HOVER_DELAY_NORMAL_MET: u64 = fnv1a64(b"fret-ui-kit.imui.hover.delay_normal_met.v1");

// ImGui default: `MouseDragThreshold = 6`.
const DEFAULT_DRAG_THRESHOLD_PX: f32 = 6.0;
// ImGui default: `ImGuiStyle::DisabledAlpha = 0.60f`.
const DEFAULT_DISABLED_ALPHA: f32 = 0.60;
const LONG_PRESS_DELAY: Duration = Duration::from_millis(450);
// ImGui defaults:
// - `HoverStationaryDelay ~= 0.15 sec`
// - `HoverDelayShort ~= 0.15 sec`
// - `HoverDelayNormal ~= 0.40 sec`
const HOVER_STATIONARY_DELAY: Duration = Duration::from_millis(150);
const HOVER_DELAY_SHORT: Duration = Duration::from_millis(150);
const HOVER_DELAY_NORMAL: Duration = Duration::from_millis(400);
const DRAG_KIND_MASK: u64 = 0x8000_0000_0000_0000;

pub(super) fn snap_point_to_device_pixels(scale_factor: f32, p: Point) -> Point {
    dpi::snap_point_to_device_pixels(scale_factor, p)
}

pub(super) fn snap_size_to_device_pixels(scale_factor: f32, s: Size) -> Size {
    dpi::snap_size_to_device_pixels(scale_factor, s)
}

fn point_sub(a: Point, b: Point) -> Point {
    Point::new(Px(a.x.0 - b.x.0), Px(a.y.0 - b.y.0))
}

fn point_add(a: Point, b: Point) -> Point {
    Point::new(Px(a.x.0 + b.x.0), Px(a.y.0 + b.y.0))
}

fn model_value_changed_for<H: UiHost, T>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    current: T,
) -> bool
where
    T: Clone + PartialEq + 'static,
{
    cx.state_for(
        id,
        || current.clone(),
        |previous| {
            let changed = previous != &current;
            if changed {
                *previous = current.clone();
            }
            changed
        },
    )
}

fn slider_step_or_default(step: f32) -> f32 {
    if step.is_finite() && step > 0.0 {
        step
    } else {
        1.0
    }
}

fn slider_normalize_range(min: f32, max: f32) -> (f32, f32) {
    if min <= max { (min, max) } else { (max, min) }
}

fn slider_clamp_and_snap(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let (min, max) = slider_normalize_range(min, max);
    if !value.is_finite() {
        return min;
    }
    if (max - min).abs() <= f32::EPSILON {
        return min;
    }
    let step = slider_step_or_default(step);
    let snapped = min + ((value - min) / step).round() * step;
    snapped.clamp(min, max)
}

fn slider_value_from_pointer(bounds: Rect, pointer: Point, min: f32, max: f32, step: f32) -> f32 {
    let (min, max) = slider_normalize_range(min, max);
    if (max - min).abs() <= f32::EPSILON {
        return min;
    }

    let width = bounds.size.width.0.max(1.0);
    let t = ((pointer.x.0 - bounds.origin.x.0) / width).clamp(0.0, 1.0);
    let raw = min + (max - min) * t;
    slider_clamp_and_snap(raw, min, max, step)
}

#[derive(Debug, Clone, Copy)]
pub struct FloatingWindowResizeOptions {
    pub min_size: Size,
    pub max_size: Option<Size>,
}

impl Default for FloatingWindowResizeOptions {
    fn default() -> Self {
        Self {
            min_size: Size::new(Px(120.0), Px(72.0)),
            max_size: None,
        }
    }
}

/// Behavior flags for in-window floating windows.
///
/// This is an ecosystem-level facade surface (not a mechanism-layer contract). The goal is to
/// provide ImGui-like control over common floating window behavior without introducing a parallel
/// runtime or duplicating canonical policy.
#[derive(Debug, Clone, Copy)]
pub struct FloatingWindowOptions {
    /// When true, the window can be moved by dragging the title bar.
    pub movable: bool,
    /// When true, resize handles are active when the window is rendered with an initial size.
    pub resizable: bool,
    /// When true, title-bar double click toggles collapse/expand.
    pub collapsible: bool,
    /// When true and an `open` model is provided, the close button and `Escape`-to-close are enabled.
    pub closable: bool,
    /// When true, pointer down inside the window requests focus for the surface (even if
    /// activation is disabled).
    ///
    /// This is useful to model ImGui's `NoBringToFrontOnFocus` behavior: you may want a window to
    /// take focus when clicked without also being activated for z-order.
    pub focus_on_click: bool,
    /// When true, pointer down anywhere in the window activates it for z-order (when nested under
    /// `floating_layer(...)`).
    pub activate_on_click: bool,
    /// When false, the window is rendered but pointer interactions are blocked (no activation,
    /// drag, resize, or child clicks).
    pub inputs_enabled: bool,
    /// When true, the window is rendered but is inert for pointer and keyboard navigation:
    /// it does not participate in pointer hit-testing and is skipped by focus traversal.
    ///
    /// This is intended to model Dear ImGui's `NoInputs` window flag, which implies mouse
    /// pass-through and disables nav/focus participation.
    ///
    /// Note: `no_inputs=true` is different from `inputs_enabled=false`:
    /// - `inputs_enabled=false` blocks pointer hits (not click-through) but still participates
    ///   in focus traversal.
    /// - `no_inputs=true` is click-through and is skipped by focus traversal.
    pub no_inputs: bool,
    /// When true, the floating window is hit-test transparent (pointer events pass through to
    /// underlay content).
    ///
    /// This is intended to model Dear ImGui's "mouse pass-through" style behavior (`NoMouseInputs`
    /// for in-window floating surfaces. In Fret's current facade, this is pointer pass-through
    /// only: the subtree remains present for focus traversal / keyboard navigation.
    ///
    /// Note: `inputs_enabled=false` is *not* click-through; it is "non-interactive but blocks
    /// pointer hits". Use `pointer_passthrough=true` when you explicitly want click-through.
    pub pointer_passthrough: bool,
}

impl Default for FloatingWindowOptions {
    fn default() -> Self {
        Self {
            movable: true,
            resizable: true,
            collapsible: true,
            closable: true,
            focus_on_click: true,
            activate_on_click: true,
            inputs_enabled: true,
            no_inputs: false,
            pointer_passthrough: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WindowOptions {
    /// Optional `open` model controlling whether the window is rendered.
    ///
    /// When present, close actions update the model to `false`.
    pub open: Option<fret_runtime::Model<bool>>,
    /// Optional fixed initial size for the floating window.
    ///
    /// When absent, the window uses content-driven sizing and `resize` is ignored.
    pub size: Option<Size>,
    /// Optional resize policy for sized windows.
    ///
    /// This only takes effect when `size` is also set.
    pub resize: Option<FloatingWindowResizeOptions>,
    /// Behavior flags for the floating window surface.
    pub behavior: FloatingWindowOptions,
}

impl WindowOptions {
    pub fn with_open(mut self, open: &fret_runtime::Model<bool>) -> Self {
        self.open = Some(open.clone());
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_resize(mut self, resize: FloatingWindowResizeOptions) -> Self {
        self.resize = Some(resize);
        self
    }

    pub fn with_behavior(mut self, behavior: FloatingWindowOptions) -> Self {
        self.behavior = behavior;
        self
    }
}

#[derive(Debug, Clone)]
pub struct FloatingAreaOptions {
    /// A stable semantics test-id prefix used when `test_id` is not provided.
    ///
    /// The final test id is `{test_id_prefix}{id}`.
    pub test_id_prefix: &'static str,
    /// Explicitly overrides the semantics test-id for the floating area root element.
    pub test_id: Option<Arc<str>>,
    /// When true, the floating area root is hit-test transparent (pointer events pass through).
    ///
    /// This is a facade-level policy knob intended for click-through / pass-through floating
    /// surfaces. It wraps the area in a `HitTestGate` so the subtree does not intercept pointer
    /// input while still allowing focus traversal.
    pub hit_test_passthrough: bool,
    /// When true, the floating area is rendered but is inert for pointer and focus traversal:
    /// it is click-through and skipped by focus traversal.
    ///
    /// This wraps the area in an `InteractivityGate(present=true, interactive=false)` to model
    /// ImGui-style `NoInputs` behavior.
    ///
    /// Precedence: when `no_inputs == true`, `hit_test_passthrough` is ignored.
    pub no_inputs: bool,
}

impl Default for FloatingAreaOptions {
    fn default() -> Self {
        Self {
            test_id_prefix: "imui.float_area.area:",
            test_id: None,
            hit_test_passthrough: false,
            no_inputs: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FloatingAreaContext {
    pub id: GlobalElementId,
    pub position: Point,
    pub drag_kind: fret_runtime::DragKindId,
}

/// A minimal `UiWriter` implementation used by facade container helpers (e.g. floating windows).
///
/// This mirrors the `fret-imui::ImUi` pattern without depending on the `fret-imui` crate.
pub struct ImUiFacade<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    out: &'cx mut Vec<AnyElement>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
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

    pub fn horizontal(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.horizontal_with_options(HorizontalOptions::default(), f);
    }

    pub fn horizontal_with_options(
        &mut self,
        options: HorizontalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element =
            self.with_cx_mut(|cx| horizontal_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn menu_bar(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.menu_bar_with_options(MenuBarOptions::default(), f);
    }

    pub fn menu_bar_with_options(
        &mut self,
        options: MenuBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self
            .with_cx_mut(|cx| menu_family_controls::menu_bar_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn tab_bar(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        self.tab_bar_with_options(id, TabBarOptions::default(), f)
    }

    pub fn tab_bar_with_options(
        &mut self,
        id: &str,
        options: TabBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        let build_focus = self.build_focus.clone();
        let (element, response) = self.with_cx_mut(|cx| {
            tab_family_controls::tab_bar_element(cx, id, build_focus, options, f)
        });
        self.add(element);
        response
    }

    pub fn vertical(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.vertical_with_options(VerticalOptions::default(), f);
    }

    pub fn vertical_with_options(
        &mut self,
        options: VerticalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element =
            self.with_cx_mut(|cx| vertical_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn grid(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.grid_with_options(GridOptions::default(), f);
    }

    pub fn grid_with_options(
        &mut self,
        options: GridOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self.with_cx_mut(|cx| grid_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn table(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) {
        self.table_with_options(id, columns, TableOptions::default(), f);
    }

    pub fn table_with_options(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        options: TableOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self.with_cx_mut(|cx| {
            table_controls::table_element(cx, id, columns, build_focus, options, f)
        });
        self.add(element);
    }

    pub fn virtual_list<K, R>(
        &mut self,
        id: &str,
        len: usize,
        key_at: K,
        row: R,
    ) -> VirtualListResponse
    where
        K: FnMut(usize) -> fret_ui::ItemKey,
        R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
    {
        self.virtual_list_with_options(id, len, VirtualListOptions::default(), key_at, row)
    }

    pub fn virtual_list_with_options<K, R>(
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
        let build_focus = self.build_focus.clone();
        let (element, response) = self.with_cx_mut(|cx| {
            virtual_list_controls::virtual_list_element(
                cx,
                id,
                len,
                build_focus,
                options,
                key_at,
                row,
            )
        });
        self.add(element);
        response
    }

    pub fn scroll(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.scroll_with_options(ScrollOptions::default(), f);
    }

    pub fn scroll_with_options(
        &mut self,
        options: ScrollOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self.with_cx_mut(|cx| scroll_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn child_region(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.child_region_with_options(id, ChildRegionOptions::default(), f);
    }

    pub fn child_region_with_options(
        &mut self,
        id: &str,
        options: ChildRegionOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self
            .with_cx_mut(|cx| child_region::child_region_element(cx, id, build_focus, options, f));
        self.add(element);
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

    pub fn button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::button(self, label);
        let enabled = self.with_cx_mut(|cx| !imui_is_disabled(cx));
        self.record_focusable(resp.id, enabled);
        resp
    }

    pub fn action_button(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
    ) -> ResponseExt {
        self.action_button_with_options(label, action, ButtonOptions::default())
    }

    pub fn action_button_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        options: ButtonOptions,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::action_button_with_options(
            self, label, action, options,
        );
        self.record_focusable(resp.id, resp.enabled);
        resp
    }

    pub fn button_command(&mut self, command: impl Into<CommandId>) -> ResponseExt {
        self.button_command_with_options(command, ButtonOptions::default())
    }

    pub fn button_command_with_options(
        &mut self,
        command: impl Into<CommandId>,
        options: ButtonOptions,
    ) -> ResponseExt {
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::button_command_with_options(self, command, options);
        self.record_focusable(resp.id, resp.enabled);
        resp
    }

    pub fn menu_item(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        self.menu_item_with_options(label, MenuItemOptions::default())
    }

    pub fn menu_item_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_with_options(self, label, options);
        self.record_focusable(resp.id, enabled);
        resp
    }

    pub fn menu_item_checkbox_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_checkbox_with_options(
            self, label, checked, options,
        );
        self.record_focusable(resp.id, enabled);
        resp
    }

    pub fn menu_item_radio_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_radio_with_options(
            self, label, checked, options,
        );
        self.record_focusable(resp.id, enabled);
        resp
    }

    pub fn menu_item_action(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
    ) -> ResponseExt {
        self.menu_item_action_with_options(label, action, MenuItemOptions::default())
    }

    pub fn menu_item_action_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_action_with_options(
            self, label, action, options,
        );
        self.record_focusable(resp.id, resp.enabled);
        resp
    }

    pub fn begin_menu(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.begin_menu_with_options(id, label, BeginMenuOptions::default(), f)
    }

    pub fn begin_menu_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: BeginMenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        menu_family_controls::begin_menu_with_options(self, id, label.into(), options, f)
    }

    pub fn begin_submenu(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.begin_submenu_with_options(id, label, BeginSubmenuOptions::default(), f)
    }

    pub fn begin_submenu_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: BeginSubmenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        menu_family_controls::begin_submenu_with_options(self, id, label.into(), options, f)
    }

    pub fn menu_item_command(&mut self, command: impl Into<CommandId>) -> ResponseExt {
        self.menu_item_command_with_options(command, MenuItemOptions::default())
    }

    pub fn menu_item_command_with_options(
        &mut self,
        command: impl Into<CommandId>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_command_with_options(
            self, command, options,
        );
        self.record_focusable(resp.id, resp.enabled);
        resp
    }

    pub fn selectable(&mut self, label: impl Into<Arc<str>>, selected: bool) -> ResponseExt {
        self.selectable_with_options(
            label,
            SelectableOptions {
                selected,
                ..Default::default()
            },
        )
    }

    pub fn selectable_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: SelectableOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::selectable_with_options(self, label, options);
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn multi_selectable<K: Clone + PartialEq + 'static>(
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

    pub fn multi_selectable_with_options<K: Clone + PartialEq + 'static>(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<ImUiMultiSelectState<K>>,
        all_keys: &[K],
        key: K,
        options: SelectableOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::multi_selectable_with_options(
            self, label, model, all_keys, key, options,
        );
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn combo(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        preview: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ComboResponse {
        self.combo_with_options(id, label, preview, ComboOptions::default(), f)
    }

    pub fn combo_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        preview: impl Into<Arc<str>>,
        options: ComboOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ComboResponse {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::combo_with_options(
            self, id, label, preview, options, f,
        );
        self.record_focusable(resp.id(), focusable);
        resp
    }

    pub fn collapsing_header(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.collapsing_header_with_options(id, label, CollapsingHeaderOptions::default(), f)
    }

    pub fn collapsing_header_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: CollapsingHeaderOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::collapsing_header_with_options(
            self, id, label, options, f,
        );
        self.record_focusable(resp.id(), enabled);
        resp
    }

    pub fn tree_node(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.tree_node_with_options(id, label, TreeNodeOptions::default(), f)
    }

    pub fn tree_node_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: TreeNodeOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::tree_node_with_options(self, id, label, options, f);
        self.record_focusable(resp.id(), enabled);
        resp
    }

    pub fn input_text_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.input_text_model_with_options(model, InputTextOptions::default())
    }

    pub fn input_text_model_with_options(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: InputTextOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::input_text_model_with_options(self, model, options);
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn textarea_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.textarea_model_with_options(model, TextAreaOptions::default())
    }

    pub fn textarea_model_with_options(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: TextAreaOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::textarea_model_with_options(self, model, options);
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn checkbox_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.checkbox_model_with_options(label, model, CheckboxOptions::default())
    }

    pub fn checkbox_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: CheckboxOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::checkbox_model_with_options(
            self, label, model, options,
        );
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn switch_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.switch_model_with_options(label, model, SwitchOptions::default())
    }

    pub fn switch_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: SwitchOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::switch_model_with_options(
            self, label, model, options,
        );
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn slider_f32_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
    ) -> ResponseExt {
        self.slider_f32_model_with_options(label, model, SliderOptions::default())
    }

    pub fn slider_f32_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
        options: SliderOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::slider_f32_model_with_options(
            self, label, model, options,
        );
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn combo_model(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
    ) -> ResponseExt {
        self.combo_model_with_options(id, label, model, items, ComboModelOptions::default())
    }

    pub fn combo_model_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
        options: ComboModelOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::combo_model_with_options(
            self, id, label, model, items, options,
        );
        self.record_focusable(resp.id, focusable);
        resp
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
        // ImGui-style item flow: avoid flex main-axis shrink so text never "compresses" and
        // overlaps subsequent items when the container is shorter than the intrinsic text height.
        let element = self.with_cx_mut(|cx| {
            let mut props = fret_ui::element::TextProps::new(text);
            props.layout.flex.shrink = 0.0;
            cx.text_props(props)
        });
        self.add(element);
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
    ) {
        self.table_with_options(id, columns, TableOptions::default(), f);
    }

    fn table_with_options(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        options: TableOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) {
        let element =
            self.with_cx_mut(|cx| table_controls::table_element(cx, id, columns, None, options, f));
        self.add(element);
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
    ) {
        self.child_region_with_options(id, ChildRegionOptions::default(), f);
    }

    fn child_region_with_options(
        &mut self,
        id: &str,
        options: ChildRegionOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element =
            self.with_cx_mut(|cx| child_region::child_region_element(cx, id, None, options, f));
        self.add(element);
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
        let element = self.with_cx_mut(|cx| floating_layer_element(cx, id, f));
        self.add(element);
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
        let (element, response) =
            self.with_cx_mut(|cx| floating_area_element(cx, id, initial_position, options, f));
        self.add(element);
        response
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
        self.with_cx_mut(|cx| {
            floating_area_drag_surface_element(cx, area, props, None, true, true, setup, f)
        })
    }

    /// Returns the internal open model for a named popup scope.
    ///
    /// This is intended to support ImGui-like `OpenPopup` / `BeginPopup` splits without forcing
    /// callers to allocate a dedicated `Model<bool>` per popup.
    fn popup_open_model(&mut self, id: &str) -> fret_runtime::Model<bool> {
        popup_overlay::popup_open_model(self, id)
    }

    /// Drops all internal state for a named popup scope.
    ///
    /// This is primarily intended for ephemeral/dynamic scopes where the id space could grow
    /// without bound (e.g. popups keyed by user-generated strings). Dropping a scope will close the
    /// popup (if open) and release the internal models if no other references exist.
    fn drop_popup_scope(&mut self, id: &str) {
        popup_overlay::drop_popup_scope(self, id);
    }

    fn open_popup(&mut self, id: &str) {
        popup_overlay::open_popup(self, id);
    }

    fn open_popup_at(&mut self, id: &str, anchor: fret_core::Rect) {
        popup_overlay::open_popup_at(self, id, anchor);
    }

    fn close_popup(&mut self, id: &str) {
        popup_overlay::close_popup(self, id);
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
        popup_overlay::begin_popup_menu_with_options(self, id, trigger, options, f)
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
        popup_overlay::begin_popup_modal_with_options(self, id, trigger, options, f)
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
        tooltip_overlay::tooltip_text_with_options(self, id, trigger, text.into(), options)
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
        tooltip_overlay::tooltip_with_options(self, id, trigger, options, f)
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
        drag_drop::drag_source_with_options(self, trigger, payload, options)
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
        drag_drop::drop_target_with_options(self, trigger, options)
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
        popup_overlay::begin_popup_context_menu_with_options(self, id, trigger, options, f)
    }

    fn button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        self.button_with_options(label, ButtonOptions::default())
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

    /// Render a minimal in-window floating window.
    ///
    /// This is intentionally v1-small:
    /// - in-window (not an OS window / viewport),
    /// - draggable via the title bar,
    /// - position is stored as element-local state under the window id scope.
    ///
    /// Notes:
    /// - `id` must be stable across frames (mirrors Dear ImGui's "window name is the id" rule).
    /// - Z-order and focus arbitration are tracked as a separate work item (see workstream TODO).
    fn window(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        floating_window::floating_window_show(self, id, title, initial_position, f)
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
        floating_window::floating_window_show_with_options(
            self,
            id,
            title,
            initial_position,
            options,
            f,
        )
    }
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterImUiFacadeExt<H> for W {}

//! Optional integration helpers for immediate-mode authoring frontends.
//!
//! This module lives in `fret-ui-kit` (not `fret-imui`) to preserve dependency direction:
//!
//! - `fret-imui` stays policy-light and depends only on `fret-ui` (+ `fret-authoring` contract).
//! - `fret-ui-kit` can optionally provide bridges that allow `UiBuilder<T>` patch vocabulary to be
//!   used from immediate-style control flow.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use fret_authoring::Response;
use fret_authoring::UiWriter;
use fret_core::{
    AppWindowId, Corners, CursorIcon, Edges, KeyCode, MouseButton, Point, Px, Rect, SemanticsRole,
    Size,
};
use fret_runtime::{DragPhase, FrameId};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{
    DismissReason, DismissRequestCx, OnDismissRequest, PressablePointerDownResult,
    PressablePointerUpResult,
};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow,
    PointerRegionProps, PositionStyle, PressableA11y, PressableProps, RowProps,
};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::UiBuilder;
use crate::primitives::menu::root as menu_root;
use crate::primitives::popper;
use crate::{OverlayController, OverlayPresence, OverlayRequest};
use crate::{UiIntoElement, UiPatchTarget};

pub mod adapters;
mod floating_window_on_area;

/// A value that can be rendered into a declarative element within an `ElementContext`.
///
/// This is used to bridge the `UiBuilder<T>` ecosystem authoring surface (ADR 0175) into
/// immediate-mode frontends (`UiWriter`).
pub trait UiKitIntoElement<H: UiHost> {
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}

impl<H: UiHost, T> UiKitIntoElement<H> for UiBuilder<T>
where
    T: UiPatchTarget + UiIntoElement,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, B> UiKitIntoElement<H> for UiBuilder<crate::ui::FlexBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, B> UiKitIntoElement<H> for UiBuilder<crate::ui::ContainerBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::ContainerBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::StackBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::ScrollAreaBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, B> UiKitIntoElement<H> for UiBuilder<crate::ui::ScrollAreaBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

/// Extension trait bridging `fret-ui-kit` authoring (`UiBuilder<T>`) into an immediate-mode output.
pub trait UiWriterUiKitExt<H: UiHost>: UiWriter<H> {
    /// Render a `UiBuilder<T>` (or other supported authoring value) into the current output list.
    #[track_caller]
    fn add_ui<B>(&mut self, value: B)
    where
        B: UiKitIntoElement<H>,
    {
        let element = self.with_cx_mut(|cx| value.into_any_element(cx));
        self.add(element);
    }
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterUiKitExt<H> for W {}

/// A richer interaction result intended for immediate-mode facade helpers.
///
/// This is a ui-kit-level convenience wrapper: it extends the minimal `fret-authoring::Response`
/// contract with additional commonly requested signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct DragResponse {
    pub started: bool,
    pub dragging: bool,
    pub stopped: bool,
    pub delta: Point,
    pub total: Point,
}

/// A richer interaction result intended for immediate-mode facade helpers.
///
/// This is a ui-kit-level convenience wrapper: it extends the minimal `fret-authoring::Response`
/// contract with additional commonly requested signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResponseExt {
    pub core: Response,
    pub id: Option<GlobalElementId>,
    /// True when the item is focused and the window's focus-visible policy indicates keyboard
    /// navigation is active.
    ///
    /// This is intended as an immediate-mode equivalent of ImGui's "nav highlight under nav"
    /// behavior used by `IsItemHovered()` when `NavHighlightItemUnderNav` is active.
    pub nav_highlighted: bool,
    pub secondary_clicked: bool,
    pub double_clicked: bool,
    pub long_pressed: bool,
    pub press_holding: bool,
    pub context_menu_requested: bool,
    pub context_menu_anchor: Option<Point>,
    pub drag: DragResponse,
}

#[derive(Debug, Clone, Copy)]
pub struct FloatingAreaResponse {
    pub id: GlobalElementId,
    pub rect: Option<Rect>,
    pub position: Point,
    pub dragging: bool,
    pub drag_kind: fret_runtime::DragKindId,
}

impl FloatingAreaResponse {
    pub fn rect(self) -> Option<Rect> {
        self.rect
    }

    pub fn position(self) -> Point {
        self.position
    }

    pub fn dragging(self) -> bool {
        self.dragging
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FloatingWindowResponse {
    pub area: FloatingAreaResponse,
    pub size: Option<Size>,
    pub resizing: bool,
    pub collapsed: bool,
}

impl FloatingWindowResponse {
    pub fn rect(self) -> Option<Rect> {
        self.area.rect
    }

    pub fn position(self) -> Point {
        self.area.position
    }

    pub fn size(self) -> Option<Size> {
        self.size
    }

    pub fn dragging(self) -> bool {
        self.area.dragging
    }

    pub fn resizing(self) -> bool {
        self.resizing
    }

    pub fn collapsed(self) -> bool {
        self.collapsed
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct FloatingWindowChromeResponse {
    size: Option<Size>,
    resizing: bool,
    collapsed: bool,
}

impl ResponseExt {
    pub fn clicked(self) -> bool {
        self.core.clicked()
    }

    pub fn changed(self) -> bool {
        self.core.changed()
    }

    pub fn secondary_clicked(self) -> bool {
        self.secondary_clicked
    }

    pub fn double_clicked(self) -> bool {
        self.double_clicked
    }

    pub fn long_pressed(self) -> bool {
        self.long_pressed
    }

    pub fn press_holding(self) -> bool {
        self.press_holding
    }

    pub fn context_menu_requested(self) -> bool {
        self.context_menu_requested
    }

    pub fn context_menu_anchor(self) -> Option<Point> {
        self.context_menu_anchor
    }

    pub fn nav_highlighted(self) -> bool {
        self.nav_highlighted
    }

    /// ImGui-style "hovered" default: pointer-hover OR nav-highlight.
    ///
    /// Note: this does not currently implement ImGui's hovered flags (e.g.
    /// `AllowWhenBlockedByPopup`, hover delays, `AllowWhenDisabled`).
    pub fn hovered_like_imgui(self) -> bool {
        self.core.hovered || self.nav_highlighted
    }

    pub fn drag_started(self) -> bool {
        self.drag.started
    }

    pub fn dragging(self) -> bool {
        self.drag.dragging
    }

    pub fn drag_stopped(self) -> bool {
        self.drag.stopped
    }

    pub fn drag_delta(self) -> Point {
        self.drag.delta
    }

    pub fn drag_total(self) -> Point {
        self.drag.total
    }
}

#[derive(Debug, Default)]
struct DragReportState {
    last_position: Option<Point>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct LongPressSignalState {
    timer: Option<fret_runtime::TimerToken>,
    holding: bool,
}

#[derive(Default)]
struct ImUiContextMenuAnchorStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<Option<Point>>>,
}

#[derive(Default)]
struct ImUiLongPressStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<LongPressSignalState>>,
}

#[derive(Default)]
struct ImUiFloatWindowCollapsedStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<bool>>,
}

fn context_menu_anchor_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<Option<Point>> {
    cx.app
        .with_global_mut_untracked(ImUiContextMenuAnchorStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(None::<Point>))
                .clone()
        })
}

fn long_press_signal_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<LongPressSignalState> {
    cx.app
        .with_global_mut_untracked(ImUiLongPressStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(LongPressSignalState::default()))
                .clone()
        })
}

fn float_window_collapsed_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<bool> {
    cx.app
        .with_global_mut_untracked(ImUiFloatWindowCollapsedStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(false))
                .clone()
        })
}

#[derive(Debug, Clone, Copy)]
pub struct PopupMenuOptions {
    pub placement: popper::PopperContentPlacement,
    pub estimated_size: Size,
    pub modal: bool,
}

impl Default for PopupMenuOptions {
    fn default() -> Self {
        Self {
            placement: popper::PopperContentPlacement::new(
                popper::LayoutDirection::Ltr,
                popper::Side::Bottom,
                popper::Align::Start,
                Px(4.0),
            ),
            estimated_size: Size::new(Px(160.0), Px(120.0)),
            modal: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PopupModalOptions {
    pub size: Size,
    pub close_on_outside_press: bool,
}

impl Default for PopupModalOptions {
    fn default() -> Self {
        Self {
            size: Size::new(Px(320.0), Px(200.0)),
            close_on_outside_press: false,
        }
    }
}

#[derive(Debug, Clone)]
struct ImUiMenuNavState {
    items: Rc<RefCell<Vec<GlobalElementId>>>,
}

#[derive(Debug, Clone)]
pub struct MenuItemOptions {
    pub enabled: bool,
    pub close_popup: Option<fret_runtime::Model<bool>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for MenuItemOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            close_popup: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputTextOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub a11y_role: Option<SemanticsRole>,
    pub placeholder: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub submit_command: Option<fret_runtime::CommandId>,
    pub cancel_command: Option<fret_runtime::CommandId>,
}

impl Default for InputTextOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            a11y_role: Some(SemanticsRole::TextField),
            placeholder: None,
            test_id: None,
            submit_command: None,
            cancel_command: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextAreaOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub min_height: Px,
}

impl Default for TextAreaOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            min_height: Px(80.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwitchOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for SwitchOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
        }
    }
}

pub type ToggleOptions = SwitchOptions;

#[derive(Debug, Clone)]
pub struct SliderOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            min: 0.0,
            max: 100.0,
            step: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    /// Optional stable popup scope id override.
    ///
    /// When set, `select_model_ex` will use this id for its internal popup scope instead of
    /// deriving one from `test_id`/`label`. This is useful to avoid accidental collisions (e.g.
    /// multiple selects with the same label) and to keep popup store growth bounded when call sites
    /// generate dynamic labels.
    pub popup_scope_id: Option<Arc<str>>,
    pub placeholder: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
}

impl Default for SelectOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            popup_scope_id: None,
            placeholder: Some(Arc::from("Select?")),
            popup: PopupMenuOptions::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HorizontalOptions {
    pub gap: crate::MetricRef,
    pub justify: crate::Justify,
    pub items: crate::Items,
    pub wrap: bool,
}

impl Default for HorizontalOptions {
    fn default() -> Self {
        Self {
            gap: crate::MetricRef::space(crate::Space::N0),
            justify: crate::Justify::Start,
            items: crate::Items::Center,
            wrap: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerticalOptions {
    pub gap: crate::MetricRef,
    pub justify: crate::Justify,
    pub items: crate::Items,
    pub wrap: bool,
}

impl Default for VerticalOptions {
    fn default() -> Self {
        Self {
            gap: crate::MetricRef::space(crate::Space::N0),
            justify: crate::Justify::Start,
            items: crate::Items::Stretch,
            wrap: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridOptions {
    pub columns: usize,
    pub column_gap: crate::MetricRef,
    pub row_gap: crate::MetricRef,
    pub row_justify: crate::Justify,
    pub row_items: crate::Items,
}

impl Default for GridOptions {
    fn default() -> Self {
        Self {
            columns: 1,
            column_gap: crate::MetricRef::space(crate::Space::N0),
            row_gap: crate::MetricRef::space(crate::Space::N0),
            row_justify: crate::Justify::Start,
            row_items: crate::Items::Center,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScrollOptions {
    pub axis: fret_ui::element::ScrollAxis,
    pub show_scrollbar_x: bool,
    pub show_scrollbar_y: bool,
    pub handle: Option<fret_ui::scroll::ScrollHandle>,
}

impl Default for ScrollOptions {
    fn default() -> Self {
        Self {
            axis: fret_ui::element::ScrollAxis::Y,
            show_scrollbar_x: false,
            show_scrollbar_y: true,
            handle: None,
        }
    }
}

#[derive(Clone)]
struct PopupStoreState {
    open: fret_runtime::Model<bool>,
    anchor: fret_runtime::Model<Option<fret_core::Rect>>,
    panel_id: Option<GlobalElementId>,
    /// Last frame id where the popup was "kept alive" by a `begin_popup_*` call.
    keep_alive_frame: Option<FrameId>,
}

#[derive(Default)]
struct PopupStoreWindowState {
    by_id: HashMap<Arc<str>, PopupStoreState>,
    prepared_frame: Option<FrameId>,
}

#[derive(Default)]
struct ImUiPopupStore {
    by_window: HashMap<AppWindowId, PopupStoreWindowState>,
}

fn prepare_popup_store_for_frame<H: UiHost>(
    store: &mut ImUiPopupStore,
    app: &mut H,
    window: AppWindowId,
    frame_id: FrameId,
) {
    let state = store.by_window.entry(window).or_default();
    if state.prepared_frame == Some(frame_id) {
        return;
    }
    state.prepared_frame = Some(frame_id);

    let required_keep_alive = FrameId(frame_id.0.saturating_sub(1));
    for st in state.by_id.values_mut() {
        let is_open = app.models().get_copied(&st.open).unwrap_or(false);
        if !is_open {
            continue;
        }
        if st.keep_alive_frame == Some(required_keep_alive) {
            continue;
        }
        let _ = app.models_mut().update(&st.open, |v| *v = false);
        let _ = app.models_mut().update(&st.anchor, |v| *v = None);
        st.panel_id = None;
    }
}

fn with_popup_store_for_id<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    f: impl FnOnce(&mut PopupStoreState, &mut H) -> R,
) -> R {
    let window = cx.window;
    let frame_id = cx.frame_id;
    cx.app
        .with_global_mut_untracked(ImUiPopupStore::default, |store, app| {
            prepare_popup_store_for_frame(store, app, window, frame_id);

            let state = store.by_window.entry(window).or_default();
            if let Some(existing) = state.by_id.get_mut(id) {
                return f(existing, app);
            }

            let key: Arc<str> = Arc::from(id);
            let entry = state.by_id.entry(key).or_insert_with(|| PopupStoreState {
                open: app.models_mut().insert(false),
                anchor: app.models_mut().insert(None::<fret_core::Rect>),
                panel_id: None,
                keep_alive_frame: None,
            });
            f(entry, app)
        })
}

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
const KEY_DRAG_STARTED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_started.v1");
const KEY_DRAG_STOPPED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_stopped.v1");

// ImGui default: `MouseDragThreshold = 6`.
const DEFAULT_DRAG_THRESHOLD_PX: f32 = 6.0;
const LONG_PRESS_DELAY: Duration = Duration::from_millis(450);
const DRAG_KIND_MASK: u64 = 0x8000_0000_0000_0000;

fn drag_kind_for_element(element: GlobalElementId) -> fret_runtime::DragKindId {
    fret_runtime::DragKindId(DRAG_KIND_MASK | element.0)
}

fn drag_threshold_sq_for<H: UiHost>(cx: &ElementContext<'_, H>) -> f32 {
    let theme = fret_ui::Theme::global(&*cx.app);
    let px = theme
        .metric_by_key(crate::theme_tokens::metric::COMPONENT_IMUI_DRAG_THRESHOLD_PX)
        .unwrap_or(Px(DEFAULT_DRAG_THRESHOLD_PX));
    let v = px.0.max(0.0);
    v * v
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
    cx.with_state_for(
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

fn text_model_changed_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    current: &str,
) -> bool {
    model_value_changed_for(cx, id, current.to_string())
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

fn default_text_area_style_from_theme(theme: &fret_ui::Theme) -> fret_ui::TextAreaStyle {
    let input_style = crate::recipes::input::default_text_input_style(theme);
    let mut preedit_bg_color = input_style.selection_color;
    preedit_bg_color.a = (preedit_bg_color.a * 0.35).clamp(0.0, 1.0);

    fret_ui::TextAreaStyle {
        padding_x: input_style.padding.left,
        padding_y: input_style.padding.top,
        background: input_style.background,
        border: input_style.border,
        border_color: input_style.border_color,
        focus_ring: input_style.focus_ring,
        corner_radii: input_style.corner_radii,
        text_color: input_style.text_color,
        selection_color: input_style.selection_color,
        caret_color: input_style.caret_color,
        preedit_bg_color,
        preedit_underline_color: input_style.preedit_color,
    }
}

fn build_imui_children_with_focus<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) {
    let mut ui = ImUiFacade {
        cx,
        out,
        build_focus,
    };
    f(&mut ui);
}

fn horizontal_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: HorizontalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut builder = crate::ui::h_flex_build(cx, move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .gap_metric(options.gap)
        .justify(options.justify)
        .items(options.items);
    if options.wrap {
        builder = builder.wrap();
    } else {
        builder = builder.no_wrap();
    }
    builder.into_element(cx)
}

fn vertical_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: VerticalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut builder = crate::ui::v_flex_build(cx, move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .gap_metric(options.gap)
        .justify(options.justify)
        .items(options.items);
    if options.wrap {
        builder = builder.wrap();
    } else {
        builder = builder.no_wrap();
    }
    builder.into_element(cx)
}

fn scroll_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ScrollOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut builder = crate::ui::scroll_area_build(cx, move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .axis(options.axis)
        .show_scrollbars(options.show_scrollbar_x, options.show_scrollbar_y);
    if let Some(handle) = options.handle {
        builder = builder.handle(handle);
    }
    builder.into_element(cx)
}

fn grid_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: GridOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut cells: Vec<AnyElement> = Vec::new();
    build_imui_children_with_focus(cx, &mut cells, build_focus, f);

    let columns = options.columns.max(1);
    let mut rows: Vec<AnyElement> = Vec::new();
    let mut row_index = 0usize;
    let mut iter = cells.into_iter();

    loop {
        let mut row_cells: Vec<AnyElement> = Vec::with_capacity(columns);
        for _ in 0..columns {
            let Some(cell) = iter.next() else {
                break;
            };
            row_cells.push(cell);
        }
        if row_cells.is_empty() {
            break;
        }

        let row = cx.keyed(row_index, |cx| {
            crate::ui::h_flex(cx, move |_cx| row_cells)
                .gap_metric(options.column_gap.clone())
                .justify(options.row_justify)
                .items(options.row_items)
                .no_wrap()
                .into_element(cx)
        });
        rows.push(row);
        row_index += 1;
    }

    crate::ui::v_flex(cx, move |_cx| rows)
        .gap_metric(options.row_gap)
        .justify(crate::Justify::Start)
        .items(crate::Items::Stretch)
        .no_wrap()
        .into_element(cx)
}
fn arm_long_press_timer_for(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    model: &fret_runtime::Model<LongPressSignalState>,
) {
    let token = host.next_timer_token();
    let previous = host
        .update_model(model, |state| {
            let previous = state.timer.take();
            state.timer = Some(token);
            state.holding = false;
            previous
        })
        .flatten();
    if let Some(previous) = previous {
        host.push_effect(fret_runtime::Effect::CancelTimer { token: previous });
    }
    host.push_effect(fret_runtime::Effect::SetTimer {
        window: Some(action_cx.window),
        token,
        after: LONG_PRESS_DELAY,
        repeat: None,
    });
}

fn cancel_long_press_timer_for(
    host: &mut dyn fret_ui::action::UiActionHost,
    model: &fret_runtime::Model<LongPressSignalState>,
) {
    let previous = host
        .update_model(model, |state| {
            let previous = state.timer.take();
            state.holding = false;
            previous
        })
        .flatten();
    if let Some(previous) = previous {
        host.push_effect(fret_runtime::Effect::CancelTimer { token: previous });
    }
}

fn emit_long_press_if_matching(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    model: &fret_runtime::Model<LongPressSignalState>,
    token: fret_runtime::TimerToken,
) -> bool {
    let fired = host
        .update_model(model, |state| {
            if state.timer != Some(token) {
                return false;
            }
            state.timer = None;
            state.holding = true;
            true
        })
        .unwrap_or(false);
    if fired {
        host.record_transient_event(action_cx, KEY_LONG_PRESSED);
        host.notify(action_cx);
    }
    fired
}

const FLOAT_WINDOW_DRAG_KIND_MASK: u64 = 0x4000_0000_0000_0000;

fn float_window_drag_kind_for_element(element: GlobalElementId) -> fret_runtime::DragKindId {
    fret_runtime::DragKindId(FLOAT_WINDOW_DRAG_KIND_MASK | element.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FloatWindowResizeHandle {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
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
            activate_on_click: true,
            inputs_enabled: true,
            no_inputs: false,
            pointer_passthrough: false,
        }
    }
}

const FLOAT_WINDOW_RESIZE_KIND_BASE: u64 = fnv1a64(b"fret-ui-kit.imui.float_window.resize.v1");

fn float_window_resize_kind_for_element(
    element: GlobalElementId,
    handle: FloatWindowResizeHandle,
) -> fret_runtime::DragKindId {
    let handle_tag = match handle {
        FloatWindowResizeHandle::Left => 1,
        FloatWindowResizeHandle::Right => 2,
        FloatWindowResizeHandle::Top => 3,
        FloatWindowResizeHandle::Bottom => 4,
        FloatWindowResizeHandle::TopLeft => 5,
        FloatWindowResizeHandle::TopRight => 6,
        FloatWindowResizeHandle::BottomLeft => 7,
        FloatWindowResizeHandle::BottomRight => 8,
    };
    fret_runtime::DragKindId(
        FLOAT_WINDOW_RESIZE_KIND_BASE ^ element.0.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ handle_tag,
    )
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

const KEY_FLOAT_WINDOW_ACTIVATE: u64 = fnv1a64(b"fret-ui-kit.imui.float_window.activate.v1");
const KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED: u64 =
    fnv1a64(b"fret-ui-kit.imui.float_window.toggle_collapsed.v1");

type OnFloatingAreaLeftDoubleClick =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiPointerActionHost, fret_ui::action::ActionCx) + 'static>;

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
        self.horizontal_ex(HorizontalOptions::default(), f);
    }

    pub fn horizontal_ex(
        &mut self,
        options: HorizontalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element =
            self.with_cx_mut(|cx| horizontal_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn vertical(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.vertical_ex(VerticalOptions::default(), f);
    }

    pub fn vertical_ex(
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
        self.grid_ex(GridOptions::default(), f);
    }

    pub fn grid_ex(
        &mut self,
        options: GridOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self.with_cx_mut(|cx| grid_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn scroll(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.scroll_ex(ScrollOptions::default(), f);
    }

    pub fn scroll_ex(
        &mut self,
        options: ScrollOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self.with_cx_mut(|cx| scroll_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::button(self, label);
        self.record_focusable(resp.id, true);
        resp
    }

    pub fn menu_item(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        self.menu_item_ex(label, MenuItemOptions::default())
    }

    pub fn menu_item_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let enabled = options.enabled;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_ex(self, label, options);
        self.record_focusable(resp.id, enabled);
        resp
    }

    pub fn menu_item_checkbox_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let enabled = options.enabled;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_checkbox_ex(
            self, label, checked, options,
        );
        self.record_focusable(resp.id, enabled);
        resp
    }

    pub fn menu_item_radio_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let enabled = options.enabled;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::menu_item_radio_ex(self, label, checked, options);
        self.record_focusable(resp.id, enabled);
        resp
    }

    pub fn menu_item_close(
        &mut self,
        label: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_close(self, label, open);
        self.record_focusable(resp.id, true);
        resp
    }

    pub fn menu_item_close_popup(
        &mut self,
        popup_id: &str,
        label: impl Into<Arc<str>>,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_close_popup(self, popup_id, label);
        self.record_focusable(resp.id, true);
        resp
    }

    pub fn input_text_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.input_text_model_ex(model, InputTextOptions::default())
    }

    pub fn input_text_model_ex(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: InputTextOptions,
    ) -> ResponseExt {
        let focusable = options.enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::input_text_model_ex(self, model, options);
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn textarea_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.textarea_model_ex(model, TextAreaOptions::default())
    }

    pub fn textarea_model_ex(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: TextAreaOptions,
    ) -> ResponseExt {
        let focusable = options.enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::textarea_model_ex(self, model, options);
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn toggle_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.toggle_model_ex(label, model, ToggleOptions::default())
    }

    pub fn toggle_model_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: ToggleOptions,
    ) -> ResponseExt {
        let focusable = options.enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::toggle_model_ex(self, label, model, options);
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn switch_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.switch_model_ex(label, model, SwitchOptions::default())
    }

    pub fn switch_model_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: SwitchOptions,
    ) -> ResponseExt {
        let focusable = options.enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::switch_model_ex(self, label, model, options);
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn slider_f32_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
    ) -> ResponseExt {
        self.slider_f32_model_ex(label, model, SliderOptions::default())
    }

    pub fn slider_f32_model_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
        options: SliderOptions,
    ) -> ResponseExt {
        let focusable = options.enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::slider_f32_model_ex(self, label, model, options);
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn select_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
    ) -> ResponseExt {
        self.select_model_ex(label, model, items, SelectOptions::default())
    }

    pub fn select_model_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
        options: SelectOptions,
    ) -> ResponseExt {
        let focusable = options.enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::select_model_ex(self, label, model, items, options);
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

#[derive(Debug)]
struct FloatingAreaState {
    position: Point,
    last_drag_position: Option<Point>,
    test_id: Arc<str>,
}

#[derive(Debug)]
struct FloatWindowState {
    size: Size,
    last_resize_position: Option<Point>,
    title_bar_test_id: Arc<str>,
    close_button_test_id: Arc<str>,
    resize_left_test_id: Arc<str>,
    resize_right_test_id: Arc<str>,
    resize_top_test_id: Arc<str>,
    resize_bottom_test_id: Arc<str>,
    resize_top_left_test_id: Arc<str>,
    resize_top_right_test_id: Arc<str>,
    resize_bottom_left_test_id: Arc<str>,
    resize_corner_test_id: Arc<str>,
}

fn float_layer_bring_to_front_if_activated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    surface_id: GlobalElementId,
    window_id: GlobalElementId,
) {
    if !cx.take_transient_for(surface_id, KEY_FLOAT_WINDOW_ACTIVATE) {
        return;
    }
    let Some(marker) = cx.inherited_state::<FloatWindowLayerMarker>() else {
        return;
    };
    cx.with_state_for(marker.layer, FloatWindowLayerZOrder::default, |st| {
        st.bring_to_front(window_id);
    });
}

#[derive(Debug, Clone, Copy)]
struct FloatWindowLayerMarker {
    layer: GlobalElementId,
}

#[derive(Debug, Default)]
struct FloatWindowLayerZOrder {
    order: Vec<GlobalElementId>,
    dirty: bool,
    snapshot: FloatWindowLayerZOrderSnapshot,
}

impl FloatWindowLayerZOrder {
    fn ensure_present(&mut self, window: GlobalElementId) {
        if self.order.contains(&window) {
            return;
        }
        self.order.push(window);
        self.dirty = true;
    }

    fn bring_to_front(&mut self, window: GlobalElementId) {
        self.ensure_present(window);
        let Some(idx) = self.order.iter().position(|w| *w == window) else {
            return;
        };
        if idx + 1 == self.order.len() {
            return;
        }
        self.order.remove(idx);
        self.order.push(window);
        self.dirty = true;
    }

    fn prune_missing(&mut self, windows: &[AnyElement]) {
        let before = self.order.len();
        self.order.retain(|id| windows.iter().any(|w| w.id == *id));
        if self.order.len() != before {
            self.dirty = true;
        }
    }

    fn snapshot(&mut self) -> FloatWindowLayerZOrderSnapshot {
        if !self.dirty {
            return self.snapshot.clone();
        }

        let order: Arc<[GlobalElementId]> = self.order.clone().into();
        let mut rank = HashMap::with_capacity(order.len());
        for (ix, id) in order.iter().enumerate() {
            rank.insert(*id, ix);
        }

        self.snapshot = FloatWindowLayerZOrderSnapshot {
            order,
            rank: Arc::new(rank),
        };
        self.dirty = false;
        self.snapshot.clone()
    }
}

#[derive(Debug, Clone, Default)]
struct FloatWindowLayerZOrderSnapshot {
    #[allow(dead_code)]
    order: Arc<[GlobalElementId]>,
    rank: Arc<HashMap<GlobalElementId, usize>>,
}

fn floating_area_drag_surface_element<H: UiHost, Setup, Build>(
    cx: &mut ElementContext<'_, H>,
    area: FloatingAreaContext,
    props: PointerRegionProps,
    on_left_double_click: Option<OnFloatingAreaLeftDoubleClick>,
    enable_drag: bool,
    enable_activation: bool,
    setup: Setup,
    build: Build,
) -> AnyElement
where
    Setup: FnOnce(&mut ElementContext<'_, H>, GlobalElementId),
    Build: for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
{
    let mut build = Some(build);
    let mut setup = Some(setup);
    let on_left_double_click_for_down = on_left_double_click.clone();
    cx.pointer_region(props, move |cx| {
        let region_id = cx.root_id();
        float_layer_bring_to_front_if_activated(cx, region_id, area.id);

        if let Some(setup) = setup.take() {
            setup(cx, region_id);
        }

        let drag_kind = area.drag_kind;
        cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
            if down.button != MouseButton::Left {
                return false;
            }

            host.request_focus(acx.target);
            if enable_drag {
                host.capture_pointer();
                if host.drag(down.pointer_id).is_none() {
                    host.begin_drag_with_kind(
                        down.pointer_id,
                        drag_kind,
                        acx.window,
                        down.position,
                    );
                }
            }
            if down.click_count == 2
                && let Some(on_left_double_click) = on_left_double_click_for_down.as_ref()
            {
                on_left_double_click(
                    host,
                    fret_ui::action::ActionCx {
                        window: acx.window,
                        target: area.id,
                    },
                );
            }
            if enable_activation {
                host.record_transient_event(acx, KEY_FLOAT_WINDOW_ACTIVATE);
            }
            host.notify(acx);
            false
        }));

        let drag_threshold_sq = drag_threshold_sq_for(cx);
        cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
            if !enable_drag {
                return false;
            }
            let Some(drag) = host.drag_mut(mv.pointer_id) else {
                return false;
            };
            if drag.kind != drag_kind || drag.source_window != acx.window {
                return false;
            }

            drag.current_window = acx.window;
            drag.position = mv.position;

            if !mv.buttons.left {
                drag.phase = DragPhase::Canceled;
                host.cancel_drag(mv.pointer_id);
                host.release_pointer_capture();
                host.notify(acx);
                return false;
            }

            let d = point_sub(drag.position, drag.start_position);
            let dist_sq = d.x.0 * d.x.0 + d.y.0 * d.y.0;
            if !drag.dragging && dist_sq >= drag_threshold_sq {
                drag.dragging = true;
                drag.phase = DragPhase::Dragging;
            }

            host.notify(acx);
            false
        }));

        cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
            if !enable_drag {
                return false;
            }
            if let Some(drag) = host.drag(up.pointer_id)
                && drag.kind == drag_kind
                && drag.source_window == acx.window
            {
                host.cancel_drag(up.pointer_id);
            }
            host.release_pointer_capture();
            host.notify(acx);
            false
        }));

        let mut out = Vec::new();
        if let Some(build) = build.take() {
            let mut ui = ImUiFacade {
                cx,
                out: &mut out,
                build_focus: None,
            };
            build(&mut ui);
        }
        out
    })
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

    fn text(&mut self, text: impl Into<Arc<str>>) {
        let element = self.with_cx_mut(|cx| cx.text(text));
        self.add(element);
    }

    fn separator(&mut self) {
        let element = self.with_cx_mut(|cx| {
            let mut props = fret_ui::element::ContainerProps::default();
            let theme = fret_ui::Theme::global(&*cx.app);
            props.background = Some(theme.color_required("border"));
            props.layout.size.width = fret_ui::element::Length::Fill;
            props.layout.size.height = fret_ui::element::Length::Px(fret_core::Px(1.0));
            cx.container(props, |_| Vec::new())
        });
        self.add(element);
    }

    fn horizontal(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.horizontal_ex(HorizontalOptions::default(), f);
    }

    fn horizontal_ex(
        &mut self,
        options: HorizontalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element = self.with_cx_mut(|cx| horizontal_container_element(cx, None, options, f));
        self.add(element);
    }

    fn vertical(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.vertical_ex(VerticalOptions::default(), f);
    }

    fn vertical_ex(
        &mut self,
        options: VerticalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element = self.with_cx_mut(|cx| vertical_container_element(cx, None, options, f));
        self.add(element);
    }

    fn grid(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.grid_ex(GridOptions::default(), f);
    }

    fn grid_ex(
        &mut self,
        options: GridOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element = self.with_cx_mut(|cx| grid_container_element(cx, None, options, f));
        self.add(element);
    }

    fn scroll(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.scroll_ex(ScrollOptions::default(), f);
    }

    fn scroll_ex(
        &mut self,
        options: ScrollOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element = self.with_cx_mut(|cx| scroll_container_element(cx, None, options, f));
        self.add(element);
    }

    /// Render a window-scoped floating window layer that manages z-order (bring-to-front).
    ///
    /// Notes:
    /// - This is an opt-in container; a plain `floating_area(...)` / `floating_window(...)` call
    ///   sequence keeps call-order z.
    /// - Call this late in the parent tree to ensure the layer paints above base content.
    fn floating_layer(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let element = self.with_cx_mut(|cx| {
            cx.named(id, |cx| {
                let layer_id = cx.root_id();
                cx.with_state_for(
                    layer_id,
                    || FloatWindowLayerMarker { layer: layer_id },
                    |st| st.layer = layer_id,
                );

                let mut windows: Vec<AnyElement> = Vec::new();
                {
                    let mut ui = ImUiFacade {
                        cx,
                        out: &mut windows,
                        build_focus: None,
                    };
                    f(&mut ui);
                }

                let z_order = cx.with_state_for(layer_id, FloatWindowLayerZOrder::default, |st| {
                    for w in windows.iter() {
                        st.ensure_present(w.id);
                    }
                    st.prune_missing(&windows);
                    st.snapshot()
                });

                let mut indexed: Vec<(usize, usize, AnyElement)> = windows
                    .into_iter()
                    .enumerate()
                    .map(|(original, w)| {
                        let idx = z_order.rank.get(&w.id).copied().unwrap_or(usize::MAX);
                        (idx, original, w)
                    })
                    .collect();

                indexed.sort_by_key(|(idx, original, _)| (*idx, *original));
                let windows_sorted: Vec<AnyElement> =
                    indexed.into_iter().map(|(_, _, w)| w).collect();

                let mut props = fret_ui::element::StackProps::default();
                props.layout.position = PositionStyle::Absolute;
                props.layout.inset = InsetStyle {
                    left: Some(Px(0.0)),
                    right: Some(Px(0.0)),
                    top: Some(Px(0.0)),
                    bottom: Some(Px(0.0)),
                };
                props.layout.overflow = Overflow::Visible;
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;

                cx.stack_props(props, move |_cx| windows_sorted)
            })
        });

        self.add(element);
    }

    /// Render a minimal in-window floating area primitive.
    ///
    /// This is the lowest-level building block for ImGui-like floating surfaces in-window:
    ///
    /// - always in-window (not an OS window / viewport),
    /// - position is stored as element-local state under the area id scope,
    /// - movement is driven by a caller-provided drag surface (via `floating_area_drag_surface_ex`),
    /// - optional z-order activation when nested inside `floating_layer(...)`.
    ///
    /// Notes:
    /// - `id` must be stable across frames (mirrors Dear ImGui's "name is the id" rule).
    fn floating_area(
        &mut self,
        id: &str,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
    ) {
        let _ = self.floating_area_show(id, initial_position, f);
    }

    fn area(
        &mut self,
        id: &str,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
    ) -> FloatingAreaResponse {
        self.floating_area_show(id, initial_position, f)
    }

    fn floating_area_ex(
        &mut self,
        id: &str,
        initial_position: Point,
        options: FloatingAreaOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
    ) {
        let _ = self.floating_area_show_ex(id, initial_position, options, f);
    }

    fn floating_area_show(
        &mut self,
        id: &str,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
    ) -> FloatingAreaResponse {
        self.floating_area_show_ex(id, initial_position, FloatingAreaOptions::default(), f)
    }

    fn floating_area_show_ex(
        &mut self,
        id: &str,
        initial_position: Point,
        options: FloatingAreaOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
    ) -> FloatingAreaResponse {
        let (element, response) = self.with_cx_mut(|cx| {
            cx.named(id, |cx| {
                let area_id = cx.root_id();
                if let Some(marker) = cx.inherited_state::<FloatWindowLayerMarker>() {
                    cx.with_state_for(marker.layer, FloatWindowLayerZOrder::default, |st| {
                        st.ensure_present(area_id);
                    });
                }

                let drag_kind = float_window_drag_kind_for_element(area_id);
                let drag_snapshot = cx
                    .app
                    .find_drag_pointer_id(|d| {
                        d.kind == drag_kind
                            && d.source_window == cx.window
                            && d.current_window == cx.window
                    })
                    .and_then(|pointer_id| cx.app.drag(pointer_id))
                    .filter(|drag| drag.kind == drag_kind)
                    .map(|drag| (drag.dragging, drag.position, drag.start_position));
                let dragging = drag_snapshot
                    .map(|(dragging, _, _)| dragging)
                    .unwrap_or(false);

                let (position, test_id) = cx.with_state_for(
                    area_id,
                    || FloatingAreaState {
                        position: initial_position,
                        last_drag_position: None,
                        test_id: options.test_id.clone().unwrap_or_else(|| {
                            Arc::from(format!("{}{id}", options.test_id_prefix))
                        }),
                    },
                    |st| {
                        if let Some(test_id) = options.test_id.clone() {
                            st.test_id = test_id;
                        }

                        if let Some((dragging, current, start)) = drag_snapshot {
                            if dragging {
                                let prev = st.last_drag_position.unwrap_or(start);
                                st.position = point_add(st.position, point_sub(current, prev));
                                st.last_drag_position = Some(current);
                            } else {
                                st.last_drag_position = None;
                            }
                        } else {
                            st.last_drag_position = None;
                        }
                        (st.position, st.test_id.clone())
                    },
                );

                let ctx = FloatingAreaContext {
                    id: area_id,
                    position,
                    drag_kind,
                };

                let mut out: Vec<AnyElement> = Vec::new();
                {
                    let mut ui = ImUiFacade {
                        cx,
                        out: &mut out,
                        build_focus: None,
                    };
                    f(&mut ui, ctx);
                }

                let (final_position, final_test_id) = cx.with_state_for(
                    area_id,
                    || FloatingAreaState {
                        position,
                        last_drag_position: None,
                        test_id: test_id.clone(),
                    },
                    |st| (st.position, st.test_id.clone()),
                );

                let mut props = ContainerProps::default();
                props.layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        left: Some(final_position.x),
                        top: Some(final_position.y),
                        ..Default::default()
                    },
                    overflow: Overflow::Visible,
                    ..Default::default()
                };

                let area = if options.no_inputs {
                    let layout = props.layout;
                    let mut gate = cx.interactivity_gate_props(
                        fret_ui::element::InteractivityGateProps {
                            layout,
                            present: true,
                            interactive: false,
                        },
                        |_cx| out,
                    );
                    gate.id = area_id;
                    gate.attach_semantics(
                        fret_ui::element::SemanticsDecoration::default().test_id(final_test_id),
                    )
                } else if options.hit_test_passthrough {
                    let layout = props.layout;
                    let mut gate = cx.hit_test_gate_props(
                        fret_ui::element::HitTestGateProps {
                            layout,
                            hit_test: false,
                        },
                        |_cx| out,
                    );
                    gate.id = area_id;
                    gate.attach_semantics(
                        fret_ui::element::SemanticsDecoration::default().test_id(final_test_id),
                    )
                } else {
                    let mut area = cx.container(props, move |_cx| out);
                    // `cx.container(...)` introduces a fresh scoped id; normalize the outer area
                    // element id back to the named scope id so z-order state can track areas by
                    // `area_id`.
                    area.id = area_id;
                    area.attach_semantics(
                        fret_ui::element::SemanticsDecoration::default().test_id(final_test_id),
                    )
                };

                let response = FloatingAreaResponse {
                    id: area_id,
                    rect: cx.last_bounds_for_element(area_id),
                    position: final_position,
                    dragging,
                    drag_kind,
                };

                (area, response)
            })
        });

        self.add(element);
        response
    }

    /// Build a drag surface that moves a floating area (ImGui-style).
    ///
    /// The returned element should be placed as part of the area content (e.g. a title bar).
    fn floating_area_drag_surface_ex(
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
        self.with_cx_mut(|cx| with_popup_store_for_id(cx, id, |st, _app| st.open.clone()))
    }

    /// Drops all internal state for a named popup scope.
    ///
    /// This is primarily intended for ephemeral/dynamic scopes where the id space could grow
    /// without bound (e.g. popups keyed by user-generated strings). Dropping a scope will close the
    /// popup (if open) and release the internal models if no other references exist.
    fn drop_popup_scope(&mut self, id: &str) {
        self.with_cx_mut(|cx| {
            cx.app
                .with_global_mut_untracked(ImUiPopupStore::default, |st, app| {
                    prepare_popup_store_for_frame(st, app, cx.window, cx.frame_id);
                    let Some(window_state) = st.by_window.get_mut(&cx.window) else {
                        return;
                    };
                    let Some(entry) = window_state.by_id.remove(id) else {
                        return;
                    };
                    let _ = app.models_mut().update(&entry.open, |v| *v = false);
                    let _ = app.models_mut().update(&entry.anchor, |v| *v = None);
                });
            cx.app.request_redraw(cx.window);
        });
    }

    fn open_popup(&mut self, id: &str) {
        self.with_cx_mut(|cx| {
            let keep_alive_frame = cx.frame_id;
            let open = with_popup_store_for_id(cx, id, move |st, _app| {
                st.keep_alive_frame = Some(keep_alive_frame);
                st.open.clone()
            });
            let _ = cx.app.models_mut().update(&open, |v| *v = true);
            cx.app.request_redraw(cx.window);
        });
    }

    fn open_popup_at(&mut self, id: &str, anchor: fret_core::Rect) {
        self.with_cx_mut(|cx| {
            let keep_alive_frame = cx.frame_id;
            let (open, anchor_model) = with_popup_store_for_id(cx, id, move |st, _app| {
                st.keep_alive_frame = Some(keep_alive_frame);
                (st.open.clone(), st.anchor.clone())
            });
            let _ = cx
                .app
                .models_mut()
                .update(&anchor_model, |v| *v = Some(anchor));
            let _ = cx.app.models_mut().update(&open, |v| *v = true);
            cx.app.request_redraw(cx.window);
        });
    }

    fn close_popup(&mut self, id: &str) {
        self.with_cx_mut(|cx| {
            let open = with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
            let _ = cx.app.models_mut().update(&open, |v| *v = false);
            cx.app.request_redraw(cx.window);
        });
    }

    fn begin_popup_menu(
        &mut self,
        id: &str,
        trigger: Option<GlobalElementId>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        self.begin_popup_menu_ex(id, trigger, PopupMenuOptions::default(), f)
    }

    fn begin_popup_menu_ex(
        &mut self,
        id: &str,
        trigger: Option<GlobalElementId>,
        options: PopupMenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        self.with_cx_mut(|cx| {
            let (open, anchor_model, panel_id) = with_popup_store_for_id(cx, id, |st, _app| {
                (st.open.clone(), st.anchor.clone(), st.panel_id)
            });
            let is_open = cx
                .read_model(&open, fret_ui::Invalidation::Paint, |_app, v| *v)
                .unwrap_or(false);
            if !is_open {
                return false;
            }

            let anchor = cx
                .read_model(&anchor_model, fret_ui::Invalidation::Paint, |_app, v| *v)
                .unwrap_or(None);
            let Some(anchor) = anchor else {
                let _ = cx.app.models_mut().update(&open, |v| *v = false);
                let _ = cx.app.models_mut().update(&anchor_model, |v| *v = None);
                with_popup_store_for_id(cx, id, |st, _app| {
                    st.panel_id = None;
                    st.keep_alive_frame = None;
                });
                cx.app.request_redraw(cx.window);
                return false;
            };

            let keep_alive_frame = cx.frame_id;
            with_popup_store_for_id(cx, id, move |st, _app| {
                st.keep_alive_frame = Some(keep_alive_frame);
            });

            let overlay_key = format!("fret-ui-kit.imui.popup.overlay.{id}");
            let overlay_id = cx.named(overlay_key.as_str(), |cx| cx.root_id());

            let root_name = OverlayController::popover_root_name(overlay_id);
            let desired = panel_id
                .and_then(|id| cx.last_bounds_for_element(id).map(|r| r.size))
                .unwrap_or(options.estimated_size);

            let layout =
                popper::popper_content_layout_sized(cx.bounds, anchor, desired, options.placement);

            let (popover, border) = {
                let theme = fret_ui::Theme::global(&*cx.app);
                (
                    theme.color_required("popover"),
                    theme.color_required("border"),
                )
            };

            let nav_items = Rc::new(RefCell::new(Vec::<GlobalElementId>::new()));
            let nav_items_for_state = nav_items.clone();
            let mut menu_id_for_focus: Option<GlobalElementId> = None;
            let mut build = Some(f);
            let panel = cx.with_root_name(root_name.as_str(), |cx| {
                cx.named("fret-ui-kit.imui.popup.panel", |cx| {
                    let mut semantics = fret_ui::element::SemanticsProps::default();
                    semantics.role = SemanticsRole::Menu;
                    semantics.test_id = Some(Arc::from(format!("imui-popup-{id}")));
                    semantics.layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            left: Some(layout.rect.origin.x),
                            top: Some(layout.rect.origin.y),
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    };

                    let menu = cx.semantics_with_id(semantics, move |cx, menu_id| {
                        cx.with_state_for(
                            menu_id,
                            || ImUiMenuNavState {
                                items: nav_items_for_state.clone(),
                            },
                            |st| st.items.borrow_mut().clear(),
                        );

                        let mut panel_props = ContainerProps::default();
                        panel_props.background = Some(popover);
                        panel_props.border = Edges::all(Px(1.0));
                        panel_props.border_color = Some(border);
                        panel_props.corner_radii = Corners::all(Px(6.0));
                        panel_props.padding = Edges::all(Px(6.0));

                        vec![cx.container(panel_props, move |cx| {
                            let mut col = ColumnProps::default();
                            col.gap = Px(2.0);
                            col.layout.size.width = Length::Auto;
                            col.layout.size.height = Length::Auto;
                            vec![cx.column(col, move |cx| {
                                let mut out: Vec<AnyElement> = Vec::new();
                                let mut ui = ImUiFacade {
                                    cx,
                                    out: &mut out,
                                    build_focus: None,
                                };
                                if let Some(f) = build.take() {
                                    f(&mut ui);
                                }
                                out
                            })]
                        })]
                    });
                    menu_id_for_focus = Some(menu.id);
                    with_popup_store_for_id(cx, id, |st, _app| st.panel_id = Some(menu.id));
                    menu
                })
            });

            let trigger_id = trigger.unwrap_or(overlay_id);
            let first_item = nav_items.borrow().first().copied();
            let initial_focus = menu_root::MenuInitialFocusTargets::new()
                .keyboard_entry_focus(first_item)
                .pointer_content_focus(menu_id_for_focus);
            let req = menu_root::dismissible_menu_request_with_modal(
                cx,
                overlay_id,
                trigger_id,
                open.clone(),
                OverlayPresence::instant(true),
                vec![panel],
                root_name.clone(),
                initial_focus,
                None,
                None,
                None,
                options.modal,
            );
            OverlayController::request(cx, req);

            true
        })
    }

    fn begin_popup_modal(
        &mut self,
        id: &str,
        trigger: Option<GlobalElementId>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        self.begin_popup_modal_ex(id, trigger, PopupModalOptions::default(), f)
    }

    fn begin_popup_modal_ex(
        &mut self,
        id: &str,
        trigger: Option<GlobalElementId>,
        options: PopupModalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        self.with_cx_mut(|cx| {
            let open = with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
            let is_open = cx
                .read_model(&open, fret_ui::Invalidation::Paint, |_app, v| *v)
                .unwrap_or(false);
            if !is_open {
                return false;
            }

            let keep_alive_frame = cx.frame_id;
            with_popup_store_for_id(cx, id, move |st, _app| {
                st.keep_alive_frame = Some(keep_alive_frame);
            });

            let overlay_key = format!("fret-ui-kit.imui.popup_modal.overlay.{id}");
            let overlay_id = cx.named(overlay_key.as_str(), |cx| cx.root_id());

            let root_name = OverlayController::modal_root_name(overlay_id);

            let (popover, border) = {
                let theme = fret_ui::Theme::global(&*cx.app);
                (
                    theme.color_required("popover"),
                    theme.color_required("border"),
                )
            };

            let dim = fret_core::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.4,
            };

            let size = options.size;
            let left =
                Px(cx.bounds.origin.x.0 + (cx.bounds.size.width.0 - size.width.0).max(0.0) * 0.5);
            let top =
                Px(cx.bounds.origin.y.0 + (cx.bounds.size.height.0 - size.height.0).max(0.0) * 0.5);

            let close_on_outside_press = options.close_on_outside_press;
            let open_for_dismiss = open.clone();
            let on_dismiss_request: OnDismissRequest = Arc::new(
                move |host, acx, req: &mut DismissRequestCx| match req.reason {
                    DismissReason::Escape => {
                        let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
                        host.notify(acx);
                    }
                    DismissReason::OutsidePress { .. } if close_on_outside_press => {
                        let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
                        host.notify(acx);
                    }
                    _ => {
                        req.prevent_default();
                    }
                },
            );

            let focus_state = Rc::new(Cell::new(None::<GlobalElementId>));
            let focus_state_for_build = focus_state.clone();
            let mut panel_id_for_focus: Option<GlobalElementId> = None;
            let mut build = Some(f);

            let layer = cx.with_root_name(root_name.as_str(), |cx| {
                cx.named("fret-ui-kit.imui.popup_modal.layer", |cx| {
                    let mut stack = fret_ui::element::StackProps::default();
                    stack.layout.position = PositionStyle::Absolute;
                    stack.layout.inset = InsetStyle {
                        left: Some(Px(0.0)),
                        right: Some(Px(0.0)),
                        top: Some(Px(0.0)),
                        bottom: Some(Px(0.0)),
                    };
                    stack.layout.size.width = Length::Fill;
                    stack.layout.size.height = Length::Fill;
                    stack.layout.overflow = Overflow::Visible;

                    cx.stack_props(stack, |cx| {
                        let backdrop = cx.container(
                            {
                                let mut props = ContainerProps::default();
                                props.layout.position = PositionStyle::Absolute;
                                props.layout.inset = InsetStyle {
                                    left: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    top: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                };
                                props.layout.size.width = Length::Fill;
                                props.layout.size.height = Length::Fill;
                                props.background = Some(dim);
                                props
                            },
                            |_cx| Vec::<AnyElement>::new(),
                        );

                        let panel = cx.named("fret-ui-kit.imui.popup_modal.panel", |cx| {
                            let mut semantics = fret_ui::element::SemanticsProps::default();
                            semantics.role = SemanticsRole::Dialog;
                            semantics.test_id = Some(Arc::from(format!("imui-popup-modal-{id}")));
                            semantics.layout = LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(left),
                                    top: Some(top),
                                    ..Default::default()
                                },
                                size: fret_ui::element::SizeStyle {
                                    width: Length::Px(size.width),
                                    height: Length::Px(size.height),
                                    ..Default::default()
                                },
                                ..Default::default()
                            };

                            let modal = cx.semantics_with_id(semantics, move |cx, _id| {
                                let mut panel_props = ContainerProps::default();
                                panel_props.background = Some(popover);
                                panel_props.border = Edges::all(Px(1.0));
                                panel_props.border_color = Some(border);
                                panel_props.corner_radii = Corners::all(Px(8.0));
                                panel_props.padding = Edges::all(Px(10.0));
                                panel_props.layout.size.width = Length::Fill;
                                panel_props.layout.size.height = Length::Fill;

                                vec![cx.container(panel_props, move |cx| {
                                    let mut out: Vec<AnyElement> = Vec::new();
                                    {
                                        let mut ui = ImUiFacade {
                                            cx,
                                            out: &mut out,
                                            build_focus: Some(focus_state_for_build.clone()),
                                        };
                                        if let Some(f) = build.take() {
                                            f(&mut ui);
                                        }
                                    }
                                    out
                                })]
                            });
                            panel_id_for_focus = Some(modal.id);
                            modal
                        });

                        vec![backdrop, panel]
                    })
                })
            });

            let mut req = OverlayRequest::modal(
                overlay_id,
                trigger,
                open.clone(),
                OverlayPresence::instant(true),
                vec![layer],
            );
            req.root_name = Some(root_name);
            req.dismissible_on_dismiss_request = Some(on_dismiss_request);
            req.initial_focus = focus_state.get().or(panel_id_for_focus);
            OverlayController::request(cx, req);

            true
        })
    }

    fn menu_separator(&mut self) {
        self.separator();
    }

    fn menu_item(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        self.menu_item_ex(label, MenuItemOptions::default())
    }

    fn menu_item_close(
        &mut self,
        label: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.menu_item_ex(
            label,
            MenuItemOptions {
                close_popup: Some(open.clone()),
                ..Default::default()
            },
        )
    }

    fn menu_item_close_popup(&mut self, popup_id: &str, label: impl Into<Arc<str>>) -> ResponseExt {
        let open = self.popup_open_model(popup_id);
        self.menu_item_close(label, &open)
    }

    fn menu_item_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let label = label.into();
        self.menu_item_impl(label, options, SemanticsRole::MenuItem, None)
    }

    fn menu_item_checkbox_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let label = label.into();
        self.menu_item_impl(
            label,
            options,
            SemanticsRole::MenuItemCheckbox,
            Some(checked),
        )
    }

    fn menu_item_radio_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let label = label.into();
        self.menu_item_impl(label, options, SemanticsRole::MenuItemRadio, Some(checked))
    }

    fn menu_item_impl(
        &mut self,
        label: Arc<str>,
        options: MenuItemOptions,
        role: SemanticsRole,
        checked: Option<bool>,
    ) -> ResponseExt {
        let mut response = ResponseExt::default();

        let element = self.with_cx_mut(|cx| {
            let mut stack = fret_ui::element::StackProps::default();
            stack.layout.size.width = Length::Fill;
            stack.layout.size.height = Length::Auto;

            let mut panel = ContainerProps::default();
            panel.layout.size.width = Length::Fill;
            panel.layout.size.height = Length::Auto;
            panel.padding = Edges {
                left: Px(8.0),
                right: Px(8.0),
                top: Px(4.0),
                bottom: Px(4.0),
            };

            let close_popup = options.close_popup.clone();
            let test_id = options.test_id.clone();
            let enabled = options.enabled;
            let label_for_visuals = label.clone();
            let role = role;
            let checked = checked;

            cx.stack_props(stack, |cx| {
                let visuals = cx.container(panel, move |cx| {
                    let mut row = RowProps::default();
                    row.layout.size.width = Length::Fill;
                    row.layout.size.height = Length::Auto;
                    row.gap = Px(8.0);

                    let indicator = match (role, checked) {
                        (SemanticsRole::MenuItemCheckbox, Some(true)) => {
                            Some(Arc::from("\u{2713}"))
                        }
                        (SemanticsRole::MenuItemCheckbox, Some(false)) => Some(Arc::from(" ")),
                        (SemanticsRole::MenuItemRadio, Some(true)) => Some(Arc::from("\u{25CF}")),
                        (SemanticsRole::MenuItemRadio, Some(false)) => Some(Arc::from(" ")),
                        _ => None,
                    };

                    vec![cx.row(row, move |cx| {
                        let mut out: Vec<AnyElement> = Vec::new();
                        if let Some(indicator) = indicator.clone() {
                            out.push(cx.text(indicator));
                        }
                        out.push(cx.text(label_for_visuals.clone()));
                        out
                    })]
                });

                let mut props = PressableProps::default();
                props.enabled = enabled;
                props.focusable = enabled;
                props.layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        left: Some(Px(0.0)),
                        right: Some(Px(0.0)),
                        top: Some(Px(0.0)),
                        bottom: Some(Px(0.0)),
                    },
                    size: fret_ui::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                props.a11y = PressableA11y {
                    role: Some(role),
                    label: Some(label.clone()),
                    test_id,
                    checked,
                    ..Default::default()
                };

                let pressable = cx.pressable_with_id(props, |cx, state, id| {
                    cx.pressable_clear_on_pointer_down();
                    cx.pressable_clear_on_pointer_up();
                    cx.key_clear_on_key_down_for(id);

                    let close_popup = close_popup.clone();
                    cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                        if let Some(open) = close_popup.as_ref() {
                            let _ = host.update_model(open, |v| *v = false);
                        }
                        host.record_transient_event(acx, KEY_CLICKED);
                        host.notify(acx);
                    }));

                    let nav_items = cx
                        .inherited_state::<ImUiMenuNavState>()
                        .map(|st| st.items.clone());
                    if enabled {
                        if let Some(nav_items) = nav_items.as_ref() {
                            nav_items.borrow_mut().push(id);
                        }
                    }
                    if let Some(nav_items) = nav_items {
                        let item_id = id;
                        cx.key_on_key_down_for(
                            id,
                            Arc::new(move |host, acx, down| {
                                if down.repeat {
                                    return false;
                                }
                                if down.modifiers != fret_core::Modifiers::default() {
                                    return false;
                                }

                                let (dir, jump_to) = match down.key {
                                    KeyCode::ArrowDown => (1isize, None),
                                    KeyCode::ArrowUp => (-1isize, None),
                                    KeyCode::Home => (0isize, Some(0usize)),
                                    KeyCode::End => (0isize, Some(usize::MAX)),
                                    _ => return false,
                                };

                                let items = nav_items.borrow();
                                if items.is_empty() {
                                    return false;
                                }
                                let len = items.len();
                                let idx = items.iter().position(|id| *id == item_id);
                                let next_idx = if let Some(jump) = jump_to {
                                    if jump == usize::MAX {
                                        len - 1
                                    } else {
                                        jump.min(len - 1)
                                    }
                                } else {
                                    let current =
                                        idx.unwrap_or_else(|| if dir < 0 { len - 1 } else { 0 });
                                    ((current as isize + dir + len as isize) % len as isize)
                                        as usize
                                };

                                host.request_focus(items[next_idx]);
                                host.notify(acx);
                                true
                            }),
                        );
                    }

                    response.core.hovered = state.hovered;
                    response.core.pressed = state.pressed;
                    response.core.focused = state.focused;
                    response.nav_highlighted = state.focused
                        && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
                    response.id = Some(id);
                    response.core.clicked = cx.take_transient_for(id, KEY_CLICKED);
                    response.core.rect = cx.last_bounds_for_element(id);

                    Vec::<AnyElement>::new()
                });

                vec![visuals, pressable]
            })
        });

        self.add(element);
        response
    }

    fn begin_popup_context_menu(
        &mut self,
        id: &str,
        trigger: ResponseExt,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        self.begin_popup_context_menu_ex(id, trigger, PopupMenuOptions::default(), f)
    }

    fn begin_popup_context_menu_ex(
        &mut self,
        id: &str,
        trigger: ResponseExt,
        options: PopupMenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> bool {
        if trigger.context_menu_requested() {
            let anchor = trigger
                .context_menu_anchor()
                .map(|p| fret_core::Rect::new(p, Size::new(Px(1.0), Px(1.0))))
                .or(trigger.core.rect);
            if let Some(anchor) = anchor {
                self.open_popup_at(id, anchor);
            }
        }

        self.begin_popup_menu_ex(id, trigger.id, options, f)
    }

    fn button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        let label = label.into();
        let mut response = ResponseExt::default();

        let element = self.with_cx_mut(|cx| {
            let mut props = fret_ui::element::PressableProps::default();
            props.a11y = fret_ui::element::PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(label.clone()),
                ..Default::default()
            };

            cx.pressable_with_id(props, |cx, state, id| {
                cx.pressable_clear_on_pointer_down();
                cx.pressable_clear_on_pointer_move();
                cx.pressable_clear_on_pointer_up();
                cx.key_clear_on_key_down_for(id);

                let context_anchor_model = context_menu_anchor_model_for(cx, id);
                let context_anchor_model_for_report = context_anchor_model.clone();
                let long_press_signal_model = long_press_signal_model_for(cx, id);
                let long_press_signal_model_for_timer = long_press_signal_model.clone();
                let long_press_signal_model_for_down = long_press_signal_model.clone();
                let long_press_signal_model_for_move = long_press_signal_model.clone();
                let long_press_signal_model_for_up = long_press_signal_model.clone();

                cx.timer_on_timer_for(
                    id,
                    Arc::new(move |host, action_cx, token| {
                        emit_long_press_if_matching(
                            host,
                            action_cx,
                            &long_press_signal_model_for_timer,
                            token,
                        )
                    }),
                );

                cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                    host.record_transient_event(acx, KEY_CLICKED);
                    host.notify(acx);
                }));

                cx.key_on_key_down_for(
                    id,
                    Arc::new(move |host, acx, down| {
                        let is_menu_key = down.key == KeyCode::ContextMenu;
                        let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                        if !(is_menu_key || is_shift_f10) {
                            return false;
                        }

                        host.record_transient_event(acx, KEY_CONTEXT_MENU_REQUESTED);
                        host.notify(acx);
                        true
                    }),
                );

                cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                    if down.button != MouseButton::Left {
                        return PressablePointerDownResult::Continue;
                    }

                    arm_long_press_timer_for(host, acx, &long_press_signal_model_for_down);

                    if host.drag(down.pointer_id).is_none() {
                        host.begin_drag_with_kind(
                            down.pointer_id,
                            drag_kind_for_element(acx.target),
                            acx.window,
                            down.position,
                        );
                    }

                    PressablePointerDownResult::Continue
                }));

                let drag_threshold_sq = drag_threshold_sq_for(cx);
                cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
                    let mut cancel_long_press = false;

                    let Some(drag) = host.drag_mut(mv.pointer_id) else {
                        return false;
                    };
                    if drag.kind != drag_kind_for_element(acx.target)
                        || drag.source_window != acx.window
                    {
                        return false;
                    }

                    drag.current_window = acx.window;
                    drag.position = mv.position;

                    if !mv.buttons.left {
                        cancel_long_press = true;
                        if drag.dragging {
                            drag.phase = DragPhase::Canceled;
                            host.record_transient_event(acx, KEY_DRAG_STOPPED);
                        }
                        host.cancel_drag(mv.pointer_id);
                        if cancel_long_press {
                            cancel_long_press_timer_for(host, &long_press_signal_model_for_move);
                        }
                        host.notify(acx);
                        return false;
                    }

                    let d = point_sub(drag.position, drag.start_position);
                    let dist_sq = d.x.0 * d.x.0 + d.y.0 * d.y.0;
                    if !drag.dragging && dist_sq >= drag_threshold_sq {
                        cancel_long_press = true;
                        drag.dragging = true;
                        drag.phase = DragPhase::Dragging;
                        host.record_transient_event(acx, KEY_DRAG_STARTED);
                    }

                    if cancel_long_press {
                        cancel_long_press_timer_for(host, &long_press_signal_model_for_move);
                    }
                    host.notify(acx);
                    false
                }));

                cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                    if up.button == MouseButton::Left {
                        cancel_long_press_timer_for(host, &long_press_signal_model_for_up);
                    }

                    if let Some(drag) = host.drag(up.pointer_id)
                        && drag.kind == drag_kind_for_element(acx.target)
                        && drag.source_window == acx.window
                    {
                        if drag.dragging {
                            host.record_transient_event(acx, KEY_DRAG_STOPPED);
                        }
                        host.cancel_drag(up.pointer_id);
                        host.notify(acx);
                    }

                    if up.is_click && up.button == fret_core::MouseButton::Right {
                        let _ =
                            host.update_model(&context_anchor_model, |v| *v = Some(up.position));
                        host.record_transient_event(acx, KEY_SECONDARY_CLICKED);
                        host.record_transient_event(acx, KEY_CONTEXT_MENU_REQUESTED);
                        host.notify(acx);
                        return PressablePointerUpResult::SkipActivate;
                    }

                    if up.is_click
                        && up.button == fret_core::MouseButton::Left
                        && up.click_count == 2
                    {
                        host.record_transient_event(acx, KEY_DOUBLE_CLICKED);
                        host.notify(acx);
                    }

                    PressablePointerUpResult::Continue
                }));

                response.core.hovered = state.hovered;
                response.core.pressed = state.pressed;
                response.core.focused = state.focused;
                response.nav_highlighted = state.focused
                    && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
                response.id = Some(id);
                response.core.clicked = cx.take_transient_for(id, KEY_CLICKED);
                response.secondary_clicked = cx.take_transient_for(id, KEY_SECONDARY_CLICKED);
                response.double_clicked = cx.take_transient_for(id, KEY_DOUBLE_CLICKED);
                response.long_pressed = cx.take_transient_for(id, KEY_LONG_PRESSED);
                response.press_holding = cx
                    .read_model(
                        &long_press_signal_model,
                        fret_ui::Invalidation::Paint,
                        |_app, value| value.holding,
                    )
                    .unwrap_or(false);
                response.context_menu_requested =
                    cx.take_transient_for(id, KEY_CONTEXT_MENU_REQUESTED);
                response.context_menu_anchor = cx
                    .read_model(
                        &context_anchor_model_for_report,
                        fret_ui::Invalidation::Paint,
                        |_app, v| *v,
                    )
                    .unwrap_or(None);
                response.drag.started = cx.take_transient_for(id, KEY_DRAG_STARTED);
                response.drag.stopped = cx.take_transient_for(id, KEY_DRAG_STOPPED);
                response.drag.dragging = false;
                response.drag.delta = Point::default();
                response.drag.total = Point::default();
                let kind = drag_kind_for_element(id);
                let pointer_id = cx.app.find_drag_pointer_id(|d| {
                    d.kind == kind && d.source_window == cx.window && d.current_window == cx.window
                });
                let drag_snapshot = pointer_id.and_then(|pointer_id| {
                    cx.app
                        .drag(pointer_id)
                        .filter(|drag| drag.kind == kind)
                        .map(|drag| (drag.dragging, drag.position, drag.start_position))
                });
                if let Some((dragging, current, start)) = drag_snapshot {
                    response.drag.dragging = dragging;
                    let (delta, total) = cx.with_state_for(id, DragReportState::default, |st| {
                        let prev = st.last_position.unwrap_or(current);
                        st.last_position = Some(current);
                        (point_sub(current, prev), point_sub(current, start))
                    });
                    if dragging {
                        response.drag.delta = delta;
                        response.drag.total = total;
                    }
                } else {
                    cx.with_state_for(id, DragReportState::default, |st| {
                        st.last_position = None;
                    });
                }
                response.core.rect = cx.last_bounds_for_element(id);

                vec![cx.text(label.clone())]
            })
        });

        self.add(element);
        response
    }

    fn checkbox_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        let label = label.into();
        let model = model.clone();
        let mut response = ResponseExt::default();

        let element = self.with_cx_mut(|cx| {
            let value = cx
                .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| *v)
                .unwrap_or(false);

            let mut props = fret_ui::element::PressableProps::default();
            props.a11y = fret_ui::element::PressableA11y {
                role: Some(SemanticsRole::Checkbox),
                label: Some(label.clone()),
                checked: Some(value),
                ..Default::default()
            };

            cx.pressable_with_id(props, |cx, state, id| {
                cx.pressable_clear_on_pointer_down();
                cx.pressable_clear_on_pointer_move();
                cx.pressable_clear_on_pointer_up();
                cx.key_clear_on_key_down_for(id);

                let context_anchor_model = context_menu_anchor_model_for(cx, id);
                let context_anchor_model_for_report = context_anchor_model.clone();
                let long_press_signal_model = long_press_signal_model_for(cx, id);
                let long_press_signal_model_for_timer = long_press_signal_model.clone();
                let long_press_signal_model_for_down = long_press_signal_model.clone();
                let long_press_signal_model_for_move = long_press_signal_model.clone();
                let long_press_signal_model_for_up = long_press_signal_model.clone();

                cx.timer_on_timer_for(
                    id,
                    Arc::new(move |host, action_cx, token| {
                        emit_long_press_if_matching(
                            host,
                            action_cx,
                            &long_press_signal_model_for_timer,
                            token,
                        )
                    }),
                );

                let model = model.clone();
                cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                    let _ = host.update_model(&model, |v: &mut bool| *v = !*v);
                    host.record_transient_event(acx, KEY_CHANGED);
                    host.notify(acx);
                }));

                cx.key_on_key_down_for(
                    id,
                    Arc::new(move |host, acx, down| {
                        let is_menu_key = down.key == KeyCode::ContextMenu;
                        let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                        if !(is_menu_key || is_shift_f10) {
                            return false;
                        }

                        host.record_transient_event(acx, KEY_CONTEXT_MENU_REQUESTED);
                        host.notify(acx);
                        true
                    }),
                );

                cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                    if down.button != MouseButton::Left {
                        return PressablePointerDownResult::Continue;
                    }

                    arm_long_press_timer_for(host, acx, &long_press_signal_model_for_down);

                    if host.drag(down.pointer_id).is_none() {
                        host.begin_drag_with_kind(
                            down.pointer_id,
                            drag_kind_for_element(acx.target),
                            acx.window,
                            down.position,
                        );
                    }

                    PressablePointerDownResult::Continue
                }));

                let drag_threshold_sq = drag_threshold_sq_for(cx);
                cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
                    let mut cancel_long_press = false;

                    let Some(drag) = host.drag_mut(mv.pointer_id) else {
                        return false;
                    };
                    if drag.kind != drag_kind_for_element(acx.target)
                        || drag.source_window != acx.window
                    {
                        return false;
                    }

                    drag.current_window = acx.window;
                    drag.position = mv.position;

                    if !mv.buttons.left {
                        cancel_long_press = true;
                        if drag.dragging {
                            drag.phase = DragPhase::Canceled;
                            host.record_transient_event(acx, KEY_DRAG_STOPPED);
                        }
                        host.cancel_drag(mv.pointer_id);
                        if cancel_long_press {
                            cancel_long_press_timer_for(host, &long_press_signal_model_for_move);
                        }
                        host.notify(acx);
                        return false;
                    }

                    let d = point_sub(drag.position, drag.start_position);
                    let dist_sq = d.x.0 * d.x.0 + d.y.0 * d.y.0;
                    if !drag.dragging && dist_sq >= drag_threshold_sq {
                        cancel_long_press = true;
                        drag.dragging = true;
                        drag.phase = DragPhase::Dragging;
                        host.record_transient_event(acx, KEY_DRAG_STARTED);
                    }

                    if cancel_long_press {
                        cancel_long_press_timer_for(host, &long_press_signal_model_for_move);
                    }
                    host.notify(acx);
                    false
                }));

                cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                    if up.button == MouseButton::Left {
                        cancel_long_press_timer_for(host, &long_press_signal_model_for_up);
                    }

                    if let Some(drag) = host.drag(up.pointer_id)
                        && drag.kind == drag_kind_for_element(acx.target)
                        && drag.source_window == acx.window
                    {
                        if drag.dragging {
                            host.record_transient_event(acx, KEY_DRAG_STOPPED);
                        }
                        host.cancel_drag(up.pointer_id);
                        host.notify(acx);
                    }

                    if up.is_click && up.button == MouseButton::Right {
                        let _ =
                            host.update_model(&context_anchor_model, |v| *v = Some(up.position));
                        host.record_transient_event(acx, KEY_SECONDARY_CLICKED);
                        host.record_transient_event(acx, KEY_CONTEXT_MENU_REQUESTED);
                        host.notify(acx);
                        return PressablePointerUpResult::SkipActivate;
                    }

                    if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
                        host.record_transient_event(acx, KEY_DOUBLE_CLICKED);
                        host.notify(acx);
                    }

                    PressablePointerUpResult::Continue
                }));

                response.core.hovered = state.hovered;
                response.core.pressed = state.pressed;
                response.core.focused = state.focused;
                response.nav_highlighted = state.focused
                    && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
                response.id = Some(id);
                response.core.changed = cx.take_transient_for(id, KEY_CHANGED);
                response.secondary_clicked = cx.take_transient_for(id, KEY_SECONDARY_CLICKED);
                response.double_clicked = cx.take_transient_for(id, KEY_DOUBLE_CLICKED);
                response.long_pressed = cx.take_transient_for(id, KEY_LONG_PRESSED);
                response.press_holding = cx
                    .read_model(
                        &long_press_signal_model,
                        fret_ui::Invalidation::Paint,
                        |_app, value| value.holding,
                    )
                    .unwrap_or(false);
                response.context_menu_requested =
                    cx.take_transient_for(id, KEY_CONTEXT_MENU_REQUESTED);
                response.context_menu_anchor = cx
                    .read_model(
                        &context_anchor_model_for_report,
                        fret_ui::Invalidation::Paint,
                        |_app, v| *v,
                    )
                    .unwrap_or(None);
                response.drag.started = cx.take_transient_for(id, KEY_DRAG_STARTED);
                response.drag.stopped = cx.take_transient_for(id, KEY_DRAG_STOPPED);
                response.drag.dragging = false;
                response.drag.delta = Point::default();
                response.drag.total = Point::default();
                let kind = drag_kind_for_element(id);
                let pointer_id = cx.app.find_drag_pointer_id(|d| {
                    d.kind == kind && d.source_window == cx.window && d.current_window == cx.window
                });
                let drag_snapshot = pointer_id.and_then(|pointer_id| {
                    cx.app
                        .drag(pointer_id)
                        .filter(|drag| drag.kind == kind)
                        .map(|drag| (drag.dragging, drag.position, drag.start_position))
                });
                if let Some((dragging, current, start)) = drag_snapshot {
                    response.drag.dragging = dragging;
                    let (delta, total) = cx.with_state_for(id, DragReportState::default, |st| {
                        let prev = st.last_position.unwrap_or(current);
                        st.last_position = Some(current);
                        (point_sub(current, prev), point_sub(current, start))
                    });
                    if dragging {
                        response.drag.delta = delta;
                        response.drag.total = total;
                    }
                } else {
                    cx.with_state_for(id, DragReportState::default, |st| {
                        st.last_position = None;
                    });
                }
                response.core.rect = cx.last_bounds_for_element(id);

                let prefix: Arc<str> = if value {
                    Arc::from("[x] ")
                } else {
                    Arc::from("[ ] ")
                };
                vec![cx.text(Arc::from(format!("{prefix}{label}")))]
            })
        });

        self.add(element);
        response
    }

    fn toggle_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.toggle_model_ex(label, model, ToggleOptions::default())
    }

    fn toggle_model_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: ToggleOptions,
    ) -> ResponseExt {
        self.switch_model_ex(label, model, options)
    }

    fn switch_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.switch_model_ex(label, model, SwitchOptions::default())
    }

    fn switch_model_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: SwitchOptions,
    ) -> ResponseExt {
        let label = label.into();
        let model = model.clone();
        let mut response = ResponseExt::default();

        let element = self.with_cx_mut(|cx| {
            let value = cx
                .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| *v)
                .unwrap_or(false);

            let mut props = PressableProps::default();
            props.enabled = options.enabled;
            props.focusable = options.focusable;
            props.a11y = crate::primitives::switch::switch_a11y(
                options.a11y_label.clone().or_else(|| Some(label.clone())),
                value,
            );
            props.a11y.test_id = options.test_id.clone();

            cx.pressable_with_id(props, |cx, state, id| {
                cx.pressable_clear_on_pointer_down();
                cx.pressable_clear_on_pointer_move();
                cx.pressable_clear_on_pointer_up();

                let model_for_activate = model.clone();
                cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                    let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
                    host.record_transient_event(acx, KEY_CLICKED);
                    host.record_transient_event(acx, KEY_CHANGED);
                    host.notify(acx);
                }));

                response.core.hovered = state.hovered;
                response.core.pressed = state.pressed;
                response.core.focused = state.focused;
                response.nav_highlighted = state.focused
                    && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
                response.id = Some(id);
                response.core.clicked = cx.take_transient_for(id, KEY_CLICKED);
                response.core.changed = cx.take_transient_for(id, KEY_CHANGED);
                response.core.rect = cx.last_bounds_for_element(id);

                let prefix: Arc<str> = if value {
                    Arc::from("[on] ")
                } else {
                    Arc::from("[off] ")
                };
                vec![cx.text(Arc::from(format!("{prefix}{label}")))]
            })
        });

        self.add(element);
        response
    }

    fn slider_f32_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
    ) -> ResponseExt {
        self.slider_f32_model_ex(label, model, SliderOptions::default())
    }

    fn slider_f32_model_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
        options: SliderOptions,
    ) -> ResponseExt {
        let label = label.into();
        let model = model.clone();
        let mut response = ResponseExt::default();

        let min = options.min;
        let max = options.max;
        let step = options.step;

        let element = self.with_cx_mut(|cx| {
            let mut props = PressableProps::default();
            props.enabled = options.enabled;
            props.focusable = options.focusable;
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Px(Px(24.0));

            props.a11y = PressableA11y {
                role: Some(SemanticsRole::Slider),
                label: options.a11y_label.clone().or_else(|| Some(label.clone())),
                test_id: options.test_id.clone(),
                ..Default::default()
            };

            cx.pressable_with_id(props, |cx, state, id| {
                cx.pressable_clear_on_pointer_down();
                cx.pressable_clear_on_pointer_move();
                cx.pressable_clear_on_pointer_up();
                cx.key_clear_on_key_down_for(id);

                let model_for_down = model.clone();
                cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                    if down.button != MouseButton::Left {
                        return PressablePointerDownResult::Continue;
                    }

                    host.capture_pointer();
                    host.request_focus(acx.target);

                    let next =
                        slider_value_from_pointer(host.bounds(), down.position, min, max, step);
                    let mut changed = false;
                    let _ = host.update_model(&model_for_down, |value: &mut f32| {
                        let current = slider_clamp_and_snap(*value, min, max, step);
                        if (current - next).abs() > f32::EPSILON {
                            *value = next;
                            changed = true;
                        }
                    });
                    if changed {
                        host.record_transient_event(acx, KEY_CHANGED);
                        host.notify(acx);
                    }

                    PressablePointerDownResult::Continue
                }));

                let model_for_move = model.clone();
                cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
                    if !mv.buttons.left {
                        host.release_pointer_capture();
                        return false;
                    }

                    let next =
                        slider_value_from_pointer(host.bounds(), mv.position, min, max, step);
                    let mut changed = false;
                    let _ = host.update_model(&model_for_move, |value: &mut f32| {
                        let current = slider_clamp_and_snap(*value, min, max, step);
                        if (current - next).abs() > f32::EPSILON {
                            *value = next;
                            changed = true;
                        }
                    });
                    if changed {
                        host.record_transient_event(acx, KEY_CHANGED);
                        host.notify(acx);
                    }
                    changed
                }));

                cx.pressable_on_pointer_up(Arc::new(move |host, _acx, up| {
                    if up.button == MouseButton::Left {
                        host.release_pointer_capture();
                    }
                    PressablePointerUpResult::Continue
                }));

                let model_for_key = model.clone();
                cx.key_on_key_down_for(
                    id,
                    Arc::new(move |host, acx, down| {
                        let (min, max) = slider_normalize_range(min, max);
                        let step = slider_step_or_default(step);
                        let delta = match down.key {
                            KeyCode::ArrowLeft | KeyCode::ArrowDown => Some(-step),
                            KeyCode::ArrowRight | KeyCode::ArrowUp => Some(step),
                            KeyCode::PageDown => Some(-step * 10.0),
                            KeyCode::PageUp => Some(step * 10.0),
                            _ => None,
                        };

                        let mut changed = false;
                        let _ = host.update_model(&model_for_key, |value: &mut f32| {
                            let current = slider_clamp_and_snap(*value, min, max, step);
                            let next = match down.key {
                                KeyCode::Home => min,
                                KeyCode::End => max,
                                _ => {
                                    let Some(delta) = delta else {
                                        return;
                                    };
                                    slider_clamp_and_snap(current + delta, min, max, step)
                                }
                            };
                            if (current - next).abs() > f32::EPSILON {
                                *value = next;
                                changed = true;
                            }
                        });

                        if changed {
                            host.record_transient_event(acx, KEY_CHANGED);
                            host.notify(acx);
                        }

                        changed
                    }),
                );

                let current = cx
                    .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| {
                        slider_clamp_and_snap(*v, min, max, step)
                    })
                    .unwrap_or_else(|_| slider_clamp_and_snap(min, min, max, step));

                response.core.hovered = state.hovered;
                response.core.pressed = state.pressed;
                response.core.focused = state.focused;
                response.nav_highlighted = state.focused
                    && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
                response.id = Some(id);
                response.core.changed = cx.take_transient_for(id, KEY_CHANGED);
                response.core.rect = cx.last_bounds_for_element(id);

                vec![cx.text(Arc::from(format!("{label}: {current:.2}")))]
            })
        });

        self.add(element);
        response
    }

    fn select_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
    ) -> ResponseExt {
        self.select_model_ex(label, model, items, SelectOptions::default())
    }

    fn select_model_ex(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
        options: SelectOptions,
    ) -> ResponseExt {
        let label = label.into();
        let model = model.clone();

        let selected = self.with_cx_mut(|cx| {
            cx.read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
                .unwrap_or(None)
        });

        let selected_label: Arc<str> = selected
            .clone()
            .or_else(|| options.placeholder.clone())
            .unwrap_or_else(|| Arc::from("Select..."));
        let trigger_text: Arc<str> = Arc::from(format!("{label}: {selected_label}"));

        let popup_scope_id: Arc<str> = options.popup_scope_id.clone().unwrap_or_else(|| {
            let base = options
                .test_id
                .as_deref()
                .map(str::to_owned)
                .unwrap_or_else(|| label.as_ref().to_string());
            let mut normalized = String::with_capacity(base.len());
            for ch in base.chars() {
                if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                    normalized.push(ch);
                } else {
                    normalized.push('-');
                }
            }
            Arc::from(format!("imui-select-popup-{normalized}"))
        });
        let popup_open = self.popup_open_model(popup_scope_id.as_ref());

        let trigger = self.push_id(format!("{popup_scope_id}.trigger"), |ui| {
            ui.menu_item_ex(
                trigger_text,
                MenuItemOptions {
                    enabled: options.enabled,
                    test_id: options.test_id.clone(),
                    ..Default::default()
                },
            )
        });

        if options.enabled && trigger.clicked() {
            if let Some(anchor) = trigger.core.rect {
                self.open_popup_at(popup_scope_id.as_ref(), anchor);
            }
        }

        let selected_before = selected.clone();
        let model_for_pick = model.clone();
        let popup_open_for_items = popup_open.clone();
        let trigger_test_id = options.test_id.clone();
        let popup_opened = self.begin_popup_menu_ex(
            popup_scope_id.as_ref(),
            trigger.id,
            options.popup,
            move |ui| {
                for (index, item) in items.iter().enumerate() {
                    let checked = selected_before
                        .as_ref()
                        .is_some_and(|current| current.as_ref() == item.as_ref());
                    let item_test_id = trigger_test_id
                        .as_ref()
                        .map(|id| Arc::from(format!("{id}.option.{index}")));
                    let item_response = ui.menu_item_radio_ex(
                        item.clone(),
                        checked,
                        MenuItemOptions {
                            test_id: item_test_id,
                            ..Default::default()
                        },
                    );
                    if item_response.clicked() {
                        if !checked {
                            let next_value = Some(item.clone());
                            let _ = ui
                                .cx_mut()
                                .app
                                .models_mut()
                                .update(&model_for_pick, |value| *value = next_value.clone());
                        }
                        let _ = ui
                            .cx_mut()
                            .app
                            .models_mut()
                            .update(&popup_open_for_items, |value| *value = false);
                    }
                }
            },
        );

        if !options.enabled && popup_opened {
            self.close_popup(popup_scope_id.as_ref());
        }

        let selected_now = self.with_cx_mut(|cx| {
            cx.read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
                .unwrap_or(None)
        });

        let mut response = trigger;
        response.core.changed = response.id.is_some_and(|id| {
            self.with_cx_mut(|cx| model_value_changed_for(cx, id, selected_now.clone()))
        });
        response
    }

    fn input_text_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.input_text_model_ex(model, InputTextOptions::default())
    }

    fn input_text_model_ex(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: InputTextOptions,
    ) -> ResponseExt {
        let model = model.clone();
        let mut response = ResponseExt::default();

        let element = self.with_cx_mut(|cx| {
            cx.scope(|cx| {
                let id = cx.root_id();
                let current = cx
                    .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
                    .unwrap_or_default();

                response.id = Some(id);
                response.core.focused = cx.is_focused_element(id);
                response.core.changed = text_model_changed_for(cx, id, &current);
                response.core.rect = cx.last_bounds_for_element(id);

                let theme = fret_ui::Theme::global(&*cx.app).clone();
                let mut props = fret_ui::element::TextInputProps::new(model.clone());
                props.enabled = options.enabled;
                props.focusable = options.focusable;
                props.a11y_label = options.a11y_label.clone();
                props.a11y_role = options.a11y_role;
                props.test_id = options.test_id.clone();
                props.placeholder = options.placeholder.clone();
                props.submit_command = options.submit_command.clone();
                props.cancel_command = options.cancel_command.clone();
                props.chrome = crate::recipes::input::default_text_input_style(&theme);

                let mut element = cx.text_input(props);
                element.id = id;
                element
            })
        });

        self.add(element);
        response
    }

    fn textarea_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.textarea_model_ex(model, TextAreaOptions::default())
    }

    fn textarea_model_ex(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: TextAreaOptions,
    ) -> ResponseExt {
        let model = model.clone();
        let mut response = ResponseExt::default();

        let element = self.with_cx_mut(|cx| {
            cx.scope(|cx| {
                let id = cx.root_id();
                let current = cx
                    .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
                    .unwrap_or_default();

                response.id = Some(id);
                response.core.focused = cx.is_focused_element(id);
                response.core.changed = text_model_changed_for(cx, id, &current);
                response.core.rect = cx.last_bounds_for_element(id);

                let theme = fret_ui::Theme::global(&*cx.app).clone();
                let mut props = fret_ui::element::TextAreaProps::new(model.clone());
                props.enabled = options.enabled;
                props.focusable = options.focusable;
                props.a11y_label = options.a11y_label.clone();
                props.test_id = options.test_id.clone();
                props.min_height = options.min_height;
                props.chrome = default_text_area_style_from_theme(&theme);

                let mut element = cx.text_area(props);
                element.id = id;
                element
            })
        });

        self.add(element);
        response
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
    fn floating_window(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let _ = self.floating_window_show(id, title, initial_position, f);
    }

    fn window(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_show(id, title, initial_position, f)
    }

    fn floating_window_show(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_show(id, title.into(), None, initial_position, None, None, f)
    }

    /// Render a floating window controlled by an `open` model (ImGui-style `bool* p_open`).
    ///
    /// When the close button is activated, the model is set to `false`.
    fn floating_window_open(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let _ = self.floating_window_open_show(id, title, open, initial_position, f);
    }

    fn window_open(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_open_show(id, title, open, initial_position, f)
    }

    fn floating_window_open_show(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_show(
            id,
            title.into(),
            Some(open),
            initial_position,
            None,
            None,
            f,
        )
    }

    /// Render a resizable in-window floating window with a fixed initial size.
    ///
    /// This installs minimal resize handles (right edge, bottom edge, bottom-right corner).
    fn floating_window_resizable(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        initial_size: Size,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let _ = self.floating_window_resizable_show(id, title, initial_position, initial_size, f);
    }

    fn window_resizable(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        initial_size: Size,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_resizable_show(id, title, initial_position, initial_size, f)
    }

    fn floating_window_resizable_show(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        initial_size: Size,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_resizable_ex_show(
            id,
            title,
            initial_position,
            initial_size,
            FloatingWindowResizeOptions::default(),
            f,
        )
    }

    fn floating_window_resizable_ex(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        initial_size: Size,
        resize: FloatingWindowResizeOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let _ = self.floating_window_resizable_ex_show(
            id,
            title.into(),
            initial_position,
            initial_size,
            resize,
            f,
        );
    }

    fn floating_window_resizable_ex_show(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        initial_size: Size,
        resize: FloatingWindowResizeOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_show(
            id,
            title.into(),
            None,
            initial_position,
            Some(initial_size),
            Some(resize),
            f,
        )
    }

    /// Render a resizable floating window controlled by an `open` model.
    fn floating_window_open_resizable(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        initial_size: Size,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let _ = self.floating_window_open_resizable_show(
            id,
            title,
            open,
            initial_position,
            initial_size,
            f,
        );
    }

    fn window_open_resizable(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        initial_size: Size,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_open_resizable_show(id, title, open, initial_position, initial_size, f)
    }

    fn floating_window_open_resizable_show(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        initial_size: Size,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_open_resizable_ex_show(
            id,
            title,
            open,
            initial_position,
            initial_size,
            FloatingWindowResizeOptions::default(),
            f,
        )
    }

    fn floating_window_open_resizable_ex(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        initial_size: Size,
        resize: FloatingWindowResizeOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let _ = self.floating_window_open_resizable_ex_show(
            id,
            title.into(),
            open,
            initial_position,
            initial_size,
            resize,
            f,
        );
    }

    fn floating_window_open_resizable_ex_show(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        initial_size: Size,
        resize: FloatingWindowResizeOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_show(
            id,
            title.into(),
            Some(open),
            initial_position,
            Some(initial_size),
            Some(resize),
            f,
        )
    }

    fn floating_window_impl_show(
        &mut self,
        id: &str,
        title: Arc<str>,
        open: Option<&fret_runtime::Model<bool>>,
        initial_position: Point,
        initial_size: Option<Size>,
        resize: Option<FloatingWindowResizeOptions>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_on_area_show_with_options(
            id,
            title,
            open,
            initial_position,
            initial_size,
            resize,
            FloatingWindowOptions::default(),
            f,
        )
    }

    fn floating_window_impl_show_with_options(
        &mut self,
        id: &str,
        title: Arc<str>,
        open: Option<&fret_runtime::Model<bool>>,
        initial_position: Point,
        initial_size: Option<Size>,
        resize: Option<FloatingWindowResizeOptions>,
        options: FloatingWindowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_on_area_show_with_options(
            id,
            title,
            open,
            initial_position,
            initial_size,
            resize,
            options,
            f,
        )
    }

    fn floating_window_impl(
        &mut self,
        id: &str,
        title: Arc<str>,
        open: Option<&fret_runtime::Model<bool>>,
        initial_position: Point,
        initial_size: Option<Size>,
        resize: Option<FloatingWindowResizeOptions>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let _ = self.floating_window_impl_show(
            id,
            title,
            open,
            initial_position,
            initial_size,
            resize,
            f,
        );
    }

    fn floating_window_impl_on_area(
        &mut self,
        id: &str,
        title: Arc<str>,
        open: Option<&fret_runtime::Model<bool>>,
        initial_position: Point,
        initial_size: Option<Size>,
        resize: Option<FloatingWindowResizeOptions>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let _ = self.floating_window_impl_on_area_show(
            id,
            title,
            open,
            initial_position,
            initial_size,
            resize,
            f,
        );
    }

    fn floating_window_impl_on_area_show(
        &mut self,
        id: &str,
        title: Arc<str>,
        open: Option<&fret_runtime::Model<bool>>,
        initial_position: Point,
        initial_size: Option<Size>,
        resize: Option<FloatingWindowResizeOptions>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_on_area_show_with_options(
            id,
            title,
            open,
            initial_position,
            initial_size,
            resize,
            FloatingWindowOptions::default(),
            f,
        )
    }

    fn floating_window_impl_on_area_show_with_options(
        &mut self,
        id: &str,
        title: Arc<str>,
        open: Option<&fret_runtime::Model<bool>>,
        initial_position: Point,
        initial_size: Option<Size>,
        resize: Option<FloatingWindowResizeOptions>,
        options: FloatingWindowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        if let Some(open) = open {
            let is_open = self
                .with_cx_mut(|cx| cx.read_model(open, fret_ui::Invalidation::Paint, |_app, v| *v))
                .unwrap_or(false);
            if !is_open {
                return FloatingWindowResponse {
                    area: FloatingAreaResponse {
                        id: GlobalElementId(0),
                        rect: None,
                        position: initial_position,
                        dragging: false,
                        drag_kind: float_window_drag_kind_for_element(GlobalElementId(0)),
                    },
                    size: initial_size,
                    resizing: false,
                    collapsed: false,
                };
            }
        }

        let open_model = open.map(|m| m.clone());

        let chrome = Rc::new(Cell::new(FloatingWindowChromeResponse::default()));
        let chrome_out = chrome.clone();

        let area = self.floating_area_show_ex(
            id,
            initial_position,
            FloatingAreaOptions {
                test_id_prefix: "imui.float_window.window:",
                test_id: None,
                hit_test_passthrough: options.pointer_passthrough,
                no_inputs: options.no_inputs,
            },
            move |ui, area| {
                let chrome = floating_window_on_area::render_floating_window_in_area(
                    ui,
                    area,
                    id,
                    title,
                    open_model.clone(),
                    initial_position,
                    initial_size,
                    resize,
                    options,
                    f,
                );
                chrome_out.set(chrome);
            },
        );

        let chrome = chrome.get();
        FloatingWindowResponse {
            area,
            size: chrome.size,
            resizing: chrome.resizing,
            collapsed: chrome.collapsed,
        }
    }

    /// Render a floating window with explicit behavior flags.
    fn window_ex(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        options: FloatingWindowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_show_with_options(
            id,
            title.into(),
            None,
            initial_position,
            None,
            None,
            options,
            f,
        )
    }

    /// Render an `open`-model floating window with explicit behavior flags.
    fn window_open_ex(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        options: FloatingWindowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_show_with_options(
            id,
            title.into(),
            Some(open),
            initial_position,
            None,
            None,
            options,
            f,
        )
    }

    /// Render a resizable floating window with explicit behavior flags.
    fn window_resizable_ex(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        initial_position: Point,
        initial_size: Size,
        resize: FloatingWindowResizeOptions,
        options: FloatingWindowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_show_with_options(
            id,
            title.into(),
            None,
            initial_position,
            Some(initial_size),
            Some(resize),
            options,
            f,
        )
    }

    /// Render an `open`-model resizable floating window with explicit behavior flags.
    fn window_open_resizable_ex(
        &mut self,
        id: &str,
        title: impl Into<Arc<str>>,
        open: &fret_runtime::Model<bool>,
        initial_position: Point,
        initial_size: Size,
        resize: FloatingWindowResizeOptions,
        options: FloatingWindowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> FloatingWindowResponse {
        self.floating_window_impl_show_with_options(
            id,
            title.into(),
            Some(open),
            initial_position,
            Some(initial_size),
            Some(resize),
            options,
            f,
        )
    }

    fn floating_window_impl_legacy(
        &mut self,
        id: &str,
        title: Arc<str>,
        open: Option<&fret_runtime::Model<bool>>,
        initial_position: Point,
        initial_size: Option<Size>,
        resize: Option<FloatingWindowResizeOptions>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        if let Some(open) = open {
            let is_open = self
                .with_cx_mut(|cx| cx.read_model(open, fret_ui::Invalidation::Paint, |_app, v| *v))
                .unwrap_or(false);
            if !is_open {
                return;
            }
        }

        let element = self.with_cx_mut(|cx| {
            cx.named(id, |cx| {
                let open_model = open.map(|m| m.clone());

                let window_id = cx.root_id();
                if let Some(marker) = cx.inherited_state::<FloatWindowLayerMarker>() {
                    cx.with_state_for(marker.layer, FloatWindowLayerZOrder::default, |st| {
                        st.ensure_present(window_id);
                    });
                }
                let drag_kind = float_window_drag_kind_for_element(window_id);

                let drag_snapshot = cx
                    .app
                    .find_drag_pointer_id(|d| {
                        d.kind == drag_kind
                            && d.source_window == cx.window
                            && d.current_window == cx.window
                    })
                    .and_then(|pointer_id| cx.app.drag(pointer_id))
                    .filter(|drag| drag.kind == drag_kind)
                    .map(|drag| (drag.dragging, drag.position, drag.start_position));

                let resizable = initial_size.is_some();
                let resize_snapshot = if resizable {
                    [
                        FloatWindowResizeHandle::Left,
                        FloatWindowResizeHandle::Right,
                        FloatWindowResizeHandle::Top,
                        FloatWindowResizeHandle::Bottom,
                        FloatWindowResizeHandle::TopLeft,
                        FloatWindowResizeHandle::TopRight,
                        FloatWindowResizeHandle::BottomLeft,
                        FloatWindowResizeHandle::BottomRight,
                    ]
                    .into_iter()
                    .find_map(|handle| {
                        let kind = float_window_resize_kind_for_element(window_id, handle);
                        cx.app
                            .find_drag_pointer_id(|d| {
                                d.kind == kind
                                    && d.source_window == cx.window
                                    && d.current_window == cx.window
                            })
                            .and_then(|pointer_id| cx.app.drag(pointer_id))
                            .filter(|drag| drag.kind == kind)
                            .map(|drag| (handle, drag.dragging, drag.position, drag.start_position))
                    })
                } else {
                    None
                };

                let (position_after_drag, window_test_id) = cx.with_state_for(
                    window_id,
                    || FloatingAreaState {
                        position: initial_position,
                        last_drag_position: None,
                        test_id: Arc::from(format!("imui.float_window.window:{id}")),
                    },
                    |st| {
                        if let Some((dragging, current, start)) = drag_snapshot {
                            if dragging {
                                let prev = st.last_drag_position.unwrap_or(start);
                                st.position = point_add(st.position, point_sub(current, prev));
                                st.last_drag_position = Some(current);
                            } else {
                                st.last_drag_position = None;
                            }
                        } else {
                            st.last_drag_position = None;
                        }
                        (st.position, st.test_id.clone())
                    },
                );

                let (
                    position,
                    size,
                    title_bar_test_id,
                    close_button_test_id,
                    resize_left_test_id,
                    resize_right_test_id,
                    resize_top_test_id,
                    resize_bottom_test_id,
                    resize_top_left_test_id,
                    resize_top_right_test_id,
                    resize_bottom_left_test_id,
                    resize_corner_test_id,
                ) = cx.with_state_for(
                    window_id,
                    || FloatWindowState {
                        size: initial_size.unwrap_or_else(|| Size::new(Px(0.0), Px(0.0))),
                        last_resize_position: None,
                        title_bar_test_id: Arc::from(format!("imui.float_window.title_bar:{id}")),
                        close_button_test_id: Arc::from(format!("imui.float_window.close:{id}")),
                        resize_left_test_id: Arc::from(format!(
                            "imui.float_window.resize.left:{id}"
                        )),
                        resize_right_test_id: Arc::from(format!(
                            "imui.float_window.resize.right:{id}"
                        )),
                        resize_top_test_id: Arc::from(format!("imui.float_window.resize.top:{id}")),
                        resize_bottom_test_id: Arc::from(format!(
                            "imui.float_window.resize.bottom:{id}"
                        )),
                        resize_top_left_test_id: Arc::from(format!(
                            "imui.float_window.resize.top_left:{id}"
                        )),
                        resize_top_right_test_id: Arc::from(format!(
                            "imui.float_window.resize.top_right:{id}"
                        )),
                        resize_bottom_left_test_id: Arc::from(format!(
                            "imui.float_window.resize.bottom_left:{id}"
                        )),
                        resize_corner_test_id: Arc::from(format!(
                            "imui.float_window.resize.corner:{id}"
                        )),
                    },
                    |st| {
                        let mut position = position_after_drag;

                        let resize_cfg = resize.unwrap_or_default();
                        let min = resize_cfg.min_size;
                        let max = resize_cfg.max_size;
                        let clamp_width = |value: f32| -> Px {
                            let mut out = value.max(min.width.0);
                            if let Some(max) = max {
                                out = out.min(max.width.0);
                            }
                            Px(out)
                        };
                        let clamp_height = |value: f32| -> Px {
                            let mut out = value.max(min.height.0);
                            if let Some(max) = max {
                                out = out.min(max.height.0);
                            }
                            Px(out)
                        };

                        if resizable {
                            st.size.width = clamp_width(st.size.width.0);
                            st.size.height = clamp_height(st.size.height.0);
                        }

                        if let Some((handle, dragging, current, start)) = resize_snapshot {
                            if dragging {
                                let prev = st.last_resize_position.unwrap_or(start);
                                let delta = point_sub(current, prev);

                                match handle {
                                    FloatWindowResizeHandle::Left => {
                                        let right = Px(position.x.0 + st.size.width.0);
                                        let width = clamp_width(st.size.width.0 - delta.x.0);
                                        st.size.width = width;
                                        position.x = Px(right.0 - width.0);
                                    }
                                    FloatWindowResizeHandle::Right => {
                                        st.size.width = clamp_width(st.size.width.0 + delta.x.0);
                                    }
                                    FloatWindowResizeHandle::Top => {
                                        let bottom = Px(position.y.0 + st.size.height.0);
                                        let height = clamp_height(st.size.height.0 - delta.y.0);
                                        st.size.height = height;
                                        position.y = Px(bottom.0 - height.0);
                                    }
                                    FloatWindowResizeHandle::Bottom => {
                                        st.size.height = clamp_height(st.size.height.0 + delta.y.0);
                                    }
                                    FloatWindowResizeHandle::TopLeft => {
                                        let right = Px(position.x.0 + st.size.width.0);
                                        let bottom = Px(position.y.0 + st.size.height.0);

                                        let width = clamp_width(st.size.width.0 - delta.x.0);
                                        let height = clamp_height(st.size.height.0 - delta.y.0);
                                        st.size.width = width;
                                        st.size.height = height;
                                        position.x = Px(right.0 - width.0);
                                        position.y = Px(bottom.0 - height.0);
                                    }
                                    FloatWindowResizeHandle::TopRight => {
                                        let bottom = Px(position.y.0 + st.size.height.0);
                                        st.size.width = clamp_width(st.size.width.0 + delta.x.0);
                                        let height = clamp_height(st.size.height.0 - delta.y.0);
                                        st.size.height = height;
                                        position.y = Px(bottom.0 - height.0);
                                    }
                                    FloatWindowResizeHandle::BottomLeft => {
                                        let right = Px(position.x.0 + st.size.width.0);
                                        let width = clamp_width(st.size.width.0 - delta.x.0);
                                        st.size.width = width;
                                        position.x = Px(right.0 - width.0);
                                        st.size.height = clamp_height(st.size.height.0 + delta.y.0);
                                    }
                                    FloatWindowResizeHandle::BottomRight => {
                                        st.size.width = clamp_width(st.size.width.0 + delta.x.0);
                                        st.size.height = clamp_height(st.size.height.0 + delta.y.0);
                                    }
                                }

                                st.last_resize_position = Some(current);
                            } else {
                                st.last_resize_position = None;
                            }
                        } else {
                            st.last_resize_position = None;
                        }

                        (
                            position,
                            st.size,
                            st.title_bar_test_id.clone(),
                            st.close_button_test_id.clone(),
                            st.resize_left_test_id.clone(),
                            st.resize_right_test_id.clone(),
                            st.resize_top_test_id.clone(),
                            st.resize_bottom_test_id.clone(),
                            st.resize_top_left_test_id.clone(),
                            st.resize_top_right_test_id.clone(),
                            st.resize_bottom_left_test_id.clone(),
                            st.resize_corner_test_id.clone(),
                        )
                    },
                );

                if position != position_after_drag {
                    cx.with_state_for(
                        window_id,
                        || FloatingAreaState {
                            position,
                            last_drag_position: None,
                            test_id: window_test_id.clone(),
                        },
                        |st| {
                            st.position = position;
                        },
                    );
                }

                let (popover, border, muted) = {
                    let theme = fret_ui::Theme::global(&*cx.app);
                    (
                        theme.color_required("popover"),
                        theme.color_required("border"),
                        theme.color_required("muted"),
                    )
                };

                let mut window_props = ContainerProps::default();
                window_props.layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        left: Some(position.x),
                        top: Some(position.y),
                        ..Default::default()
                    },
                    overflow: Overflow::Visible,
                    ..Default::default()
                };
                if resizable {
                    window_props.layout.size.width = Length::Px(size.width);
                    window_props.layout.size.height = Length::Px(size.height);
                }
                window_props.background = Some(popover);
                window_props.border = Edges::all(Px(1.0));
                window_props.border_color = Some(border);
                window_props.corner_radii = Corners::all(Px(8.0));

                let mut window = cx.container(window_props, |cx| {
                    let mut col = ColumnProps::default();
                    col.layout.size.width = Length::Auto;
                    col.layout.size.height = Length::Auto;

                    let title_bar = cx.container(
                        {
                            let mut props = ContainerProps::default();
                            props.layout.size.width = Length::Fill;
                            props.layout.size.height = Length::Px(Px(24.0));
                            props.padding = Edges {
                                left: Px(8.0),
                                right: Px(6.0),
                                top: Px(4.0),
                                bottom: Px(4.0),
                            };
                            props.background = Some(muted);
                            props.border = Edges {
                                left: Px(0.0),
                                right: Px(0.0),
                                top: Px(0.0),
                                bottom: Px(1.0),
                            };
                            props.border_color = Some(border);
                            props.corner_radii = Corners {
                                top_left: Px(8.0),
                                top_right: Px(8.0),
                                bottom_left: Px(0.0),
                                bottom_right: Px(0.0),
                            };
                            props
                        },
                        |cx| {
                            let mut row = RowProps::default();
                            row.layout.size.width = Length::Fill;
                            row.layout.size.height = Length::Fill;
                            row.gap = Px(6.0);

                            let title = title.clone();
                            let title_bar_test_id = title_bar_test_id.clone();
                            let open_for_key = open_model.clone();
                            let window_id = window_id;
                            let drag_surface = cx.pointer_region(
                                PointerRegionProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Fill;
                                        layout
                                    },
                                    ..Default::default()
                                },
                                move |cx| {
                                    let region_id = cx.root_id();

                                    float_layer_bring_to_front_if_activated(
                                        cx, region_id, window_id,
                                    );

                                    cx.key_clear_on_key_down_for(region_id);
                                    if let Some(open) = open_for_key.clone() {
                                        cx.key_on_key_down_for(
                                            region_id,
                                            Arc::new(move |host, acx, down| {
                                                if down.key != KeyCode::Escape || down.repeat {
                                                    return false;
                                                }
                                                let _ = host.update_model(&open, |v: &mut bool| {
                                                    *v = false;
                                                });
                                                host.notify(acx);
                                                true
                                            }),
                                        );
                                    }

                                    cx.pointer_region_on_pointer_down(Arc::new(
                                        move |host, acx, down| {
                                            if down.button != MouseButton::Left {
                                                return false;
                                            }

                                            host.request_focus(acx.target);
                                            host.capture_pointer();
                                            if host.drag(down.pointer_id).is_none() {
                                                host.begin_drag_with_kind(
                                                    down.pointer_id,
                                                    drag_kind,
                                                    acx.window,
                                                    down.position,
                                                );
                                            }
                                            host.record_transient_event(
                                                acx,
                                                KEY_FLOAT_WINDOW_ACTIVATE,
                                            );
                                            host.notify(acx);
                                            false
                                        },
                                    ));

                                    let drag_threshold_sq = drag_threshold_sq_for(cx);
                                    cx.pointer_region_on_pointer_move(Arc::new(
                                        move |host, acx, mv| {
                                            let Some(drag) = host.drag_mut(mv.pointer_id) else {
                                                return false;
                                            };
                                            if drag.kind != drag_kind
                                                || drag.source_window != acx.window
                                            {
                                                return false;
                                            }

                                            drag.current_window = acx.window;
                                            drag.position = mv.position;

                                            if !mv.buttons.left {
                                                drag.phase = DragPhase::Canceled;
                                                host.cancel_drag(mv.pointer_id);
                                                host.release_pointer_capture();
                                                host.notify(acx);
                                                return false;
                                            }

                                            let d = point_sub(drag.position, drag.start_position);
                                            let dist_sq = d.x.0 * d.x.0 + d.y.0 * d.y.0;
                                            if !drag.dragging && dist_sq >= drag_threshold_sq {
                                                drag.dragging = true;
                                                drag.phase = DragPhase::Dragging;
                                            }

                                            host.notify(acx);
                                            false
                                        },
                                    ));

                                    cx.pointer_region_on_pointer_up(Arc::new(
                                        move |host, acx, up| {
                                            if let Some(drag) = host.drag(up.pointer_id)
                                                && drag.kind == drag_kind
                                                && drag.source_window == acx.window
                                            {
                                                host.cancel_drag(up.pointer_id);
                                            }
                                            host.release_pointer_capture();
                                            host.notify(acx);
                                            false
                                        },
                                    ));

                                    vec![
                                        cx.text(title.clone()).attach_semantics(
                                            fret_ui::element::SemanticsDecoration::default()
                                                .test_id(title_bar_test_id.clone()),
                                        ),
                                    ]
                                },
                            );

                            let close = open_model.clone().map(|open| {
                                let mut props = PressableProps::default();
                                props.a11y = PressableA11y {
                                    role: Some(SemanticsRole::Button),
                                    label: Some(Arc::from("Close")),
                                    test_id: Some(close_button_test_id.clone()),
                                    ..Default::default()
                                };
                                props.layout.size.width = Length::Px(Px(20.0));
                                props.layout.size.height = Length::Px(Px(20.0));
                                cx.pressable(props, move |cx, _state| {
                                    cx.pressable_on_activate(Arc::new(
                                        move |host, acx, _reason| {
                                            let _ = host.update_model(&open, |v: &mut bool| {
                                                *v = false;
                                            });
                                            host.notify(acx);
                                        },
                                    ));
                                    vec![cx.text("\u{00D7}")]
                                })
                            });

                            vec![cx.row(row, move |_cx| {
                                let mut out = vec![drag_surface];
                                if let Some(close) = close {
                                    out.push(close);
                                }
                                out
                            })]
                        },
                    );

                    let content = cx.container(
                        {
                            let mut props = ContainerProps::default();
                            props.padding = Edges::all(Px(8.0));
                            props
                        },
                        |cx| {
                            let mut out = Vec::new();
                            {
                                let mut ui = ImUiFacade {
                                    cx,
                                    out: &mut out,
                                    build_focus: None,
                                };
                                f(&mut ui);
                            }
                            let mut content_col = ColumnProps::default();
                            content_col.layout.size.width = Length::Fill;
                            vec![cx.column(content_col, |_cx| out)]
                        },
                    );

                    let body = cx.column(col, |_cx| vec![title_bar, content]);
                    if !resizable {
                        return vec![body];
                    }

                    let window_id = window_id;
                    let mut resize_handle = |handle: FloatWindowResizeHandle, test_id: Arc<str>| {
                        let (cursor, layout) = match handle {
                            FloatWindowResizeHandle::Left => {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Absolute;
                                layout.inset = InsetStyle {
                                    left: Some(Px(0.0)),
                                    top: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    ..Default::default()
                                };
                                layout.size.width = Length::Px(Px(6.0));
                                layout.size.height = Length::Fill;
                                (CursorIcon::ColResize, layout)
                            }
                            FloatWindowResizeHandle::Right => {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Absolute;
                                layout.inset = InsetStyle {
                                    right: Some(Px(0.0)),
                                    top: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    ..Default::default()
                                };
                                layout.size.width = Length::Px(Px(6.0));
                                layout.size.height = Length::Fill;
                                (CursorIcon::ColResize, layout)
                            }
                            FloatWindowResizeHandle::Top => {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Absolute;
                                layout.inset = InsetStyle {
                                    left: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    top: Some(Px(0.0)),
                                    ..Default::default()
                                };
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Px(Px(6.0));
                                (CursorIcon::RowResize, layout)
                            }
                            FloatWindowResizeHandle::Bottom => {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Absolute;
                                layout.inset = InsetStyle {
                                    left: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    ..Default::default()
                                };
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Px(Px(6.0));
                                (CursorIcon::RowResize, layout)
                            }
                            FloatWindowResizeHandle::TopLeft => {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Absolute;
                                layout.inset = InsetStyle {
                                    left: Some(Px(0.0)),
                                    top: Some(Px(0.0)),
                                    ..Default::default()
                                };
                                layout.size.width = Length::Px(Px(10.0));
                                layout.size.height = Length::Px(Px(10.0));
                                (CursorIcon::NwseResize, layout)
                            }
                            FloatWindowResizeHandle::TopRight => {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Absolute;
                                layout.inset = InsetStyle {
                                    right: Some(Px(0.0)),
                                    top: Some(Px(0.0)),
                                    ..Default::default()
                                };
                                layout.size.width = Length::Px(Px(10.0));
                                layout.size.height = Length::Px(Px(10.0));
                                (CursorIcon::NeswResize, layout)
                            }
                            FloatWindowResizeHandle::BottomLeft => {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Absolute;
                                layout.inset = InsetStyle {
                                    left: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    ..Default::default()
                                };
                                layout.size.width = Length::Px(Px(10.0));
                                layout.size.height = Length::Px(Px(10.0));
                                (CursorIcon::NeswResize, layout)
                            }
                            FloatWindowResizeHandle::BottomRight => {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Absolute;
                                layout.inset = InsetStyle {
                                    right: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    ..Default::default()
                                };
                                layout.size.width = Length::Px(Px(10.0));
                                layout.size.height = Length::Px(Px(10.0));
                                (CursorIcon::NwseResize, layout)
                            }
                        };

                        let kind = float_window_resize_kind_for_element(window_id, handle);
                        cx.pointer_region(
                            PointerRegionProps {
                                layout,
                                ..Default::default()
                            },
                            move |cx| {
                                let region_id = cx.root_id();
                                float_layer_bring_to_front_if_activated(cx, region_id, window_id);

                                cx.pointer_region_clear_on_pointer_down();
                                cx.pointer_region_clear_on_pointer_move();
                                cx.pointer_region_clear_on_pointer_up();

                                cx.pointer_region_on_pointer_down(Arc::new(
                                    move |host, acx, down| {
                                        if down.button != MouseButton::Left {
                                            return false;
                                        }

                                        host.request_focus(acx.target);
                                        host.capture_pointer();
                                        host.set_cursor_icon(cursor);
                                        if host.drag(down.pointer_id).is_none() {
                                            host.begin_drag_with_kind(
                                                down.pointer_id,
                                                kind,
                                                acx.window,
                                                down.position,
                                            );
                                        }
                                        host.record_transient_event(acx, KEY_FLOAT_WINDOW_ACTIVATE);
                                        host.notify(acx);
                                        false
                                    },
                                ));

                                cx.pointer_region_on_pointer_move(Arc::new(
                                    move |host, acx, mv| {
                                        host.set_cursor_icon(cursor);

                                        let Some(drag) = host.drag_mut(mv.pointer_id) else {
                                            return false;
                                        };
                                        if drag.kind != kind || drag.source_window != acx.window {
                                            return false;
                                        }

                                        drag.current_window = acx.window;
                                        drag.position = mv.position;
                                        drag.dragging = true;
                                        drag.phase = DragPhase::Dragging;

                                        if !mv.buttons.left {
                                            drag.phase = DragPhase::Canceled;
                                            host.cancel_drag(mv.pointer_id);
                                            host.release_pointer_capture();
                                            host.notify(acx);
                                            return false;
                                        }

                                        host.notify(acx);
                                        false
                                    },
                                ));

                                cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
                                    if let Some(drag) = host.drag(up.pointer_id)
                                        && drag.kind == kind
                                        && drag.source_window == acx.window
                                    {
                                        host.cancel_drag(up.pointer_id);
                                    }
                                    host.release_pointer_capture();
                                    host.notify(acx);
                                    false
                                }));

                                Vec::new()
                            },
                        )
                        .attach_semantics(
                            fret_ui::element::SemanticsDecoration::default()
                                .test_id(test_id.clone()),
                        )
                    };

                    let left =
                        resize_handle(FloatWindowResizeHandle::Left, resize_left_test_id.clone());
                    let right =
                        resize_handle(FloatWindowResizeHandle::Right, resize_right_test_id.clone());
                    let top =
                        resize_handle(FloatWindowResizeHandle::Top, resize_top_test_id.clone());
                    let bottom = resize_handle(
                        FloatWindowResizeHandle::Bottom,
                        resize_bottom_test_id.clone(),
                    );
                    let top_left = resize_handle(
                        FloatWindowResizeHandle::TopLeft,
                        resize_top_left_test_id.clone(),
                    );
                    let top_right = resize_handle(
                        FloatWindowResizeHandle::TopRight,
                        resize_top_right_test_id.clone(),
                    );
                    let bottom_left = resize_handle(
                        FloatWindowResizeHandle::BottomLeft,
                        resize_bottom_left_test_id.clone(),
                    );
                    let corner = resize_handle(
                        FloatWindowResizeHandle::BottomRight,
                        resize_corner_test_id.clone(),
                    );

                    vec![cx.stack(|_cx| {
                        vec![
                            body,
                            left,
                            right,
                            top,
                            bottom,
                            top_left,
                            top_right,
                            bottom_left,
                            corner,
                        ]
                    })]
                });
                // `cx.container(...)` introduces a fresh scoped id; normalize the outer window element
                // id back to the named scope id so z-order state can track windows by `window_id`.
                window.id = window_id;
                window.attach_semantics(
                    fret_ui::element::SemanticsDecoration::default().test_id(window_test_id),
                )
            })
        });

        self.add(element);
    }
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterImUiFacadeExt<H> for W {}

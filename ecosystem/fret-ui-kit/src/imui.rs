//! Optional integration helpers for immediate-mode authoring frontends.
//!
//! This module lives in `fret-ui-kit` (not `fret-imui`) to preserve dependency direction:
//!
//! - `fret-imui` stays policy-light and depends only on `fret-ui` (+ `fret-authoring` contract).
//! - `fret-ui-kit` can optionally provide bridges that allow `UiBuilder<T>` patch vocabulary to be
//!   used from immediate-style control flow.

use std::sync::Arc;

use fret_authoring::Response;
use fret_authoring::UiWriter;
use fret_core::{Corners, CursorIcon, Edges, KeyCode, MouseButton, Point, Px, SemanticsRole, Size};
use fret_runtime::DragPhase;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow,
    PointerRegionProps, PositionStyle, PressableA11y, PressableProps, RowProps,
};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::UiBuilder;
use crate::{UiIntoElement, UiPatchTarget};

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
    pub secondary_clicked: bool,
    pub double_clicked: bool,
    pub context_menu_requested: bool,
    pub drag: DragResponse,
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

    pub fn context_menu_requested(self) -> bool {
        self.context_menu_requested
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
const KEY_CONTEXT_MENU_REQUESTED: u64 = fnv1a64(b"fret-ui-kit.imui.context_menu_requested.v1");
const KEY_DRAG_STARTED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_started.v1");
const KEY_DRAG_STOPPED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_stopped.v1");

const DRAG_THRESHOLD_PX: f32 = 4.0;
const DRAG_KIND_MASK: u64 = 0x8000_0000_0000_0000;

fn drag_kind_for_element(element: GlobalElementId) -> fret_runtime::DragKindId {
    fret_runtime::DragKindId(DRAG_KIND_MASK | element.0)
}

fn point_sub(a: Point, b: Point) -> Point {
    Point::new(Px(a.x.0 - b.x.0), Px(a.y.0 - b.y.0))
}

fn point_add(a: Point, b: Point) -> Point {
    Point::new(Px(a.x.0 + b.x.0), Px(a.y.0 + b.y.0))
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

const KEY_FLOAT_WINDOW_ACTIVATE: u64 = fnv1a64(b"fret-ui-kit.imui.float_window.activate.v1");

/// A minimal `UiWriter` implementation used by facade container helpers (e.g. floating windows).
///
/// This mirrors the `fret-imui::ImUi` pattern without depending on the `fret-imui` crate.
pub struct ImUiFacade<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    out: &'cx mut Vec<AnyElement>,
}

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn cx_mut(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
    }

    pub fn add(&mut self, element: AnyElement) {
        self.out.push(element);
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
struct FloatWindowState {
    position: Point,
    last_drag_position: Option<Point>,
    size: Size,
    last_resize_position: Option<Point>,
    window_test_id: Arc<str>,
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

#[derive(Debug, Clone, Copy)]
struct FloatWindowLayerMarker {
    layer: GlobalElementId,
}

#[derive(Debug, Default)]
struct FloatWindowLayerZOrder {
    order: Vec<GlobalElementId>,
}

impl FloatWindowLayerZOrder {
    fn ensure_present(&mut self, window: GlobalElementId) {
        if self.order.contains(&window) {
            return;
        }
        self.order.push(window);
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
    }
}

/// Immediate-mode facade helpers for any authoring frontend that implements `UiWriter`.
///
/// This is intentionally a small convenience layer. It aims to feel closer to egui/imgui while
/// still compiling down to Fret's declarative element tree and delegating complex policy to
/// higher-level components.
pub trait UiWriterImUiFacadeExt<H: UiHost>: UiWriter<H> {
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

    /// Render a window-scoped floating window layer that manages z-order (bring-to-front).
    ///
    /// Notes:
    /// - This is an opt-in container; a plain `floating_window(...)` call sequence keeps call-order z.
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
                    };
                    f(&mut ui);
                }

                let mut z_order =
                    cx.with_state_for(layer_id, FloatWindowLayerZOrder::default, |st| {
                        for w in windows.iter() {
                            st.ensure_present(w.id);
                        }
                        st.order.clone()
                    });

                // Ensure we never rank missing windows above present ones if the order vector is stale.
                z_order.retain(|id| windows.iter().any(|w| w.id == *id));

                let mut indexed: Vec<(usize, usize, AnyElement)> = windows
                    .into_iter()
                    .enumerate()
                    .map(|(original, w)| {
                        let idx = z_order
                            .iter()
                            .position(|id| *id == w.id)
                            .unwrap_or(usize::MAX);
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

                cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
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
                        if drag.dragging {
                            drag.phase = DragPhase::Canceled;
                            host.record_transient_event(acx, KEY_DRAG_STOPPED);
                        }
                        host.cancel_drag(mv.pointer_id);
                        host.notify(acx);
                        return false;
                    }

                    let d = point_sub(drag.position, drag.start_position);
                    let dist_sq = d.x.0 * d.x.0 + d.y.0 * d.y.0;
                    if !drag.dragging && dist_sq >= DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX {
                        drag.dragging = true;
                        drag.phase = DragPhase::Dragging;
                        host.record_transient_event(acx, KEY_DRAG_STARTED);
                    }

                    host.notify(acx);
                    false
                }));

                cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
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
                response.core.clicked = cx.take_transient_for(id, KEY_CLICKED);
                response.secondary_clicked = cx.take_transient_for(id, KEY_SECONDARY_CLICKED);
                response.double_clicked = cx.take_transient_for(id, KEY_DOUBLE_CLICKED);
                response.context_menu_requested =
                    cx.take_transient_for(id, KEY_CONTEXT_MENU_REQUESTED);
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

                cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
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
                        if drag.dragging {
                            drag.phase = DragPhase::Canceled;
                            host.record_transient_event(acx, KEY_DRAG_STOPPED);
                        }
                        host.cancel_drag(mv.pointer_id);
                        host.notify(acx);
                        return false;
                    }

                    let d = point_sub(drag.position, drag.start_position);
                    let dist_sq = d.x.0 * d.x.0 + d.y.0 * d.y.0;
                    if !drag.dragging && dist_sq >= DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX {
                        drag.dragging = true;
                        drag.phase = DragPhase::Dragging;
                        host.record_transient_event(acx, KEY_DRAG_STARTED);
                    }

                    host.notify(acx);
                    false
                }));

                cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
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
                response.core.changed = cx.take_transient_for(id, KEY_CHANGED);
                response.secondary_clicked = cx.take_transient_for(id, KEY_SECONDARY_CLICKED);
                response.double_clicked = cx.take_transient_for(id, KEY_DOUBLE_CLICKED);
                response.context_menu_requested =
                    cx.take_transient_for(id, KEY_CONTEXT_MENU_REQUESTED);
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
        self.floating_window_impl(id, title.into(), None, initial_position, None, None, f);
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
        self.floating_window_impl(
            id,
            title.into(),
            Some(open),
            initial_position,
            None,
            None,
            f,
        );
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
        self.floating_window_resizable_ex(
            id,
            title,
            initial_position,
            initial_size,
            FloatingWindowResizeOptions::default(),
            f,
        );
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
        self.floating_window_impl(
            id,
            title.into(),
            None,
            initial_position,
            Some(initial_size),
            Some(resize),
            f,
        );
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
        self.floating_window_open_resizable_ex(
            id,
            title,
            open,
            initial_position,
            initial_size,
            FloatingWindowResizeOptions::default(),
            f,
        );
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
        self.floating_window_impl(
            id,
            title.into(),
            Some(open),
            initial_position,
            Some(initial_size),
            Some(resize),
            f,
        );
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

                let (
                    position,
                    size,
                    window_test_id,
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
                        position: initial_position,
                        last_drag_position: None,
                        size: initial_size.unwrap_or_else(|| Size::new(Px(0.0), Px(0.0))),
                        last_resize_position: None,
                        window_test_id: Arc::from(format!("imui.float_window.window:{id}")),
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

                        if let Some((handle, dragging, current, start)) = resize_snapshot {
                            if dragging {
                                let prev = st.last_resize_position.unwrap_or(start);
                                let delta = point_sub(current, prev);

                                match handle {
                                    FloatWindowResizeHandle::Left => {
                                        let right = Px(st.position.x.0 + st.size.width.0);
                                        let width = clamp_width(st.size.width.0 - delta.x.0);
                                        st.size.width = width;
                                        st.position.x = Px(right.0 - width.0);
                                    }
                                    FloatWindowResizeHandle::Right => {
                                        st.size.width = clamp_width(st.size.width.0 + delta.x.0);
                                    }
                                    FloatWindowResizeHandle::Top => {
                                        let bottom = Px(st.position.y.0 + st.size.height.0);
                                        let height = clamp_height(st.size.height.0 - delta.y.0);
                                        st.size.height = height;
                                        st.position.y = Px(bottom.0 - height.0);
                                    }
                                    FloatWindowResizeHandle::Bottom => {
                                        st.size.height = clamp_height(st.size.height.0 + delta.y.0);
                                    }
                                    FloatWindowResizeHandle::TopLeft => {
                                        let right = Px(st.position.x.0 + st.size.width.0);
                                        let bottom = Px(st.position.y.0 + st.size.height.0);

                                        let width = clamp_width(st.size.width.0 - delta.x.0);
                                        let height = clamp_height(st.size.height.0 - delta.y.0);
                                        st.size.width = width;
                                        st.size.height = height;
                                        st.position.x = Px(right.0 - width.0);
                                        st.position.y = Px(bottom.0 - height.0);
                                    }
                                    FloatWindowResizeHandle::TopRight => {
                                        let bottom = Px(st.position.y.0 + st.size.height.0);
                                        st.size.width = clamp_width(st.size.width.0 + delta.x.0);
                                        let height = clamp_height(st.size.height.0 - delta.y.0);
                                        st.size.height = height;
                                        st.position.y = Px(bottom.0 - height.0);
                                    }
                                    FloatWindowResizeHandle::BottomLeft => {
                                        let right = Px(st.position.x.0 + st.size.width.0);
                                        let width = clamp_width(st.size.width.0 - delta.x.0);
                                        st.size.width = width;
                                        st.position.x = Px(right.0 - width.0);
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
                            st.position,
                            st.size,
                            st.window_test_id.clone(),
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

                                    if cx.take_transient_for(region_id, KEY_FLOAT_WINDOW_ACTIVATE) {
                                        if let Some(marker) =
                                            cx.inherited_state::<FloatWindowLayerMarker>()
                                        {
                                            cx.with_state_for(
                                                marker.layer,
                                                FloatWindowLayerZOrder::default,
                                                |st| {
                                                    st.bring_to_front(window_id);
                                                },
                                            );
                                        }
                                    }

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
                                            if !drag.dragging
                                                && dist_sq >= DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX
                                            {
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
                                let mut ui = ImUiFacade { cx, out: &mut out };
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

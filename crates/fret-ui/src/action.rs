use crate::UiHost;
use fret_core::{
    AppWindowId, Axis, CursorIcon, InternalDragKind, KeyCode, Modifiers, MouseButton, Point,
    PointerId, PointerType,
};
use fret_runtime::{
    CommandId, DefaultAction, DragHost, DragKindId, DragSession, Effect, Model, ModelStore, TickId,
    TimerToken, WeakModel,
};
use std::any::Any;
use std::sync::Arc;

/// Context passed to component-owned action handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionCx {
    pub window: AppWindowId,
    pub target: crate::GlobalElementId,
}

/// Why an element was activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivateReason {
    Pointer,
    Keyboard,
}

/// Result of a component-owned `Pressable` pointer down hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressablePointerDownResult {
    /// Continue with the default `Pressable` pointer down behavior (focus, capture, pressed state).
    Continue,
    /// Skip the default behavior but allow the event to keep propagating.
    SkipDefault,
    /// Skip the default behavior and stop propagation at this pressable.
    SkipDefaultAndStopPropagation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressablePointerUpResult {
    /// Continue with the default `Pressable` pointer-up behavior (activate when pressed+hovered).
    Continue,
    /// Skip the activation step (but still run default cleanup like releasing capture).
    SkipActivate,
}

/// Why an overlay is requesting dismissal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DismissReason {
    Escape,
    OutsidePress {
        pointer: Option<OutsidePressCx>,
    },
    /// Focus moved outside the dismissable layer subtree (Radix `onFocusOutside` outcome).
    FocusOutside,
    /// The trigger (or another registered subtree) was scrolled.
    ///
    /// This is used for Radix-aligned tooltip semantics: a tooltip should close when its trigger
    /// is inside the scroll target that received a wheel/scroll gesture.
    Scroll,
}

/// Context passed to overlay dismissal handlers.
///
/// This mirrors the DOM/Radix contract where `onInteractOutside` / `onPointerDownOutside` /
/// `onFocusOutside` may "prevent default" to keep the overlay open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DismissRequestCx {
    pub reason: DismissReason,
    default_prevented: bool,
}

impl DismissRequestCx {
    pub fn new(reason: DismissReason) -> Self {
        Self {
            reason,
            default_prevented: false,
        }
    }

    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    pub fn default_prevented(&self) -> bool {
        self.default_prevented
    }
}

/// Context passed to auto-focus handlers.
///
/// This mirrors the DOM/Radix contract where `onOpenAutoFocus` / `onCloseAutoFocus` may "prevent
/// default" to take full control of focus movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoFocusRequestCx {
    default_prevented: bool,
}

impl AutoFocusRequestCx {
    pub fn new() -> Self {
        Self {
            default_prevented: false,
        }
    }

    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    pub fn default_prevented(&self) -> bool {
        self.default_prevented
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutsidePressCx {
    pub pointer_id: PointerId,
    pub pointer_type: PointerType,
    pub button: MouseButton,
    pub modifiers: Modifiers,
    pub click_count: u8,
}

/// Pointer down payload for component-owned pointer handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerDownCx {
    pub pointer_id: PointerId,
    pub position: Point,
    pub tick_id: TickId,
    /// Pixels-per-point (a.k.a. window scale factor) for `position`.
    ///
    /// This is required for DPI-stable interactions (e.g. viewport tools, gizmos).
    pub pixels_per_point: f32,
    pub button: MouseButton,
    pub modifiers: Modifiers,
    /// See `PointerEvent::{Down,Up}.click_count` for normalization rules.
    pub click_count: u8,
    pub pointer_type: PointerType,
}

/// Pointer move payload for component-owned pointer handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerMoveCx {
    pub pointer_id: PointerId,
    pub position: Point,
    pub tick_id: TickId,
    /// Pixels-per-point (a.k.a. window scale factor) for `position`.
    pub pixels_per_point: f32,
    pub buttons: fret_core::MouseButtons,
    pub modifiers: Modifiers,
    pub pointer_type: PointerType,
}

/// Wheel payload for component-owned wheel handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelCx {
    pub pointer_id: PointerId,
    pub position: Point,
    pub tick_id: TickId,
    /// Pixels-per-point (a.k.a. window scale factor) for `position`.
    pub pixels_per_point: f32,
    pub delta: Point,
    pub modifiers: Modifiers,
    pub pointer_type: PointerType,
}

/// Pinch (magnify) gesture payload for component-owned pinch handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PinchGestureCx {
    pub pointer_id: PointerId,
    pub position: Point,
    pub tick_id: TickId,
    /// Pixels-per-point (a.k.a. window scale factor) for `position`.
    pub pixels_per_point: f32,
    /// Positive for magnification (zoom in) and negative for shrinking (zoom out).
    ///
    /// This may be NaN depending on the platform backend; callers should guard accordingly.
    pub delta: f32,
    pub modifiers: Modifiers,
    pub pointer_type: PointerType,
}

/// Pointer cancel payload for component-owned pointer handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerCancelCx {
    pub pointer_id: PointerId,
    /// When provided by the platform, this is the last known pointer position (logical pixels).
    pub position: Option<Point>,
    pub tick_id: TickId,
    /// Pixels-per-point (a.k.a. window scale factor) for `position`.
    pub pixels_per_point: f32,
    pub buttons: fret_core::MouseButtons,
    pub modifiers: Modifiers,
    pub pointer_type: PointerType,
    pub reason: fret_core::PointerCancelReason,
}

/// Pointer up payload for component-owned pointer handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerUpCx {
    pub pointer_id: PointerId,
    pub position: Point,
    pub tick_id: TickId,
    /// Pixels-per-point (a.k.a. window scale factor) for `position`.
    pub pixels_per_point: f32,
    pub button: MouseButton,
    pub modifiers: Modifiers,
    /// Whether this pointer-up completes a "true click" (press + release without exceeding click
    /// slop).
    ///
    /// See `PointerEvent::Up.is_click` for normalization rules.
    pub is_click: bool,
    /// See `PointerEvent::{Down,Up}.click_count` for normalization rules.
    pub click_count: u8,
    pub pointer_type: PointerType,
}

/// Key down payload for component-owned key handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyDownCx {
    pub key: KeyCode,
    pub modifiers: Modifiers,
    pub repeat: bool,
}

/// Object-safe host surface for action handlers.
///
/// This intentionally exposes only non-generic operations so handlers can be stored in element
/// state and invoked by the runtime without coupling to `H: UiHost` (see ADR 0074).
pub trait UiActionHost {
    fn models_mut(&mut self) -> &mut ModelStore;
    fn push_effect(&mut self, effect: Effect);
    fn request_redraw(&mut self, window: AppWindowId);
    fn next_timer_token(&mut self) -> TimerToken;

    /// Mark the nearest view-cache root for `cx.target` as dirty (GPUI-style `notify`).
    ///
    /// Notes:
    /// - This is intentionally optional: hosts that are not running inside a UI tree event
    ///   dispatch can leave this as a no-op.
    /// - When view caching is enabled, this forces a rerender (skips reuse) for the nearest cache
    ///   root so declarative UI that depends on non-model state can still update deterministically.
    fn notify(&mut self, _cx: ActionCx) {}

    fn dispatch_command(&mut self, window: Option<AppWindowId>, command: CommandId) {
        self.push_effect(Effect::Command { window, command });
    }
}

/// Extra runtime-provided operations available to non-pointer action hooks.
///
/// This is used by keyboard hooks and other global hooks that need to move focus as a policy
/// decision (e.g. menu submenu focus transfer).
pub trait UiFocusActionHost: UiActionHost {
    fn request_focus(&mut self, target: crate::GlobalElementId);
}

/// Host operations for internal (app-owned) drag sessions.
///
/// This is intentionally object-safe so drag flows can be authored via stored action hooks.
/// Payload should typically live in models/globals (not in the drag session payload) to avoid
/// generic APIs in this surface.
pub trait UiDragActionHost: UiActionHost {
    fn begin_drag_with_kind(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
    );

    fn begin_cross_window_drag_with_kind(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
    );

    fn drag(&self, pointer_id: PointerId) -> Option<&DragSession>;
    fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession>;
    fn cancel_drag(&mut self, pointer_id: PointerId);
}

pub trait UiActionHostExt: UiActionHost {
    fn read_weak_model<T: Any, R>(
        &mut self,
        model: &WeakModel<T>,
        f: impl FnOnce(&T) -> R,
    ) -> Option<R> {
        let model = model.upgrade()?;
        self.models_mut().read(&model, f).ok()
    }

    fn update_model<T: Any, R>(
        &mut self,
        model: &Model<T>,
        f: impl FnOnce(&mut T) -> R,
    ) -> Option<R> {
        self.models_mut().update(model, f).ok()
    }

    fn update_weak_model<T: Any, R>(
        &mut self,
        model: &WeakModel<T>,
        f: impl FnOnce(&mut T) -> R,
    ) -> Option<R> {
        let model = model.upgrade()?;
        self.update_model(&model, f)
    }
}

impl<T> UiActionHostExt for T where T: UiActionHost + ?Sized {}

/// Extra runtime-provided operations available during pointer event hooks.
///
/// This is intentionally separate from `UiActionHost` because pointer capture and cursor updates
/// are mediated by the UI runtime (`UiTree`), not by the app host (`UiHost`).
pub trait UiPointerActionHost: UiFocusActionHost + UiDragActionHost {
    fn bounds(&self) -> fret_core::Rect;
    fn capture_pointer(&mut self);
    fn release_pointer_capture(&mut self);
    fn set_cursor_icon(&mut self, icon: CursorIcon);
    /// Suppress a runtime default action for the current event dispatch.
    ///
    /// This is primarily used to prevent "focus on pointer down" while still allowing propagation
    /// and other policies (overlays, global shortcuts, outside-press) to observe the event.
    fn prevent_default(&mut self, action: DefaultAction);

    /// Request a node-level invalidation for the current pointer region / pressable.
    ///
    /// This is intentionally separate from `notify()`: it enables paint-only updates (e.g. hover
    /// chrome) under view-cache reuse without forcing a rerender.
    fn invalidate(&mut self, _invalidation: crate::widget::Invalidation) {}
}

pub struct UiActionHostAdapter<'a, H: UiHost> {
    pub app: &'a mut H,
}

impl<'a, H: UiHost> UiActionHost for UiActionHostAdapter<'a, H> {
    fn models_mut(&mut self) -> &mut ModelStore {
        self.app.models_mut()
    }

    fn push_effect(&mut self, effect: Effect) {
        self.app.push_effect(effect);
    }

    fn request_redraw(&mut self, window: AppWindowId) {
        self.app.request_redraw(window);
    }

    fn next_timer_token(&mut self) -> TimerToken {
        self.app.next_timer_token()
    }
}

impl<'a, H: UiHost> UiDragActionHost for UiActionHostAdapter<'a, H> {
    fn begin_drag_with_kind(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
    ) {
        DragHost::begin_drag_with_kind(&mut *self.app, pointer_id, kind, source_window, start, ());
    }

    fn begin_cross_window_drag_with_kind(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
    ) {
        DragHost::begin_cross_window_drag_with_kind(
            &mut *self.app,
            pointer_id,
            kind,
            source_window,
            start,
            (),
        );
    }

    fn drag(&self, pointer_id: PointerId) -> Option<&DragSession> {
        DragHost::drag(&*self.app, pointer_id)
    }

    fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession> {
        DragHost::drag_mut(&mut *self.app, pointer_id)
    }

    fn cancel_drag(&mut self, pointer_id: PointerId) {
        DragHost::cancel_drag(&mut *self.app, pointer_id);
    }
}

/// Internal drag event payload for component-owned internal drag handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InternalDragCx {
    pub pointer_id: PointerId,
    pub position: Point,
    pub tick_id: TickId,
    pub kind: InternalDragKind,
    pub modifiers: Modifiers,
}

pub type OnInternalDrag =
    Arc<dyn Fn(&mut dyn UiDragActionHost, ActionCx, InternalDragCx) -> bool + 'static>;

#[derive(Default)]
pub(crate) struct InternalDragActionHooks {
    pub on_internal_drag: Option<OnInternalDrag>,
}

pub type OnActivate = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, ActivateReason) + 'static>;
pub type OnPressablePointerDown = Arc<
    dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerDownCx) -> PressablePointerDownResult
        + 'static,
>;
pub type OnPressablePointerMove =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerMoveCx) -> bool + 'static>;
pub type OnPressablePointerUp = Arc<
    dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerUpCx) -> PressablePointerUpResult
        + 'static,
>;

#[derive(Default)]
pub(crate) struct PressableActionHooks {
    pub on_activate: Option<OnActivate>,
    pub on_pointer_down: Option<OnPressablePointerDown>,
    pub on_pointer_move: Option<OnPressablePointerMove>,
    pub on_pointer_up: Option<OnPressablePointerUp>,
}

pub type OnHoverChange = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, bool) + 'static>;

#[derive(Default)]
pub(crate) struct PressableHoverActionHooks {
    pub on_hover_change: Option<OnHoverChange>,
}

pub type OnDismissRequest =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, &mut DismissRequestCx) + 'static>;

pub type OnOpenAutoFocus =
    Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, &mut AutoFocusRequestCx) + 'static>;

pub type OnCloseAutoFocus =
    Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, &mut AutoFocusRequestCx) + 'static>;

/// Pointer move observer hook for `DismissibleLayer`.
///
/// This is intentionally `UiActionHost` (not `UiPointerActionHost`) so dismissible roots can
/// observe pointer movement without participating in hit-testing or capture.
pub type OnDismissiblePointerMove =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, PointerMoveCx) -> bool + 'static>;

#[derive(Default)]
pub(crate) struct DismissibleActionHooks {
    pub on_dismiss_request: Option<OnDismissRequest>,
    pub on_pointer_move: Option<OnDismissiblePointerMove>,
}

pub type OnPointerDown =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerDownCx) -> bool + 'static>;

pub type OnPointerMove =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerMoveCx) -> bool + 'static>;

pub type OnWheel = Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, WheelCx) -> bool + 'static>;

pub type OnPinchGesture =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PinchGestureCx) -> bool + 'static>;

pub type OnPointerUp =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerUpCx) -> bool + 'static>;

pub type OnPointerCancel =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerCancelCx) -> bool + 'static>;

#[derive(Default)]
pub(crate) struct PointerActionHooks {
    pub on_pointer_down: Option<OnPointerDown>,
    pub on_pointer_move: Option<OnPointerMove>,
    pub on_wheel: Option<OnWheel>,
    pub on_pinch_gesture: Option<OnPinchGesture>,
    pub on_pointer_up: Option<OnPointerUp>,
    pub on_pointer_cancel: Option<OnPointerCancel>,
}

pub type OnKeyDown = Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, KeyDownCx) -> bool + 'static>;

#[derive(Default)]
pub(crate) struct KeyActionHooks {
    pub on_key_down: Option<OnKeyDown>,
}

pub type OnCommand = Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, CommandId) -> bool + 'static>;

#[derive(Default)]
pub(crate) struct CommandActionHooks {
    pub on_command: Option<OnCommand>,
}

pub trait UiCommandAvailabilityActionHost {
    fn models_mut(&mut self) -> &mut fret_runtime::ModelStore;
}

#[derive(Debug, Clone)]
pub struct CommandAvailabilityActionCx {
    pub window: fret_core::AppWindowId,
    pub target: crate::GlobalElementId,
    pub node: fret_core::NodeId,
    pub focus: Option<fret_core::NodeId>,
    pub focus_in_subtree: bool,
    pub input_ctx: fret_runtime::InputContext,
}

pub type OnCommandAvailability = Arc<
    dyn Fn(
            &mut dyn UiCommandAvailabilityActionHost,
            CommandAvailabilityActionCx,
            CommandId,
        ) -> crate::widget::CommandAvailability
        + 'static,
>;

#[derive(Default)]
pub(crate) struct CommandAvailabilityActionHooks {
    pub on_command_availability: Option<OnCommandAvailability>,
}

pub type OnTimer = Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, TimerToken) -> bool + 'static>;

#[derive(Default)]
pub(crate) struct TimerActionHooks {
    pub on_timer: Option<OnTimer>,
}

#[derive(Debug, Clone)]
pub struct RovingTypeaheadCx {
    pub input: char,
    pub current: Option<usize>,
    pub len: usize,
    pub disabled: Arc<[bool]>,
    pub wrap: bool,
    pub tick: u64,
}

pub type OnRovingActiveChange = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, usize) + 'static>;

pub type OnRovingTypeahead =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, RovingTypeaheadCx) -> Option<usize> + 'static>;

#[derive(Debug, Clone)]
pub struct RovingNavigateCx {
    pub key: KeyCode,
    pub modifiers: Modifiers,
    pub repeat: bool,
    pub axis: Axis,
    pub current: Option<usize>,
    pub len: usize,
    pub disabled: Arc<[bool]>,
    pub wrap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RovingNavigateResult {
    NotHandled,
    Handled { target: Option<usize> },
}

pub type OnRovingNavigate = Arc<
    dyn Fn(&mut dyn UiActionHost, ActionCx, RovingNavigateCx) -> RovingNavigateResult + 'static,
>;

#[derive(Default)]
pub(crate) struct RovingActionHooks {
    pub on_active_change: Option<OnRovingActiveChange>,
    pub on_typeahead: Option<OnRovingTypeahead>,
    pub on_navigate: Option<OnRovingNavigate>,
    pub on_key_down: Option<OnKeyDown>,
}

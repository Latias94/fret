use crate::UiHost;
use fret_core::{AppWindowId, CursorIcon, KeyCode, Modifiers, MouseButton, Point, TimerToken};
use fret_runtime::{CommandId, Effect, ModelStore};
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

/// Why an overlay is requesting dismissal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DismissReason {
    Escape,
    OutsidePress,
}

/// Pointer down payload for component-owned pointer handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerDownCx {
    pub position: Point,
    pub button: MouseButton,
    pub modifiers: Modifiers,
}

/// Pointer move payload for component-owned pointer handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerMoveCx {
    pub position: Point,
    pub buttons: fret_core::MouseButtons,
    pub modifiers: Modifiers,
}

/// Pointer up payload for component-owned pointer handlers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerUpCx {
    pub position: Point,
    pub button: MouseButton,
    pub modifiers: Modifiers,
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

    fn dispatch_command(&mut self, window: Option<AppWindowId>, command: CommandId) {
        self.push_effect(Effect::Command { window, command });
    }
}

/// Extra runtime-provided operations available during pointer event hooks.
///
/// This is intentionally separate from `UiActionHost` because pointer capture and cursor updates
/// are mediated by the UI runtime (`UiTree`), not by the app host (`UiHost`).
pub trait UiPointerActionHost: UiActionHost {
    fn bounds(&self) -> fret_core::Rect;
    fn capture_pointer(&mut self);
    fn release_pointer_capture(&mut self);
    fn set_cursor_icon(&mut self, icon: CursorIcon);
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

pub type OnActivate = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, ActivateReason) + 'static>;

#[derive(Default)]
pub(crate) struct PressableActionHooks {
    pub on_activate: Option<OnActivate>,
}

pub type OnHoverChange = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, bool) + 'static>;

#[derive(Default)]
pub(crate) struct PressableHoverActionHooks {
    pub on_hover_change: Option<OnHoverChange>,
}

pub type OnDismissRequest = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, DismissReason) + 'static>;

#[derive(Default)]
pub(crate) struct DismissibleActionHooks {
    pub on_dismiss_request: Option<OnDismissRequest>,
}

pub type OnPointerDown =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerDownCx) -> bool + 'static>;

pub type OnPointerMove =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerMoveCx) -> bool + 'static>;

pub type OnPointerUp =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, PointerUpCx) -> bool + 'static>;

#[derive(Default)]
pub(crate) struct PointerActionHooks {
    pub on_pointer_down: Option<OnPointerDown>,
    pub on_pointer_move: Option<OnPointerMove>,
    pub on_pointer_up: Option<OnPointerUp>,
}

pub type OnKeyDown = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, KeyDownCx) -> bool + 'static>;

#[derive(Default)]
pub(crate) struct KeyActionHooks {
    pub on_key_down: Option<OnKeyDown>,
}

pub type OnTimer = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, TimerToken) -> bool + 'static>;

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

#[derive(Default)]
pub(crate) struct RovingActionHooks {
    pub on_active_change: Option<OnRovingActiveChange>,
    pub on_typeahead: Option<OnRovingTypeahead>,
}

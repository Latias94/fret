//! Radix `ContextMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/context-menu>
//!
//! In Radix, `ContextMenu` is built on top of `Menu` with a different trigger/open policy.
//! In Fret we share the same underlying behavior via `crate::primitives::menu` and expose
//! Radix-named entry points here for reuse outside the shadcn layer.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{MouseButton, Point, PointerId, PointerType, Px, Rect};
use fret_runtime::{Effect, Model, ModelId, TimerToken};
use fret_ui::action::{
    OnPointerCancel, OnPointerDown, OnPointerMove, OnPointerUp, PointerCancelCx, PointerDownCx,
    PointerMoveCx, PointerUpCx, UiActionHost, UiPointerActionHost,
};
use fret_ui::UiHost;

use crate::primitives::popper;

pub use crate::primitives::menu::*;

pub use crate::primitives::menu::root::dismissible_menu_request as context_menu_dismissible_request;
pub use crate::primitives::menu::root::dismissible_menu_request_with_dismiss_handler as context_menu_dismissible_request_with_dismiss_handler;
pub use crate::primitives::menu::root::menu_overlay_root_name as context_menu_root_name;
pub use crate::primitives::menu::root::with_root_name_sync_root_open_and_ensure_submenu as context_menu_sync_root_open_and_ensure_submenu;
pub use crate::primitives::menu::trigger::wire_open_on_shift_f10 as wire_context_menu_open_on_shift_f10;

/// Touch long-press delay aligned with Base UI `ContextMenu.Trigger`.
pub const CONTEXT_MENU_TOUCH_LONG_PRESS_DELAY: Duration = Duration::from_millis(500);

/// Touch move threshold (in logical px) before canceling a pending long-press open.
pub const CONTEXT_MENU_TOUCH_LONG_PRESS_MOVE_THRESHOLD_PX: f32 = 10.0;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ContextMenuTouchLongPressState {
    pub pointer_id: Option<PointerId>,
    pub origin: Option<Point>,
    pub timer: Option<TimerToken>,
}

pub type ContextMenuTouchLongPress = Arc<Mutex<ContextMenuTouchLongPressState>>;

pub fn context_menu_touch_long_press() -> ContextMenuTouchLongPress {
    Arc::new(Mutex::new(ContextMenuTouchLongPressState::default()))
}

#[derive(Default)]
struct ContextMenuAnchorStore {
    by_open_model: Option<Model<HashMap<ModelId, Point>>>,
}

/// Returns a shared anchor store keyed by the context menu's open model id.
///
/// This is intended for context menus that need to anchor by cursor position even when the trigger
/// is not a `PointerRegion` (e.g. viewport tools opened via `Effect::ViewportInput`).
pub fn context_menu_anchor_store_model<H: UiHost>(app: &mut H) -> Model<HashMap<ModelId, Point>> {
    app.with_global_mut_untracked(ContextMenuAnchorStore::default, |st, app| {
        if let Some(model) = st.by_open_model.clone() {
            return model;
        }
        let model = app.models_mut().insert(HashMap::<ModelId, Point>::new());
        st.by_open_model = Some(model.clone());
        model
    })
}

/// Updates the anchor point for the given open model.
pub fn set_context_menu_anchor_for_open_model<H: UiHost>(
    app: &mut H,
    open: &Model<bool>,
    position: Point,
) {
    let open_model_id = open.id();
    let anchor_store_model = context_menu_anchor_store_model(app);
    let _ = app.models_mut().update(&anchor_store_model, |map| {
        map.insert(open_model_id, position);
    });
}

/// A Radix-aligned pointer-down policy for opening a context menu.
///
/// Mirrors the common desktop behavior:
/// - Right click opens.
/// - (macOS) Ctrl + left click opens.
///
/// Usage (typical):
/// - wrap your trigger in a `PointerRegion`,
/// - call `cx.pointer_region_on_pointer_down(context_menu_pointer_down_policy(open.clone()))`,
/// - read `PointerRegionState::last_down` to anchor the popup at the click position.
///
/// Note: `PointerRegionState::last_down` is per-element state; if you need the anchor to persist
/// across re-renders (or want to decouple it from element identity), copy `down.position` into an
/// app-owned model (e.g. `Model<Option<Point>>`, or a map keyed by your `open` model id).
pub fn context_menu_pointer_down_policy(open: Model<bool>) -> OnPointerDown {
    Arc::new(
        move |host: &mut dyn UiPointerActionHost,
              cx: fret_ui::action::ActionCx,
              down: PointerDownCx| {
            let is_right_click = down.button == MouseButton::Right;
            let is_macos_ctrl_click = cfg!(target_os = "macos")
                && down.button == MouseButton::Left
                && down.modifiers.ctrl;

            if !is_right_click && !is_macos_ctrl_click {
                return false;
            }

            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(cx.window);
            true
        },
    )
}

fn touch_long_press_is_touch_left_down(down: PointerDownCx) -> bool {
    down.pointer_type == PointerType::Touch && down.button == MouseButton::Left
}

fn touch_long_press_exceeds_move_threshold(origin: Point, position: Point) -> bool {
    let dx = origin.x.0 - position.x.0;
    let dy = origin.y.0 - position.y.0;
    (dx * dx + dy * dy)
        > CONTEXT_MENU_TOUCH_LONG_PRESS_MOVE_THRESHOLD_PX
            * CONTEXT_MENU_TOUCH_LONG_PRESS_MOVE_THRESHOLD_PX
}

fn clear_touch_long_press_inner(
    host: &mut dyn UiActionHost,
    state: &mut ContextMenuTouchLongPressState,
) {
    if let Some(token) = state.timer.take() {
        host.push_effect(Effect::CancelTimer { token });
    }
    state.pointer_id = None;
    state.origin = None;
}

pub fn context_menu_touch_long_press_clear(
    long_press: &ContextMenuTouchLongPress,
    host: &mut dyn UiActionHost,
) {
    let mut state = long_press.lock().unwrap_or_else(|e| e.into_inner());
    clear_touch_long_press_inner(host, &mut state);
}

pub fn context_menu_touch_long_press_on_pointer_down(
    long_press: &ContextMenuTouchLongPress,
    host: &mut dyn UiPointerActionHost,
    cx: fret_ui::action::ActionCx,
    down: PointerDownCx,
) -> bool {
    if !touch_long_press_is_touch_left_down(down) {
        return false;
    }

    let token = host.next_timer_token();
    {
        let mut state = long_press.lock().unwrap_or_else(|e| e.into_inner());
        clear_touch_long_press_inner(host, &mut state);
        state.pointer_id = Some(down.pointer_id);
        state.origin = Some(down.position);
        state.timer = Some(token);
    }

    host.push_effect(Effect::SetTimer {
        window: Some(cx.window),
        token,
        after: CONTEXT_MENU_TOUCH_LONG_PRESS_DELAY,
        repeat: None,
    });
    false
}

pub fn context_menu_touch_long_press_on_pointer_move(
    long_press: &ContextMenuTouchLongPress,
    host: &mut dyn UiPointerActionHost,
    mv: PointerMoveCx,
) -> bool {
    if mv.pointer_type != PointerType::Touch {
        return false;
    }

    let mut state = long_press.lock().unwrap_or_else(|e| e.into_inner());
    if state.pointer_id != Some(mv.pointer_id) {
        return false;
    }
    if let Some(origin) = state.origin {
        if touch_long_press_exceeds_move_threshold(origin, mv.position) {
            clear_touch_long_press_inner(host, &mut state);
        }
    }
    false
}

pub fn context_menu_touch_long_press_on_pointer_up(
    long_press: &ContextMenuTouchLongPress,
    host: &mut dyn UiPointerActionHost,
    up: PointerUpCx,
) -> bool {
    let mut state = long_press.lock().unwrap_or_else(|e| e.into_inner());
    if state.pointer_id != Some(up.pointer_id) {
        return false;
    }
    clear_touch_long_press_inner(host, &mut state);
    false
}

pub fn context_menu_touch_long_press_on_pointer_cancel(
    long_press: &ContextMenuTouchLongPress,
    host: &mut dyn UiPointerActionHost,
    cancel: PointerCancelCx,
) -> bool {
    let mut state = long_press.lock().unwrap_or_else(|e| e.into_inner());
    if state.pointer_id != Some(cancel.pointer_id) {
        return false;
    }
    clear_touch_long_press_inner(host, &mut state);
    false
}

pub fn context_menu_touch_long_press_take_anchor_on_timer(
    long_press: &ContextMenuTouchLongPress,
    token: TimerToken,
) -> Option<Point> {
    let mut state = long_press.lock().unwrap_or_else(|e| e.into_inner());
    if state.timer != Some(token) {
        return None;
    }

    state.timer = None;
    state.pointer_id = None;
    state.origin.take()
}

pub fn context_menu_touch_long_press_pointer_handlers(
    long_press: ContextMenuTouchLongPress,
) -> (OnPointerMove, OnPointerUp, OnPointerCancel) {
    let on_move: OnPointerMove = Arc::new({
        let long_press = long_press.clone();
        move |host, _cx, mv| context_menu_touch_long_press_on_pointer_move(&long_press, host, mv)
    });
    let on_up: OnPointerUp = Arc::new({
        let long_press = long_press.clone();
        move |host, _cx, up| context_menu_touch_long_press_on_pointer_up(&long_press, host, up)
    });
    let on_cancel: OnPointerCancel = Arc::new(move |host, _cx, cancel| {
        context_menu_touch_long_press_on_pointer_cancel(&long_press, host, cancel)
    });
    (on_move, on_up, on_cancel)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContextMenuPopperVars {
    pub available_width: Px,
    pub available_height: Px,
    pub trigger_width: Px,
    pub trigger_height: Px,
}

pub fn context_menu_popper_desired_width(outer: Rect, anchor: Rect, min_width: Px) -> Px {
    popper::popper_desired_width(outer, anchor, min_width)
}

/// Compute Radix-like "context menu popper vars" (`--radix-context-menu-*`) for recipes.
///
/// Upstream Radix re-namespaces these from `@radix-ui/react-popper`:
/// - `--radix-context-menu-content-available-width`
/// - `--radix-context-menu-content-available-height`
/// - `--radix-context-menu-trigger-width`
/// - `--radix-context-menu-trigger-height`
///
/// In Fret, we compute the same concepts as a structured return value so recipes can constrain
/// their content without relying on CSS variables.
pub fn context_menu_popper_vars(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: popper::PopperContentPlacement,
) -> ContextMenuPopperVars {
    let metrics =
        popper::popper_available_metrics_for_placement(outer, anchor, min_width, placement);
    ContextMenuPopperVars {
        available_width: metrics.available_width,
        available_height: metrics.available_height,
        trigger_width: metrics.anchor_width,
        trigger_height: metrics.anchor_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButtons, Point, PointerCancelReason, PointerId, PointerType,
        Size,
    };
    use fret_runtime::{Effect, ModelStore};
    use fret_ui::action::{
        ActionCx, UiActionHost, UiDragActionHost, UiFocusActionHost, UiPointerActionHost,
    };

    #[derive(Default)]
    struct PointerHost {
        app: App,
    }

    impl UiActionHost for PointerHost {
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

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.app.next_clipboard_token()
        }
    }

    impl UiFocusActionHost for PointerHost {
        fn request_focus(&mut self, _target: fret_ui::elements::GlobalElementId) {}
    }

    impl UiDragActionHost for PointerHost {
        fn begin_drag_with_kind(
            &mut self,
            _pointer_id: PointerId,
            _kind: fret_runtime::DragKindId,
            _source_window: AppWindowId,
            _start: Point,
        ) {
        }

        fn begin_cross_window_drag_with_kind(
            &mut self,
            _pointer_id: PointerId,
            _kind: fret_runtime::DragKindId,
            _source_window: AppWindowId,
            _start: Point,
        ) {
        }

        fn drag(&self, _pointer_id: PointerId) -> Option<&fret_runtime::DragSession> {
            None
        }

        fn drag_mut(&mut self, _pointer_id: PointerId) -> Option<&mut fret_runtime::DragSession> {
            None
        }

        fn cancel_drag(&mut self, _pointer_id: PointerId) {}
    }

    impl UiPointerActionHost for PointerHost {
        fn bounds(&self) -> Rect {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            )
        }

        fn capture_pointer(&mut self) {}

        fn release_pointer_capture(&mut self) {}

        fn set_cursor_icon(&mut self, _icon: fret_core::CursorIcon) {}

        fn prevent_default(&mut self, _action: fret_runtime::DefaultAction) {}
    }

    #[test]
    fn context_menu_popper_vars_available_height_tracks_flipped_side_space() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor = Rect::new(Point::new(Px(10.0), Px(70.0)), Size::new(Px(1.0), Px(1.0)));

        let placement = popper::PopperContentPlacement::new(
            popper::LayoutDirection::Ltr,
            popper::Side::Bottom,
            popper::Align::Start,
            Px(0.0),
        );
        let vars = context_menu_popper_vars(outer, anchor, Px(0.0), placement);
        assert!(vars.available_height.0 > 60.0 && vars.available_height.0 < 90.0);
    }

    #[test]
    fn touch_long_press_arms_timer_and_returns_anchor_on_fire() {
        let window = AppWindowId::default();
        let action_cx = ActionCx {
            window,
            target: fret_ui::elements::GlobalElementId(1),
        };
        let mut host = PointerHost::default();
        let long_press = context_menu_touch_long_press();

        let pointer_id = PointerId(7);
        let origin = Point::new(Px(120.0), Px(88.0));
        let tick_id = host.app.tick_id();
        let handled = context_menu_touch_long_press_on_pointer_down(
            &long_press,
            &mut host,
            action_cx,
            PointerDownCx {
                pointer_id,
                position: origin,
                tick_id,
                pixels_per_point: 1.0,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Touch,
            },
        );
        assert!(!handled);

        let effects = host.app.flush_effects();
        let token = effects.iter().find_map(|effect| match effect {
            Effect::SetTimer { token, after, .. }
                if *after == CONTEXT_MENU_TOUCH_LONG_PRESS_DELAY =>
            {
                Some(*token)
            }
            _ => None,
        });
        let Some(token) = token else {
            panic!("expected long-press timer effect; effects={effects:?}");
        };

        let anchor = context_menu_touch_long_press_take_anchor_on_timer(&long_press, token);
        assert_eq!(anchor, Some(origin));
    }

    #[test]
    fn touch_long_press_clears_when_pointer_moves_far() {
        let window = AppWindowId::default();
        let action_cx = ActionCx {
            window,
            target: fret_ui::elements::GlobalElementId(1),
        };
        let mut host = PointerHost::default();
        let long_press = context_menu_touch_long_press();

        let pointer_id = PointerId(9);
        let origin = Point::new(Px(10.0), Px(10.0));
        let tick_id = host.app.tick_id();
        let _ = context_menu_touch_long_press_on_pointer_down(
            &long_press,
            &mut host,
            action_cx,
            PointerDownCx {
                pointer_id,
                position: origin,
                tick_id,
                pixels_per_point: 1.0,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Touch,
            },
        );

        let effects = host.app.flush_effects();
        let token = effects.iter().find_map(|effect| match effect {
            Effect::SetTimer { token, after, .. }
                if *after == CONTEXT_MENU_TOUCH_LONG_PRESS_DELAY =>
            {
                Some(*token)
            }
            _ => None,
        });
        let Some(token) = token else {
            panic!("expected long-press timer effect; effects={effects:?}");
        };

        let _ = context_menu_touch_long_press_on_pointer_move(
            &long_press,
            &mut host,
            PointerMoveCx {
                pointer_id,
                position: Point::new(Px(40.0), Px(40.0)),
                tick_id,
                pixels_per_point: 1.0,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Touch,
            },
        );

        let anchor = context_menu_touch_long_press_take_anchor_on_timer(&long_press, token);
        assert!(anchor.is_none(), "moved too far; long-press should cancel");

        let cancel_effects = host.app.flush_effects();
        assert!(
            cancel_effects
                .iter()
                .any(|effect| matches!(effect, Effect::CancelTimer { token: t } if *t == token)),
            "expected timer cancellation effect after touch move; effects={cancel_effects:?}"
        );
    }

    #[test]
    fn touch_long_press_clears_on_pointer_cancel() {
        let window = AppWindowId::default();
        let action_cx = ActionCx {
            window,
            target: fret_ui::elements::GlobalElementId(1),
        };
        let mut host = PointerHost::default();
        let long_press = context_menu_touch_long_press();

        let pointer_id = PointerId(5);
        let tick_id = host.app.tick_id();
        let _ = context_menu_touch_long_press_on_pointer_down(
            &long_press,
            &mut host,
            action_cx,
            PointerDownCx {
                pointer_id,
                position: Point::new(Px(50.0), Px(60.0)),
                tick_id,
                pixels_per_point: 1.0,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Touch,
            },
        );

        let _ = host.app.flush_effects();

        let _ = context_menu_touch_long_press_on_pointer_cancel(
            &long_press,
            &mut host,
            PointerCancelCx {
                pointer_id,
                position: None,
                tick_id,
                pixels_per_point: 1.0,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Touch,
                reason: PointerCancelReason::LeftWindow,
            },
        );

        let state = long_press.lock().unwrap_or_else(|e| e.into_inner());
        assert!(state.pointer_id.is_none());
        assert!(state.origin.is_none());
        assert!(state.timer.is_none());
    }
}

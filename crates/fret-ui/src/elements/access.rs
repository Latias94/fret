use std::any::{Any, TypeId};

use fret_core::{AppWindowId, NodeId};

use crate::UiHost;
use crate::action::DismissibleActionHooks;
use crate::widget::Invalidation;

use fret_runtime::{ModelId, TimerToken};

use super::{ElementRuntime, GlobalElementId, WindowElementState};

pub fn with_element_state<H: UiHost, S: Any, R>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    init: impl FnOnce() -> S,
    f: impl FnOnce(&mut S) -> R,
) -> R {
    let frame_id = app.frame_id();
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let window_state = runtime.for_window_mut(window);

        let key = (element, TypeId::of::<S>());
        window_state.record_state_key_access(key);
        let mut value = window_state
            .take_state_box(&key)
            .unwrap_or_else(|| Box::new(init()));

        let out = {
            let state = value
                .downcast_mut::<S>()
                .expect("element state type mismatch");
            f(state)
        };

        window_state.insert_state_box(key, value);
        out
    })
}

pub(crate) fn with_observed_models_for_element<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    f: impl FnOnce(&[(ModelId, Invalidation)]) -> R,
) -> R {
    let frame_id = app.frame_id();
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let window_state = runtime.for_window_mut(window);
        let items = window_state
            .observed_models_next
            .get(&element)
            .or_else(|| window_state.observed_models_rendered.get(&element))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        f(items)
    })
}

pub(crate) fn with_observed_globals_for_element<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    f: impl FnOnce(&[(TypeId, Invalidation)]) -> R,
) -> R {
    with_window_state(app, window, |window_state| {
        let items = window_state
            .observed_globals_next
            .get(&element)
            .or_else(|| window_state.observed_globals_rendered.get(&element))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        f(items)
    })
}

pub(crate) fn with_window_state<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    f: impl FnOnce(&mut WindowElementState) -> R,
) -> R {
    let frame_id = app.frame_id();
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let window_state = runtime.for_window_mut(window);
        f(window_state)
    })
}

pub(crate) fn record_timer_target<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    token: TimerToken,
    target: GlobalElementId,
) {
    with_window_state(app, window, |st| {
        st.timer_targets.insert(token, target);
    });
}

pub(crate) fn clear_timer_target<H: UiHost>(app: &mut H, window: AppWindowId, token: TimerToken) {
    with_window_state(app, window, |st| {
        st.timer_targets.remove(&token);
    });
}

pub(crate) fn record_transient_event<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    key: u64,
) {
    with_window_state(app, window, |st| {
        st.record_transient_event(element, key);
    });
}

pub(crate) fn timer_target_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    token: TimerToken,
) -> Option<NodeId> {
    with_window_state(app, window, |st| {
        let element = st.timer_targets.get(&token).copied()?;
        st.node_entry(element).map(|e| e.node)
    })
}

pub(crate) fn update_hovered_pressable<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    next: Option<GlobalElementId>,
) -> (
    Option<GlobalElementId>,
    Option<NodeId>,
    Option<GlobalElementId>,
    Option<NodeId>,
) {
    with_window_state(app, window, |st| {
        let prev = st.hovered_pressable;
        if prev == next {
            return (None, None, None, None);
        }
        let prev_node = prev.and_then(|id| st.node_entry(id).map(|e| e.node));
        let next_node = next.and_then(|id| st.node_entry(id).map(|e| e.node));
        st.hovered_pressable = next;
        (prev, prev_node, next, next_node)
    })
}

pub(crate) fn update_hovered_hover_region<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    next: Option<GlobalElementId>,
) -> (
    Option<GlobalElementId>,
    Option<NodeId>,
    Option<GlobalElementId>,
    Option<NodeId>,
) {
    with_window_state(app, window, |st| {
        let prev = st.hovered_hover_region;
        if prev == next {
            return (None, None, None, None);
        }
        let prev_node = prev.and_then(|id| st.node_entry(id).map(|e| e.node));
        let next_node = next.and_then(|id| st.node_entry(id).map(|e| e.node));
        st.hovered_hover_region = next;
        (prev, prev_node, next, next_node)
    })
}

pub(crate) fn set_pressed_pressable<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pressed: Option<GlobalElementId>,
) -> Option<NodeId> {
    with_window_state(app, window, |st| {
        let prev = st.pressed_pressable;
        if prev == pressed {
            return None;
        }
        let prev_node = prev.and_then(|id| st.node_entry(id).map(|e| e.node));
        st.pressed_pressable = pressed;
        prev_node
    })
}

pub(crate) fn is_pressed_pressable<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> bool {
    with_window_state(app, window, |st| st.pressed_pressable == Some(element))
}

pub fn take_element_state<H: UiHost, S: Any>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<S> {
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, app| {
        runtime.prepare_window_for_frame(window, app.frame_id());
        let window_state = runtime.for_window_mut(window);
        let key = (element, TypeId::of::<S>());
        window_state
            .take_state_box(&key)
            .and_then(|e| e.downcast::<S>().ok())
            .map(|b| *b)
    })
}

/// Returns `true` if the given element currently has an `on_pointer_move` hook installed for
/// `DismissibleLayer`.
///
/// This is intended for diagnostics and cross-crate UI policy tests.
pub fn dismissible_has_pointer_move_handler<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> bool {
    with_window_state(app, window, |st| {
        let key = (element, TypeId::of::<DismissibleActionHooks>());
        st.state_any_ref(&key)
            .and_then(|any| any.downcast_ref::<DismissibleActionHooks>())
            .is_some_and(|hooks| hooks.on_pointer_move.is_some())
    })
}

use std::any::{Any, TypeId};

use fret_core::{AppWindowId, NodeId};

use crate::UiHost;
use crate::action::DismissibleActionHooks;
use crate::widget::Invalidation;

use fret_runtime::{ModelId, TimerToken};

use super::{ElementRuntime, GlobalElementId, WindowElementState};

fn update_element_target(
    current_element: &mut Option<GlobalElementId>,
    current_node: &mut Option<NodeId>,
    next_element: Option<GlobalElementId>,
    next_node: Option<NodeId>,
) -> (
    Option<GlobalElementId>,
    Option<NodeId>,
    Option<GlobalElementId>,
    Option<NodeId>,
) {
    let prev_element = *current_element;
    let prev_node = *current_node;

    if prev_element == next_element {
        *current_node = next_node;
        return (None, None, None, None);
    }

    *current_element = next_element;
    *current_node = next_node;
    (prev_element, prev_node, next_element, next_node)
}

pub(crate) fn with_observed_deps_for_element<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    f: impl FnOnce(&[(ModelId, Invalidation)], &[(TypeId, Invalidation)]) -> R,
) -> R {
    let frame_id = app.frame_id();
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let window_state = runtime.for_window_mut(window);

        let models = window_state
            .observed_models_next
            .get(&element)
            .or_else(|| window_state.observed_models_rendered.get(&element))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let globals = window_state
            .observed_globals_next
            .get(&element)
            .or_else(|| window_state.observed_globals_rendered.get(&element))
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        f(models, globals)
    })
}

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
        window_state.with_state_mut(element, init, f)
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
        st.timer_targets.insert(
            token,
            crate::elements::runtime::TimerTarget::Element(target),
        );
    });
}

pub(crate) fn record_timer_target_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    token: TimerToken,
    node: NodeId,
) {
    with_window_state(app, window, |st| {
        st.timer_targets
            .insert(token, crate::elements::runtime::TimerTarget::Node(node));
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

pub(crate) fn timer_target<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    token: TimerToken,
) -> Option<crate::elements::runtime::TimerTarget> {
    with_window_state(app, window, |st| st.timer_targets.get(&token).copied())
}

pub(crate) fn update_hovered_pressable_with_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    next: Option<(GlobalElementId, NodeId)>,
) -> (
    Option<GlobalElementId>,
    Option<NodeId>,
    Option<GlobalElementId>,
    Option<NodeId>,
) {
    with_window_state(app, window, |st| {
        let next_element = next.map(|(element, _)| element);
        let next_node = next.map(|(_, node)| node);
        update_element_target(
            &mut st.hovered_pressable,
            &mut st.hovered_pressable_node,
            next_element,
            next_node,
        )
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
        update_element_target(
            &mut st.hovered_pressable,
            &mut st.hovered_pressable_node,
            next,
            None,
        )
    })
}

pub(crate) fn update_hovered_pressable_raw_with_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    next: Option<(GlobalElementId, NodeId)>,
) -> (
    Option<GlobalElementId>,
    Option<NodeId>,
    Option<GlobalElementId>,
    Option<NodeId>,
) {
    with_window_state(app, window, |st| {
        let next_element = next.map(|(element, _)| element);
        let next_node = next.map(|(_, node)| node);
        update_element_target(
            &mut st.hovered_pressable_raw,
            &mut st.hovered_pressable_raw_node,
            next_element,
            next_node,
        )
    })
}

pub(crate) fn update_hovered_pressable_raw_below_barrier_with_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    next: Option<(GlobalElementId, NodeId)>,
) -> (
    Option<GlobalElementId>,
    Option<NodeId>,
    Option<GlobalElementId>,
    Option<NodeId>,
) {
    with_window_state(app, window, |st| {
        let next_element = next.map(|(element, _)| element);
        let next_node = next.map(|(_, node)| node);
        update_element_target(
            &mut st.hovered_pressable_raw_below_barrier,
            &mut st.hovered_pressable_raw_below_barrier_node,
            next_element,
            next_node,
        )
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
        update_element_target(
            &mut st.hovered_hover_region,
            &mut st.hovered_hover_region_node,
            next,
            None,
        )
    })
}

pub(crate) fn update_hovered_hover_region_with_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    next: Option<(GlobalElementId, NodeId)>,
) -> (
    Option<GlobalElementId>,
    Option<NodeId>,
    Option<GlobalElementId>,
    Option<NodeId>,
) {
    with_window_state(app, window, |st| {
        let next_element = next.map(|(element, _)| element);
        let next_node = next.map(|(_, node)| node);
        update_element_target(
            &mut st.hovered_hover_region,
            &mut st.hovered_hover_region_node,
            next_element,
            next_node,
        )
    })
}

pub(crate) fn set_pressed_pressable_with_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pressed: Option<(GlobalElementId, NodeId)>,
) -> Option<NodeId> {
    with_window_state(app, window, |st| {
        let next_element = pressed.map(|(element, _)| element);
        let next_node = pressed.map(|(_, node)| node);
        let prev_element = st.pressed_pressable;
        let prev_node = st.pressed_pressable_node;
        if prev_element == next_element {
            st.pressed_pressable_node = next_node;
            return None;
        }
        st.pressed_pressable = next_element;
        st.pressed_pressable_node = next_node;
        prev_node
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
        let prev_node = st.pressed_pressable_node;
        st.pressed_pressable = pressed;
        st.pressed_pressable_node = None;
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::strict_runtime::strict_runtime_enabled;
    use slotmap::KeyData;
    use std::any::TypeId;

    #[test]
    fn with_element_state_recovers_from_type_mismatch_in_non_strict_mode() {
        if strict_runtime_enabled() {
            return;
        }

        let mut app = crate::test_host::TestHost::new();
        let window = AppWindowId::default();
        let element = GlobalElementId(1);

        with_window_state(&mut app, window, |st| {
            st.insert_state_box(
                (element, TypeId::of::<u32>()),
                Box::new("corrupt".to_string()),
            );
        });

        let out = with_element_state(
            &mut app,
            window,
            element,
            || 0u32,
            |v| {
                *v = 9;
                *v
            },
        );
        assert_eq!(out, 9);

        let out = with_element_state(&mut app, window, element, || 0u32, |v| *v);
        assert_eq!(out, 9);
    }

    #[test]
    fn with_element_state_restores_state_on_panic() {
        let mut app = crate::test_host::TestHost::new();
        let window = AppWindowId::default();
        let element = GlobalElementId(2);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_element_state(
                &mut app,
                window,
                element,
                || 1u32,
                |v| {
                    *v = 2;
                    panic!("boom");
                },
            );
        }));
        assert!(result.is_err());

        let out = with_element_state(&mut app, window, element, || 0u32, |v| *v);
        assert_eq!(out, 2);
    }

    #[test]
    fn hovered_hover_region_tracks_node_id_for_redraw_on_exit() {
        let mut app = crate::test_host::TestHost::new();
        let window = AppWindowId::default();
        let element = GlobalElementId(1);
        let node = NodeId::from(KeyData::from_ffi(1));

        let (prev_element, prev_node, next_element, next_node) =
            update_hovered_hover_region_with_node(&mut app, window, Some((element, node)));
        assert_eq!(prev_element, None);
        assert_eq!(prev_node, None);
        assert_eq!(next_element, Some(element));
        assert_eq!(next_node, Some(node));

        let (prev_element, prev_node, next_element, next_node) =
            update_hovered_hover_region_with_node(&mut app, window, None);
        assert_eq!(prev_element, Some(element));
        assert_eq!(prev_node, Some(node));
        assert_eq!(next_element, None);
        assert_eq!(next_node, None);
    }

    #[test]
    fn hovered_pressable_clear_uses_latest_node_for_same_element() {
        let mut app = crate::test_host::TestHost::new();
        let window = AppWindowId::default();
        let element = GlobalElementId(11);
        let node_a = NodeId::from(KeyData::from_ffi(11));
        let node_b = NodeId::from(KeyData::from_ffi(12));

        let (_prev_element, _prev_node, next_element, next_node) =
            update_hovered_pressable_with_node(&mut app, window, Some((element, node_a)));
        assert_eq!(next_element, Some(element));
        assert_eq!(next_node, Some(node_a));

        let (prev_element, prev_node, next_element, next_node) =
            update_hovered_pressable_with_node(&mut app, window, Some((element, node_b)));
        assert_eq!(
            (prev_element, prev_node, next_element, next_node),
            (None, None, None, None)
        );

        let (prev_element, prev_node, next_element, next_node) =
            update_hovered_pressable(&mut app, window, None);
        assert_eq!(prev_element, Some(element));
        assert_eq!(prev_node, Some(node_b));
        assert_eq!(next_element, None);
        assert_eq!(next_node, None);
    }

    #[test]
    fn pressed_pressable_clear_uses_latest_node_for_same_element() {
        let mut app = crate::test_host::TestHost::new();
        let window = AppWindowId::default();
        let element = GlobalElementId(21);
        let node_a = NodeId::from(KeyData::from_ffi(21));
        let node_b = NodeId::from(KeyData::from_ffi(22));

        assert_eq!(
            set_pressed_pressable_with_node(&mut app, window, Some((element, node_a))),
            None
        );
        assert_eq!(
            set_pressed_pressable_with_node(&mut app, window, Some((element, node_b))),
            None
        );
        assert_eq!(
            set_pressed_pressable(&mut app, window, None),
            Some(node_b),
            "expected clearing the pressed target to invalidate the latest authoritative node for the element"
        );
    }
}

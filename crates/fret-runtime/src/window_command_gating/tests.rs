use std::collections::HashMap;
use std::sync::Arc;

use fret_core::AppWindowId;

use super::*;
use crate::{CommandId, CommandScope, InputContext};

#[test]
fn base_snapshot_is_visible_when_no_stack_overrides_exist() {
    let window = AppWindowId::default();
    let mut svc = WindowCommandGatingService::default();

    let base_ctx = InputContext {
        focus_is_text_input: true,
        ..Default::default()
    };
    svc.set_base_snapshot(
        window,
        WindowCommandGatingSnapshot::new(base_ctx, HashMap::new()),
    );

    assert!(
        svc.base_snapshot(window)
            .is_some_and(|s| s.input_ctx().focus_is_text_input),
        "expected base snapshot getter to return the stored base snapshot"
    );
    assert!(
        svc.snapshot(window)
            .is_some_and(|s| s.input_ctx().focus_is_text_input),
        "expected snapshot() to fall back to base snapshot"
    );
}

#[test]
fn snapshot_prefers_stack_top_and_falls_back_to_base() {
    let window = AppWindowId::default();
    let mut svc = WindowCommandGatingService::default();

    let base_ctx = InputContext {
        focus_is_text_input: true,
        ..Default::default()
    };
    svc.set_base_snapshot(
        window,
        WindowCommandGatingSnapshot::new(base_ctx, HashMap::new()),
    );
    assert!(
        svc.snapshot(window)
            .is_some_and(|s| s.input_ctx().focus_is_text_input),
        "expected base snapshot to be visible"
    );

    let overlay_ctx = InputContext {
        ui_has_modal: true,
        focus_is_text_input: false,
        ..Default::default()
    };
    let handle = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(overlay_ctx, HashMap::new()),
    );
    assert!(
        svc.snapshot(window)
            .is_some_and(|s| s.input_ctx().ui_has_modal && !s.input_ctx().focus_is_text_input),
        "expected stack top snapshot to win"
    );

    svc.pop_snapshot(handle).expect("remove pushed snapshot");
    assert!(
        svc.snapshot(window)
            .is_some_and(|s| s.input_ctx().focus_is_text_input && !s.input_ctx().ui_has_modal),
        "expected fallback to base snapshot after popping"
    );

    svc.clear_base_snapshot(window);
    assert!(
        svc.snapshot(window).is_none(),
        "expected window to be cleared"
    );
}

#[test]
fn action_availability_disables_widget_scope_commands_only() {
    let command = CommandId::new("test.widget");

    let mut availability: HashMap<CommandId, bool> = HashMap::new();
    availability.insert(command.clone(), false);

    let snapshot = WindowCommandGatingSnapshot::new(InputContext::default(), HashMap::new())
        .with_action_availability(Some(Arc::new(availability)));

    assert!(
        !snapshot.is_enabled_for_meta(&command, CommandScope::Widget, None),
        "expected widget-scope command to be disabled by action availability"
    );
    assert!(
        snapshot.is_enabled_for_meta(&command, CommandScope::Window, None),
        "expected non-widget scopes to ignore action availability"
    );
}

#[test]
fn set_snapshot_does_not_override_stack_top() {
    let window = AppWindowId::default();
    let mut svc = WindowCommandGatingService::default();

    let outer_ctx = InputContext {
        focus_is_text_input: true,
        ..Default::default()
    };
    let token = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(outer_ctx, HashMap::new()),
    );

    let base_ctx = InputContext {
        ui_has_modal: true,
        ..Default::default()
    };
    svc.set_snapshot(
        window,
        WindowCommandGatingSnapshot::new(base_ctx, HashMap::new()),
    );

    assert!(
        svc.snapshot(window)
            .is_some_and(|s| s.input_ctx().focus_is_text_input && !s.input_ctx().ui_has_modal),
        "expected stack top to remain effective after set_snapshot"
    );

    svc.pop_snapshot(token).expect("remove pushed snapshot");
    assert!(
        svc.snapshot(window)
            .is_some_and(|s| s.input_ctx().ui_has_modal && !s.input_ctx().focus_is_text_input),
        "expected base snapshot to become effective after popping stack"
    );
}

#[test]
fn pushed_snapshots_can_be_removed_out_of_order() {
    let window = AppWindowId::default();
    let mut svc = WindowCommandGatingService::default();

    let outer_ctx = InputContext {
        ui_has_modal: true,
        ..Default::default()
    };
    let outer = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(outer_ctx, HashMap::new()),
    );

    let inner_ctx = InputContext {
        dispatch_phase: crate::InputDispatchPhase::Capture,
        ..Default::default()
    };
    let inner = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(inner_ctx, HashMap::new()),
    );

    assert_eq!(
        svc.snapshot(window)
            .expect("snapshot")
            .input_ctx()
            .dispatch_phase,
        crate::InputDispatchPhase::Capture
    );

    svc.pop_snapshot(outer).expect("remove outer");
    assert_eq!(
        svc.snapshot(window)
            .expect("snapshot")
            .input_ctx()
            .dispatch_phase,
        crate::InputDispatchPhase::Capture,
        "expected inner snapshot to remain effective"
    );

    svc.pop_snapshot(inner).expect("remove inner");
    assert!(
        svc.snapshot(window).is_none(),
        "expected all snapshots removed"
    );
}
#[test]
fn clearing_base_snapshot_does_not_remove_active_overlay_snapshot() {
    let window = AppWindowId::default();
    let mut svc = WindowCommandGatingService::default();

    let base_ctx = InputContext {
        focus_is_text_input: true,
        ..Default::default()
    };
    svc.set_base_snapshot(
        window,
        WindowCommandGatingSnapshot::new(base_ctx, HashMap::new()),
    );

    let overlay_ctx = InputContext {
        ui_has_modal: true,
        ..Default::default()
    };
    let handle = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(overlay_ctx, HashMap::new()),
    );

    svc.clear_base_snapshot(window);
    assert!(
        svc.snapshot(window)
            .is_some_and(|s| s.input_ctx().ui_has_modal && !s.input_ctx().focus_is_text_input),
        "expected overlay snapshot to remain effective after clearing base"
    );

    svc.pop_snapshot(handle).expect("remove pushed snapshot");
    assert!(
        svc.snapshot(window).is_none(),
        "expected window to be cleared after removing the last overlay snapshot"
    );
}

#[test]
fn setting_base_snapshot_does_not_override_stack_top() {
    let window = AppWindowId::default();
    let mut svc = WindowCommandGatingService::default();

    let overlay_ctx = InputContext {
        ui_has_modal: true,
        focus_is_text_input: false,
        ..Default::default()
    };
    let handle = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(overlay_ctx, HashMap::new()),
    );

    let base_ctx = InputContext {
        ui_has_modal: false,
        focus_is_text_input: true,
        ..Default::default()
    };
    svc.set_base_snapshot(
        window,
        WindowCommandGatingSnapshot::new(base_ctx, HashMap::new()),
    );

    assert!(
        svc.snapshot(window)
            .is_some_and(|s| { s.input_ctx().ui_has_modal && !s.input_ctx().focus_is_text_input }),
        "expected stack top snapshot to remain effective after set_snapshot"
    );

    svc.pop_snapshot(handle).expect("remove pushed snapshot");
    assert!(
        svc.snapshot(window)
            .is_some_and(|s| !s.input_ctx().ui_has_modal && s.input_ctx().focus_is_text_input),
        "expected base snapshot to take effect after popping the overlay"
    );
}

#[test]
fn updating_pushed_snapshot_only_affects_that_entry() {
    let window = AppWindowId::default();
    let mut svc = WindowCommandGatingService::default();

    let outer_ctx = InputContext {
        ui_has_modal: true,
        ..Default::default()
    };
    let outer = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(outer_ctx, HashMap::new()),
    );

    let inner_ctx = InputContext {
        dispatch_phase: crate::InputDispatchPhase::Capture,
        ..Default::default()
    };
    let inner = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(inner_ctx, HashMap::new()),
    );

    let updated_outer_ctx = InputContext {
        dispatch_phase: crate::InputDispatchPhase::Preview,
        ..Default::default()
    };
    assert!(
        svc.update_pushed_snapshot(
            outer,
            WindowCommandGatingSnapshot::new(updated_outer_ctx, HashMap::new())
        ),
        "expected update to succeed"
    );

    assert_eq!(
        svc.snapshot(window)
            .expect("snapshot")
            .input_ctx()
            .dispatch_phase,
        crate::InputDispatchPhase::Capture,
        "expected inner snapshot to remain effective"
    );

    svc.pop_snapshot(inner).expect("remove inner");
    assert_eq!(
        svc.snapshot(window)
            .expect("snapshot")
            .input_ctx()
            .dispatch_phase,
        crate::InputDispatchPhase::Preview,
        "expected updated outer snapshot to become effective after popping inner"
    );
}

#[test]
fn removing_inner_snapshot_restores_outer_snapshot() {
    let window = AppWindowId::default();
    let mut svc = WindowCommandGatingService::default();

    let outer_ctx = InputContext {
        ui_has_modal: true,
        ..Default::default()
    };
    let outer = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(outer_ctx, HashMap::new()),
    );

    let inner_ctx = InputContext {
        dispatch_phase: crate::InputDispatchPhase::Capture,
        ..Default::default()
    };
    let inner = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(inner_ctx, HashMap::new()),
    );

    svc.pop_snapshot(inner).expect("remove inner");
    assert!(
        svc.snapshot(window)
            .is_some_and(|s| s.input_ctx().ui_has_modal),
        "expected outer snapshot to become effective after popping inner"
    );

    svc.pop_snapshot(outer).expect("remove outer");
    assert!(
        svc.snapshot(window).is_none(),
        "expected all snapshots removed"
    );
}

#[test]
fn clear_snapshot_only_clears_base_not_pushed_overrides() {
    let window = AppWindowId::default();
    let mut svc = WindowCommandGatingService::default();

    let base_ctx = InputContext {
        focus_is_text_input: true,
        ..Default::default()
    };
    svc.set_base_snapshot(
        window,
        WindowCommandGatingSnapshot::new(base_ctx, HashMap::new()),
    );

    let overlay_ctx = InputContext {
        ui_has_modal: true,
        ..Default::default()
    };
    let handle = svc.push_snapshot(
        window,
        WindowCommandGatingSnapshot::new(overlay_ctx, HashMap::new()),
    );

    svc.clear_base_snapshot(window);
    assert!(
        svc.snapshot(window)
            .is_some_and(|s| s.input_ctx().ui_has_modal),
        "expected pushed override to remain after clearing base snapshot"
    );

    svc.pop_snapshot(handle).expect("remove pushed snapshot");
    assert!(
        svc.snapshot(window).is_none(),
        "expected window to be cleared after removing last pushed override and base"
    );
}

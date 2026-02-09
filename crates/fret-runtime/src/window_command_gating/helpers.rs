/// Best-effort: builds a `WindowCommandGatingSnapshot` from the currently published services.
use fret_core::AppWindowId;

use crate::WindowCommandActionAvailabilityService;
use crate::{CommandId, CommandScope, CommandsHost, GlobalsHost, InputContext};
use crate::{WindowCommandEnabledService, WindowInputContextService};

use super::{WindowCommandGatingService, WindowCommandGatingSnapshot};

pub fn snapshot_for_window(
    app: &impl GlobalsHost,
    window: AppWindowId,
) -> WindowCommandGatingSnapshot {
    snapshot_for_window_with_input_ctx_fallback(app, window, InputContext::default())
}

/// Best-effort: returns a `WindowCommandGatingSnapshot` from a previously published override if
/// present (`WindowCommandGatingService`), otherwise falls back to `snapshot_for_window`.
pub fn best_effort_snapshot_for_window(
    app: &impl GlobalsHost,
    window: AppWindowId,
) -> WindowCommandGatingSnapshot {
    best_effort_snapshot_for_window_with_input_ctx_fallback(app, window, InputContext::default())
}

/// Best-effort: returns a `WindowCommandGatingSnapshot` from a previously published override if
/// present (`WindowCommandGatingService`), otherwise falls back to
/// `snapshot_for_window_with_input_ctx_fallback`.
pub fn best_effort_snapshot_for_window_with_input_ctx_fallback(
    app: &impl GlobalsHost,
    window: AppWindowId,
    fallback_input_ctx: InputContext,
) -> WindowCommandGatingSnapshot {
    app.global::<WindowCommandGatingService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .unwrap_or_else(|| {
            snapshot_for_window_with_input_ctx_fallback(app, window, fallback_input_ctx)
        })
}

pub fn snapshot_for_window_with_input_ctx_fallback(
    app: &impl GlobalsHost,
    window: AppWindowId,
    fallback_input_ctx: InputContext,
) -> WindowCommandGatingSnapshot {
    let input_ctx = app
        .global::<WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .unwrap_or(fallback_input_ctx);

    let enabled_overrides = app
        .global::<WindowCommandEnabledService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .unwrap_or_default();

    let action_availability = app
        .global::<WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.snapshot_arc(window));

    WindowCommandGatingSnapshot::new(input_ctx, enabled_overrides)
        .with_action_availability(action_availability)
}

/// Returns whether `command` is enabled according to the best-effort window gating snapshot.
///
/// This is intended for cross-surface checks (OS menus, in-window menus, command palettes,
/// shortcuts, effect filtering) that need consistent results without depending on UI internals.
pub fn command_is_enabled_for_window_with_input_ctx_fallback(
    app: &(impl GlobalsHost + CommandsHost),
    window: AppWindowId,
    command: &CommandId,
    fallback_input_ctx: InputContext,
) -> bool {
    let gating =
        best_effort_snapshot_for_window_with_input_ctx_fallback(app, window, fallback_input_ctx);
    if let Some(meta) = app.commands().get(command.clone()) {
        gating.is_enabled_for_command(command, meta)
    } else {
        gating.is_enabled_for_meta(command, CommandScope::App, None)
    }
}

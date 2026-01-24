use std::collections::HashMap;
use std::sync::Arc;

use fret_core::AppWindowId;

use crate::WindowCommandActionAvailabilityService;
use crate::{CommandId, CommandMeta, CommandScope, InputContext, WhenExpr};
use crate::{CommandsHost, GlobalsHost};
use crate::{WindowCommandEnabledService, WindowInputContextService};

#[derive(Debug, Default)]
pub struct WindowCommandGatingService {
    by_window: HashMap<AppWindowId, WindowCommandGatingSnapshot>,
}

impl WindowCommandGatingService {
    pub fn snapshot(&self, window: AppWindowId) -> Option<&WindowCommandGatingSnapshot> {
        self.by_window.get(&window)
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, snapshot: WindowCommandGatingSnapshot) {
        self.by_window.insert(window, snapshot);
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}

/// Window-scoped snapshot that aggregates data-only command gating inputs.
///
/// This is a consumption-focused seam intended for runner/platform and UI-kit layers:
/// menus, command palette, shortcut help, etc.
#[derive(Debug, Default, Clone)]
pub struct WindowCommandGatingSnapshot {
    input_ctx: InputContext,
    enabled_overrides: HashMap<CommandId, bool>,
    action_availability: Option<Arc<HashMap<CommandId, bool>>>,
}

impl WindowCommandGatingSnapshot {
    pub fn new(input_ctx: InputContext, enabled_overrides: HashMap<CommandId, bool>) -> Self {
        Self {
            input_ctx,
            enabled_overrides,
            action_availability: None,
        }
    }

    pub fn input_ctx(&self) -> &InputContext {
        &self.input_ctx
    }

    pub fn with_input_ctx(mut self, input_ctx: InputContext) -> Self {
        self.input_ctx = input_ctx;
        self
    }

    pub fn enabled_overrides(&self) -> &HashMap<CommandId, bool> {
        &self.enabled_overrides
    }

    pub fn action_availability(&self) -> Option<&HashMap<CommandId, bool>> {
        self.action_availability.as_deref()
    }

    /// GPUI naming parity: query the latest published dispatch-path availability, if present.
    ///
    /// This is only meaningful for `CommandScope::Widget` commands; other scopes are not modeled
    /// as dispatch-path availability entries today.
    pub fn is_action_available(&self, command: &CommandId) -> Option<bool> {
        self.action_availability
            .as_ref()
            .and_then(|map| map.get(command).copied())
    }

    pub fn with_action_availability(
        mut self,
        action_availability: Option<Arc<HashMap<CommandId, bool>>>,
    ) -> Self {
        self.action_availability = action_availability;
        self
    }

    pub fn is_enabled_for_meta(
        &self,
        command: &CommandId,
        scope: CommandScope,
        when: Option<&WhenExpr>,
    ) -> bool {
        if scope == CommandScope::Widget
            && let Some(map) = self.action_availability.as_ref()
            && let Some(is_available) = map.get(command).copied()
            && !is_available
        {
            return false;
        }
        if when.is_some_and(|w| !w.eval(&self.input_ctx)) {
            return false;
        }
        self.enabled_overrides.get(command).copied().unwrap_or(true)
    }

    pub fn is_enabled_for_command(&self, command: &CommandId, meta: &CommandMeta) -> bool {
        self.is_enabled_for_meta(command, meta.scope, meta.when.as_ref())
    }
}

/// Best-effort: builds a `WindowCommandGatingSnapshot` from the currently published services.
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

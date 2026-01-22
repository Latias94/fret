use std::collections::HashMap;

use fret_core::AppWindowId;

use crate::GlobalsHost;
use crate::{CommandId, CommandMeta, CommandScope, InputContext, WhenExpr};
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
}

impl WindowCommandGatingSnapshot {
    pub fn new(input_ctx: InputContext, enabled_overrides: HashMap<CommandId, bool>) -> Self {
        Self {
            input_ctx,
            enabled_overrides,
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

    pub fn is_enabled_for_meta(
        &self,
        command: &CommandId,
        _scope: CommandScope,
        when: Option<&WhenExpr>,
    ) -> bool {
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

    WindowCommandGatingSnapshot::new(input_ctx, enabled_overrides)
}

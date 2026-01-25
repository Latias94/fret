use std::collections::HashMap;
use std::sync::Arc;

use fret_core::AppWindowId;

use crate::WindowCommandActionAvailabilityService;
use crate::{CommandId, CommandMeta, CommandScope, InputContext, WhenExpr};
use crate::{CommandsHost, GlobalsHost};
use crate::{WindowCommandEnabledService, WindowInputContextService};

/// Token identifying a pushed, overlay-scoped gating override.
///
/// The intent is to allow nested overlays (command palette -> menu -> sub-menu, etc.) to publish
/// independent gating snapshots without clobbering each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowCommandGatingToken(u64);

#[derive(Debug, Default)]
struct WindowCommandGatingWindowState {
    base: Option<WindowCommandGatingSnapshot>,
    stack: Vec<(WindowCommandGatingToken, WindowCommandGatingSnapshot)>,
}

#[derive(Debug, Default)]
pub struct WindowCommandGatingService {
    by_window: HashMap<AppWindowId, WindowCommandGatingWindowState>,
    next_token: u64,
}

impl WindowCommandGatingService {
    pub fn snapshot(&self, window: AppWindowId) -> Option<&WindowCommandGatingSnapshot> {
        self.by_window.get(&window).and_then(|state| {
            state
                .stack
                .last()
                .map(|(_, snap)| snap)
                .or(state.base.as_ref())
        })
    }

    /// Sets the base override snapshot for the window.
    ///
    /// Nested overlays should prefer `push_snapshot` so they do not overwrite other overrides.
    pub fn set_snapshot(&mut self, window: AppWindowId, snapshot: WindowCommandGatingSnapshot) {
        let state = self.by_window.entry(window).or_default();
        state.base = Some(snapshot);
        self.gc_window(window);
    }

    pub fn clear_snapshot(&mut self, window: AppWindowId) {
        if let Some(state) = self.by_window.get_mut(&window) {
            state.base = None;
        }
        self.gc_window(window);
    }

    /// Pushes an overlay-scoped gating snapshot and returns a token that can later remove it.
    ///
    /// The most recently pushed snapshot wins (`snapshot()` returns the stack top).
    pub fn push_snapshot(
        &mut self,
        window: AppWindowId,
        snapshot: WindowCommandGatingSnapshot,
    ) -> WindowCommandGatingToken {
        let token = WindowCommandGatingToken(self.next_token.max(1));
        self.next_token = token.0.saturating_add(1);
        let state = self.by_window.entry(window).or_default();
        state.stack.push((token, snapshot));
        token
    }

    pub fn update_pushed_snapshot(
        &mut self,
        window: AppWindowId,
        token: WindowCommandGatingToken,
        snapshot: WindowCommandGatingSnapshot,
    ) -> bool {
        let Some(state) = self.by_window.get_mut(&window) else {
            return false;
        };
        for (t, s) in &mut state.stack {
            if *t == token {
                *s = snapshot;
                return true;
            }
        }
        false
    }

    pub fn remove_pushed_snapshot(
        &mut self,
        window: AppWindowId,
        token: WindowCommandGatingToken,
    ) -> Option<WindowCommandGatingSnapshot> {
        let state = self.by_window.get_mut(&window)?;
        let idx = state.stack.iter().position(|(t, _)| *t == token)?;
        let (_, snapshot) = state.stack.remove(idx);
        self.gc_window(window);
        Some(snapshot)
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }

    fn gc_window(&mut self, window: AppWindowId) {
        let remove = self
            .by_window
            .get(&window)
            .is_some_and(|state| state.base.is_none() && state.stack.is_empty());
        if remove {
            self.by_window.remove(&window);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_prefers_stack_top_and_falls_back_to_base() {
        let window = AppWindowId::default();
        let mut svc = WindowCommandGatingService::default();

        let mut base_ctx = InputContext::default();
        base_ctx.focus_is_text_input = true;
        svc.set_snapshot(
            window,
            WindowCommandGatingSnapshot::new(base_ctx, HashMap::new()),
        );
        assert!(
            svc.snapshot(window)
                .is_some_and(|s| s.input_ctx().focus_is_text_input),
            "expected base snapshot to be visible"
        );

        let mut overlay_ctx = InputContext::default();
        overlay_ctx.ui_has_modal = true;
        overlay_ctx.focus_is_text_input = false;
        let token = svc.push_snapshot(
            window,
            WindowCommandGatingSnapshot::new(overlay_ctx, HashMap::new()),
        );
        assert!(
            svc.snapshot(window)
                .is_some_and(|s| s.input_ctx().ui_has_modal && !s.input_ctx().focus_is_text_input),
            "expected stack top snapshot to win"
        );

        svc.remove_pushed_snapshot(window, token)
            .expect("remove pushed snapshot");
        assert!(
            svc.snapshot(window)
                .is_some_and(|s| s.input_ctx().focus_is_text_input && !s.input_ctx().ui_has_modal),
            "expected fallback to base snapshot after popping"
        );

        svc.clear_snapshot(window);
        assert!(
            svc.snapshot(window).is_none(),
            "expected window to be cleared"
        );
    }

    #[test]
    fn pushed_snapshots_can_be_removed_out_of_order() {
        let window = AppWindowId::default();
        let mut svc = WindowCommandGatingService::default();

        let mut outer_ctx = InputContext::default();
        outer_ctx.ui_has_modal = true;
        let outer = svc.push_snapshot(
            window,
            WindowCommandGatingSnapshot::new(outer_ctx, HashMap::new()),
        );

        let mut inner_ctx = InputContext::default();
        inner_ctx.dispatch_phase = crate::InputDispatchPhase::Capture;
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

        svc.remove_pushed_snapshot(window, outer)
            .expect("remove outer");
        assert_eq!(
            svc.snapshot(window)
                .expect("snapshot")
                .input_ctx()
                .dispatch_phase,
            crate::InputDispatchPhase::Capture,
            "expected inner snapshot to remain effective"
        );

        svc.remove_pushed_snapshot(window, inner)
            .expect("remove inner");
        assert!(
            svc.snapshot(window).is_none(),
            "expected all snapshots removed"
        );
    }

    #[test]
    fn clearing_base_snapshot_does_not_remove_active_overlay_snapshot() {
        let window = AppWindowId::default();
        let mut svc = WindowCommandGatingService::default();

        let mut base_ctx = InputContext::default();
        base_ctx.focus_is_text_input = true;
        svc.set_snapshot(
            window,
            WindowCommandGatingSnapshot::new(base_ctx, HashMap::new()),
        );

        let mut overlay_ctx = InputContext::default();
        overlay_ctx.ui_has_modal = true;
        let token = svc.push_snapshot(
            window,
            WindowCommandGatingSnapshot::new(overlay_ctx, HashMap::new()),
        );

        svc.clear_snapshot(window);
        assert!(
            svc.snapshot(window)
                .is_some_and(|s| s.input_ctx().ui_has_modal && !s.input_ctx().focus_is_text_input),
            "expected overlay snapshot to remain effective after clearing base"
        );

        svc.remove_pushed_snapshot(window, token)
            .expect("remove pushed snapshot");
        assert!(
            svc.snapshot(window).is_none(),
            "expected window to be cleared after removing the last overlay snapshot"
        );
    }

    #[test]
    fn setting_base_snapshot_does_not_override_stack_top() {
        let window = AppWindowId::default();
        let mut svc = WindowCommandGatingService::default();

        let mut overlay_ctx = InputContext::default();
        overlay_ctx.ui_has_modal = true;
        overlay_ctx.focus_is_text_input = false;
        let token = svc.push_snapshot(
            window,
            WindowCommandGatingSnapshot::new(overlay_ctx, HashMap::new()),
        );

        let mut base_ctx = InputContext::default();
        base_ctx.ui_has_modal = false;
        base_ctx.focus_is_text_input = true;
        svc.set_snapshot(
            window,
            WindowCommandGatingSnapshot::new(base_ctx, HashMap::new()),
        );

        assert!(
            svc.snapshot(window).is_some_and(|s| {
                s.input_ctx().ui_has_modal && !s.input_ctx().focus_is_text_input
            }),
            "expected stack top snapshot to remain effective after set_snapshot"
        );

        svc.remove_pushed_snapshot(window, token)
            .expect("remove pushed snapshot");
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

        let mut outer_ctx = InputContext::default();
        outer_ctx.ui_has_modal = true;
        let outer = svc.push_snapshot(
            window,
            WindowCommandGatingSnapshot::new(outer_ctx, HashMap::new()),
        );

        let mut inner_ctx = InputContext::default();
        inner_ctx.dispatch_phase = crate::InputDispatchPhase::Capture;
        let inner = svc.push_snapshot(
            window,
            WindowCommandGatingSnapshot::new(inner_ctx, HashMap::new()),
        );

        let mut updated_outer_ctx = InputContext::default();
        updated_outer_ctx.dispatch_phase = crate::InputDispatchPhase::Preview;
        assert!(
            svc.update_pushed_snapshot(
                window,
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

        svc.remove_pushed_snapshot(window, inner)
            .expect("remove inner");
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

        let mut outer_ctx = InputContext::default();
        outer_ctx.ui_has_modal = true;
        let outer = svc.push_snapshot(
            window,
            WindowCommandGatingSnapshot::new(outer_ctx, HashMap::new()),
        );

        let mut inner_ctx = InputContext::default();
        inner_ctx.dispatch_phase = crate::InputDispatchPhase::Capture;
        let inner = svc.push_snapshot(
            window,
            WindowCommandGatingSnapshot::new(inner_ctx, HashMap::new()),
        );

        svc.remove_pushed_snapshot(window, inner)
            .expect("remove inner");
        assert!(
            svc.snapshot(window)
                .is_some_and(|s| s.input_ctx().ui_has_modal),
            "expected outer snapshot to become effective after popping inner"
        );

        svc.remove_pushed_snapshot(window, outer)
            .expect("remove outer");
        assert!(
            svc.snapshot(window).is_none(),
            "expected all snapshots removed"
        );
    }
}

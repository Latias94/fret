//! NavigationMenu primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing navigation menu behavior in
//! recipes. It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/navigation-menu/src/navigation-menu.tsx`

use std::sync::Arc;
use std::time::Duration;

use fret_core::PointerType;
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::{ElementContext, UiHost};

/// Radix `delayDuration` default (milliseconds).
pub const DEFAULT_DELAY_DURATION_MS: u64 = 200;
/// Radix `skipDelayDuration` default (milliseconds).
pub const DEFAULT_SKIP_DELAY_DURATION_MS: u64 = 300;
/// Radix `startCloseTimer` default (milliseconds).
pub const DEFAULT_CLOSE_DELAY_MS: u64 = 150;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationMenuConfig {
    pub delay_duration: Duration,
    pub skip_delay_duration: Duration,
    pub close_delay_duration: Duration,
}

impl Default for NavigationMenuConfig {
    fn default() -> Self {
        Self {
            delay_duration: Duration::from_millis(DEFAULT_DELAY_DURATION_MS),
            skip_delay_duration: Duration::from_millis(DEFAULT_SKIP_DELAY_DURATION_MS),
            close_delay_duration: Duration::from_millis(DEFAULT_CLOSE_DELAY_MS),
        }
    }
}

impl NavigationMenuConfig {
    pub fn new(
        delay_duration: Duration,
        skip_delay_duration: Duration,
        close_delay_duration: Duration,
    ) -> Self {
        Self {
            delay_duration,
            skip_delay_duration,
            close_delay_duration,
        }
    }
}

/// Returns a selected-value model that behaves like Radix `useControllableState` (`value` /
/// `defaultValue`).
///
/// Radix uses an empty string to represent "closed". In Fret we use `Option<Arc<str>>` (`None`
/// means closed).
pub fn navigation_menu_use_value_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Option<Arc<str>>>>,
    default_value: impl FnOnce() -> Option<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Option<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

fn cancel_timer(host: &mut dyn UiActionHost, token: &mut Option<TimerToken>) {
    if let Some(token) = token.take() {
        host.push_effect(Effect::CancelTimer { token });
    }
}

fn arm_timer(
    host: &mut dyn UiActionHost,
    window: fret_core::AppWindowId,
    after: Duration,
    token_out: &mut Option<TimerToken>,
) -> TimerToken {
    cancel_timer(host, token_out);
    let token = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window: Some(window),
        token,
        after,
        repeat: None,
    });
    *token_out = Some(token);
    token
}

#[derive(Debug, Clone)]
pub struct NavigationMenuRootState {
    open_timer: Option<TimerToken>,
    close_timer: Option<TimerToken>,
    skip_delay_timer: Option<TimerToken>,
    pending_open_value: Option<Arc<str>>,
    is_open_delayed: bool,
}

impl Default for NavigationMenuRootState {
    fn default() -> Self {
        Self {
            open_timer: None,
            close_timer: None,
            skip_delay_timer: None,
            pending_open_value: None,
            is_open_delayed: true,
        }
    }
}

impl NavigationMenuRootState {
    pub fn is_open_delayed(&self) -> bool {
        self.is_open_delayed
    }

    pub fn clear_timers(&mut self, host: &mut dyn UiActionHost) {
        cancel_timer(host, &mut self.open_timer);
        cancel_timer(host, &mut self.close_timer);
        cancel_timer(host, &mut self.skip_delay_timer);
        self.pending_open_value = None;
    }

    fn note_opened(&mut self, host: &mut dyn UiActionHost, cfg: NavigationMenuConfig) {
        cancel_timer(host, &mut self.skip_delay_timer);
        // Radix only skips open delays when `skipDelayDuration > 0`.
        self.is_open_delayed = cfg.skip_delay_duration.is_zero();
    }

    fn note_closed(
        &mut self,
        host: &mut dyn UiActionHost,
        window: fret_core::AppWindowId,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.skip_delay_timer);
        self.is_open_delayed = true;
        if cfg.skip_delay_duration.is_zero() {
            return;
        }
        // Mirror Radix: after `skipDelayDuration`, re-enable delayed opening.
        arm_timer(
            host,
            window,
            cfg.skip_delay_duration,
            &mut self.skip_delay_timer,
        );
        // While the timer is armed we keep `is_open_delayed=false` (immediate-open window).
        self.is_open_delayed = false;
    }

    pub fn on_trigger_enter(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        item_value: Arc<str>,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.open_timer);

        let current = host
            .models_mut()
            .read(value_model, |v| v.clone())
            .ok()
            .flatten();

        // Always cancel close when entering a trigger.
        cancel_timer(host, &mut self.close_timer);

        if !self.is_open_delayed {
            let _ = host
                .models_mut()
                .update(value_model, |v| *v = Some(item_value.clone()));
            self.note_opened(host, cfg);
            host.request_redraw(acx.window);
            return;
        }

        // Delayed open: if the item is already open, just clear the close timer (done above).
        if current.as_deref() == Some(item_value.as_ref()) {
            return;
        }

        self.pending_open_value = Some(item_value);
        arm_timer(host, acx.window, cfg.delay_duration, &mut self.open_timer);
        host.request_redraw(acx.window);
    }

    pub fn on_trigger_leave(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.open_timer);
        self.pending_open_value = None;
        self.start_close_timer(host, acx, value_model, cfg);
    }

    pub fn on_content_enter(&mut self, host: &mut dyn UiActionHost) {
        cancel_timer(host, &mut self.close_timer);
    }

    pub fn on_content_leave(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) {
        self.start_close_timer(host, acx, value_model, cfg);
    }

    fn start_close_timer(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) {
        if cfg.close_delay_duration.is_zero() {
            let _ = host.models_mut().update(value_model, |v| *v = None);
            self.note_closed(host, acx.window, cfg);
            host.request_redraw(acx.window);
            return;
        }
        arm_timer(
            host,
            acx.window,
            cfg.close_delay_duration,
            &mut self.close_timer,
        );
        host.request_redraw(acx.window);
    }

    pub fn on_item_select(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        item_value: Arc<str>,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.open_timer);
        cancel_timer(host, &mut self.close_timer);
        self.pending_open_value = None;

        let current = host
            .models_mut()
            .read(value_model, |v| v.clone())
            .ok()
            .flatten();
        if current.as_deref() == Some(item_value.as_ref()) {
            let _ = host.models_mut().update(value_model, |v| *v = None);
            self.note_closed(host, acx.window, cfg);
        } else {
            let _ = host
                .models_mut()
                .update(value_model, |v| *v = Some(item_value.clone()));
            self.note_opened(host, cfg);
        }

        host.request_redraw(acx.window);
    }

    pub fn on_item_dismiss(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.open_timer);
        cancel_timer(host, &mut self.close_timer);
        self.pending_open_value = None;

        let _ = host.models_mut().update(value_model, |v| *v = None);
        self.note_closed(host, acx.window, cfg);
        host.request_redraw(acx.window);
    }

    /// Handle a timer callback (open/close/skip-delay).
    ///
    /// Returns `true` when it updates state and a redraw should be requested by the caller.
    pub fn on_timer(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        token: TimerToken,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) -> bool {
        if self.open_timer == Some(token) {
            self.open_timer = None;
            let Some(value) = self.pending_open_value.take() else {
                return false;
            };
            cancel_timer(host, &mut self.close_timer);
            let _ = host.models_mut().update(value_model, |v| *v = Some(value));
            self.note_opened(host, cfg);
            host.request_redraw(acx.window);
            return true;
        }

        if self.close_timer == Some(token) {
            self.close_timer = None;
            let _ = host.models_mut().update(value_model, |v| *v = None);
            self.note_closed(host, acx.window, cfg);
            host.request_redraw(acx.window);
            return true;
        }

        if self.skip_delay_timer == Some(token) {
            self.skip_delay_timer = None;
            self.is_open_delayed = true;
            host.request_redraw(acx.window);
            return true;
        }

        false
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NavigationMenuTriggerState {
    pub has_pointer_move_opened: bool,
    pub was_click_close: bool,
    pub was_escape_close: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationMenuTriggerPointerMoveAction {
    Open,
    Ignore,
}

/// Mirror Radix `NavigationMenuTrigger` hover-open gating.
pub fn navigation_menu_trigger_pointer_move_action(
    pointer_type: PointerType,
    disabled: bool,
    state: NavigationMenuTriggerState,
) -> NavigationMenuTriggerPointerMoveAction {
    match pointer_type {
        PointerType::Touch | PointerType::Pen => NavigationMenuTriggerPointerMoveAction::Ignore,
        PointerType::Mouse | PointerType::Unknown => {
            if disabled
                || state.was_click_close
                || state.was_escape_close
                || state.has_pointer_move_opened
            {
                NavigationMenuTriggerPointerMoveAction::Ignore
            } else {
                NavigationMenuTriggerPointerMoveAction::Open
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::AppWindowId;
    use fret_ui::GlobalElementId;
    use fret_ui::action::UiActionHostAdapter;

    fn acx(window: AppWindowId) -> ActionCx {
        ActionCx {
            window,
            target: GlobalElementId(0x1),
        }
    }

    #[test]
    fn trigger_enter_is_delayed_by_default_and_opens_after_timer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let mut host = UiActionHostAdapter { app: &mut app };

        let mut st = NavigationMenuRootState::default();
        let cfg = NavigationMenuConfig::default();

        st.on_trigger_enter(&mut host, acx(window), &value, Arc::from("a"), cfg);
        assert_eq!(
            host.models_mut().read(&value, |v| v.clone()).ok().flatten(),
            None
        );

        let effects = host.app.flush_effects();
        let token = effects
            .iter()
            .find_map(|e| match e {
                Effect::SetTimer { token, after, .. } if *after == cfg.delay_duration => {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected open timer");

        assert!(st.on_timer(&mut host, acx(window), token, &value, cfg));
        assert_eq!(
            host.models_mut()
                .read(&value, |v| v.clone())
                .ok()
                .flatten()
                .as_deref(),
            Some("a")
        );
        assert!(
            !st.is_open_delayed(),
            "expected skip-delay window to be active"
        );
    }

    #[test]
    fn closing_enables_immediate_open_within_skip_delay_window() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let value = app.models_mut().insert(Some(Arc::from("a")));
        let mut host = UiActionHostAdapter { app: &mut app };

        let mut st = NavigationMenuRootState::default();
        let cfg = NavigationMenuConfig::default();

        // Mark as opened (Radix sets isOpenDelayed=false while open).
        st.note_opened(&mut host, cfg);
        assert!(!st.is_open_delayed());

        // Dismiss closes and arms the skip-delay timer, while keeping `is_open_delayed=false`
        // until it fires.
        st.on_item_dismiss(&mut host, acx(window), &value, cfg);
        assert_eq!(
            host.models_mut().read(&value, |v| v.clone()).ok().flatten(),
            None
        );
        assert!(!st.is_open_delayed());

        host.app.flush_effects();

        // Within the skip window: entering a trigger opens immediately (no open timer).
        st.on_trigger_enter(&mut host, acx(window), &value, Arc::from("b"), cfg);
        assert_eq!(
            host.models_mut()
                .read(&value, |v| v.clone())
                .ok()
                .flatten()
                .as_deref(),
            Some("b")
        );
        let effects = host.app.flush_effects();
        assert!(
            effects.iter().all(
                |e| !matches!(e, Effect::SetTimer { after, .. } if *after == cfg.delay_duration)
            ),
            "expected immediate open (no delayed-open timer)"
        );
    }

    #[test]
    fn trigger_leave_starts_close_timer_and_content_enter_cancels_it() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let value = app.models_mut().insert(Some(Arc::from("a")));
        let mut host = UiActionHostAdapter { app: &mut app };

        let mut st = NavigationMenuRootState::default();
        let cfg = NavigationMenuConfig::default();

        st.on_trigger_leave(&mut host, acx(window), &value, cfg);
        let effects = host.app.flush_effects();
        let close_token = effects
            .iter()
            .find_map(|e| match e {
                Effect::SetTimer { token, after, .. } if *after == cfg.close_delay_duration => {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected close timer");

        // Content enter cancels the close timer.
        st.on_content_enter(&mut host);
        let effects = host.app.flush_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::CancelTimer { token } if *token == close_token)),
            "expected close timer cancellation"
        );
    }

    #[test]
    fn trigger_pointer_move_gate_matches_radix_outcomes() {
        let st = NavigationMenuTriggerState::default();
        assert_eq!(
            navigation_menu_trigger_pointer_move_action(PointerType::Mouse, false, st),
            NavigationMenuTriggerPointerMoveAction::Open
        );
        assert_eq!(
            navigation_menu_trigger_pointer_move_action(PointerType::Touch, false, st),
            NavigationMenuTriggerPointerMoveAction::Ignore
        );

        let st = NavigationMenuTriggerState {
            has_pointer_move_opened: true,
            ..Default::default()
        };
        assert_eq!(
            navigation_menu_trigger_pointer_move_action(PointerType::Mouse, false, st),
            NavigationMenuTriggerPointerMoveAction::Ignore
        );
    }
}

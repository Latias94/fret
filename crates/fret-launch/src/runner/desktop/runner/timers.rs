use fret_core::time::Instant;

use fret_app::Effect;
use fret_core::Event;

use super::window::TimerEntry;
use super::{WinitEventContext, WinitRunner};

impl<D: super::WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_set_timer_effect(&mut self, now: Instant, effect: &Effect) {
        let Effect::SetTimer {
            window,
            token,
            after,
            repeat,
        } = effect
        else {
            return;
        };
        self.timers.insert(
            *token,
            TimerEntry {
                window: *window,
                deadline: now + *after,
                repeat: *repeat,
                last_fired_tick: None,
            },
        );
    }

    pub(super) fn handle_cancel_timer_effect(&mut self, token: fret_runtime::TimerToken) {
        self.timers.remove(&token);
    }

    pub(super) fn fire_due_timers(&mut self, now: Instant) -> bool {
        let mut fired_any = false;
        let mut due: Vec<fret_runtime::TimerToken> = Vec::new();
        for (token, entry) in &self.timers {
            if entry.deadline <= now && entry.last_fired_tick != Some(self.tick_id) {
                due.push(*token);
            }
        }

        for token in due {
            let Some(entry) = self.timers.get(&token).cloned() else {
                continue;
            };
            fired_any = true;

            let all_windows = self.windows.keys().collect::<Vec<_>>();
            if let Some(asset_reload) = self.asset_reload.as_mut()
                && asset_reload.handle_timer(&mut self.app, token, &all_windows)
            {
                self.finish_fired_timer(token, entry.repeat);
                continue;
            }

            let target = entry
                .window
                .or(self.main_window)
                .and_then(|w| self.windows.contains_key(w).then_some(w));

            if let Some(window) = target {
                let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                if let Some(state) = self.windows.get_mut(window) {
                    self.driver.handle_event(
                        WinitEventContext {
                            app: &mut self.app,
                            services,
                            window,
                            state: &mut state.user,
                        },
                        &Event::Timer { token },
                    );
                }
            }

            self.finish_fired_timer(token, entry.repeat);
        }

        fired_any
    }

    pub(super) fn finish_fired_timer(
        &mut self,
        token: fret_runtime::TimerToken,
        repeat: Option<std::time::Duration>,
    ) {
        match repeat {
            Some(interval) => {
                if let Some(e) = self.timers.get_mut(&token) {
                    // Re-arm repeating timers relative to the completion of the handler rather
                    // than the start of the drain turn. A diagnostics/script keepalive timer can
                    // legitimately perform expensive work (or request redraw + inject events).
                    // If we schedule from the stale pre-handler `now`, the same repeating timer
                    // can already be overdue before `drain_effects` reaches its next fixed-point
                    // iteration, causing catch-up self-spin inside one event-loop turn and starving
                    // the platform `RedrawRequested` that the handler just requested.
                    e.deadline = Instant::now() + interval;
                    e.last_fired_tick = Some(self.tick_id);
                }
            }
            None => {
                self.timers.remove(&token);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use fret_app::App;
    use fret_core::{AppWindowId, Event, TimerToken, time::Instant};

    use crate::{WinitAppDriver, WinitEventContext, WinitRenderContext};

    use super::super::{WinitRunner, WinitRunnerConfig};
    use super::TimerEntry;

    struct TimerTestDriver {
        fired: std::rc::Rc<Cell<u32>>,
    }

    struct TimerTestWindowState;

    impl WinitAppDriver for TimerTestDriver {
        type WindowState = TimerTestWindowState;

        fn create_window_state(
            &mut self,
            _app: &mut App,
            _window: AppWindowId,
        ) -> Self::WindowState {
            TimerTestWindowState
        }

        fn handle_event(
            &mut self,
            _context: WinitEventContext<'_, Self::WindowState>,
            event: &Event,
        ) {
            if matches!(event, Event::Timer { .. }) {
                self.fired.set(self.fired.get().saturating_add(1));
            }
        }

        fn render(&mut self, _context: WinitRenderContext<'_, Self::WindowState>) {}
    }

    #[test]
    fn repeating_timer_rearms_from_handler_completion_time() {
        let fired = std::rc::Rc::new(Cell::new(0));
        let mut runner = WinitRunner::new(
            WinitRunnerConfig::default(),
            App::new(),
            TimerTestDriver {
                fired: fired.clone(),
            },
        );
        let token = TimerToken(42);
        let before_finish = Instant::now();
        let stale_now = before_finish - Duration::from_secs(30);
        let interval = Duration::from_millis(16);

        runner.timers.insert(
            token,
            TimerEntry {
                window: None,
                deadline: stale_now,
                repeat: Some(interval),
                last_fired_tick: None,
            },
        );

        runner.finish_fired_timer(token, Some(interval));

        let next = runner.timers.get(&token).expect("repeating timer remains");
        assert!(
            next.deadline >= before_finish + interval,
            "repeating timer should be rearmed from handler completion time, not stale drain time"
        );
        assert_eq!(fired.get(), 0);
    }

    #[test]
    fn due_repeating_timer_fires_at_most_once_per_runner_tick() {
        let fired = std::rc::Rc::new(Cell::new(0));
        let mut runner = WinitRunner::new(
            WinitRunnerConfig::default(),
            App::new(),
            TimerTestDriver {
                fired: fired.clone(),
            },
        );

        let token = TimerToken(7);
        let interval = Duration::from_millis(1);
        let due_at = Instant::now() - Duration::from_secs(1);
        runner.timers.insert(
            token,
            TimerEntry {
                window: None,
                deadline: due_at,
                repeat: Some(interval),
                last_fired_tick: None,
            },
        );

        assert!(runner.fire_due_timers(Instant::now()));
        assert_eq!(
            fired.get(),
            0,
            "this unit only needs timer scheduling; no window is required"
        );

        if let Some(entry) = runner.timers.get_mut(&token) {
            entry.deadline = due_at;
        }

        assert!(
            !runner.fire_due_timers(Instant::now()),
            "same runner tick should not fire the same repeating timer twice"
        );
        assert_eq!(fired.get(), 0);

        runner.tick_id = fret_runtime::TickId(runner.tick_id.0.saturating_add(1));
        assert!(runner.fire_due_timers(Instant::now()));
        assert_eq!(fired.get(), 0);
    }

    #[test]
    fn overlapping_repeating_timers_do_not_catch_up_inside_one_runner_tick() {
        let fired = std::rc::Rc::new(Cell::new(0));
        let mut runner = WinitRunner::new(
            WinitRunnerConfig::default(),
            App::new(),
            TimerTestDriver {
                fired: fired.clone(),
            },
        );

        let script_keepalive = TimerToken(100);
        let asset_reload_poll = TimerToken(101);
        let interval = Duration::from_secs(60);
        let due_at = Instant::now() - Duration::from_secs(1);

        runner.timers.insert(
            script_keepalive,
            TimerEntry {
                window: Some(AppWindowId::default()),
                deadline: due_at,
                repeat: Some(interval),
                last_fired_tick: None,
            },
        );
        runner.timers.insert(
            asset_reload_poll,
            TimerEntry {
                window: None,
                deadline: due_at,
                repeat: Some(interval),
                last_fired_tick: None,
            },
        );

        let first_tick = runner.tick_id;
        assert!(runner.fire_due_timers(Instant::now()));

        for token in [script_keepalive, asset_reload_poll] {
            let entry = runner.timers.get(&token).expect("repeating timer remains");
            assert_eq!(entry.last_fired_tick, Some(first_tick));
        }

        // Model a slow handler / stale drain timestamp by making both timers overdue again before
        // the runner advances to another event-loop tick. Neither the script keepalive timer nor an
        // asset-reload-style windowless poll timer may catch up inside the same tick.
        for token in [script_keepalive, asset_reload_poll] {
            runner
                .timers
                .get_mut(&token)
                .expect("timer remains")
                .deadline = due_at;
        }

        assert!(
            !runner.fire_due_timers(Instant::now()),
            "overlapping repeating timers should not self-spin inside one runner tick"
        );
        assert_eq!(
            fired.get(),
            0,
            "this scheduling stress gate does not require a real winit window"
        );

        runner.tick_id = fret_runtime::TickId(runner.tick_id.0.saturating_add(1));
        assert!(runner.fire_due_timers(Instant::now()));
        for token in [script_keepalive, asset_reload_poll] {
            let entry = runner.timers.get(&token).expect("repeating timer remains");
            assert_eq!(entry.last_fired_tick, Some(runner.tick_id));
        }
    }
}

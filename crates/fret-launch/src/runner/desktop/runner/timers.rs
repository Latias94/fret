use std::time::Instant;

use fret_app::Effect;
use fret_core::Event;

use super::window::TimerEntry;
use super::{WinitEventContext, WinitRunner};

impl<D: super::WinitAppDriver> WinitRunner<D> {
    pub(super) fn schedule_timer(&mut self, now: Instant, effect: &Effect) {
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
            },
        );
    }

    pub(super) fn fire_due_timers(&mut self, now: Instant) -> bool {
        let mut fired_any = false;
        let mut due: Vec<fret_runtime::TimerToken> = Vec::new();
        for (token, entry) in &self.timers {
            if entry.deadline <= now {
                due.push(*token);
            }
        }

        for token in due {
            let Some(entry) = self.timers.get(&token).cloned() else {
                continue;
            };
            fired_any = true;

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

            match entry.repeat {
                Some(interval) => {
                    if let Some(e) = self.timers.get_mut(&token) {
                        e.deadline = now + interval;
                    }
                }
                None => {
                    self.timers.remove(&token);
                }
            }
        }

        fired_any
    }
}

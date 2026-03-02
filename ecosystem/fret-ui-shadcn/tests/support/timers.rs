use fret_app::{App, Effect};
use fret_core::{Event, TimerToken};
use fret_ui::tree::UiTree;
use std::time::Duration;

#[derive(Default)]
pub(crate) struct TimerQueue {
    pending: Vec<(TimerToken, Duration)>,
}

impl TimerQueue {
    pub(crate) fn ingest_effects(&mut self, app: &mut App) {
        let effects = app.flush_effects();
        for effect in effects {
            match effect {
                Effect::SetTimer { token, after, .. } => self.pending.push((token, after)),
                Effect::CancelTimer { token } => self.pending.retain(|(t, _)| *t != token),
                other => app.push_effect(other),
            }
        }
    }

    pub(crate) fn fire_all(
        &mut self,
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
    ) {
        let fire: Vec<TimerToken> = self.pending.drain(..).map(|(t, _)| t).collect();
        for token in fire {
            ui.dispatch_event(app, services, &Event::Timer { token });
        }
    }
}

use std::collections::HashSet;

use fret_core::time::Instant;
use winit::event_loop::ActiveEventLoop;

use super::WinitRunner;

impl<D: super::WinitAppDriver> WinitRunner<D> {
    pub(super) fn drain_inboxes(&mut self, window: Option<fret_core::AppWindowId>) -> bool {
        let did_work = self.app.with_global_mut_untracked(
            fret_runtime::InboxDrainRegistry::default,
            |registry, app| registry.drain_all(app, window),
        );
        tracing::trace!(?window, did_work, "driver: drain_inboxes");
        did_work
    }

    pub(super) fn drain_effects(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut should_exit = false;
        crate::runner::common::fixed_point::drain_bounded(|| {
            let now = Instant::now();
            let mut did_work = self.dispatcher.drain_turn(now);
            did_work |= self.drain_inboxes(None);
            did_work |= self.apply_pending_system_font_rescan_result(now);
            let effects = self.app.flush_effects();
            let (effects, mut stats, ack_count) = self.process_streaming_upload_effects(effects);
            tracing::trace!(
                did_work,
                effects = effects.len(),
                acks = ack_count,
                "driver: drain_effects turn"
            );

            did_work |= self.poll_watch_restart_trigger(now);
            did_work |= self.poll_hotpatch_trigger(now);
            did_work |= !effects.is_empty();
            let mut window_state_dirty: HashSet<fret_core::AppWindowId> = HashSet::new();

            if self.dispatch_effect_queue(
                event_loop,
                effects,
                &mut stats,
                &mut window_state_dirty,
                now,
            ) {
                should_exit = true;
                return false;
            }

            self.publish_streaming_upload_diagnostics(&stats);

            for window in window_state_dirty {
                if let Some(state) = self.windows.get_mut(window) {
                    state.platform.prepare_frame(state.window.as_ref());
                }
            }

            did_work |= self.fire_due_timers(now);
            did_work |= self.clear_internal_drag_hover_if_needed();
            did_work |= self.propagate_model_changes();
            did_work |= self.propagate_global_changes();

            self.request_pending_streaming_upload_redraws();

            if !did_work {
                return false;
            }
            true
        });
        if should_exit {}
    }
}

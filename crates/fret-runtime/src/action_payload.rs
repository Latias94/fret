use std::any::Any;
use std::collections::HashMap;

use fret_core::AppWindowId;

use crate::{ActionId, TickId};

#[derive(Debug)]
struct PendingActionPayloadV1 {
    tick_id: TickId,
    window: AppWindowId,
    action: ActionId,
    payload: Box<dyn Any + Send + Sync>,
}

/// Window-scoped, tick-local payload store for parameterized actions (v2 prototype).
///
/// This is intentionally best-effort and transient:
/// - callers should record a payload immediately before dispatching an `ActionId`,
/// - handlers should consume the payload at dispatch time (or treat missing payload as not handled),
/// - entries expire after a small tick TTL.
///
/// See ADR 0312.
#[derive(Default)]
pub struct WindowPendingActionPayloadService {
    per_window: HashMap<AppWindowId, Vec<PendingActionPayloadV1>>,
}

impl WindowPendingActionPayloadService {
    const MAX_PENDING_PER_WINDOW: usize = 32;
    const PENDING_TTL_TICKS: u64 = 64;

    pub fn record(
        &mut self,
        window: AppWindowId,
        tick_id: TickId,
        action: ActionId,
        payload: Box<dyn Any + Send + Sync>,
    ) {
        let pending = PendingActionPayloadV1 {
            tick_id,
            window,
            action,
            payload,
        };
        let entries = self.per_window.entry(window).or_default();
        entries.push(pending);
        if entries.len() > Self::MAX_PENDING_PER_WINDOW {
            let extra = entries.len().saturating_sub(Self::MAX_PENDING_PER_WINDOW);
            entries.drain(0..extra);
        }
    }

    pub fn consume(
        &mut self,
        window: AppWindowId,
        tick_id: TickId,
        action: &ActionId,
    ) -> Option<Box<dyn Any + Send + Sync>> {
        let entries = self.per_window.get_mut(&window)?;

        // Drop stale pending entries.
        //
        // Like `WindowPendingCommandDispatchSourceService`, payload is best-effort: the actual
        // command dispatch may be handled on a later tick (effect flushing deferral, scheduled
        // work). Keep a small TTL window to preserve usability without making this a durable
        // storage mechanism.
        let min_tick = TickId(tick_id.0.saturating_sub(Self::PENDING_TTL_TICKS));
        entries.retain(|e| e.tick_id.0 >= min_tick.0 && e.tick_id.0 <= tick_id.0);

        let pos = entries
            .iter()
            .rposition(|e| &e.action == action && e.window == window)?;
        Some(entries.remove(pos).payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_payload_expires_across_ticks() {
        let mut svc = WindowPendingActionPayloadService::default();
        let window = AppWindowId::default();
        let action = ActionId::from("test.payload_action");

        svc.record(
            window,
            TickId(10),
            action.clone(),
            Box::new(123u32) as Box<dyn Any + Send + Sync>,
        );

        assert!(svc.consume(window, TickId(10), &action).is_some());

        svc.record(
            window,
            TickId(10),
            action.clone(),
            Box::new(456u32) as Box<dyn Any + Send + Sync>,
        );

        // TTL is 64 ticks: at tick 10 + 65, the earlier entry should be stale.
        assert!(svc.consume(window, TickId(75), &action).is_none());
    }

    #[test]
    fn pending_payload_consumes_most_recent() {
        let mut svc = WindowPendingActionPayloadService::default();
        let window = AppWindowId::default();
        let action = ActionId::from("test.payload_action");

        svc.record(
            window,
            TickId(10),
            action.clone(),
            Box::new(1u32) as Box<dyn Any + Send + Sync>,
        );
        svc.record(
            window,
            TickId(11),
            action.clone(),
            Box::new(2u32) as Box<dyn Any + Send + Sync>,
        );

        let payload = svc
            .consume(window, TickId(11), &action)
            .expect("payload must exist");
        let payload = payload.downcast::<u32>().expect("type must match");
        assert_eq!(*payload, 2);
    }
}

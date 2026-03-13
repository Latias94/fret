/// ADR 0034 default bound for effect/event draining within one runner turn.
pub(crate) const MAX_EFFECT_DRAIN_TURNS: usize = 8;

/// Keep draining same-turn work until the turn goes idle or the configured bound is reached.
pub(crate) fn drain_bounded(turn: impl FnMut() -> bool) -> usize {
    drain_bounded_with_limit(MAX_EFFECT_DRAIN_TURNS, turn)
}

pub(crate) fn drain_bounded_with_limit(limit: usize, mut turn: impl FnMut() -> bool) -> usize {
    let mut iterations: usize = 0;
    for _ in 0..limit {
        iterations = iterations.saturating_add(1);
        if !turn() {
            break;
        }
    }
    iterations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_bounded_stops_when_turn_goes_idle() {
        let mut remaining = 2usize;

        let iterations = drain_bounded_with_limit(8, || {
            if remaining == 0 {
                return false;
            }
            remaining = remaining.saturating_sub(1);
            true
        });

        assert_eq!(iterations, 3);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn drain_bounded_respects_limit_when_work_keeps_arriving() {
        let mut iterations_seen = 0usize;

        let iterations = drain_bounded_with_limit(3, || {
            iterations_seen = iterations_seen.saturating_add(1);
            true
        });

        assert_eq!(iterations, 3);
        assert_eq!(iterations_seen, 3);
    }

    #[test]
    fn drain_bounded_with_zero_limit_does_not_run() {
        let mut called = false;

        let iterations = drain_bounded_with_limit(0, || {
            called = true;
            true
        });

        assert_eq!(iterations, 0);
        assert!(!called);
    }
}

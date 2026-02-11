//! Budget helpers for incremental, smooth-by-default work on interactive canvases.
//!
//! This is intentionally policy-light:
//! - It does not decide which tasks should be budgeted.
//! - It does not assume a time source (vsync/RAF, profiling, etc.).
//! - It only provides a simple "work units per frame" accounting mechanism.
//!
//! Motivation and guidance: `docs/adr/0163-canvas-viewport-transform-hit-testing-and-spatial-index-v1.md`.

/// A pair of work budget limits, usually used to degrade background work while the user is
/// interacting with the view.
///
/// This type remains policy-light: it does not define what "interactive" means for a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionBudget {
    pub idle: u32,
    pub interactive: u32,
}

impl InteractionBudget {
    pub const fn new(idle: u32, interactive: u32) -> Self {
        Self { idle, interactive }
    }

    pub const fn select(&self, interacting: bool) -> u32 {
        if interacting {
            self.interactive
        } else {
            self.idle
        }
    }
}

/// A per-frame work budget expressed as "units".
///
/// Callers choose the meaning of a unit:
/// - prepared paths,
/// - shaped text blobs,
/// - index updates,
/// - refined hit tests, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkBudget {
    limit: u32,
    used: u32,
}

impl WorkBudget {
    pub fn new(limit: u32) -> Self {
        Self { limit, used: 0 }
    }

    pub fn limit(&self) -> u32 {
        self.limit
    }

    pub fn used(&self) -> u32 {
        self.used
    }

    pub fn remaining(&self) -> u32 {
        self.limit.saturating_sub(self.used)
    }

    pub fn reset(&mut self) {
        self.used = 0;
    }

    pub fn try_consume(&mut self, units: u32) -> bool {
        let Some(next) = self.used.checked_add(units) else {
            return false;
        };
        if next > self.limit {
            return false;
        }
        self.used = next;
        true
    }

    pub fn consume_up_to(&mut self, units: u32) -> u32 {
        let allowed = self.remaining().min(units);
        self.used = self.used.saturating_add(allowed);
        allowed
    }
}

impl Default for WorkBudget {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_budget_selects_profile() {
        let b = InteractionBudget::new(10, 2);
        assert_eq!(b.select(false), 10);
        assert_eq!(b.select(true), 2);
    }

    #[test]
    fn try_consume_respects_limit() {
        let mut b = WorkBudget::new(3);
        assert!(b.try_consume(1));
        assert!(b.try_consume(2));
        assert!(!b.try_consume(1));
        assert_eq!(b.used(), 3);
        assert_eq!(b.remaining(), 0);
    }

    #[test]
    fn consume_up_to_saturates() {
        let mut b = WorkBudget::new(3);
        assert_eq!(b.consume_up_to(10), 3);
        assert_eq!(b.consume_up_to(1), 0);
        assert_eq!(b.used(), 3);
    }

    #[test]
    fn reset_clears_used() {
        let mut b = WorkBudget::new(3);
        assert!(b.try_consume(3));
        b.reset();
        assert_eq!(b.used(), 0);
        assert_eq!(b.remaining(), 3);
    }
}

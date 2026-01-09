//! Small, reusable hover intent state machine (delay open/close).
//!
//! This is intended for tooltip / hover-card style overlays where the open state depends on
//! pointer hover plus a delay, and we want a deterministic, testable contract.

#[derive(Debug, Clone, Copy)]
pub struct HoverIntentConfig {
    pub open_delay_ticks: u64,
    pub close_delay_ticks: u64,
}

impl HoverIntentConfig {
    pub fn new(open_delay_ticks: u64, close_delay_ticks: u64) -> Self {
        Self {
            open_delay_ticks,
            close_delay_ticks,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct HoverIntentState {
    open: bool,
    hover_start: Option<u64>,
    leave_start: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoverIntentUpdate {
    pub open: bool,
    pub wants_continuous_ticks: bool,
}

impl HoverIntentState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn set_open(&mut self, open: bool) {
        if self.open == open {
            return;
        }
        self.open = open;
        self.hover_start = None;
        self.leave_start = None;
    }

    pub fn update(&mut self, hovered: bool, now: u64, cfg: HoverIntentConfig) -> HoverIntentUpdate {
        if hovered {
            self.leave_start = None;
            if !self.open {
                if cfg.open_delay_ticks == 0 {
                    self.open = true;
                    self.hover_start = None;
                } else {
                    let start = self.hover_start.get_or_insert(now);
                    let elapsed = now.saturating_sub(*start);
                    if elapsed >= cfg.open_delay_ticks {
                        self.open = true;
                        self.hover_start = None;
                    }
                }
            }
        } else {
            self.hover_start = None;
            if self.open {
                if cfg.close_delay_ticks == 0 {
                    self.open = false;
                    self.leave_start = None;
                } else {
                    let start = self.leave_start.get_or_insert(now);
                    let elapsed = now.saturating_sub(*start);
                    if elapsed >= cfg.close_delay_ticks {
                        self.open = false;
                        self.leave_start = None;
                    }
                }
            } else {
                self.leave_start = None;
            }
        }

        let wants_continuous_ticks = (hovered && !self.open && cfg.open_delay_ticks > 0)
            || (!hovered && self.open && cfg.close_delay_ticks > 0);

        HoverIntentUpdate {
            open: self.open,
            wants_continuous_ticks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_after_delay_and_closes_after_delay() {
        let cfg = HoverIntentConfig::new(2, 3);
        let mut st = HoverIntentState::default();

        // Hover for 0,1 ticks: still closed.
        assert_eq!(
            st.update(true, 0, cfg),
            HoverIntentUpdate {
                open: false,
                wants_continuous_ticks: true
            }
        );
        assert_eq!(
            st.update(true, 1, cfg),
            HoverIntentUpdate {
                open: false,
                wants_continuous_ticks: true
            }
        );

        // At tick 2 (elapsed >= 2): open.
        assert_eq!(
            st.update(true, 2, cfg),
            HoverIntentUpdate {
                open: true,
                wants_continuous_ticks: false
            }
        );

        // Leave: close after 3 ticks.
        assert_eq!(
            st.update(false, 3, cfg),
            HoverIntentUpdate {
                open: true,
                wants_continuous_ticks: true
            }
        );
        assert_eq!(
            st.update(false, 5, cfg),
            HoverIntentUpdate {
                open: true,
                wants_continuous_ticks: true
            }
        );
        assert_eq!(
            st.update(false, 6, cfg),
            HoverIntentUpdate {
                open: false,
                wants_continuous_ticks: false
            }
        );
    }

    #[test]
    fn zero_delays_toggle_immediately() {
        let cfg = HoverIntentConfig::new(0, 0);
        let mut st = HoverIntentState::default();

        assert_eq!(
            st.update(true, 10, cfg),
            HoverIntentUpdate {
                open: true,
                wants_continuous_ticks: false
            }
        );
        assert_eq!(
            st.update(false, 11, cfg),
            HoverIntentUpdate {
                open: false,
                wants_continuous_ticks: false
            }
        );
    }

    #[test]
    fn set_open_resets_pending_delays() {
        let cfg = HoverIntentConfig::new(5, 5);
        let mut st = HoverIntentState::default();

        // Begin opening delay.
        let out0 = st.update(true, 0, cfg);
        assert!(!out0.open);
        assert!(out0.wants_continuous_ticks);

        // Force open, then ensure we don't keep "opening" due to stale timers.
        st.set_open(true);
        let out1 = st.update(true, 1, cfg);
        assert!(out1.open);
        assert!(!out1.wants_continuous_ticks);

        // Begin closing delay, then force closed and ensure we stop delaying.
        let out2 = st.update(false, 2, cfg);
        assert!(out2.open);
        assert!(out2.wants_continuous_ticks);

        st.set_open(false);
        let out3 = st.update(false, 3, cfg);
        assert!(!out3.open);
        assert!(!out3.wants_continuous_ticks);
    }
}

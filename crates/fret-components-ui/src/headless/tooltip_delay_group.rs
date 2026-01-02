//! Tooltip provider delay-group logic (Radix-aligned outcomes).
//!
//! Radix tooltips commonly share a delay provider so:
//! - the first tooltip opens after a delay, but
//! - moving between tooltips shortly after closing skips the delay.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooltipDelayGroupConfig {
    pub delay_ticks: u64,
    pub skip_delay_ticks: u64,
}

impl TooltipDelayGroupConfig {
    pub fn new(delay_ticks: u64, skip_delay_ticks: u64) -> Self {
        Self {
            delay_ticks,
            skip_delay_ticks,
        }
    }
}

impl Default for TooltipDelayGroupConfig {
    fn default() -> Self {
        Self {
            delay_ticks: 0,
            skip_delay_ticks: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TooltipDelayGroupState {
    pub last_closed_at: Option<u64>,
}

impl TooltipDelayGroupState {
    pub fn open_delay_ticks(&self, now: u64, cfg: TooltipDelayGroupConfig) -> u64 {
        if cfg.delay_ticks == 0 {
            return 0;
        }
        let Some(last) = self.last_closed_at else {
            return cfg.delay_ticks;
        };
        if cfg.skip_delay_ticks == 0 {
            return cfg.delay_ticks;
        }
        let elapsed = now.saturating_sub(last);
        if elapsed <= cfg.skip_delay_ticks {
            0
        } else {
            cfg.delay_ticks
        }
    }

    pub fn note_closed(&mut self, now: u64) {
        self.last_closed_at = Some(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_delay_is_skipped_within_skip_window() {
        let mut st = TooltipDelayGroupState::default();
        let cfg = TooltipDelayGroupConfig::new(10, 30);

        assert_eq!(st.open_delay_ticks(100, cfg), 10);

        st.note_closed(120);
        assert_eq!(st.open_delay_ticks(121, cfg), 0);
        assert_eq!(st.open_delay_ticks(150, cfg), 0);
        assert_eq!(st.open_delay_ticks(151, cfg), 10);
    }

    #[test]
    fn zero_delay_always_opens_immediately() {
        let mut st = TooltipDelayGroupState::default();
        let cfg = TooltipDelayGroupConfig::new(0, 30);

        assert_eq!(st.open_delay_ticks(10, cfg), 0);
        st.note_closed(20);
        assert_eq!(st.open_delay_ticks(21, cfg), 0);
    }
}

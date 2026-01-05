#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Hidden,
    Opening { start_tick: u64 },
    Open,
    Closing { start_tick: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresenceOutput {
    pub present: bool,
    pub opacity: f32,
    pub animating: bool,
}

/// A tiny "presence" state machine for fade-in/fade-out animations.
///
/// This is a component-layer helper (policy/ergonomics), not a runtime contract. It is intentionally
/// time-source agnostic: the caller supplies a monotonic `tick` (typically frame count).
#[derive(Debug, Clone, Copy)]
pub struct FadePresence {
    open_ticks: u64,
    close_ticks: u64,
    phase: Phase,
}

impl Default for FadePresence {
    fn default() -> Self {
        Self {
            open_ticks: 4,
            close_ticks: 4,
            phase: Phase::Hidden,
        }
    }
}

impl FadePresence {
    pub fn fade_ticks(&self) -> u64 {
        self.open_ticks
    }

    pub fn set_fade_ticks(&mut self, fade_ticks: u64) {
        let ticks = fade_ticks.max(1);
        self.open_ticks = ticks;
        self.close_ticks = ticks;
    }

    pub fn open_ticks(&self) -> u64 {
        self.open_ticks
    }

    pub fn close_ticks(&self) -> u64 {
        self.close_ticks
    }

    pub fn set_open_ticks(&mut self, open_ticks: u64) {
        self.open_ticks = open_ticks.max(1);
    }

    pub fn set_close_ticks(&mut self, close_ticks: u64) {
        self.close_ticks = close_ticks.max(1);
    }

    pub fn set_durations(&mut self, open_ticks: u64, close_ticks: u64) {
        self.open_ticks = open_ticks.max(1);
        self.close_ticks = close_ticks.max(1);
    }

    pub fn update(&mut self, open: bool, tick: u64) -> PresenceOutput {
        if open {
            match self.phase {
                Phase::Hidden | Phase::Closing { .. } => {
                    self.phase = Phase::Opening { start_tick: tick };
                }
                Phase::Opening { .. } | Phase::Open => {}
            }
        } else {
            match self.phase {
                Phase::Open | Phase::Opening { .. } => {
                    self.phase = Phase::Closing { start_tick: tick };
                }
                Phase::Closing { .. } | Phase::Hidden => {}
            }
        }

        match self.phase {
            Phase::Hidden => PresenceOutput {
                present: false,
                opacity: 0.0,
                animating: false,
            },
            Phase::Open => PresenceOutput {
                present: true,
                opacity: 1.0,
                animating: false,
            },
            Phase::Opening { start_tick } => {
                let fade = self.open_ticks.max(1);
                let elapsed = tick.saturating_sub(start_tick).saturating_add(1);
                let t = (elapsed as f32 / fade as f32).clamp(0.0, 1.0);
                let opacity = smoothstep(t);
                if t >= 1.0 {
                    self.phase = Phase::Open;
                    PresenceOutput {
                        present: true,
                        opacity: 1.0,
                        animating: false,
                    }
                } else {
                    PresenceOutput {
                        present: true,
                        opacity,
                        animating: true,
                    }
                }
            }
            Phase::Closing { start_tick } => {
                let fade = self.close_ticks.max(1);
                let elapsed = tick.saturating_sub(start_tick).saturating_add(1);
                let t = (elapsed as f32 / fade as f32).clamp(0.0, 1.0);
                let opacity = smoothstep(1.0 - t);
                if t >= 1.0 {
                    self.phase = Phase::Hidden;
                    PresenceOutput {
                        present: false,
                        opacity: 0.0,
                        animating: false,
                    }
                } else {
                    PresenceOutput {
                        present: true,
                        opacity,
                        animating: true,
                    }
                }
            }
        }
    }
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_then_closes_and_becomes_hidden() {
        let mut p = FadePresence::default();
        p.set_fade_ticks(3);

        let a0 = p.update(true, 0);
        assert!(a0.present);
        assert!(a0.animating);
        assert!(a0.opacity >= 0.0 && a0.opacity <= 1.0);

        let a1 = p.update(true, 1);
        assert!(a1.present);

        let a3 = p.update(true, 3);
        assert!(a3.present);
        assert!(!a3.animating);
        assert_eq!(a3.opacity, 1.0);

        let c0 = p.update(false, 4);
        assert!(c0.present);
        assert!(c0.animating);

        let c3 = p.update(false, 7);
        assert!(!c3.present);
        assert!(!c3.animating);
        assert_eq!(c3.opacity, 0.0);
    }
}

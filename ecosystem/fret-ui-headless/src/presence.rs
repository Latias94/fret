use crate::transition::TransitionTimeline;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresenceOutput {
    pub present: bool,
    pub opacity: f32,
    pub animating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleFadePresenceOutput {
    pub present: bool,
    pub opacity: f32,
    pub scale: f32,
    pub animating: bool,
}

/// A tiny "presence" state machine for fade-in/fade-out animations.
///
/// This is a component-layer helper (policy/ergonomics), not a runtime contract. It is
/// intentionally time-source agnostic: the caller supplies a monotonic `tick` (typically frame
/// count).
#[derive(Debug, Clone, Copy, Default)]
pub struct FadePresence {
    timeline: TransitionTimeline,
}

impl FadePresence {
    pub fn fade_ticks(&self) -> u64 {
        self.timeline.open_ticks()
    }

    pub fn set_fade_ticks(&mut self, fade_ticks: u64) {
        let ticks = fade_ticks.max(1);
        self.timeline.set_durations(ticks, ticks);
    }

    pub fn open_ticks(&self) -> u64 {
        self.timeline.open_ticks()
    }

    pub fn close_ticks(&self) -> u64 {
        self.timeline.close_ticks()
    }

    pub fn set_open_ticks(&mut self, open_ticks: u64) {
        self.timeline.set_open_ticks(open_ticks);
    }

    pub fn set_close_ticks(&mut self, close_ticks: u64) {
        self.timeline.set_close_ticks(close_ticks);
    }

    pub fn set_durations(&mut self, open_ticks: u64, close_ticks: u64) {
        self.timeline.set_durations(open_ticks, close_ticks);
    }

    pub fn update(&mut self, open: bool, tick: u64) -> PresenceOutput {
        self.update_with_easing(open, tick, crate::easing::smoothstep)
    }

    /// Like [`FadePresence::update`], but allows callers to provide an easing function.
    ///
    /// This is useful for matching CSS-style easing curves (e.g. cubic-bezier) without changing
    /// the default `smoothstep` behavior across the codebase.
    pub fn update_with_easing(
        &mut self,
        open: bool,
        tick: u64,
        ease: fn(f32) -> f32,
    ) -> PresenceOutput {
        let out = self.timeline.update_with_easing(open, tick, ease);
        PresenceOutput {
            present: out.present,
            opacity: out.progress,
            animating: out.animating,
        }
    }
}

/// A tiny "presence" state machine for scale+fade animations (e.g. shadcn-like zoom-in/out).
///
/// This is a deterministic wrapper over `TransitionTimeline` that maps the eased progress to:
/// - opacity (`0..1`), and
/// - scale (`from_scale..to_scale`).
#[derive(Debug, Clone, Copy)]
pub struct ScaleFadePresence {
    timeline: TransitionTimeline,
    from_scale: f32,
    to_scale: f32,
}

impl Default for ScaleFadePresence {
    fn default() -> Self {
        Self {
            timeline: TransitionTimeline::default(),
            from_scale: 0.95,
            to_scale: 1.0,
        }
    }
}

impl ScaleFadePresence {
    pub fn new(from_scale: f32, to_scale: f32) -> Self {
        Self {
            timeline: TransitionTimeline::default(),
            from_scale,
            to_scale,
        }
    }

    pub fn from_scale(&self) -> f32 {
        self.from_scale
    }

    pub fn to_scale(&self) -> f32 {
        self.to_scale
    }

    pub fn set_scales(&mut self, from_scale: f32, to_scale: f32) {
        self.from_scale = from_scale;
        self.to_scale = to_scale;
    }

    pub fn open_ticks(&self) -> u64 {
        self.timeline.open_ticks()
    }

    pub fn close_ticks(&self) -> u64 {
        self.timeline.close_ticks()
    }

    pub fn set_open_ticks(&mut self, open_ticks: u64) {
        self.timeline.set_open_ticks(open_ticks);
    }

    pub fn set_close_ticks(&mut self, close_ticks: u64) {
        self.timeline.set_close_ticks(close_ticks);
    }

    pub fn set_durations(&mut self, open_ticks: u64, close_ticks: u64) {
        self.timeline.set_durations(open_ticks, close_ticks);
    }

    pub fn update(&mut self, open: bool, tick: u64) -> ScaleFadePresenceOutput {
        self.update_with_easing(open, tick, crate::easing::smoothstep)
    }

    pub fn update_with_easing(
        &mut self,
        open: bool,
        tick: u64,
        ease: fn(f32) -> f32,
    ) -> ScaleFadePresenceOutput {
        let out = self.timeline.update_with_easing(open, tick, ease);
        let scale = self.from_scale + (self.to_scale - self.from_scale) * out.progress;
        ScaleFadePresenceOutput {
            present: out.present,
            opacity: out.progress,
            scale,
            animating: out.animating,
        }
    }
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

    #[test]
    fn update_with_easing_can_use_linear_progress() {
        let mut p = FadePresence::default();
        p.set_fade_ticks(4);

        let a0 = p.update_with_easing(true, 0, crate::easing::linear);
        assert!(a0.present);
        assert!(a0.animating);
        assert!((a0.opacity - 0.25).abs() < 1e-6);

        let a3 = p.update_with_easing(true, 3, crate::easing::linear);
        assert!(a3.present);
        assert!(!a3.animating);
        assert!((a3.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn scale_fade_presence_interpolates_scale_and_opacity() {
        let mut p = ScaleFadePresence::new(0.8, 1.0);
        p.set_durations(4, 4);

        let a0 = p.update_with_easing(true, 0, crate::easing::linear);
        assert!(a0.present);
        assert!(a0.animating);
        assert!((a0.opacity - 0.25).abs() < 1e-6);
        assert!((a0.scale - 0.85).abs() < 1e-6);

        let a3 = p.update_with_easing(true, 3, crate::easing::linear);
        assert!(a3.present);
        assert!(!a3.animating);
        assert!((a3.opacity - 1.0).abs() < 1e-6);
        assert!((a3.scale - 1.0).abs() < 1e-6);
    }
}

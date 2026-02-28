#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBodySnapshot {
    pub direction: f32,
    pub duration: f32,
    pub velocity: f32,
    pub location: f32,
    pub previous_location: f32,
    pub target: f32,
}

/// Embla-style scroll integrator.
///
/// Upstream reference:
/// - `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollBody.ts`
///
/// Notes:
/// - This uses Embla's "duration" semantics (a numeric integrator parameter), not wall-clock time.
/// - `seek()` represents one integrator step (typically one rendered frame).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBody {
    base_duration: f32,
    base_friction: f32,

    scroll_velocity: f32,
    scroll_direction: f32,
    scroll_duration: f32,
    scroll_friction: f32,

    location: f32,
    offset_location: f32,
    previous_location: f32,
    target: f32,

    raw_location: f32,
    raw_location_previous: f32,
}

impl ScrollBody {
    pub fn new(location: f32, base_duration: f32, base_friction: f32) -> Self {
        Self {
            base_duration,
            base_friction,

            scroll_velocity: 0.0,
            scroll_direction: 0.0,
            scroll_duration: base_duration,
            scroll_friction: base_friction,

            location,
            offset_location: location,
            previous_location: location,
            target: location,

            raw_location: location,
            raw_location_previous: 0.0,
        }
    }

    pub fn snapshot(&self) -> ScrollBodySnapshot {
        ScrollBodySnapshot {
            direction: self.scroll_direction,
            duration: self.scroll_duration,
            velocity: self.scroll_velocity,
            location: self.location,
            previous_location: self.previous_location,
            target: self.target,
        }
    }

    pub fn location(&self) -> f32 {
        self.location
    }

    pub fn previous_location(&self) -> f32 {
        self.previous_location
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn set_location(&mut self, location: f32) {
        self.location = location;
        self.offset_location = location;
        self.previous_location = location;
        self.raw_location = location;
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn add_target(&mut self, delta: f32) {
        self.target += delta;
    }

    /// Applies a loop offset to all location-like values while preserving velocity/integration state.
    ///
    /// This mirrors Embla's `ScrollLooper` behavior where the loop distance is added to both
    /// location and target entities without resetting motion.
    pub fn add_loop_distance(&mut self, delta: f32) {
        self.location += delta;
        self.offset_location += delta;
        self.previous_location += delta;
        self.target += delta;
        self.raw_location += delta;
        self.raw_location_previous += delta;
    }

    pub fn duration(&self) -> f32 {
        self.scroll_duration
    }

    pub fn friction(&self) -> f32 {
        self.scroll_friction
    }

    pub fn use_duration(&mut self, duration: f32) -> &mut Self {
        self.scroll_duration = duration;
        self
    }

    pub fn use_friction(&mut self, friction: f32) -> &mut Self {
        self.scroll_friction = friction;
        self
    }

    pub fn set_base_duration(&mut self, base_duration: f32) {
        let was_base = (self.scroll_duration - self.base_duration).abs() <= 0.0001;
        self.base_duration = base_duration;
        if was_base {
            self.scroll_duration = base_duration;
        }
    }

    pub fn set_base_friction(&mut self, base_friction: f32) {
        let was_base = (self.scroll_friction - self.base_friction).abs() <= 0.0001;
        self.base_friction = base_friction;
        if was_base {
            self.scroll_friction = base_friction;
        }
    }

    pub fn use_base_duration(&mut self) -> &mut Self {
        self.scroll_duration = self.base_duration;
        self
    }

    pub fn use_base_friction(&mut self) -> &mut Self {
        self.scroll_friction = self.base_friction;
        self
    }

    /// One integrator step.
    pub fn seek(&mut self) -> &mut Self {
        let displacement = self.target - self.location;
        let is_instant = self.scroll_duration == 0.0;
        let scroll_distance;

        if is_instant {
            self.scroll_velocity = 0.0;
            self.previous_location = self.target;
            self.location = self.target;
            self.offset_location = self.target;
            self.raw_location = self.target;
            scroll_distance = displacement;
        } else {
            self.previous_location = self.location;

            self.scroll_velocity += displacement / self.scroll_duration;
            self.scroll_velocity *= self.scroll_friction;
            self.raw_location += self.scroll_velocity;
            self.location += self.scroll_velocity;
            self.offset_location = self.location;

            scroll_distance = self.raw_location - self.raw_location_previous;
        }

        self.scroll_direction = scroll_distance.signum();
        self.raw_location_previous = self.raw_location;
        self
    }

    pub fn settled(&self) -> bool {
        let displacement = self.target - self.offset_location;
        displacement.abs() < 0.001
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_body_instant_seek_jumps_to_target_and_sets_direction() {
        let mut body = ScrollBody::new(0.0, 25.0, 0.9);
        body.use_duration(0.0);
        body.set_target(10.0);
        body.seek();

        let snap = body.snapshot();
        assert_eq!(snap.location, 10.0);
        assert_eq!(snap.previous_location, 10.0);
        assert_eq!(snap.velocity, 0.0);
        assert_eq!(snap.direction, 1.0);
        assert!(body.settled());
    }

    #[test]
    fn scroll_body_converges_towards_target_with_friction() {
        let mut body = ScrollBody::new(0.0, 25.0, 0.9);
        body.set_target(100.0);

        for _ in 0..240 {
            body.seek();
        }

        let snap = body.snapshot();
        assert!(snap.location > 0.0);
        assert!((snap.target - snap.location).abs() < 1.0);
        assert!(body.settled());
    }

    #[test]
    fn lower_duration_moves_faster_per_tick() {
        let mut fast = ScrollBody::new(0.0, 10.0, 0.9);
        let mut slow = ScrollBody::new(0.0, 50.0, 0.9);
        fast.set_target(100.0);
        slow.set_target(100.0);

        fast.seek();
        slow.seek();

        assert!(fast.location() > slow.location());
        assert!(fast.location() > 0.0);
        assert!(slow.location() > 0.0);
    }
}

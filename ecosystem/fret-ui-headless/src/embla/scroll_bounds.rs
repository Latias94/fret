use crate::embla::limit::Limit;
use crate::embla::scroll_body::ScrollBody;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBoundsConfig {
    pub view_size: f32,
}

/// Ported from Embla `ScrollBounds`.
///
/// Upstream: `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollBounds.ts`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBounds {
    pull_back_threshold: f32,
    edge_offset_tolerance: f32,
    disabled: bool,
}

impl ScrollBounds {
    pub fn new(cfg: ScrollBoundsConfig) -> Self {
        let pull_back_threshold = cfg.view_size * 0.10;
        let edge_offset_tolerance = cfg.view_size * 0.50;
        Self {
            pull_back_threshold,
            edge_offset_tolerance,
            disabled: false,
        }
    }

    pub fn toggle_active(&mut self, active: bool) {
        self.disabled = !active;
    }

    pub fn should_constrain(&self, limit: Limit, location: f32, target: f32) -> bool {
        if self.disabled {
            return false;
        }
        if !limit.past_any_bound(target) {
            return false;
        }
        if !limit.past_any_bound(location) {
            return false;
        }
        true
    }

    pub fn constrain(&self, limit: Limit, scroll_body: &mut ScrollBody, pointer_down: bool) {
        let location = scroll_body.location();
        let target = scroll_body.target();
        if !self.should_constrain(limit, location, target) {
            return;
        }

        let is_past_min = limit.past_min_bound(location);
        let edge = if is_past_min { limit.min } else { limit.max };
        let diff_to_edge = (edge - location).abs();
        let displacement = target - location;

        let edge_offset_tolerance = self.edge_offset_tolerance.max(0.0);
        let friction = if edge_offset_tolerance == 0.0 {
            0.99
        } else {
            // Clamp to Embla's `Limit(0.1, 0.99)`.
            let raw = diff_to_edge / edge_offset_tolerance;
            raw.clamp(0.1, 0.99)
        };

        scroll_body.set_target(target - displacement * friction);

        if !pointer_down && displacement.abs() < self.pull_back_threshold {
            scroll_body.set_target(limit.clamp(scroll_body.target()));
            scroll_body.use_duration(25.0).use_base_friction();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_bounds_constrains_target_when_past_bounds() {
        let limit = Limit::new(-300.0, 0.0);
        let mut body = ScrollBody::new(-350.0, 25.0, 0.68);
        body.set_target(-350.0);

        let bounds = ScrollBounds::new(ScrollBoundsConfig { view_size: 320.0 });
        bounds.constrain(limit, &mut body, false);

        assert!(body.target() > -350.0);
    }
}

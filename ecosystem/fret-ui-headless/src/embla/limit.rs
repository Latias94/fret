/// A numeric clamp/loop helper ported from Embla `Limit`.
///
/// Upstream: `repo-ref/embla-carousel/packages/embla-carousel/src/components/Limit.ts`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limit {
    pub min: f32,
    pub max: f32,
    pub length: f32,
}

impl Limit {
    pub fn new(min: f32, max: f32) -> Self {
        let length = (min - max).abs();
        Self { min, max, length }
    }

    pub fn past_min_bound(&self, input: f32) -> bool {
        input < self.min
    }

    pub fn past_max_bound(&self, input: f32) -> bool {
        input > self.max
    }

    pub fn past_any_bound(&self, input: f32) -> bool {
        self.past_min_bound(input) || self.past_max_bound(input)
    }

    pub fn clamp(&self, input: f32) -> f32 {
        if !self.past_any_bound(input) {
            return input;
        }
        if self.past_min_bound(input) {
            self.min
        } else {
            self.max
        }
    }

    pub fn remove_offset(&self, input: f32) -> f32 {
        if self.length == 0.0 {
            return input;
        }
        // input - length * ceil((input - max) / length)
        let k = ((input - self.max) / self.length).ceil();
        input - (self.length * k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_clamps_to_bounds() {
        let limit = Limit::new(-100.0, 0.0);
        assert_eq!(limit.clamp(-150.0), -100.0);
        assert_eq!(limit.clamp(10.0), 0.0);
        assert_eq!(limit.clamp(-42.0), -42.0);
    }

    #[test]
    fn limit_remove_offset_wraps_with_length() {
        let limit = Limit::new(-100.0, 0.0);
        // length=100, max=0
        assert_eq!(limit.remove_offset(0.0), 0.0);
        assert_eq!(limit.remove_offset(-50.0), -50.0);
        assert_eq!(limit.remove_offset(-150.0), -50.0);
        assert_eq!(limit.remove_offset(50.0), -50.0);
    }
}

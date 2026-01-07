#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    domain: (f32, f32),
    range: (f32, f32),
}

impl LinearScale {
    pub fn new(domain: (f32, f32), range: (f32, f32)) -> Self {
        Self { domain, range }
    }

    pub fn map(self, v: f32) -> Option<f32> {
        if !v.is_finite() {
            return None;
        }
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;
        let denom = d1 - d0;
        if !denom.is_finite() || denom == 0.0 {
            return None;
        }
        let t = (v - d0) / denom;
        let out = r0 + (r1 - r0) * t;
        out.is_finite().then_some(out)
    }

    pub fn invert(self, v: f32) -> Option<f32> {
        if !v.is_finite() {
            return None;
        }
        let (r0, r1) = self.range;
        let (d0, d1) = self.domain;
        let denom = r1 - r0;
        if !denom.is_finite() || denom == 0.0 {
            return None;
        }
        let t = (v - r0) / denom;
        let out = d0 + (d1 - d0) * t;
        out.is_finite().then_some(out)
    }
}

use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickCircle2d {
    pub center: Vec2,
    pub radius: f32,
}

impl PickCircle2d {
    pub fn distance_to_center(self, p: Vec2) -> f32 {
        (p - self.center).length()
    }

    pub fn hit_distance(self, p: Vec2) -> Option<f32> {
        let d = self.distance_to_center(p);
        (d.is_finite() && self.radius.is_finite() && d <= self.radius).then_some(d)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickSegmentCapsule2d {
    pub a: Vec2,
    pub b: Vec2,
    pub radius: f32,
}

impl PickSegmentCapsule2d {
    pub fn distance_to_segment(self, p: Vec2) -> f32 {
        distance_point_to_segment_px(p, self.a, self.b)
    }

    pub fn hit_distance(self, p: Vec2) -> Option<f32> {
        let d = self.distance_to_segment(p);
        (d.is_finite() && self.radius.is_finite() && d <= self.radius).then_some(d)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickConvexQuad2d {
    pub points: [Vec2; 4],
}

impl PickConvexQuad2d {
    pub fn contains(self, p: Vec2) -> bool {
        point_in_convex_quad(p, self.points)
    }

    pub fn edge_distance(self, p: Vec2) -> f32 {
        quad_edge_distance(p, self.points)
    }
}

pub fn distance_point_to_segment_px(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = if ab.length_squared() > 0.0 {
        ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let q = a + ab * t;
    (p - q).length()
}

pub fn point_in_convex_quad(p: Vec2, q: [Vec2; 4]) -> bool {
    fn cross(a: Vec2, b: Vec2) -> f32 {
        a.x * b.y - a.y * b.x
    }

    let mut sign = 0.0f32;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let c = cross(b - a, p - a);
        if c.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = c;
        } else if sign.signum() != c.signum() {
            return false;
        }
    }
    true
}

pub fn quad_edge_distance(p: Vec2, q: [Vec2; 4]) -> f32 {
    let mut best = f32::INFINITY;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        best = best.min(distance_point_to_segment_px(p, a, b));
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capsule_hit_distance_matches_radius() {
        let capsule = PickSegmentCapsule2d {
            a: Vec2::new(0.0, 0.0),
            b: Vec2::new(10.0, 0.0),
            radius: 1.0,
        };

        assert!(capsule.hit_distance(Vec2::new(5.0, 0.0)).is_some());
        assert!(capsule.hit_distance(Vec2::new(5.0, 1.0)).is_some());
        assert!(capsule.hit_distance(Vec2::new(5.0, 1.1)).is_none());
    }

    #[test]
    fn convex_quad_contains_center_and_rejects_outside() {
        let q = PickConvexQuad2d {
            points: [
                Vec2::new(0.0, 0.0),
                Vec2::new(2.0, 0.0),
                Vec2::new(2.0, 2.0),
                Vec2::new(0.0, 2.0),
            ],
        };

        assert!(q.contains(Vec2::new(1.0, 1.0)));
        assert!(!q.contains(Vec2::new(3.0, 1.0)));
    }

    #[test]
    fn quad_edge_distance_is_zero_on_edge() {
        let q = PickConvexQuad2d {
            points: [
                Vec2::new(0.0, 0.0),
                Vec2::new(2.0, 0.0),
                Vec2::new(2.0, 2.0),
                Vec2::new(0.0, 2.0),
            ],
        };

        let d = q.edge_distance(Vec2::new(1.0, 0.0));
        assert!(d.abs() < 1e-5, "expected near zero, got {d}");
    }
}

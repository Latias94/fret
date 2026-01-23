use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Px(pub f32);

impl From<f32> for Px {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Add for Px {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Px {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<f32> for Px {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div<f32> for Px {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: Px,
    pub y: Px,
}

impl Point {
    pub const fn new(x: Px, y: Px) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: Px,
    pub height: Px,
}

impl Size {
    pub const fn new(width: Px, height: Px) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub fn contains(&self, point: Point) -> bool {
        let x0 = self.origin.x.0;
        let y0 = self.origin.y.0;
        let x1 = x0 + self.size.width.0;
        let y1 = y0 + self.size.height.0;

        point.x.0 >= x0 && point.x.0 < x1 && point.y.0 >= y0 && point.y.0 < y1
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RectPx {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl RectPx {
    pub const fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub const fn full(w: u32, h: u32) -> Self {
        Self { x: 0, y: 0, w, h }
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Corners {
    pub top_left: Px,
    pub top_right: Px,
    pub bottom_right: Px,
    pub bottom_left: Px,
}

impl Corners {
    pub const fn all(radius: Px) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Edges {
    pub top: Px,
    pub right: Px,
    pub bottom: Px,
    pub left: Px,
}

impl Edges {
    pub const fn all(value: Px) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: Px, vertical: Px) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// A 2D affine transform in logical pixels.
///
/// Matrix form (applied to column vectors):
///
/// ```text
/// | a  c  tx |
/// | b  d  ty |
/// | 0  0  1  |
/// ```
///
/// So:
/// - `x' = a*x + c*y + tx`
/// - `y' = b*x + d*y + ty`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub const fn translation(delta: Point) -> Self {
        Self {
            tx: delta.x.0,
            ty: delta.y.0,
            ..Self::IDENTITY
        }
    }

    pub const fn scale_uniform(s: f32) -> Self {
        Self {
            a: s,
            d: s,
            ..Self::IDENTITY
        }
    }

    pub fn rotation_radians(theta: f32) -> Self {
        let (sin, cos) = theta.sin_cos();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            ..Self::IDENTITY
        }
    }

    pub fn rotation_degrees(degrees: f32) -> Self {
        Self::rotation_radians(degrees.to_radians())
    }

    pub fn rotation_about_radians(theta: f32, center: Point) -> Self {
        let to_center = Self::translation(center);
        let from_center = Self::translation(Point::new(Px(-center.x.0), Px(-center.y.0)));
        to_center * Self::rotation_radians(theta) * from_center
    }

    pub fn rotation_about_degrees(degrees: f32, center: Point) -> Self {
        Self::rotation_about_radians(degrees.to_radians(), center)
    }

    /// Matrix composition: `self * rhs`.
    ///
    /// This means: apply `rhs` first, then `self`.
    pub fn compose(self, rhs: Self) -> Self {
        Self {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            tx: self.a * rhs.tx + self.c * rhs.ty + self.tx,
            ty: self.b * rhs.tx + self.d * rhs.ty + self.ty,
        }
    }

    pub fn apply_point(self, p: Point) -> Point {
        Point::new(
            Px(self.a * p.x.0 + self.c * p.y.0 + self.tx),
            Px(self.b * p.x.0 + self.d * p.y.0 + self.ty),
        )
    }

    pub fn inverse(self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        if !det.is_finite() || det == 0.0 {
            return None;
        }
        let inv_det = 1.0 / det;
        let ia = self.d * inv_det;
        let ib = -self.b * inv_det;
        let ic = -self.c * inv_det;
        let id = self.a * inv_det;

        let itx = -(ia * self.tx + ic * self.ty);
        let ity = -(ib * self.tx + id * self.ty);

        Some(Self {
            a: ia,
            b: ib,
            c: ic,
            d: id,
            tx: itx,
            ty: ity,
        })
    }

    /// Converts a logical-px transform to a physical-px transform.
    ///
    /// If you already have coordinates multiplied by `scale_factor`, apply the returned transform
    /// directly in physical pixels.
    pub fn to_physical_px(self, scale_factor: f32) -> Self {
        Self {
            tx: self.tx * scale_factor,
            ty: self.ty * scale_factor,
            ..self
        }
    }

    /// Returns `(scale, translation)` if this is a translation + uniform scale transform.
    pub fn as_translation_uniform_scale(self) -> Option<(f32, Point)> {
        if !self.a.is_finite()
            || !self.b.is_finite()
            || !self.c.is_finite()
            || !self.d.is_finite()
            || !self.tx.is_finite()
            || !self.ty.is_finite()
        {
            return None;
        }

        if self.b != 0.0 || self.c != 0.0 || self.a != self.d {
            return None;
        }
        Some((self.a, Point::new(Px(self.tx), Px(self.ty))))
    }
}

impl std::ops::Mul for Transform2D {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.compose(rhs)
    }
}

impl std::ops::MulAssign for Transform2D {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.compose(rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_inverse_roundtrips_point() {
        let t = Transform2D {
            a: 2.0,
            b: 1.0,
            c: -0.5,
            d: 1.5,
            tx: 10.0,
            ty: -7.0,
        };
        let inv = t.inverse().expect("invertible");
        let p = Point::new(Px(3.0), Px(4.0));
        let p2 = inv.apply_point(t.apply_point(p));
        assert!((p2.x.0 - p.x.0).abs() < 1e-4);
        assert!((p2.y.0 - p.y.0).abs() < 1e-4);
    }
}

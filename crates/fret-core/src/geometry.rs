use std::ops::{Add, Div, Mul, Sub};

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
}

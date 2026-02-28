use fret_core::WindowLogicalPosition;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct WindowLogicalSize {
    pub width: f64,
    pub height: f64,
}

impl WindowLogicalSize {
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WindowPhysicalPosition {
    pub x: i32,
    pub y: i32,
}

impl WindowPhysicalPosition {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowPosition {
    Logical(WindowLogicalPosition),
    Physical(WindowPhysicalPosition),
}

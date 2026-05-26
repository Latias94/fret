use fret_core::{Color, Point, SceneMeshVertex, UvPoint};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawVertex {
    pub position: Point,
    pub uv: UvPoint,
    pub color: Color,
}

impl DebugDrawVertex {
    pub const fn new(position: Point, uv: UvPoint, color: Color) -> Self {
        Self {
            position,
            uv,
            color,
        }
    }

    pub const fn colored(position: Point, color: Color) -> Self {
        Self {
            position,
            uv: UvPoint::ZERO,
            color,
        }
    }

    pub(in crate::imui::debug_draw_controls) fn scene_vertex(self) -> SceneMeshVertex {
        SceneMeshVertex::new(self.position, self.uv, self.color)
    }
}

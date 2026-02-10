use fret_core::{
    Axis, Color, Corners, CursorIcon, DrawOrder, Edges, Paint, Px, Rect, Scene, SceneOp, Size,
    geometry::Point,
};

#[derive(Debug, Clone, Copy)]
pub struct ResizeHandle {
    pub axis: Axis,
    /// Hit-test thickness in logical px.
    pub hit_thickness: Px,
    /// Visual thickness in *device* pixels (converted using the current scale factor).
    pub paint_device_px: f32,
}

impl ResizeHandle {
    pub fn cursor_icon(self) -> CursorIcon {
        match self.axis {
            Axis::Horizontal => CursorIcon::ColResize,
            Axis::Vertical => CursorIcon::RowResize,
        }
    }

    pub fn hit_rect(self, bounds: Rect, center: f32) -> Rect {
        match self.axis {
            Axis::Horizontal => Rect {
                origin: Point::new(Px(center - self.hit_thickness.0 * 0.5), bounds.origin.y),
                size: Size::new(self.hit_thickness, bounds.size.height),
            },
            Axis::Vertical => Rect {
                origin: Point::new(bounds.origin.x, Px(center - self.hit_thickness.0 * 0.5)),
                size: Size::new(bounds.size.width, self.hit_thickness),
            },
        }
    }

    pub fn paint_rect(self, bounds: Rect, center: f32, scale_factor: f32) -> Rect {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        let thickness = Px(self.paint_device_px / scale_factor);

        match self.axis {
            Axis::Horizontal => Rect {
                origin: Point::new(Px(center - thickness.0 * 0.5), bounds.origin.y),
                size: Size::new(thickness, bounds.size.height),
            },
            Axis::Vertical => Rect {
                origin: Point::new(bounds.origin.x, Px(center - thickness.0 * 0.5)),
                size: Size::new(bounds.size.width, thickness),
            },
        }
    }

    pub fn paint(
        self,
        scene: &mut Scene,
        order: DrawOrder,
        bounds: Rect,
        center: f32,
        scale_factor: f32,
        color: Color,
    ) {
        let rect = self.paint_rect(bounds, center, scale_factor);
        scene.push(SceneOp::Quad {
            order,
            rect,
            background: Paint::Solid(color),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT),
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}

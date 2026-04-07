use fret_core::{Point, Rect};

use crate::core::CanvasPoint;

use super::minimap_projection::{
    pan_to_center_canvas_point, project_world_rect_to_minimap, unproject_minimap_point,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MiniMapDragPlan {
    pub(super) start_canvas: Point,
    pub(super) start_pan: CanvasPoint,
    pub(super) immediate_pan: Option<CanvasPoint>,
}

pub(super) fn plan_minimap_drag_start(
    minimap: Rect,
    world: Rect,
    viewport: Rect,
    pointer: Point,
    current_pan: CanvasPoint,
    zoom: f32,
    canvas_bounds: Rect,
) -> Option<MiniMapDragPlan> {
    let start_canvas = unproject_minimap_point(minimap, world, pointer)?;
    let viewport_rect = project_world_rect_to_minimap(minimap, world, viewport);

    if viewport_rect.contains(pointer) {
        return Some(MiniMapDragPlan {
            start_canvas,
            start_pan: current_pan,
            immediate_pan: None,
        });
    }

    let centered_pan = pan_to_center_canvas_point(canvas_bounds, zoom, start_canvas);
    Some(MiniMapDragPlan {
        start_canvas,
        start_pan: centered_pan,
        immediate_pan: Some(centered_pan),
    })
}

pub(super) fn plan_minimap_drag_pan(
    minimap: Rect,
    world: Rect,
    pointer: Point,
    start_canvas: Point,
    start_pan: CanvasPoint,
) -> Option<CanvasPoint> {
    let canvas_point = unproject_minimap_point(minimap, world, pointer)?;
    let dx = canvas_point.x.0 - start_canvas.x.0;
    let dy = canvas_point.y.0 - start_canvas.y.0;
    Some(CanvasPoint {
        x: start_pan.x - dx,
        y: start_pan.y - dy,
    })
}

#[cfg(test)]
mod tests {
    use super::{MiniMapDragPlan, plan_minimap_drag_pan, plan_minimap_drag_start};
    use crate::core::CanvasPoint;
    use fret_core::{Point, Px, Rect, Size};

    fn canvas_bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        )
    }

    fn minimap_bounds() -> Rect {
        Rect::new(
            Point::new(Px(590.0), Px(470.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn minimap_drag_start_keeps_current_pan_inside_viewport() {
        let current_pan = CanvasPoint { x: 10.0, y: 20.0 };
        let viewport = canvas_bounds();
        let plan = plan_minimap_drag_start(
            minimap_bounds(),
            viewport,
            viewport,
            Point::new(Px(690.0), Px(530.0)),
            current_pan,
            1.0,
            canvas_bounds(),
        )
        .expect("drag plan");

        assert_eq!(
            plan,
            MiniMapDragPlan {
                start_canvas: Point::new(Px(400.0), Px(300.0)),
                start_pan: current_pan,
                immediate_pan: None
            }
        );
    }

    #[test]
    fn minimap_drag_start_recenters_when_pointer_is_outside_viewport_rect() {
        let world = Rect::new(
            Point::new(Px(-200.0), Px(-150.0)),
            Size::new(Px(1200.0), Px(900.0)),
        );
        let viewport = canvas_bounds();
        let plan = plan_minimap_drag_start(
            minimap_bounds(),
            world,
            viewport,
            Point::new(Px(780.0), Px(580.0)),
            CanvasPoint { x: 0.0, y: 0.0 },
            1.0,
            canvas_bounds(),
        )
        .expect("drag plan");

        assert!(plan.immediate_pan.is_some());
        assert_eq!(plan.start_pan, plan.immediate_pan.expect("immediate pan"));
    }

    #[test]
    fn minimap_drag_pan_translates_world_delta_back_to_pan_delta() {
        let world = canvas_bounds();
        let pan = plan_minimap_drag_pan(
            minimap_bounds(),
            world,
            Point::new(Px(700.0), Px(530.0)),
            Point::new(Px(400.0), Px(300.0)),
            CanvasPoint { x: 0.0, y: 0.0 },
        )
        .expect("pan plan");

        assert!((pan.x + 50.0).abs() <= 1.0e-4);
        assert!((pan.y - 0.0).abs() <= 1.0e-4);
    }
}

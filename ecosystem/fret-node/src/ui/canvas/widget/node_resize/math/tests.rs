use super::apply_resize_handle;
use crate::core::{CanvasPoint, CanvasRect, CanvasSize};
use crate::io::NodeGraphNodeOrigin;
use crate::ui::canvas::state::NodeResizeHandle;
use fret_core::{Point, Px};

#[test]
fn resize_right_increases_width_and_keeps_origin() {
    let start_pos = CanvasPoint { x: 10.0, y: 20.0 };
    let start_size_px = CanvasSize {
        width: 100.0,
        height: 50.0,
    };
    let start_pointer = Point::new(Px(0.0), Px(0.0));
    let pointer = Point::new(Px(10.0), Px(0.0));
    let zoom = 1.0;
    let min = CanvasSize {
        width: 10.0,
        height: 10.0,
    };

    let (pos, size) = apply_resize_handle(
        NodeResizeHandle::Right,
        false,
        start_pos,
        NodeGraphNodeOrigin::default(),
        start_size_px,
        start_pointer,
        pointer,
        zoom,
        min,
        None,
        None,
        None,
    );
    assert_eq!(pos, start_pos);
    assert_eq!(size.width, 110.0);
    assert_eq!(size.height, 50.0);
}

#[test]
fn resize_left_moves_origin_and_keeps_right_edge() {
    let start_pos = CanvasPoint { x: 10.0, y: 20.0 };
    let start_size_px = CanvasSize {
        width: 100.0,
        height: 50.0,
    };
    let start_pointer = Point::new(Px(0.0), Px(0.0));
    let pointer = Point::new(Px(10.0), Px(0.0));
    let zoom = 1.0;
    let min = CanvasSize {
        width: 10.0,
        height: 10.0,
    };

    let (pos, size) = apply_resize_handle(
        NodeResizeHandle::Left,
        false,
        start_pos,
        NodeGraphNodeOrigin::default(),
        start_size_px,
        start_pointer,
        pointer,
        zoom,
        min,
        None,
        None,
        None,
    );
    assert_eq!(pos.x, 20.0);
    assert_eq!(pos.y, 20.0);
    assert_eq!(size.width, 90.0);
    assert_eq!(size.height, 50.0);
}

#[test]
fn resize_respects_node_extent_bounds() {
    let start_pos = CanvasPoint { x: 0.0, y: 0.0 };
    let start_size_px = CanvasSize {
        width: 100.0,
        height: 50.0,
    };
    let start_pointer = Point::new(Px(0.0), Px(0.0));
    let pointer = Point::new(Px(200.0), Px(0.0));
    let zoom = 1.0;
    let min = CanvasSize {
        width: 10.0,
        height: 10.0,
    };
    let extent = CanvasRect {
        origin: CanvasPoint { x: 0.0, y: 0.0 },
        size: CanvasSize {
            width: 120.0,
            height: 120.0,
        },
    };

    let (_pos, size) = apply_resize_handle(
        NodeResizeHandle::Right,
        false,
        start_pos,
        NodeGraphNodeOrigin::default(),
        start_size_px,
        start_pointer,
        pointer,
        zoom,
        min,
        None,
        Some(extent),
        None,
    );
    assert_eq!(size.width, 120.0);
}

#[test]
fn resize_snaps_moved_edge_to_grid_when_enabled() {
    let start_pos = CanvasPoint { x: 0.0, y: 0.0 };
    let start_size_px = CanvasSize {
        width: 100.0,
        height: 50.0,
    };
    let start_pointer = Point::new(Px(0.0), Px(0.0));
    let pointer = Point::new(Px(7.0), Px(0.0));
    let zoom = 1.0;
    let min = CanvasSize {
        width: 10.0,
        height: 10.0,
    };
    let grid = CanvasSize {
        width: 10.0,
        height: 10.0,
    };

    let (_pos, size) = apply_resize_handle(
        NodeResizeHandle::Right,
        false,
        start_pos,
        NodeGraphNodeOrigin::default(),
        start_size_px,
        start_pointer,
        pointer,
        zoom,
        min,
        None,
        None,
        Some(grid),
    );
    assert_eq!(size.width, 110.0);
}

#[test]
fn resize_respects_max_size_constraints() {
    let start_pos = CanvasPoint { x: 0.0, y: 0.0 };
    let start_size_px = CanvasSize {
        width: 100.0,
        height: 50.0,
    };
    let start_pointer = Point::new(Px(0.0), Px(0.0));
    let pointer = Point::new(Px(200.0), Px(200.0));
    let zoom = 1.0;
    let min = CanvasSize {
        width: 10.0,
        height: 10.0,
    };
    let max = CanvasSize {
        width: 120.0,
        height: 80.0,
    };

    let (_pos, size) = apply_resize_handle(
        NodeResizeHandle::BottomRight,
        false,
        start_pos,
        NodeGraphNodeOrigin::default(),
        start_size_px,
        start_pointer,
        pointer,
        zoom,
        min,
        Some(max),
        None,
        None,
    );
    assert_eq!(size.width, 120.0);
    assert_eq!(size.height, 80.0);
}

#[test]
fn resize_keeps_aspect_ratio_for_corner_handles() {
    let start_pos = CanvasPoint { x: 10.0, y: 20.0 };
    let start_size_px = CanvasSize {
        width: 100.0,
        height: 50.0,
    };
    let start_pointer = Point::new(Px(0.0), Px(0.0));
    let pointer = Point::new(Px(20.0), Px(10.0));
    let zoom = 1.0;
    let min = CanvasSize {
        width: 1.0,
        height: 1.0,
    };

    let (pos, size) = apply_resize_handle(
        NodeResizeHandle::BottomRight,
        true,
        start_pos,
        NodeGraphNodeOrigin::default(),
        start_size_px,
        start_pointer,
        pointer,
        zoom,
        min,
        None,
        None,
        None,
    );
    assert_eq!(pos, start_pos);
    assert_eq!(size.width, 120.0);
    assert_eq!(size.height, 60.0);
}

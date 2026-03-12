use fret_core::{Point, Px};

use super::*;
use crate::core::{CanvasPoint, CanvasRect, CanvasSize, GroupId};

fn point(x: f32, y: f32) -> CanvasPoint {
    CanvasPoint { x, y }
}

fn size(width: f32, height: f32) -> CanvasSize {
    CanvasSize { width, height }
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> CanvasRect {
    CanvasRect {
        origin: point(x, y),
        size: size(width, height),
    }
}

#[test]
fn build_group_resize_ops_skips_unchanged_rect() {
    let resize = GroupResize {
        group: GroupId::new(),
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_rect: rect(0.0, 0.0, 180.0, 100.0),
        current_rect: rect(0.0, 0.0, 180.0, 100.0),
        preview_rev: 0,
    };

    let ops = build_group_resize_ops(&resize);

    assert!(ops.is_empty());
}

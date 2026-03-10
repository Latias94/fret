use crate::core::{CanvasPoint, CanvasRect, CanvasSize};

pub(in super::super) fn clamp_finite_positive(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        fallback.max(0.0)
    }
}

pub(in super::super) fn canvas_rect_intersection(a: CanvasRect, b: CanvasRect) -> CanvasRect {
    let ax0 = a.origin.x;
    let ay0 = a.origin.y;
    let ax1 = a.origin.x + a.size.width;
    let ay1 = a.origin.y + a.size.height;

    let bx0 = b.origin.x;
    let by0 = b.origin.y;
    let bx1 = b.origin.x + b.size.width;
    let by1 = b.origin.y + b.size.height;

    let x0 = ax0.max(bx0);
    let y0 = ay0.max(by0);
    let x1 = ax1.min(bx1);
    let y1 = ay1.min(by1);

    CanvasRect {
        origin: CanvasPoint { x: x0, y: y0 },
        size: CanvasSize {
            width: (x1 - x0).max(0.0),
            height: (y1 - y0).max(0.0),
        },
    }
}

pub(in super::super) fn canvas_rect_union(a: CanvasRect, b: CanvasRect) -> CanvasRect {
    let ax0 = a.origin.x;
    let ay0 = a.origin.y;
    let ax1 = a.origin.x + a.size.width;
    let ay1 = a.origin.y + a.size.height;

    let bx0 = b.origin.x;
    let by0 = b.origin.y;
    let bx1 = b.origin.x + b.size.width;
    let by1 = b.origin.y + b.size.height;

    let x0 = ax0.min(bx0);
    let y0 = ay0.min(by0);
    let x1 = ax1.max(bx1);
    let y1 = ay1.max(by1);

    CanvasRect {
        origin: CanvasPoint { x: x0, y: y0 },
        size: CanvasSize {
            width: (x1 - x0).max(0.0),
            height: (y1 - y0).max(0.0),
        },
    }
}

pub(in super::super) fn normalize_canvas_rect(mut rect: CanvasRect) -> CanvasRect {
    if rect.size.width.is_finite() {
        rect.size.width = rect.size.width.max(0.0);
    } else {
        rect.size.width = 0.0;
    }
    if rect.size.height.is_finite() {
        rect.size.height = rect.size.height.max(0.0);
    } else {
        rect.size.height = 0.0;
    }
    rect
}

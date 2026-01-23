//! Viewport overlay shapes and paint helpers.
//!
//! These are editor/app-level policies (ADR 0027 / ADR 0049). Docking is responsible only for
//! embedding viewports (render target + input forwarding). The app can paint these overlays via
//! `fret-docking`'s `DockViewportOverlayHooks`.
//!
//! Note: this module implements small 2D viewport tool overlays (translate/rotate) for the editor
//! demos. It is unrelated to the engine-pass 3D transform gizmos in `ecosystem/fret-gizmo`.

use fret_core::{
    Color, Corners, DrawOrder, Edges, Scene, SceneOp,
    geometry::{Point, Px, Rect, Size},
};

const VIEWPORT_GIZMO_AXIS_THICKNESS_PX: Px = Px(2.5);
const VIEWPORT_GIZMO_AXIS_HIGHLIGHT_THICKNESS_PX: Px = Px(4.0);
const VIEWPORT_DRAG_LINE_THICKNESS_PX: Px = Px(1.5);
const VIEWPORT_DRAG_LINE_ENDPOINT_SIZE_PX: Px = Px(6.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportOverlay {
    pub marquee: Option<ViewportMarquee>,
    pub drag_line: Option<ViewportDragLine>,
    pub selection_rect: Option<ViewportSelectionRect>,
    pub translate_gizmo: Option<ViewportTranslateGizmo>,
    pub rotate_gizmo: Option<ViewportRotateGizmo>,
    pub marker: Option<ViewportMarker>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMarquee {
    pub a_uv: (f32, f32),
    pub b_uv: (f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportSelectionRect {
    pub min_uv: (f32, f32),
    pub max_uv: (f32, f32),
    pub fill: Color,
    pub stroke: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportTranslateGizmoPart {
    X,
    Y,
    Handle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportTranslateGizmo {
    pub center_uv: (f32, f32),
    pub axis_len_px: Px,
    pub highlight: Option<ViewportTranslateGizmoPart>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportRotateGizmo {
    pub center_uv: (f32, f32),
    pub radius_px: Px,
    pub highlight: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMarker {
    pub uv: (f32, f32),
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportDragLine {
    pub a_uv: (f32, f32),
    pub b_uv: (f32, f32),
    pub color: Color,
}

pub fn paint_viewport_crosshair(
    theme: fret_ui::ThemeSnapshot,
    content: Rect,
    position: Point,
    scene: &mut Scene,
) {
    if !content.contains(position) {
        return;
    }

    let thickness = Px(1.5);
    let len = Px(12.0);
    let x = position.x;
    let y = position.y;

    let h = Rect {
        origin: Point::new(Px(x.0 - len.0), Px(y.0 - thickness.0 * 0.5)),
        size: Size::new(Px(len.0 * 2.0), thickness),
    };
    let v = Rect {
        origin: Point::new(Px(x.0 - thickness.0 * 0.5), Px(y.0 - len.0)),
        size: Size::new(thickness, Px(len.0 * 2.0)),
    };

    let color = Color {
        a: 0.65,
        ..theme.color_required("foreground")
    };

    for rect in [h, v] {
        scene.push(SceneOp::Quad {
            order: DrawOrder(5),
            rect,
            background: color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}

pub fn paint_viewport_overlay(
    theme: fret_ui::ThemeSnapshot,
    content: Rect,
    overlay: ViewportOverlay,
    scene: &mut Scene,
) {
    if let Some(sel) = overlay.selection_rect {
        paint_viewport_selection_rect(content, sel, scene);
    }
    if let Some(gizmo) = overlay.translate_gizmo {
        paint_viewport_translate_gizmo(theme, content, gizmo, scene);
    }
    if let Some(gizmo) = overlay.rotate_gizmo {
        paint_viewport_rotate_gizmo(theme, content, gizmo, scene);
    }
    if let Some(m) = overlay.marquee {
        paint_viewport_marquee(theme, content, m, scene);
    }
    if let Some(line) = overlay.drag_line {
        paint_viewport_drag_line(content, line, scene);
    }
    if let Some(marker) = overlay.marker {
        paint_viewport_marker(content, marker, scene);
    }
}

fn paint_viewport_translate_gizmo(
    theme: fret_ui::ThemeSnapshot,
    content: Rect,
    gizmo: ViewportTranslateGizmo,
    scene: &mut Scene,
) {
    let (u, v) = gizmo.center_uv;
    let x = content.origin.x.0 + content.size.width.0 * u;
    let y = content.origin.y.0 + content.size.height.0 * v;

    let len = gizmo.axis_len_px;
    let highlight = gizmo.highlight;
    let t = VIEWPORT_GIZMO_AXIS_THICKNESS_PX;
    let x_t = if highlight == Some(ViewportTranslateGizmoPart::X) {
        VIEWPORT_GIZMO_AXIS_HIGHLIGHT_THICKNESS_PX
    } else {
        t
    };
    let y_t = if highlight == Some(ViewportTranslateGizmoPart::Y) {
        VIEWPORT_GIZMO_AXIS_HIGHLIGHT_THICKNESS_PX
    } else {
        t
    };

    let x_axis = Rect::new(Point::new(Px(x), Px(y - x_t.0 * 0.5)), Size::new(len, x_t));
    let y_axis = Rect::new(
        Point::new(Px(x - y_t.0 * 0.5), Px(y - len.0)),
        Size::new(y_t, len),
    );

    let x_axis_alpha = if highlight == Some(ViewportTranslateGizmoPart::X) {
        1.0
    } else {
        0.85
    };
    let y_axis_alpha = if highlight == Some(ViewportTranslateGizmoPart::Y) {
        1.0
    } else {
        0.85
    };
    let x_color = Color {
        a: x_axis_alpha,
        ..theme.color_required("color.viewport.gizmo.x")
    };
    let y_color = Color {
        a: y_axis_alpha,
        ..theme.color_required("color.viewport.gizmo.y")
    };

    scene.push(SceneOp::Quad {
        order: DrawOrder(6),
        rect: x_axis,
        background: x_color,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(6),
        rect: y_axis,
        background: y_color,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let handle = Px(10.0);
    let handle_highlight = highlight == Some(ViewportTranslateGizmoPart::Handle);
    let handle_border = if handle_highlight { Px(2.5) } else { Px(1.5) };
    scene.push(SceneOp::Quad {
        order: DrawOrder(7),
        rect: Rect::new(
            Point::new(Px(x - handle.0 * 0.5), Px(y - handle.0 * 0.5)),
            Size::new(handle, handle),
        ),
        background: Color {
            a: 0.85,
            ..theme.color_required("color.viewport.gizmo.handle.background")
        },
        border: Edges::all(handle_border),
        border_color: Color {
            a: 0.90,
            ..theme.color_required("color.viewport.gizmo.handle.border")
        },
        corner_radii: Corners::all(Px(2.0)),
    });
}

fn paint_viewport_rotate_gizmo(
    theme: fret_ui::ThemeSnapshot,
    content: Rect,
    gizmo: ViewportRotateGizmo,
    scene: &mut Scene,
) {
    let (u, v) = gizmo.center_uv;
    let x = content.origin.x.0 + content.size.width.0 * u;
    let y = content.origin.y.0 + content.size.height.0 * v;

    let r = gizmo.radius_px;
    let t = if gizmo.highlight { Px(3.0) } else { Px(2.0) };
    let a = if gizmo.highlight { 0.95 } else { 0.75 };
    let color = Color {
        a,
        ..theme.color_required("color.viewport.rotate_gizmo")
    };

    scene.push(SceneOp::Quad {
        order: DrawOrder(6),
        rect: Rect::new(
            Point::new(Px(x - r.0), Px(y - r.0)),
            Size::new(Px(r.0 * 2.0), Px(r.0 * 2.0)),
        ),
        background: Color::TRANSPARENT,
        border: Edges::all(t),
        border_color: color,
        corner_radii: Corners::all(r),
    });
}

fn paint_viewport_selection_rect(content: Rect, rect: ViewportSelectionRect, scene: &mut Scene) {
    let (u0, v0) = rect.min_uv;
    let (u1, v1) = rect.max_uv;
    let left = content.origin.x.0 + content.size.width.0 * u0.min(u1);
    let right = content.origin.x.0 + content.size.width.0 * u0.max(u1);
    let top = content.origin.y.0 + content.size.height.0 * v0.min(v1);
    let bottom = content.origin.y.0 + content.size.height.0 * v0.max(v1);

    let inner = Rect::new(
        Point::new(Px(left), Px(top)),
        Size::new(Px((right - left).max(0.0)), Px((bottom - top).max(0.0))),
    );
    if inner.size.width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
        return;
    }

    let t = Px(2.0);
    scene.push(SceneOp::Quad {
        order: DrawOrder(4),
        rect: inner,
        background: rect.fill,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let top_rect = Rect::new(inner.origin, Size::new(inner.size.width, t));
    let bottom_rect = Rect::new(
        Point::new(
            inner.origin.x,
            Px(inner.origin.y.0 + inner.size.height.0 - t.0),
        ),
        Size::new(inner.size.width, t),
    );
    let left_rect = Rect::new(inner.origin, Size::new(t, inner.size.height));
    let right_rect = Rect::new(
        Point::new(
            Px(inner.origin.x.0 + inner.size.width.0 - t.0),
            inner.origin.y,
        ),
        Size::new(t, inner.size.height),
    );
    for r in [top_rect, bottom_rect, left_rect, right_rect] {
        scene.push(SceneOp::Quad {
            order: DrawOrder(5),
            rect: r,
            background: rect.stroke,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}

fn paint_viewport_marker(content: Rect, marker: ViewportMarker, scene: &mut Scene) {
    let (u, v) = marker.uv;
    let x = content.origin.x.0 + content.size.width.0 * u;
    let y = content.origin.y.0 + content.size.height.0 * v;

    let t = Px(2.0);
    let len = Px(10.0);
    let color = marker.color;

    let h = Rect::new(
        Point::new(Px(x - len.0), Px(y - t.0 * 0.5)),
        Size::new(Px(len.0 * 2.0), t),
    );
    let v = Rect::new(
        Point::new(Px(x - t.0 * 0.5), Px(y - len.0)),
        Size::new(t, Px(len.0 * 2.0)),
    );

    let shadow = fret_ui::element::ShadowStyle {
        primary: fret_ui::element::ShadowLayerStyle {
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            offset_x: Px(1.0),
            offset_y: Px(1.0),
            blur: Px(0.0),
            spread: Px(0.0),
        },
        secondary: None,
        corner_radii: Corners::all(Px(0.0)),
    };
    fret_ui::paint::paint_shadow(scene, DrawOrder(10), h, shadow);
    fret_ui::paint::paint_shadow(scene, DrawOrder(10), v, shadow);

    for rect in [h, v] {
        scene.push(SceneOp::Quad {
            order: DrawOrder(11),
            rect,
            background: color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });
    }

    let p = Px(7.0);
    scene.push(SceneOp::Quad {
        order: DrawOrder(12),
        rect: Rect::new(
            Point::new(Px(x - p.0 * 0.5), Px(y - p.0 * 0.5)),
            Size::new(p, p),
        ),
        background: Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: (color.a * 0.25).min(1.0),
        },
        border: Edges::all(Px(1.5)),
        border_color: color,
        corner_radii: Corners::all(Px(2.0)),
    });
}

fn paint_viewport_marquee(
    theme: fret_ui::ThemeSnapshot,
    content: Rect,
    marquee: ViewportMarquee,
    scene: &mut Scene,
) {
    let (au, av) = marquee.a_uv;
    let (bu, bv) = marquee.b_uv;
    let x0 = content.origin.x.0 + content.size.width.0 * au;
    let y0 = content.origin.y.0 + content.size.height.0 * av;
    let x1 = content.origin.x.0 + content.size.width.0 * bu;
    let y1 = content.origin.y.0 + content.size.height.0 * bv;

    let left = x0.min(x1);
    let right = x0.max(x1);
    let top = y0.min(y1);
    let bottom = y0.max(y1);

    let rect = Rect::new(
        Point::new(Px(left), Px(top)),
        Size::new(Px((right - left).max(0.0)), Px((bottom - top).max(0.0))),
    );
    // Render even for very thin drags (so users still see feedback); only skip true clicks.
    if rect.size.width.0 <= 1.0 && rect.size.height.0 <= 1.0 {
        return;
    }

    let fill = Color {
        a: 0.14,
        ..theme.color_required("primary")
    };
    let stroke = Color {
        a: 0.85,
        ..theme.color_required("primary")
    };
    let t = Px(1.5);

    scene.push(SceneOp::Quad {
        order: DrawOrder(6),
        rect,
        background: fill,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let top_rect = Rect::new(rect.origin, Size::new(rect.size.width, t));
    let bottom_rect = Rect::new(
        Point::new(
            rect.origin.x,
            Px(rect.origin.y.0 + rect.size.height.0 - t.0),
        ),
        Size::new(rect.size.width, t),
    );
    let left_rect = Rect::new(rect.origin, Size::new(t, rect.size.height));
    let right_rect = Rect::new(
        Point::new(Px(rect.origin.x.0 + rect.size.width.0 - t.0), rect.origin.y),
        Size::new(t, rect.size.height),
    );

    for r in [top_rect, bottom_rect, left_rect, right_rect] {
        scene.push(SceneOp::Quad {
            order: DrawOrder(7),
            rect: r,
            background: stroke,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}

fn paint_viewport_drag_line(content: Rect, line: ViewportDragLine, scene: &mut Scene) {
    let (au, av) = line.a_uv;
    let (bu, bv) = line.b_uv;
    let x0 = content.origin.x.0 + content.size.width.0 * au;
    let y0 = content.origin.y.0 + content.size.height.0 * av;
    let x1 = content.origin.x.0 + content.size.width.0 * bu;
    let y1 = content.origin.y.0 + content.size.height.0 * bv;

    let color = line.color;
    let t = VIEWPORT_DRAG_LINE_THICKNESS_PX;

    let h = Rect::new(
        Point::new(Px(x0.min(x1)), Px(y0 - t.0 * 0.5)),
        Size::new(Px((x1 - x0).abs().max(0.0)), t),
    );
    let v = Rect::new(
        Point::new(Px(x1 - t.0 * 0.5), Px(y0.min(y1))),
        Size::new(t, Px((y1 - y0).abs().max(0.0))),
    );

    for rect in [h, v] {
        if rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0 {
            continue;
        }
        scene.push(SceneOp::Quad {
            order: DrawOrder(8),
            rect,
            background: color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });
    }

    let p = VIEWPORT_DRAG_LINE_ENDPOINT_SIZE_PX;
    for (x, y) in [(x0, y0), (x1, y1)] {
        scene.push(SceneOp::Quad {
            order: DrawOrder(9),
            rect: Rect::new(
                Point::new(Px(x - p.0 * 0.5), Px(y - p.0 * 0.5)),
                Size::new(p, p),
            ),
            background: color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(2.0)),
        });
    }
}

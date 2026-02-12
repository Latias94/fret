use std::sync::Arc;

use fret_core::scene::{Color, DrawOrder, Paint, SceneOp};
use fret_core::vector_path::{PathCommand, PathStyle, StrokeStyle};
use fret_core::{Corners, Edges, Point, Px, Rect, Size};
use fret_ui::element::{AnyElement, CanvasProps, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::style as decl_style;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WorkflowEdgeKey {
    from_x_bits: u32,
    from_y_bits: u32,
    to_x_bits: u32,
    to_y_bits: u32,
    stroke_width_bits: u32,
}

fn px_bits(px: Px) -> u32 {
    px.0.to_bits()
}

fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;

    let uu = u * u;
    let uuu = uu * u;
    let tt = t * t;
    let ttt = tt * t;

    let x = uuu * p0.x.0 + 3.0 * uu * t * p1.x.0 + 3.0 * u * tt * p2.x.0 + ttt * p3.x.0;
    let y = uuu * p0.y.0 + 3.0 * uu * t * p1.y.0 + 3.0 * u * tt * p2.y.0 + ttt * p3.y.0;

    Point::new(Px(x), Px(y))
}

/// AI Elements-aligned workflow edge renderer (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/edge.tsx`.
///
/// Notes:
/// - Upstream uses `@xyflow/react` to compute handle coordinates and SVG paths.
/// - In Fret, apps own graph layout and supply coordinates. This module provides style-aligned
///   drawing helpers in a declarative `Canvas`.
#[derive(Clone)]
pub struct WorkflowEdgeTemporary {
    from: Point,
    to: Point,
    stroke_width: Px,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for WorkflowEdgeTemporary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowEdgeTemporary")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("stroke_width", &self.stroke_width)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl WorkflowEdgeTemporary {
    pub fn new(from: Point, to: Point) -> Self {
        Self {
            from,
            to,
            stroke_width: Px(1.0),
            test_id: None,
            layout: LayoutRefinement::default()
                .w_full()
                .h_full()
                .min_w_0()
                .min_h_0(),
        }
    }

    pub fn stroke_width(mut self, width: Px) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let ring = theme.color_required("ring");

        let mut props = CanvasProps::default();
        props.layout = decl_style::layout_style(&theme, self.layout);

        let from = self.from;
        let to = self.to;
        let stroke_width = self.stroke_width;

        let el = cx.canvas(props, move |p| {
            let key = WorkflowEdgeKey {
                from_x_bits: px_bits(from.x),
                from_y_bits: px_bits(from.y),
                to_x_bits: px_bits(to.x),
                to_y_bits: px_bits(to.y),
                stroke_width_bits: px_bits(stroke_width),
            };
            let key = p.key(&key);

            let mid_x = Px(from.x.0 + (to.x.0 - from.x.0) * 0.5);
            let ctrl1 = Point::new(mid_x, from.y);
            let ctrl2 = Point::new(mid_x, to.y);

            let commands = [
                PathCommand::MoveTo(from),
                PathCommand::CubicTo { ctrl1, ctrl2, to },
            ];

            // Note: upstream uses `strokeDasharray: "5, 5"`. Fret's current path primitive does
            // not support dash patterns, so this renders as a solid stroke for now.
            p.path(
                key,
                DrawOrder(0),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Stroke(StrokeStyle {
                    width: stroke_width,
                }),
                ring,
                p.scale_factor(),
            );
        });

        let Some(test_id) = self.test_id else {
            return el;
        };
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [el],
        )
    }
}

#[derive(Clone)]
pub struct WorkflowEdgeAnimated {
    from: Point,
    to: Point,
    stroke_width: Px,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for WorkflowEdgeAnimated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowEdgeAnimated")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("stroke_width", &self.stroke_width)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl WorkflowEdgeAnimated {
    pub fn new(from: Point, to: Point) -> Self {
        Self {
            from,
            to,
            stroke_width: Px(1.0),
            test_id: None,
            layout: LayoutRefinement::default()
                .w_full()
                .h_full()
                .min_w_0()
                .min_h_0(),
        }
    }

    pub fn stroke_width(mut self, width: Px) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let ring = theme.color_required("ring");
        let primary = theme.color_required("primary");

        let mut props = CanvasProps::default();
        props.layout = decl_style::layout_style(&theme, self.layout);

        let from = self.from;
        let to = self.to;
        let stroke_width = self.stroke_width;

        let el = cx.canvas(props, move |p| {
            p.request_animation_frame();

            let key = WorkflowEdgeKey {
                from_x_bits: px_bits(from.x),
                from_y_bits: px_bits(from.y),
                to_x_bits: px_bits(to.x),
                to_y_bits: px_bits(to.y),
                stroke_width_bits: px_bits(stroke_width),
            };
            let key = p.key(&key);

            let mid_x = Px(from.x.0 + (to.x.0 - from.x.0) * 0.5);
            let ctrl1 = Point::new(mid_x, from.y);
            let ctrl2 = Point::new(mid_x, to.y);

            let commands = [
                PathCommand::MoveTo(from),
                PathCommand::CubicTo { ctrl1, ctrl2, to },
            ];

            p.path(
                key,
                DrawOrder(0),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Stroke(StrokeStyle {
                    width: stroke_width,
                }),
                ring,
                p.scale_factor(),
            );

            let phase = (p.frame_id() % 120) as f32 / 120.0;
            let pos = cubic_bezier(from, ctrl1, ctrl2, to, phase);
            let radius = Px(4.0);
            let rect = Rect::new(
                Point::new(Px(pos.x.0 - radius.0), Px(pos.y.0 - radius.0)),
                Size::new(Px(radius.0 * 2.0), Px(radius.0 * 2.0)),
            );

            p.scene().push(SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background: Paint::Solid(primary),
                border: Edges::all(Px(0.0)),
                border_paint: Paint::Solid(Color::TRANSPARENT),
                corner_radii: Corners::all(radius),
            });
        });

        let Some(test_id) = self.test_id else {
            return el;
        };
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [el],
        )
    }
}

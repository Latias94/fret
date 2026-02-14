use std::sync::Arc;

use fret_canvas::wires::{
    arrowhead_triangle, cubic_bezier, cubic_bezier_derivative, cubic_bezier_polyline_points,
    dash_polyline_segments,
};
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

            // Upstream uses `strokeDasharray: "5, 5"`. Fret's path primitive does not currently
            // expose dash patterns, so we approximate dashes by emitting a set of independent
            // stroked line segments along a flattened Bezier polyline.
            let chord = {
                let dx = to.x.0 - from.x.0;
                let dy = to.y.0 - from.y.0;
                (dx * dx + dy * dy).sqrt().max(1.0)
            };
            let steps = ((chord / 12.0).ceil() as usize).clamp(16, 64);

            let mut points = Vec::new();
            cubic_bezier_polyline_points(from, ctrl1, ctrl2, to, steps, &mut points);

            let mut segments = Vec::new();
            dash_polyline_segments(&points, &[5.0, 5.0], 0.0, &mut segments);

            if segments.is_empty() {
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
                return;
            }

            let scope = p.key_scope(&key);
            for (ix, (a, b)) in segments.iter().enumerate() {
                let seg_key = u64::from(p.child_key(scope, &ix));
                let commands = [PathCommand::MoveTo(*a), PathCommand::LineTo(*b)];
                p.path(
                    seg_key,
                    DrawOrder(0),
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Stroke(StrokeStyle {
                        width: stroke_width,
                    }),
                    ring,
                    p.scale_factor(),
                );
            }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WorkflowEdgeMarkerEnd {
    #[default]
    None,
    Arrow,
}

#[derive(Clone)]
pub struct WorkflowEdgeAnimated {
    from: Point,
    to: Point,
    stroke_width: Px,
    marker_end: WorkflowEdgeMarkerEnd,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for WorkflowEdgeAnimated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowEdgeAnimated")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("stroke_width", &self.stroke_width)
            .field("marker_end", &self.marker_end)
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
            marker_end: WorkflowEdgeMarkerEnd::None,
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

    pub fn marker_end(mut self, marker_end: WorkflowEdgeMarkerEnd) -> Self {
        self.marker_end = marker_end;
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
        let marker_end = self.marker_end;

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

            if marker_end == WorkflowEdgeMarkerEnd::Arrow {
                let tangent = cubic_bezier_derivative(from, ctrl1, ctrl2, to, 1.0);
                let arrow_len = (stroke_width.0 * 6.0).clamp(8.0, 14.0);
                let arrow_w = arrow_len * 0.75;
                let tri = arrowhead_triangle(to, tangent, arrow_len, arrow_w);

                let scope = p.key_scope(&key);
                let marker_key = u64::from(p.child_key(scope, &0u8));
                let commands = [
                    PathCommand::MoveTo(tri[0]),
                    PathCommand::LineTo(tri[1]),
                    PathCommand::LineTo(tri[2]),
                    PathCommand::Close,
                ];

                p.path(
                    marker_key,
                    DrawOrder(1),
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(Default::default()),
                    ring,
                    p.scale_factor(),
                );
            }

            let phase = (p.frame_id() % 120) as f32 / 120.0;
            let pos = cubic_bezier(from, ctrl1, ctrl2, to, phase);
            let radius = Px(4.0);
            let rect = Rect::new(
                Point::new(Px(pos.x.0 - radius.0), Px(pos.y.0 - radius.0)),
                Size::new(Px(radius.0 * 2.0), Px(radius.0 * 2.0)),
            );

            p.scene().push(SceneOp::Quad {
                order: DrawOrder(2),
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

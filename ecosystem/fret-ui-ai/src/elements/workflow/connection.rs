use std::sync::Arc;

use fret_core::scene::{DrawOrder, Paint, SceneOp};
use fret_core::vector_path::{PathCommand, PathStyle, StrokeStyle};
use fret_core::{Corners, Edges, Point, Px, Rect, Size};
use fret_ui::element::{AnyElement, CanvasProps, Length, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::style as decl_style;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WorkflowConnectionKey {
    from_x_bits: u32,
    from_y_bits: u32,
    to_x_bits: u32,
    to_y_bits: u32,
    stroke_width_bits: u32,
}

fn px_bits(px: Px) -> u32 {
    px.0.to_bits()
}

/// AI Elements-aligned workflow connection line (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/connection.tsx`.
///
/// Notes:
/// - Upstream renders in SVG (`<path>` + `<circle>`). In Fret this is a declarative `Canvas` leaf.
/// - This surface is intended as a styling seam. Apps own interactive graph engines and coordinate
///   spaces.
#[derive(Clone)]
pub struct WorkflowConnection {
    from: Point,
    to: Point,
    stroke_width: Px,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for WorkflowConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowConnection")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("stroke_width", &self.stroke_width)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl WorkflowConnection {
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
        let background = theme.color_required("background");

        let mut layout = decl_style::layout_style(&theme, self.layout);
        layout.size.width = layout.size.width.or_fill();
        layout.size.height = layout.size.height.or_fill();

        let mut props = CanvasProps::default();
        props.layout = layout;

        let from = self.from;
        let to = self.to;
        let stroke_width = self.stroke_width;

        let el = cx.canvas(props, move |p| {
            let key = WorkflowConnectionKey {
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

            let radius = Px(3.0);
            let rect = Rect::new(
                Point::new(Px(to.x.0 - radius.0), Px(to.y.0 - radius.0)),
                Size::new(Px(radius.0 * 2.0), Px(radius.0 * 2.0)),
            );

            p.scene().push(SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background: Paint::Solid(background),
                border: Edges::all(stroke_width),
                border_paint: Paint::Solid(ring),
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

trait LengthExt {
    fn or_fill(self) -> Self;
}

impl LengthExt for Length {
    fn or_fill(self) -> Self {
        match self {
            Self::Auto => Self::Fill,
            other => other,
        }
    }
}

//! Immediate-mode debug draw helper backed by declarative `Canvas`.

use std::hash::Hash;
use std::sync::Arc;

use fret_core::scene::ImageSamplingHint;
use fret_core::scene::{DashPatternV1, Paint};
use fret_core::{
    Color, Corners, DrawOrder, Edges, FillStyle, ImageId, PathCommand, PathStyle, Point, Px, Rect,
    Size, StrokeCapV1, StrokeJoinV1, StrokeStyle, StrokeStyleV2, SvgFit, TextOverflow, TextStyle,
    TextWrap, UvRect, ViewportFit,
};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};
use fret_ui::element::{
    AnyElement, CanvasCachePolicy, CanvasProps, LayoutStyle, Length, SizeStyle,
};
use fret_ui::{ElementContext, SvgSource, UiHost};

use super::UiWriterImUiFacadeExt;

const DEFAULT_ELLIPSE_SEGMENTS: usize = 32;
const DEFAULT_PATH_ARC_SEGMENTS: usize = 12;
const DEFAULT_PATH_BEZIER_SEGMENTS: usize = 12;
const DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS: usize = 32;

#[derive(Debug, Clone)]
pub struct DebugDrawOptions {
    pub layout: LayoutStyle,
    pub test_id: Option<Arc<str>>,
    pub clip_to_bounds: bool,
}

impl Default for DebugDrawOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(120.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            test_id: None,
            clip_to_bounds: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImUiDebugDrawList {
    commands: Vec<DebugDrawCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawStrokeStyle {
    pub width: Px,
    pub join: StrokeJoinV1,
    pub cap: StrokeCapV1,
    pub miter_limit: f32,
    pub dash: Option<DashPatternV1>,
}

impl DebugDrawStrokeStyle {
    pub fn new(width: Px) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }

    pub fn with_join(mut self, join: StrokeJoinV1) -> Self {
        self.join = join;
        self
    }

    pub fn with_cap(mut self, cap: StrokeCapV1) -> Self {
        self.cap = cap;
        self
    }

    pub fn with_miter_limit(mut self, miter_limit: f32) -> Self {
        if miter_limit.is_finite() && miter_limit > 0.0 {
            self.miter_limit = miter_limit;
        }
        self
    }

    pub fn with_dash(mut self, dash: Px, gap: Px, phase: Px) -> Self {
        if dash.0 > 0.0 && gap.0 > 0.0 && phase.0.is_finite() {
            self.dash = Some(DashPatternV1::new(dash, gap, phase));
        }
        self
    }

    pub fn with_dash_pattern(mut self, dash: DashPatternV1) -> Self {
        if dash.dash.0 > 0.0 && dash.gap.0 > 0.0 && dash.phase.0.is_finite() {
            self.dash = Some(dash);
        }
        self
    }

    fn is_visible(self) -> bool {
        self.width.0 > 0.0
    }

    fn path_style(self) -> PathStyle {
        if self.join == StrokeJoinV1::Miter
            && self.cap == StrokeCapV1::Butt
            && self.miter_limit == 4.0
            && self.dash.is_none()
        {
            PathStyle::Stroke(StrokeStyle { width: self.width })
        } else {
            PathStyle::StrokeV2(StrokeStyleV2 {
                width: self.width,
                join: self.join,
                cap: self.cap,
                miter_limit: self.miter_limit,
                dash: self.dash,
            })
        }
    }
}

impl Default for DebugDrawStrokeStyle {
    fn default() -> Self {
        Self {
            width: Px(1.0),
            join: StrokeJoinV1::Miter,
            cap: StrokeCapV1::Butt,
            miter_limit: 4.0,
            dash: None,
        }
    }
}

impl From<Px> for DebugDrawStrokeStyle {
    fn from(width: Px) -> Self {
        Self::new(width)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawImageOptions {
    pub fit: ViewportFit,
    pub sampling: ImageSamplingHint,
    pub opacity: f32,
}

impl Default for DebugDrawImageOptions {
    fn default() -> Self {
        Self {
            fit: ViewportFit::Stretch,
            sampling: ImageSamplingHint::Default,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawSvgOptions {
    pub fit: SvgFit,
    pub opacity: f32,
}

impl Default for DebugDrawSvgOptions {
    fn default() -> Self {
        Self {
            fit: SvgFit::Stretch,
            opacity: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct ImUiDebugDrawPath<'a> {
    draw_list: &'a mut ImUiDebugDrawList,
    points: Vec<Point>,
}

impl ImUiDebugDrawPath<'_> {
    pub fn clear(&mut self) -> &mut Self {
        self.points.clear();
        self
    }

    pub fn line_to(&mut self, point: Point) -> &mut Self {
        self.points.push(point);
        self
    }

    pub fn line_to_merge_duplicate(&mut self, point: Point) -> &mut Self {
        if self.points.last().copied() != Some(point) {
            self.points.push(point);
        }
        self
    }

    pub fn bezier_quadratic_curve_to(
        &mut self,
        ctrl: Point,
        to: Point,
        segments: usize,
    ) -> &mut Self {
        let Some(from) = self.points.last().copied() else {
            return self;
        };
        let segments = path_bezier_segments(segments);
        for step in 1..=segments {
            let t = step as f32 / segments as f32;
            self.points.push(quadratic_bezier_point(from, ctrl, to, t));
        }
        self
    }

    pub fn bezier_cubic_curve_to(
        &mut self,
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
        segments: usize,
    ) -> &mut Self {
        let Some(from) = self.points.last().copied() else {
            return self;
        };
        let segments = path_bezier_segments(segments);
        for step in 1..=segments {
            let t = step as f32 / segments as f32;
            self.points
                .push(cubic_bezier_point(from, ctrl1, ctrl2, to, t));
        }
        self
    }

    pub fn arc_to(
        &mut self,
        center: Point,
        radius: Px,
        a_min: f32,
        a_max: f32,
        segments: usize,
    ) -> &mut Self {
        if !radius.0.is_finite() || !a_min.is_finite() || !a_max.is_finite() || radius.0 <= 0.0 {
            return self;
        }
        if radius.0 < 0.5 {
            self.points.push(center);
            return self;
        }
        append_arc_points(
            &mut self.points,
            center,
            radius,
            a_min,
            a_max,
            path_arc_segments(segments),
        );
        self
    }

    pub fn arc_to_fast(
        &mut self,
        center: Point,
        radius: Px,
        a_min_of_12: i32,
        a_max_of_12: i32,
    ) -> &mut Self {
        if !radius.0.is_finite() || radius.0 <= 0.0 {
            return self;
        }
        if radius.0 < 0.5 {
            self.points.push(center);
            return self;
        }
        let a_min = a_min_of_12 as f32 * std::f32::consts::TAU / 12.0;
        let a_max = a_max_of_12 as f32 * std::f32::consts::TAU / 12.0;
        append_arc_points(
            &mut self.points,
            center,
            radius,
            a_min,
            a_max,
            a_min_of_12.abs_diff(a_max_of_12) as usize,
        );
        self
    }

    pub fn elliptical_arc_to(
        &mut self,
        center: Point,
        radius: Size,
        rotation_radians: f32,
        a_min: f32,
        a_max: f32,
        segments: usize,
    ) -> &mut Self {
        if radius.width.0 <= 0.0
            || radius.height.0 <= 0.0
            || !radius.width.0.is_finite()
            || !radius.height.0.is_finite()
            || !rotation_radians.is_finite()
            || !a_min.is_finite()
            || !a_max.is_finite()
        {
            return self;
        }
        append_elliptical_arc_points(
            &mut self.points,
            center,
            radius,
            rotation_radians,
            a_min,
            a_max,
            path_elliptical_arc_segments(segments),
        );
        self
    }

    pub fn stroke(&mut self, color: Color, thickness: Px, closed: bool) {
        self.stroke_with_style(color, thickness, closed);
    }

    pub fn stroke_with_style(
        &mut self,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
        closed: bool,
    ) {
        let points = std::mem::take(&mut self.points);
        if points.len() < path_stroke_required_points(closed) {
            return;
        }
        self.draw_list
            .add_polyline_with_style(points, color, style, closed);
    }

    pub fn fill_convex(&mut self, color: Color) {
        let points = std::mem::take(&mut self.points);
        if points.len() < 3 {
            return;
        }
        self.draw_list.add_convex_poly_filled(points, color);
    }

    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl ImUiDebugDrawList {
    pub fn path<F>(&mut self, build: F)
    where
        F: FnOnce(&mut ImUiDebugDrawPath<'_>),
    {
        let mut path = ImUiDebugDrawPath {
            draw_list: self,
            points: Vec::new(),
        };
        build(&mut path);
    }

    pub fn add_line(&mut self, from: Point, to: Point, color: Color, thickness: Px) {
        self.add_line_with_style(from, to, color, thickness);
    }

    pub fn add_line_with_style(
        &mut self,
        from: Point,
        to: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Line {
            from,
            to,
            color,
            style: style.into(),
        });
    }

    pub fn add_polyline<I>(&mut self, points: I, color: Color, thickness: Px, closed: bool)
    where
        I: IntoIterator<Item = Point>,
    {
        self.add_polyline_with_style(points, color, thickness, closed);
    }

    pub fn add_polyline_with_style<I>(
        &mut self,
        points: I,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
        closed: bool,
    ) where
        I: IntoIterator<Item = Point>,
    {
        let points: Arc<[Point]> = Arc::from(points.into_iter().collect::<Vec<_>>());
        self.commands.push(DebugDrawCommand::Polyline {
            points,
            color,
            style: style.into(),
            closed,
        });
    }

    pub fn add_convex_poly_filled<I>(&mut self, points: I, color: Color)
    where
        I: IntoIterator<Item = Point>,
    {
        let points: Arc<[Point]> = Arc::from(points.into_iter().collect::<Vec<_>>());
        self.commands
            .push(DebugDrawCommand::ConvexPolyFilled { points, color });
    }

    pub fn add_rect(&mut self, rect: Rect, color: Color, thickness: Px) {
        self.add_rect_with_style(rect, color, thickness);
    }

    pub fn add_rect_with_style(
        &mut self,
        rect: Rect,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Rect {
            rect,
            color,
            style: style.into(),
        });
    }

    pub fn add_rect_filled(&mut self, rect: Rect, color: Color) {
        self.commands
            .push(DebugDrawCommand::RectFilled { rect, color });
    }

    pub fn add_quad(
        &mut self,
        p1: Point,
        p2: Point,
        p3: Point,
        p4: Point,
        color: Color,
        thickness: Px,
    ) {
        self.add_quad_with_style(p1, p2, p3, p4, color, thickness);
    }

    pub fn add_quad_with_style(
        &mut self,
        p1: Point,
        p2: Point,
        p3: Point,
        p4: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Quad {
            p1,
            p2,
            p3,
            p4,
            color,
            style: style.into(),
        });
    }

    pub fn add_quad_filled(&mut self, p1: Point, p2: Point, p3: Point, p4: Point, color: Color) {
        self.commands.push(DebugDrawCommand::QuadFilled {
            p1,
            p2,
            p3,
            p4,
            color,
        });
    }

    pub fn add_triangle(&mut self, p1: Point, p2: Point, p3: Point, color: Color, thickness: Px) {
        self.add_triangle_with_style(p1, p2, p3, color, thickness);
    }

    pub fn add_triangle_with_style(
        &mut self,
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Triangle {
            p1,
            p2,
            p3,
            color,
            style: style.into(),
        });
    }

    pub fn add_triangle_filled(&mut self, p1: Point, p2: Point, p3: Point, color: Color) {
        self.commands
            .push(DebugDrawCommand::TriangleFilled { p1, p2, p3, color });
    }

    pub fn add_circle(&mut self, center: Point, radius: Px, color: Color, thickness: Px) {
        self.add_circle_with_style(center, radius, color, thickness);
    }

    pub fn add_circle_with_style(
        &mut self,
        center: Point,
        radius: Px,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Circle {
            center,
            radius,
            color,
            style: style.into(),
        });
    }

    pub fn add_circle_filled(&mut self, center: Point, radius: Px, color: Color) {
        self.commands.push(DebugDrawCommand::CircleFilled {
            center,
            radius,
            color,
        });
    }

    pub fn add_ngon(
        &mut self,
        center: Point,
        radius: Px,
        segments: usize,
        color: Color,
        thickness: Px,
    ) {
        self.add_ngon_with_style(center, radius, segments, color, thickness);
    }

    pub fn add_ngon_with_style(
        &mut self,
        center: Point,
        radius: Px,
        segments: usize,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Ngon {
            center,
            radius,
            segments,
            color,
            style: style.into(),
        });
    }

    pub fn add_ngon_filled(&mut self, center: Point, radius: Px, segments: usize, color: Color) {
        self.commands.push(DebugDrawCommand::NgonFilled {
            center,
            radius,
            segments,
            color,
        });
    }

    pub fn add_ellipse(
        &mut self,
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
        thickness: Px,
    ) {
        self.add_ellipse_with_style(center, radius, rotation_radians, segments, color, thickness);
    }

    pub fn add_ellipse_with_style(
        &mut self,
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Ellipse {
            center,
            radius,
            rotation_radians,
            segments,
            color,
            style: style.into(),
        });
    }

    pub fn add_ellipse_filled(
        &mut self,
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
    ) {
        self.commands.push(DebugDrawCommand::EllipseFilled {
            center,
            radius,
            rotation_radians,
            segments,
            color,
        });
    }

    pub fn add_bezier_quadratic(
        &mut self,
        from: Point,
        ctrl: Point,
        to: Point,
        color: Color,
        thickness: Px,
    ) {
        self.add_bezier_quadratic_with_style(from, ctrl, to, color, thickness);
    }

    pub fn add_bezier_quadratic_with_style(
        &mut self,
        from: Point,
        ctrl: Point,
        to: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::BezierQuadratic {
            from,
            ctrl,
            to,
            color,
            style: style.into(),
        });
    }

    pub fn add_bezier_cubic(
        &mut self,
        from: Point,
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
        color: Color,
        thickness: Px,
    ) {
        self.add_bezier_cubic_with_style(from, ctrl1, ctrl2, to, color, thickness);
    }

    pub fn add_bezier_cubic_with_style(
        &mut self,
        from: Point,
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::BezierCubic {
            from,
            ctrl1,
            ctrl2,
            to,
            color,
            style: style.into(),
        });
    }

    pub fn push_clip_rect(&mut self, rect: Rect) {
        self.commands.push(DebugDrawCommand::PushClipRect { rect });
    }

    pub fn pop_clip_rect(&mut self) {
        self.commands.push(DebugDrawCommand::PopClipRect);
    }

    pub fn add_image(&mut self, rect: Rect, image: ImageId) {
        self.add_image_with_options(rect, image, DebugDrawImageOptions::default());
    }

    pub fn add_image_with_options(
        &mut self,
        rect: Rect,
        image: ImageId,
        options: DebugDrawImageOptions,
    ) {
        self.commands.push(DebugDrawCommand::Image {
            rect,
            image,
            options,
        });
    }

    pub fn add_image_region(
        &mut self,
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        options: DebugDrawImageOptions,
    ) {
        self.commands.push(DebugDrawCommand::ImageRegion {
            rect,
            image,
            uv,
            options,
        });
    }

    pub fn add_svg_image(&mut self, rect: Rect, svg: SvgSource) {
        self.add_svg_image_with_options(rect, svg, DebugDrawSvgOptions::default());
    }

    pub fn add_svg_image_with_options(
        &mut self,
        rect: Rect,
        svg: SvgSource,
        options: DebugDrawSvgOptions,
    ) {
        self.commands
            .push(DebugDrawCommand::SvgImage { rect, svg, options });
    }

    pub fn add_svg_mask_icon(&mut self, rect: Rect, svg: SvgSource, color: Color) {
        self.add_svg_mask_icon_with_options(rect, svg, color, DebugDrawSvgOptions::default());
    }

    pub fn add_svg_mask_icon_with_options(
        &mut self,
        rect: Rect,
        svg: SvgSource,
        color: Color,
        options: DebugDrawSvgOptions,
    ) {
        self.commands.push(DebugDrawCommand::SvgMaskIcon {
            rect,
            svg,
            color,
            options,
        });
    }

    pub fn add_text(&mut self, origin: Point, text: impl Into<Arc<str>>, color: Color, size: Px) {
        self.commands.push(DebugDrawCommand::Text {
            origin,
            text: text.into(),
            color,
            size,
        });
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for ImUiDebugDrawList {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum DebugDrawCommand {
    Line {
        from: Point,
        to: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    Polyline {
        points: Arc<[Point]>,
        color: Color,
        style: DebugDrawStrokeStyle,
        closed: bool,
    },
    ConvexPolyFilled {
        points: Arc<[Point]>,
        color: Color,
    },
    Rect {
        rect: Rect,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    RectFilled {
        rect: Rect,
        color: Color,
    },
    Quad {
        p1: Point,
        p2: Point,
        p3: Point,
        p4: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    QuadFilled {
        p1: Point,
        p2: Point,
        p3: Point,
        p4: Point,
        color: Color,
    },
    Triangle {
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    TriangleFilled {
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
    },
    Circle {
        center: Point,
        radius: Px,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    CircleFilled {
        center: Point,
        radius: Px,
        color: Color,
    },
    Ngon {
        center: Point,
        radius: Px,
        segments: usize,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    NgonFilled {
        center: Point,
        radius: Px,
        segments: usize,
        color: Color,
    },
    Ellipse {
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    EllipseFilled {
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
    },
    BezierQuadratic {
        from: Point,
        ctrl: Point,
        to: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    BezierCubic {
        from: Point,
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    PushClipRect {
        rect: Rect,
    },
    PopClipRect,
    Image {
        rect: Rect,
        image: ImageId,
        options: DebugDrawImageOptions,
    },
    ImageRegion {
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        options: DebugDrawImageOptions,
    },
    SvgImage {
        rect: Rect,
        svg: SvgSource,
        options: DebugDrawSvgOptions,
    },
    SvgMaskIcon {
        rect: Rect,
        svg: SvgSource,
        color: Color,
        options: DebugDrawSvgOptions,
    },
    Text {
        origin: Point,
        text: Arc<str>,
        color: Color,
        size: Px,
    },
}

pub(super) fn debug_draw_with_options<H, W, K, F>(
    ui: &mut W,
    id: K,
    options: DebugDrawOptions,
    draw: F,
) where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    K: Hash,
    F: FnOnce(&mut ImUiDebugDrawList),
{
    let mut list = ImUiDebugDrawList::default();
    draw(&mut list);
    let commands: Arc<[DebugDrawCommand]> = Arc::from(list.commands.into_boxed_slice());
    let element = ui.with_cx_mut(|cx| {
        cx.keyed(("fret-ui-kit.imui.debug_draw", id), |cx| {
            debug_draw_element(cx, commands, options)
        })
    });
    ui.add(element);
}

fn debug_draw_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    commands: Arc<[DebugDrawCommand]>,
    options: DebugDrawOptions,
) -> AnyElement {
    let mut props = CanvasProps {
        layout: options.layout,
        cache_policy: CanvasCachePolicy::smooth_default(),
    };
    props.cache_policy.shared_text.keep_frames = 30;
    props.cache_policy.path.keep_frames = 30;

    let clip_to_bounds = options.clip_to_bounds;
    let mut element = cx.canvas(props, move |painter| {
        if clip_to_bounds {
            let bounds = painter.bounds();
            painter.with_clip_rect(bounds, |painter| {
                paint_debug_draw_commands(painter, &commands)
            });
        } else {
            paint_debug_draw_commands(painter, &commands);
        }
    });
    if let Some(test_id) = options.test_id {
        element = element.test_id(test_id);
    }
    element
}

fn paint_debug_draw_commands(painter: &mut CanvasPainter<'_>, commands: &[DebugDrawCommand]) {
    let scale = painter.scale_factor().max(1.0);
    let mut open_clip_depth = 0usize;
    for (index, command) in commands.iter().enumerate() {
        let order = DrawOrder(index as u32);
        let key = painter.key(&("fret-ui-kit.imui.debug_draw.command", index));
        match command {
            DebugDrawCommand::PushClipRect { rect } => {
                if rect_is_empty(*rect) {
                    continue;
                }
                painter
                    .scene()
                    .push(fret_core::SceneOp::PushClipRect { rect: *rect });
                open_clip_depth += 1;
            }
            DebugDrawCommand::PopClipRect => {
                if open_clip_depth == 0 {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::PopClip);
                open_clip_depth -= 1;
            }
            DebugDrawCommand::Image {
                rect,
                image,
                options,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || rect_is_empty(*rect) {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::Image {
                    order,
                    rect: *rect,
                    image: *image,
                    fit: options.fit,
                    sampling: options.sampling,
                    opacity,
                });
            }
            DebugDrawCommand::ImageRegion {
                rect,
                image,
                uv,
                options,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || rect_is_empty(*rect) || !uv_rect_is_valid(*uv) {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::ImageRegion {
                    order,
                    rect: *rect,
                    image: *image,
                    uv: *uv,
                    sampling: options.sampling,
                    opacity,
                });
            }
            DebugDrawCommand::SvgImage { rect, svg, options } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || rect_is_empty(*rect) {
                    continue;
                }
                painter.svg_image(key, order, *rect, svg, options.fit, opacity);
            }
            DebugDrawCommand::SvgMaskIcon {
                rect,
                svg,
                color,
                options,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || color.a <= 0.0 || rect_is_empty(*rect) {
                    continue;
                }
                painter.svg_mask_icon(key, order, *rect, svg, options.fit, *color, opacity);
            }
            DebugDrawCommand::Line {
                from,
                to,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let commands = [PathCommand::MoveTo(*from), PathCommand::LineTo(*to)];
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Polyline {
                points,
                color,
                style,
                closed,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let Some(commands) = polyline_path(points, *closed) else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::ConvexPolyFilled { points, color } => {
                if color.a <= 0.0 {
                    continue;
                }
                let Some(commands) = convex_poly_fill_path(points) else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Rect { rect, color, style } => {
                if color.a <= 0.0 || !style.is_visible() || rect_is_empty(*rect) {
                    continue;
                }
                let commands = rect_path(*rect);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::RectFilled { rect, color } => {
                if color.a <= 0.0 || rect_is_empty(*rect) {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::Quad {
                    order,
                    rect: *rect,
                    background: Paint::Solid(*color).into(),
                    border: Edges::all(Px(0.0)),
                    border_paint: Paint::Solid(Color::TRANSPARENT).into(),
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
            DebugDrawCommand::Quad {
                p1,
                p2,
                p3,
                p4,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let commands = quad_path(*p1, *p2, *p3, *p4);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::QuadFilled {
                p1,
                p2,
                p3,
                p4,
                color,
            } => {
                if color.a <= 0.0 {
                    continue;
                }
                let commands = quad_path(*p1, *p2, *p3, *p4);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Triangle {
                p1,
                p2,
                p3,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() || triangle_is_degenerate(*p1, *p2, *p3) {
                    continue;
                }
                let commands = triangle_path(*p1, *p2, *p3);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::TriangleFilled { p1, p2, p3, color } => {
                if color.a <= 0.0 || triangle_is_degenerate(*p1, *p2, *p3) {
                    continue;
                }
                let commands = triangle_path(*p1, *p2, *p3);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Circle {
                center,
                radius,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() || radius.0 <= 0.0 {
                    continue;
                }
                let commands = circle_path(*center, *radius);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::CircleFilled {
                center,
                radius,
                color,
            } => {
                if color.a <= 0.0 || radius.0 <= 0.0 {
                    continue;
                }
                let commands = circle_path(*center, *radius);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Ngon {
                center,
                radius,
                segments,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let Some(commands) = ngon_path(*center, *radius, *segments) else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::NgonFilled {
                center,
                radius,
                segments,
                color,
            } => {
                if color.a <= 0.0 {
                    continue;
                }
                let Some(commands) = ngon_path(*center, *radius, *segments) else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Ellipse {
                center,
                radius,
                rotation_radians,
                segments,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let Some(commands) = ellipse_path(*center, *radius, *rotation_radians, *segments)
                else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::EllipseFilled {
                center,
                radius,
                rotation_radians,
                segments,
                color,
            } => {
                if color.a <= 0.0 {
                    continue;
                }
                let Some(commands) = ellipse_path(*center, *radius, *rotation_radians, *segments)
                else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::BezierQuadratic {
                from,
                ctrl,
                to,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let commands = bezier_quadratic_path(*from, *ctrl, *to);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::BezierCubic {
                from,
                ctrl1,
                ctrl2,
                to,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let commands = bezier_cubic_path(*from, *ctrl1, *ctrl2, *to);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Text {
                origin,
                text,
                color,
                size,
            } => {
                if color.a <= 0.0 || size.0 <= 0.0 {
                    continue;
                }
                painter.shared_text(
                    order,
                    *origin,
                    text.clone(),
                    TextStyle {
                        size: *size,
                        line_height: Some(Px(size.0 * 1.2)),
                        ..Default::default()
                    },
                    *color,
                    CanvasTextConstraints {
                        max_width: None,
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    },
                    scale,
                );
            }
        }
    }

    for _ in 0..open_clip_depth {
        painter.scene().push(fret_core::SceneOp::PopClip);
    }
}

fn rect_is_empty(rect: Rect) -> bool {
    rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0
}

fn normalized_opacity(opacity: f32) -> f32 {
    if opacity.is_finite() {
        opacity.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn uv_rect_is_valid(uv: UvRect) -> bool {
    uv.u0.is_finite()
        && uv.v0.is_finite()
        && uv.u1.is_finite()
        && uv.v1.is_finite()
        && uv.u1 > uv.u0
        && uv.v1 > uv.v0
}

fn path_stroke_required_points(closed: bool) -> usize {
    if closed { 3 } else { 2 }
}

fn path_arc_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_ARC_SEGMENTS
    } else {
        segments
    }
}

fn path_bezier_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_BEZIER_SEGMENTS
    } else {
        segments
    }
}

fn path_elliptical_arc_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS
    } else {
        segments
    }
}

fn append_arc_points(
    points: &mut Vec<Point>,
    center: Point,
    radius: Px,
    a_min: f32,
    a_max: f32,
    segments: usize,
) {
    for step in 0..=segments {
        let t = if segments == 0 {
            0.0
        } else {
            step as f32 / segments as f32
        };
        points.push(arc_point(center, radius, a_min + t * (a_max - a_min)));
    }
}

fn arc_point(center: Point, radius: Px, angle: f32) -> Point {
    let (sin, cos) = angle.sin_cos();
    Point::new(
        Px(center.x.0 + cos * radius.0),
        Px(center.y.0 + sin * radius.0),
    )
}

fn append_elliptical_arc_points(
    points: &mut Vec<Point>,
    center: Point,
    radius: Size,
    rotation_radians: f32,
    a_min: f32,
    a_max: f32,
    segments: usize,
) {
    for step in 0..=segments {
        let t = if segments == 0 {
            0.0
        } else {
            step as f32 / segments as f32
        };
        points.push(elliptical_arc_point(
            center,
            radius,
            rotation_radians,
            a_min + t * (a_max - a_min),
        ));
    }
}

fn elliptical_arc_point(center: Point, radius: Size, rotation_radians: f32, angle: f32) -> Point {
    let (angle_sin, angle_cos) = angle.sin_cos();
    let (rot_sin, rot_cos) = rotation_radians.sin_cos();
    let x = angle_cos * radius.width.0;
    let y = angle_sin * radius.height.0;
    Point::new(
        Px(center.x.0 + x * rot_cos - y * rot_sin),
        Px(center.y.0 + x * rot_sin + y * rot_cos),
    )
}

fn quadratic_bezier_point(from: Point, ctrl: Point, to: Point, t: f32) -> Point {
    let u = 1.0 - t;
    Point::new(
        Px(u * u * from.x.0 + 2.0 * u * t * ctrl.x.0 + t * t * to.x.0),
        Px(u * u * from.y.0 + 2.0 * u * t * ctrl.y.0 + t * t * to.y.0),
    )
}

fn cubic_bezier_point(from: Point, ctrl1: Point, ctrl2: Point, to: Point, t: f32) -> Point {
    let u = 1.0 - t;
    let uu = u * u;
    let tt = t * t;
    Point::new(
        Px(uu * u * from.x.0
            + 3.0 * uu * t * ctrl1.x.0
            + 3.0 * u * tt * ctrl2.x.0
            + tt * t * to.x.0),
        Px(uu * u * from.y.0
            + 3.0 * uu * t * ctrl1.y.0
            + 3.0 * u * tt * ctrl2.y.0
            + tt * t * to.y.0),
    )
}

fn polyline_path(points: &[Point], closed: bool) -> Option<Vec<PathCommand>> {
    if points.len() < path_stroke_required_points(closed) {
        return None;
    }

    let mut commands = Vec::with_capacity(points.len() + usize::from(closed));
    commands.push(PathCommand::MoveTo(points[0]));
    for point in &points[1..] {
        commands.push(PathCommand::LineTo(*point));
    }
    if closed {
        commands.push(PathCommand::Close);
    }
    Some(commands)
}

fn convex_poly_fill_path(points: &[Point]) -> Option<Vec<PathCommand>> {
    polyline_path(points, true)
}

fn rect_path(rect: Rect) -> [PathCommand; 5] {
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let x1 = Px(rect.origin.x.0 + rect.size.width.0);
    let y1 = Px(rect.origin.y.0 + rect.size.height.0);
    [
        PathCommand::MoveTo(Point::new(x0, y0)),
        PathCommand::LineTo(Point::new(x1, y0)),
        PathCommand::LineTo(Point::new(x1, y1)),
        PathCommand::LineTo(Point::new(x0, y1)),
        PathCommand::Close,
    ]
}

fn triangle_path(p1: Point, p2: Point, p3: Point) -> [PathCommand; 4] {
    [
        PathCommand::MoveTo(p1),
        PathCommand::LineTo(p2),
        PathCommand::LineTo(p3),
        PathCommand::Close,
    ]
}

fn quad_path(p1: Point, p2: Point, p3: Point, p4: Point) -> [PathCommand; 5] {
    [
        PathCommand::MoveTo(p1),
        PathCommand::LineTo(p2),
        PathCommand::LineTo(p3),
        PathCommand::LineTo(p4),
        PathCommand::Close,
    ]
}

fn triangle_is_degenerate(p1: Point, p2: Point, p3: Point) -> bool {
    let ax = p2.x.0 - p1.x.0;
    let ay = p2.y.0 - p1.y.0;
    let bx = p3.x.0 - p1.x.0;
    let by = p3.y.0 - p1.y.0;
    (ax * by - ay * bx).abs() <= f32::EPSILON
}

fn circle_path(center: Point, radius: Px) -> [PathCommand; 6] {
    let r = radius.0;
    let k = 0.552_284_8_f32 * r;
    let cx = center.x.0;
    let cy = center.y.0;
    [
        PathCommand::MoveTo(Point::new(Px(cx + r), Px(cy))),
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + r), Px(cy + k)),
            ctrl2: Point::new(Px(cx + k), Px(cy + r)),
            to: Point::new(Px(cx), Px(cy + r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - k), Px(cy + r)),
            ctrl2: Point::new(Px(cx - r), Px(cy + k)),
            to: Point::new(Px(cx - r), Px(cy)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - r), Px(cy - k)),
            ctrl2: Point::new(Px(cx - k), Px(cy - r)),
            to: Point::new(Px(cx), Px(cy - r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + k), Px(cy - r)),
            ctrl2: Point::new(Px(cx + r), Px(cy - k)),
            to: Point::new(Px(cx + r), Px(cy)),
        },
        PathCommand::Close,
    ]
}

fn ngon_path(center: Point, radius: Px, segments: usize) -> Option<Vec<PathCommand>> {
    if segments < 3 || radius.0 <= 0.0 || !radius.0.is_finite() {
        return None;
    }

    let mut commands = Vec::with_capacity(segments.checked_add(1)?);
    for index in 0..segments {
        let angle = std::f32::consts::TAU * index as f32 / segments as f32;
        let (sin, cos) = angle.sin_cos();
        let point = Point::new(
            Px(center.x.0 + cos * radius.0),
            Px(center.y.0 + sin * radius.0),
        );
        if index == 0 {
            commands.push(PathCommand::MoveTo(point));
        } else {
            commands.push(PathCommand::LineTo(point));
        }
    }
    commands.push(PathCommand::Close);
    Some(commands)
}

fn ellipse_path(
    center: Point,
    radius: Size,
    rotation_radians: f32,
    segments: usize,
) -> Option<Vec<PathCommand>> {
    let segments = if segments == 0 {
        DEFAULT_ELLIPSE_SEGMENTS
    } else {
        segments
    };
    if segments < 3
        || radius.width.0 <= 0.0
        || radius.height.0 <= 0.0
        || !radius.width.0.is_finite()
        || !radius.height.0.is_finite()
        || !rotation_radians.is_finite()
    {
        return None;
    }

    let (rot_sin, rot_cos) = rotation_radians.sin_cos();
    let mut commands = Vec::with_capacity(segments.checked_add(1)?);
    for index in 0..segments {
        let angle = std::f32::consts::TAU * index as f32 / segments as f32;
        let (angle_sin, angle_cos) = angle.sin_cos();
        let x = angle_cos * radius.width.0;
        let y = angle_sin * radius.height.0;
        let point = Point::new(
            Px(center.x.0 + x * rot_cos - y * rot_sin),
            Px(center.y.0 + x * rot_sin + y * rot_cos),
        );
        if index == 0 {
            commands.push(PathCommand::MoveTo(point));
        } else {
            commands.push(PathCommand::LineTo(point));
        }
    }
    commands.push(PathCommand::Close);
    Some(commands)
}

fn bezier_quadratic_path(from: Point, ctrl: Point, to: Point) -> [PathCommand; 2] {
    [PathCommand::MoveTo(from), PathCommand::QuadTo { ctrl, to }]
}

fn bezier_cubic_path(from: Point, ctrl1: Point, ctrl2: Point, to: Point) -> [PathCommand; 2] {
    [
        PathCommand::MoveTo(from),
        PathCommand::CubicTo { ctrl1, ctrl2, to },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Size;

    fn assert_point_near(actual: Point, expected: Point) {
        assert!(
            (actual.x.0 - expected.x.0).abs() <= 0.000_1,
            "x mismatch: actual {:?}, expected {:?}",
            actual,
            expected
        );
        assert!(
            (actual.y.0 - expected.y.0).abs() <= 0.000_1,
            "y mismatch: actual {:?}, expected {:?}",
            actual,
            expected
        );
    }

    #[test]
    fn debug_draw_list_records_commands_in_order() {
        let mut list = ImUiDebugDrawList::default();
        assert!(list.is_empty());

        list.add_line(
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(10.0), Px(10.0)),
            Color::from_srgb_hex_rgb(0xff_00_00),
            Px(1.0),
        );
        list.add_polyline(
            [
                Point::new(Px(0.0), Px(0.0)),
                Point::new(Px(4.0), Px(8.0)),
                Point::new(Px(8.0), Px(2.0)),
            ],
            Color::from_srgb_hex_rgb(0xff_ff_00),
            Px(1.0),
            false,
        );
        list.add_convex_poly_filled(
            [
                Point::new(Px(12.0), Px(62.0)),
                Point::new(Px(24.0), Px(54.0)),
                Point::new(Px(36.0), Px(62.0)),
                Point::new(Px(30.0), Px(76.0)),
                Point::new(Px(18.0), Px(76.0)),
            ],
            Color::from_srgb_hex_rgb(0x10_b9_81),
        );
        list.add_rect(
            Rect::new(Point::new(Px(2.0), Px(3.0)), Size::new(Px(4.0), Px(5.0))),
            Color::from_srgb_hex_rgb(0x00_ff_00),
            Px(2.0),
        );
        list.add_rect_filled(
            Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(2.0), Px(2.0))),
            Color::from_srgb_hex_rgb(0x00_00_ff),
        );
        list.add_quad(
            Point::new(Px(8.0), Px(8.0)),
            Point::new(Px(18.0), Px(6.0)),
            Point::new(Px(22.0), Px(18.0)),
            Point::new(Px(10.0), Px(20.0)),
            Color::from_srgb_hex_rgb(0xfb_71_85),
            Px(1.0),
        );
        list.add_quad_filled(
            Point::new(Px(24.0), Px(8.0)),
            Point::new(Px(34.0), Px(6.0)),
            Point::new(Px(38.0), Px(18.0)),
            Point::new(Px(26.0), Px(20.0)),
            Color::from_srgb_hex_rgb(0x2d_d4_bf),
        );
        list.add_triangle(
            Point::new(Px(1.0), Px(1.0)),
            Point::new(Px(5.0), Px(1.0)),
            Point::new(Px(3.0), Px(4.0)),
            Color::from_srgb_hex_rgb(0xff_00_ff),
            Px(1.0),
        );
        list.add_triangle_filled(
            Point::new(Px(2.0), Px(2.0)),
            Point::new(Px(6.0), Px(2.0)),
            Point::new(Px(4.0), Px(5.0)),
            Color::from_srgb_hex_rgb(0x00_ff_ff),
        );
        list.add_circle(
            Point::new(Px(20.0), Px(20.0)),
            Px(8.0),
            Color::from_srgb_hex_rgb(0xff_aa_00),
            Px(2.0),
        );
        list.add_circle_filled(
            Point::new(Px(40.0), Px(20.0)),
            Px(6.0),
            Color::from_srgb_hex_rgb(0xaa_00_ff),
        );
        list.add_ngon(
            Point::new(Px(56.0), Px(20.0)),
            Px(8.0),
            5,
            Color::from_srgb_hex_rgb(0x65_a3_ff),
            Px(1.0),
        );
        list.add_ngon_filled(
            Point::new(Px(76.0), Px(20.0)),
            Px(6.0),
            6,
            Color::from_srgb_hex_rgb(0xc0_84_fc),
        );
        list.add_ellipse(
            Point::new(Px(96.0), Px(20.0)),
            Size::new(Px(12.0), Px(6.0)),
            0.25,
            16,
            Color::from_srgb_hex_rgb(0x38_bd_f8),
            Px(1.0),
        );
        list.add_ellipse_filled(
            Point::new(Px(122.0), Px(20.0)),
            Size::new(Px(10.0), Px(5.0)),
            0.5,
            0,
            Color::from_srgb_hex_rgb(0xf0_ab_fc),
        );
        list.add_bezier_quadratic(
            Point::new(Px(2.0), Px(60.0)),
            Point::new(Px(20.0), Px(42.0)),
            Point::new(Px(38.0), Px(60.0)),
            Color::from_srgb_hex_rgb(0x22_d3_ee),
            Px(1.0),
        );
        list.add_bezier_cubic(
            Point::new(Px(42.0), Px(60.0)),
            Point::new(Px(54.0), Px(42.0)),
            Point::new(Px(70.0), Px(78.0)),
            Point::new(Px(82.0), Px(60.0)),
            Color::from_srgb_hex_rgb(0xf4_72_b6),
            Px(1.0),
        );
        list.add_text(
            Point::new(Px(4.0), Px(5.0)),
            "debug",
            Color::from_srgb_hex_rgb(0xff_ff_ff),
            Px(12.0),
        );

        assert_eq!(list.command_count(), 18);
        assert!(matches!(list.commands[0], DebugDrawCommand::Line { .. }));
        assert!(matches!(
            list.commands[1],
            DebugDrawCommand::Polyline { .. }
        ));
        assert!(matches!(
            list.commands[2],
            DebugDrawCommand::ConvexPolyFilled { .. }
        ));
        assert!(matches!(list.commands[3], DebugDrawCommand::Rect { .. }));
        assert!(matches!(
            list.commands[4],
            DebugDrawCommand::RectFilled { .. }
        ));
        assert!(matches!(list.commands[5], DebugDrawCommand::Quad { .. }));
        assert!(matches!(
            list.commands[6],
            DebugDrawCommand::QuadFilled { .. }
        ));
        assert!(matches!(
            list.commands[7],
            DebugDrawCommand::Triangle { .. }
        ));
        assert!(matches!(
            list.commands[8],
            DebugDrawCommand::TriangleFilled { .. }
        ));
        assert!(matches!(list.commands[9], DebugDrawCommand::Circle { .. }));
        assert!(matches!(
            list.commands[10],
            DebugDrawCommand::CircleFilled { .. }
        ));
        assert!(matches!(list.commands[11], DebugDrawCommand::Ngon { .. }));
        assert!(matches!(
            list.commands[12],
            DebugDrawCommand::NgonFilled { .. }
        ));
        assert!(matches!(
            list.commands[13],
            DebugDrawCommand::Ellipse { .. }
        ));
        assert!(matches!(
            list.commands[14],
            DebugDrawCommand::EllipseFilled { .. }
        ));
        assert!(matches!(
            list.commands[15],
            DebugDrawCommand::BezierQuadratic { .. }
        ));
        assert!(matches!(
            list.commands[16],
            DebugDrawCommand::BezierCubic { .. }
        ));
        assert!(matches!(list.commands[17], DebugDrawCommand::Text { .. }));
    }

    #[test]
    fn debug_draw_list_records_clip_stack_commands() {
        let mut list = ImUiDebugDrawList::default();
        list.push_clip_rect(Rect::new(
            Point::new(Px(2.0), Px(3.0)),
            Size::new(Px(40.0), Px(50.0)),
        ));
        list.pop_clip_rect();

        assert_eq!(list.command_count(), 2);
        assert!(matches!(
            list.commands[0],
            DebugDrawCommand::PushClipRect { .. }
        ));
        assert!(matches!(list.commands[1], DebugDrawCommand::PopClipRect));
    }

    #[test]
    fn debug_draw_list_records_image_overlay_commands() {
        let mut list = ImUiDebugDrawList::default();
        let image = ImageId::default();
        let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(24.0), Px(16.0)));
        let image_options = DebugDrawImageOptions {
            fit: ViewportFit::Contain,
            sampling: ImageSamplingHint::Nearest,
            opacity: 0.5,
        };
        let svg_options = DebugDrawSvgOptions {
            fit: SvgFit::Contain,
            opacity: 0.75,
        };

        list.add_image_with_options(rect, image, image_options);
        list.add_image_region(rect, image, UvRect::FULL, image_options);
        list.add_svg_image_with_options(rect, SvgSource::Static(b"<svg/>"), svg_options);
        list.add_svg_mask_icon_with_options(
            rect,
            SvgSource::Static(b"<svg/>"),
            Color::from_srgb_hex_rgb(0xff_ff_ff),
            svg_options,
        );

        assert_eq!(list.command_count(), 4);
        assert!(matches!(list.commands[0], DebugDrawCommand::Image { .. }));
        assert!(matches!(
            list.commands[1],
            DebugDrawCommand::ImageRegion { .. }
        ));
        assert!(matches!(
            list.commands[2],
            DebugDrawCommand::SvgImage { .. }
        ));
        assert!(matches!(
            list.commands[3],
            DebugDrawCommand::SvgMaskIcon { .. }
        ));
    }

    #[test]
    fn debug_draw_path_builder_records_stroke_and_fill_commands() {
        let mut list = ImUiDebugDrawList::default();
        let p0 = Point::new(Px(0.0), Px(0.0));
        let p1 = Point::new(Px(12.0), Px(0.0));
        let p2 = Point::new(Px(12.0), Px(10.0));
        let p3 = Point::new(Px(0.0), Px(10.0));

        list.path(|path| {
            assert!(path.is_empty());
            path.line_to(p0)
                .line_to_merge_duplicate(p0)
                .line_to_merge_duplicate(p1)
                .line_to(p2);
            assert_eq!(path.point_count(), 3);

            path.stroke_with_style(
                Color::from_srgb_hex_rgb(0xff_aa_00),
                DebugDrawStrokeStyle::new(Px(2.0)).with_join(StrokeJoinV1::Round),
                true,
            );
            assert!(path.is_empty());

            path.line_to(p0).line_to(p1).line_to(p2).line_to(p3);
            path.fill_convex(Color::from_srgb_hex_rgb(0x22_c5_5e));
        });

        assert_eq!(list.command_count(), 2);
        let DebugDrawCommand::Polyline {
            points,
            style,
            closed,
            ..
        } = &list.commands[0]
        else {
            panic!("path stroke should record a polyline command");
        };
        assert_eq!(&**points, &[p0, p1, p2]);
        assert_eq!(style.width, Px(2.0));
        assert_eq!(style.join, StrokeJoinV1::Round);
        assert!(*closed);

        let DebugDrawCommand::ConvexPolyFilled { points, .. } = &list.commands[1] else {
            panic!("path fill should record a convex fill command");
        };
        assert_eq!(&**points, &[p0, p1, p2, p3]);
    }

    #[test]
    fn debug_draw_path_builder_appends_bezier_curve_samples() {
        let mut list = ImUiDebugDrawList::default();
        let start = Point::new(Px(0.0), Px(0.0));
        let quad_mid = Point::new(Px(10.0), Px(5.0));
        let quad_end = Point::new(Px(20.0), Px(0.0));
        let cubic_mid = Point::new(Px(30.0), Px(5.0));
        let cubic_end = Point::new(Px(40.0), Px(10.0));

        list.path(|path| {
            path.line_to(start)
                .bezier_quadratic_curve_to(Point::new(Px(10.0), Px(10.0)), quad_end, 2)
                .bezier_cubic_curve_to(
                    Point::new(Px(30.0), Px(0.0)),
                    Point::new(Px(30.0), Px(10.0)),
                    cubic_end,
                    2,
                );
            assert_eq!(path.point_count(), 5);
            path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
        });

        let DebugDrawCommand::Polyline { points, .. } = &list.commands[0] else {
            panic!("path Bezier helpers should record a sampled polyline command");
        };
        assert_eq!(
            &**points,
            &[start, quad_mid, quad_end, cubic_mid, cubic_end]
        );
    }

    #[test]
    fn debug_draw_path_builder_bezier_helpers_require_a_start_point_and_default_segments() {
        let mut list = ImUiDebugDrawList::default();
        let start = Point::new(Px(0.0), Px(0.0));
        let ctrl = Point::new(Px(10.0), Px(10.0));
        let end = Point::new(Px(20.0), Px(0.0));

        list.path(|path| {
            path.bezier_quadratic_curve_to(ctrl, end, 2);
            assert!(path.is_empty());

            path.line_to(start).bezier_quadratic_curve_to(ctrl, end, 0);
            assert_eq!(path.point_count(), DEFAULT_PATH_BEZIER_SEGMENTS + 1);
            path.clear();

            path.bezier_cubic_curve_to(ctrl, ctrl, end, 2);
            assert!(path.is_empty());
        });

        assert_eq!(list.command_count(), 0);
    }

    #[test]
    fn debug_draw_path_builder_appends_arc_samples() {
        let mut list = ImUiDebugDrawList::default();
        let center = Point::new(Px(10.0), Px(20.0));

        list.path(|path| {
            path.arc_to(center, Px(8.0), 0.0, std::f32::consts::PI, 2);
            assert_eq!(path.point_count(), 3);
            path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
        });

        let DebugDrawCommand::Polyline { points, .. } = &list.commands[0] else {
            panic!("path arc helper should record a sampled polyline command");
        };
        assert_eq!(points.len(), 3);
        assert_point_near(points[0], Point::new(Px(18.0), Px(20.0)));
        assert_point_near(points[1], Point::new(Px(10.0), Px(28.0)));
        assert_point_near(points[2], Point::new(Px(2.0), Px(20.0)));
    }

    #[test]
    fn debug_draw_path_builder_arc_helpers_handle_fast_default_and_degenerate_inputs() {
        let mut list = ImUiDebugDrawList::default();
        let center = Point::new(Px(10.0), Px(20.0));

        list.path(|path| {
            path.arc_to(center, Px(0.25), 0.0, std::f32::consts::PI, 4);
            assert_eq!(path.point_count(), 1);
            assert_eq!(path.clear().point_count(), 0);

            path.arc_to(center, Px(8.0), f32::NAN, std::f32::consts::PI, 4);
            path.arc_to(center, Px(0.0), 0.0, std::f32::consts::PI, 4);
            assert!(path.is_empty());

            path.arc_to(center, Px(8.0), 0.0, std::f32::consts::FRAC_PI_2, 0);
            assert_eq!(path.point_count(), DEFAULT_PATH_ARC_SEGMENTS + 1);
            path.clear();

            path.arc_to_fast(center, Px(8.0), 0, 3);
            assert_eq!(path.point_count(), 4);
            path.clear();

            path.arc_to_fast(center, Px(8.0), 3, 0);
            assert_eq!(path.point_count(), 4);
            path.clear();
        });

        assert_eq!(list.command_count(), 0);
    }

    #[test]
    fn debug_draw_path_builder_appends_elliptical_arc_samples() {
        let mut list = ImUiDebugDrawList::default();
        let center = Point::new(Px(10.0), Px(20.0));

        list.path(|path| {
            path.elliptical_arc_to(
                center,
                Size::new(Px(8.0), Px(4.0)),
                0.0,
                0.0,
                std::f32::consts::PI,
                2,
            );
            assert_eq!(path.point_count(), 3);
            path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
        });

        let DebugDrawCommand::Polyline { points, .. } = &list.commands[0] else {
            panic!("path elliptical arc helper should record a sampled polyline command");
        };
        assert_eq!(points.len(), 3);
        assert_point_near(points[0], Point::new(Px(18.0), Px(20.0)));
        assert_point_near(points[1], Point::new(Px(10.0), Px(24.0)));
        assert_point_near(points[2], Point::new(Px(2.0), Px(20.0)));
    }

    #[test]
    fn debug_draw_path_builder_elliptical_arc_handles_rotation_default_and_invalid_inputs() {
        let mut list = ImUiDebugDrawList::default();
        let center = Point::new(Px(10.0), Px(20.0));

        list.path(|path| {
            path.elliptical_arc_to(
                center,
                Size::new(Px(8.0), Px(4.0)),
                std::f32::consts::FRAC_PI_2,
                0.0,
                std::f32::consts::PI,
                2,
            );
            assert_eq!(path.point_count(), 3);
            assert_point_near(path.points[0], Point::new(Px(10.0), Px(28.0)));
            assert_point_near(path.points[1], Point::new(Px(6.0), Px(20.0)));
            assert_point_near(path.points[2], Point::new(Px(10.0), Px(12.0)));
            path.clear();

            path.elliptical_arc_to(
                center,
                Size::new(Px(8.0), Px(4.0)),
                0.0,
                0.0,
                std::f32::consts::FRAC_PI_2,
                0,
            );
            assert_eq!(path.point_count(), DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS + 1);
            path.clear();

            path.elliptical_arc_to(
                center,
                Size::new(Px(0.0), Px(4.0)),
                0.0,
                0.0,
                std::f32::consts::PI,
                2,
            );
            path.elliptical_arc_to(
                center,
                Size::new(Px(8.0), Px(4.0)),
                f32::NAN,
                0.0,
                std::f32::consts::PI,
                2,
            );
            path.elliptical_arc_to(
                center,
                Size::new(Px(8.0), Px(4.0)),
                0.0,
                f32::NAN,
                std::f32::consts::PI,
                2,
            );
            assert!(path.is_empty());
        });

        assert_eq!(list.command_count(), 0);
    }

    #[test]
    fn debug_draw_path_builder_clears_invalid_finished_paths_without_recording() {
        let mut list = ImUiDebugDrawList::default();
        let p0 = Point::new(Px(0.0), Px(0.0));
        let p1 = Point::new(Px(8.0), Px(0.0));

        list.path(|path| {
            path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
            assert!(path.is_empty());

            path.line_to(p0);
            path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
            assert!(path.is_empty());

            path.line_to(p0).line_to(p1);
            path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), true);
            assert!(path.is_empty());

            path.line_to(p0).line_to(p1);
            path.fill_convex(Color::from_srgb_hex_rgb(0xff_ff_ff));
            assert!(path.is_empty());

            path.line_to(p0).line_to(p1);
            assert_eq!(path.point_count(), 2);
            path.clear();
            assert!(path.is_empty());
        });

        assert_eq!(list.command_count(), 0);
    }

    #[test]
    fn image_overlay_helpers_sanitize_opacity_and_uv_rects() {
        assert_eq!(normalized_opacity(-1.0), 0.0);
        assert_eq!(normalized_opacity(2.0), 1.0);
        assert_eq!(normalized_opacity(f32::NAN), 1.0);

        assert!(uv_rect_is_valid(UvRect::FULL));
        assert!(!uv_rect_is_valid(UvRect {
            u0: 0.5,
            v0: 0.0,
            u1: 0.25,
            v1: 1.0,
        }));
    }

    #[test]
    fn rect_path_closes_clockwise_edges() {
        let path = rect_path(Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        ));

        assert_eq!(
            path,
            [
                PathCommand::MoveTo(Point::new(Px(10.0), Px(20.0))),
                PathCommand::LineTo(Point::new(Px(40.0), Px(20.0))),
                PathCommand::LineTo(Point::new(Px(40.0), Px(60.0))),
                PathCommand::LineTo(Point::new(Px(10.0), Px(60.0))),
                PathCommand::Close,
            ]
        );
    }

    #[test]
    fn debug_draw_stroke_style_uses_v1_for_default_and_v2_for_explicit_policy() {
        let default_style = DebugDrawStrokeStyle::new(Px(2.0));
        assert_eq!(
            default_style.path_style(),
            PathStyle::Stroke(StrokeStyle { width: Px(2.0) })
        );

        let styled = DebugDrawStrokeStyle::new(Px(3.0))
            .with_join(StrokeJoinV1::Round)
            .with_cap(StrokeCapV1::Round)
            .with_miter_limit(8.0)
            .with_dash(Px(6.0), Px(4.0), Px(1.0));

        let PathStyle::StrokeV2(stroke) = styled.path_style() else {
            panic!("explicit debug-draw stroke policy should use StrokeV2");
        };
        assert_eq!(stroke.width, Px(3.0));
        assert_eq!(stroke.join, StrokeJoinV1::Round);
        assert_eq!(stroke.cap, StrokeCapV1::Round);
        assert_eq!(stroke.miter_limit, 8.0);
        assert_eq!(
            stroke.dash,
            Some(DashPatternV1::new(Px(6.0), Px(4.0), Px(1.0)))
        );
    }

    #[test]
    fn debug_draw_stroke_style_ignores_invalid_dash_and_miter_inputs() {
        let style = DebugDrawStrokeStyle::new(Px(2.0))
            .with_miter_limit(f32::NAN)
            .with_dash(Px(0.0), Px(4.0), Px(0.0))
            .with_dash_pattern(DashPatternV1::new(Px(4.0), Px(-1.0), Px(0.0)));

        assert_eq!(style.miter_limit, 4.0);
        assert_eq!(style.dash, None);
        assert_eq!(
            style.path_style(),
            PathStyle::Stroke(StrokeStyle { width: Px(2.0) })
        );
    }

    #[test]
    fn polyline_path_requires_enough_points_and_closes_when_requested() {
        assert!(polyline_path(&[Point::new(Px(0.0), Px(0.0))], false).is_none());
        assert!(
            polyline_path(
                &[Point::new(Px(0.0), Px(0.0)), Point::new(Px(1.0), Px(1.0))],
                true,
            )
            .is_none()
        );

        let path = polyline_path(
            &[
                Point::new(Px(0.0), Px(0.0)),
                Point::new(Px(10.0), Px(0.0)),
                Point::new(Px(10.0), Px(10.0)),
            ],
            true,
        )
        .unwrap();

        assert_eq!(
            path,
            vec![
                PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
                PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
                PathCommand::LineTo(Point::new(Px(10.0), Px(10.0))),
                PathCommand::Close,
            ]
        );
    }

    #[test]
    fn convex_poly_fill_path_requires_three_points_and_closes() {
        assert!(convex_poly_fill_path(&[Point::new(Px(0.0), Px(0.0))]).is_none());
        assert!(
            convex_poly_fill_path(&[Point::new(Px(0.0), Px(0.0)), Point::new(Px(10.0), Px(0.0)),])
                .is_none()
        );

        let path = convex_poly_fill_path(&[
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(10.0), Px(0.0)),
            Point::new(Px(12.0), Px(8.0)),
            Point::new(Px(2.0), Px(10.0)),
        ])
        .unwrap();

        assert_eq!(
            path,
            vec![
                PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
                PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
                PathCommand::LineTo(Point::new(Px(12.0), Px(8.0))),
                PathCommand::LineTo(Point::new(Px(2.0), Px(10.0))),
                PathCommand::Close,
            ]
        );
    }

    #[test]
    fn triangle_path_closes_and_degenerate_triangles_are_detected() {
        let p1 = Point::new(Px(0.0), Px(0.0));
        let p2 = Point::new(Px(10.0), Px(0.0));
        let p3 = Point::new(Px(5.0), Px(8.0));

        assert_eq!(
            triangle_path(p1, p2, p3),
            [
                PathCommand::MoveTo(p1),
                PathCommand::LineTo(p2),
                PathCommand::LineTo(p3),
                PathCommand::Close,
            ]
        );
        assert!(!triangle_is_degenerate(p1, p2, p3));
        assert!(triangle_is_degenerate(p1, Point::new(Px(5.0), Px(0.0)), p2));
    }

    #[test]
    fn quad_path_closes_four_ordered_points() {
        let p1 = Point::new(Px(0.0), Px(0.0));
        let p2 = Point::new(Px(10.0), Px(2.0));
        let p3 = Point::new(Px(12.0), Px(12.0));
        let p4 = Point::new(Px(2.0), Px(10.0));

        assert_eq!(
            quad_path(p1, p2, p3, p4),
            [
                PathCommand::MoveTo(p1),
                PathCommand::LineTo(p2),
                PathCommand::LineTo(p3),
                PathCommand::LineTo(p4),
                PathCommand::Close,
            ]
        );
    }

    #[test]
    fn circle_path_uses_four_cubic_arcs_and_closes() {
        let path = circle_path(Point::new(Px(10.0), Px(20.0)), Px(8.0));

        assert_eq!(path.len(), 6);
        assert_eq!(path[0], PathCommand::MoveTo(Point::new(Px(18.0), Px(20.0))));
        assert!(matches!(path[1], PathCommand::CubicTo { .. }));
        assert!(matches!(path[2], PathCommand::CubicTo { .. }));
        assert!(matches!(path[3], PathCommand::CubicTo { .. }));
        assert!(matches!(path[4], PathCommand::CubicTo { .. }));
        assert_eq!(path[5], PathCommand::Close);
    }

    #[test]
    fn ngon_path_requires_three_segments_and_positive_radius() {
        assert!(ngon_path(Point::new(Px(0.0), Px(0.0)), Px(8.0), 2).is_none());
        assert!(ngon_path(Point::new(Px(0.0), Px(0.0)), Px(0.0), 3).is_none());

        let path = ngon_path(Point::new(Px(10.0), Px(20.0)), Px(8.0), 4).unwrap();

        assert_eq!(path.len(), 5);
        assert_eq!(path[0], PathCommand::MoveTo(Point::new(Px(18.0), Px(20.0))));
        assert!(matches!(path[1], PathCommand::LineTo(_)));
        assert!(matches!(path[2], PathCommand::LineTo(_)));
        assert!(matches!(path[3], PathCommand::LineTo(_)));
        assert_eq!(path[4], PathCommand::Close);
    }

    #[test]
    fn ellipse_path_defaults_segments_and_supports_rotation() {
        assert!(
            ellipse_path(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(8.0), Px(4.0)),
                0.0,
                2
            )
            .is_none()
        );
        assert!(
            ellipse_path(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(0.0), Px(4.0)),
                0.0,
                4
            )
            .is_none()
        );
        assert!(
            ellipse_path(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(8.0), Px(4.0)),
                f32::NAN,
                4,
            )
            .is_none()
        );

        let default_path = ellipse_path(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(8.0), Px(4.0)),
            0.0,
            0,
        )
        .unwrap();
        assert_eq!(default_path.len(), DEFAULT_ELLIPSE_SEGMENTS + 1);
        assert_eq!(
            default_path[0],
            PathCommand::MoveTo(Point::new(Px(18.0), Px(20.0)))
        );
        assert_eq!(default_path[DEFAULT_ELLIPSE_SEGMENTS], PathCommand::Close);

        let rotated_path = ellipse_path(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(8.0), Px(4.0)),
            std::f32::consts::FRAC_PI_2,
            4,
        )
        .unwrap();
        let PathCommand::MoveTo(point) = &rotated_path[0] else {
            panic!("rotated ellipse should start with a MoveTo");
        };
        assert!((point.x.0 - 10.0).abs() <= 0.000_1);
        assert!((point.y.0 - 28.0).abs() <= 0.000_1);
        assert_eq!(rotated_path[4], PathCommand::Close);
    }

    #[test]
    fn bezier_paths_use_native_quad_and_cubic_commands() {
        let from = Point::new(Px(0.0), Px(0.0));
        let ctrl = Point::new(Px(10.0), Px(20.0));
        let ctrl1 = Point::new(Px(8.0), Px(16.0));
        let ctrl2 = Point::new(Px(18.0), Px(-6.0));
        let to = Point::new(Px(24.0), Px(0.0));

        assert_eq!(
            bezier_quadratic_path(from, ctrl, to),
            [PathCommand::MoveTo(from), PathCommand::QuadTo { ctrl, to }]
        );
        assert_eq!(
            bezier_cubic_path(from, ctrl1, ctrl2, to),
            [
                PathCommand::MoveTo(from),
                PathCommand::CubicTo { ctrl1, ctrl2, to },
            ]
        );
    }
}

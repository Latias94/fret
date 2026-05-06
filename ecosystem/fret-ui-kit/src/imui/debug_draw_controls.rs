//! Immediate-mode debug draw helper backed by declarative `Canvas`.

use std::hash::Hash;
use std::sync::Arc;

use fret_core::scene::DashPatternV1;
use fret_core::scene::ImageSamplingHint;
use fret_core::{
    Color, ImageId, PathStyle, Point, Px, Rect, SceneMeshVertex, Size, StrokeCapV1, StrokeJoinV1,
    StrokeStyle, StrokeStyleV2, SvgFit, UvPoint, UvRect, ViewportFit,
};
use fret_ui::action::ActivateReason;
use fret_ui::element::{
    AnyElement, CanvasCachePolicy, CanvasProps, LayoutStyle, Length, PressableA11y, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, SvgSource, UiHost};

use super::{ResponseExt, UiWriterImUiFacadeExt};

mod commands;
mod geometry;
mod paint;
mod paths;

use commands::DebugDrawCommand;
pub use commands::{DebugDrawCommandKind, DebugDrawCommandSummary, DebugDrawListSummary};
#[cfg(test)]
use geometry::triangle_is_degenerate;
use geometry::{rect_is_empty, rect_is_finite, sequential_triangle_indices};
use paint::paint_debug_draw_commands;
#[cfg(test)]
use paint::{
    corner_radii_are_visible, normalized_opacity, rounded_rect_corner_radii, uv_rect_is_valid,
};
use paths::{
    append_arc_points, append_elliptical_arc_points, append_path_rect_points, bezier_cubic_path,
    bezier_quadratic_path, circle_path, concave_poly_fill_path, convex_poly_fill_path,
    cubic_bezier_point, ellipse_path, ngon_path, path_arc_segments, path_bezier_segments,
    path_elliptical_arc_segments, path_stroke_required_points, polyline_path, quad_path,
    quadratic_bezier_point, rect_path, triangle_path,
};

#[cfg(test)]
use fret_core::PathCommand;

const DEFAULT_ELLIPSE_SEGMENTS: usize = 32;
const DEFAULT_PATH_ARC_SEGMENTS: usize = 12;
const DEFAULT_PATH_BEZIER_SEGMENTS: usize = 12;
const DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS: usize = 32;

#[derive(Debug, Clone)]
pub struct DebugDrawOptions {
    pub layout: LayoutStyle,
    pub test_id: Option<Arc<str>>,
    pub clip_to_bounds: bool,
    pub interaction: DebugDrawInteractionOptions,
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
            interaction: DebugDrawInteractionOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DebugDrawInteractionOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
}

impl DebugDrawInteractionOptions {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn with_a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct DebugDrawResponse {
    pub response: ResponseExt,
    pub list_summary: DebugDrawListSummary,
    pub command_summaries: Arc<[DebugDrawCommandSummary]>,
}

impl DebugDrawResponse {
    pub fn command_summaries(&self) -> &[DebugDrawCommandSummary] {
        &self.command_summaries
    }

    pub fn list_summary(&self) -> DebugDrawListSummary {
        self.list_summary
    }

    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    pub fn hovered_like_imgui(&self) -> bool {
        self.response.hovered_like_imgui()
    }

    pub fn rect(&self) -> Option<Rect> {
        self.response.core.rect
    }
}

#[derive(Debug, Clone)]
pub struct ImUiDebugDrawList {
    commands: Vec<DebugDrawCommand>,
    channel_split: Option<DebugDrawChannelSplit>,
}

#[derive(Debug, Clone)]
struct DebugDrawChannelSplit {
    channels: Vec<Vec<DebugDrawCommand>>,
    current: usize,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugDrawRoundCorners(u8);

impl DebugDrawRoundCorners {
    pub const NONE: Self = Self(0);
    pub const TOP_LEFT: Self = Self(1 << 0);
    pub const TOP_RIGHT: Self = Self(1 << 1);
    pub const BOTTOM_RIGHT: Self = Self(1 << 2);
    pub const BOTTOM_LEFT: Self = Self(1 << 3);
    pub const TOP: Self = Self(Self::TOP_LEFT.0 | Self::TOP_RIGHT.0);
    pub const BOTTOM: Self = Self(Self::BOTTOM_LEFT.0 | Self::BOTTOM_RIGHT.0);
    pub const LEFT: Self = Self(Self::TOP_LEFT.0 | Self::BOTTOM_LEFT.0);
    pub const RIGHT: Self = Self(Self::TOP_RIGHT.0 | Self::BOTTOM_RIGHT.0);
    pub const ALL: Self = Self(Self::TOP.0 | Self::BOTTOM.0);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl Default for DebugDrawRoundCorners {
    fn default() -> Self {
        Self::ALL
    }
}

impl std::ops::BitOr for DebugDrawRoundCorners {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DebugDrawRoundCorners {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
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
pub struct DebugDrawImageQuadOptions {
    pub sampling: ImageSamplingHint,
    pub tint: Color,
    pub opacity: f32,
}

impl Default for DebugDrawImageQuadOptions {
    fn default() -> Self {
        Self {
            sampling: ImageSamplingHint::Default,
            tint: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawVertex {
    pub position: Point,
    pub uv: UvPoint,
    pub color: Color,
}

impl DebugDrawVertex {
    pub const fn new(position: Point, uv: UvPoint, color: Color) -> Self {
        Self {
            position,
            uv,
            color,
        }
    }

    pub const fn colored(position: Point, color: Color) -> Self {
        Self {
            position,
            uv: UvPoint::ZERO,
            color,
        }
    }

    fn scene_vertex(self) -> SceneMeshVertex {
        SceneMeshVertex::new(self.position, self.uv, self.color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawImageMeshOptions {
    pub sampling: ImageSamplingHint,
    pub opacity: f32,
}

impl Default for DebugDrawImageMeshOptions {
    fn default() -> Self {
        Self {
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

    pub fn rect(&mut self, rect: Rect) -> &mut Self {
        self.rect_with_rounding(rect, Px(0.0), DebugDrawRoundCorners::ALL)
    }

    pub fn rect_with_rounding(
        &mut self,
        rect: Rect,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    ) -> &mut Self {
        if rect_is_empty(rect) || !rect_is_finite(rect) || !rounding.0.is_finite() {
            return self;
        }
        append_path_rect_points(&mut self.points, rect, rounding, corners);
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

    pub fn fill_concave(&mut self, color: Color) {
        let points = std::mem::take(&mut self.points);
        if points.len() < 3 {
            return;
        }
        self.draw_list.add_concave_poly_filled(points, color);
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

    pub fn channels_split(&mut self, count: usize) {
        if count <= 1 || self.channel_split.is_some() {
            return;
        }
        self.channel_split = Some(DebugDrawChannelSplit {
            channels: (0..count).map(|_| Vec::new()).collect(),
            current: 0,
        });
    }

    pub fn channels_set_current(&mut self, channel: usize) {
        let Some(split) = self.channel_split.as_mut() else {
            return;
        };
        if channel >= split.channels.len() || channel == split.current {
            return;
        }

        std::mem::swap(&mut split.channels[split.current], &mut self.commands);
        std::mem::swap(&mut split.channels[channel], &mut self.commands);
        split.current = channel;
    }

    pub fn channels_merge(&mut self) {
        let Some(mut split) = self.channel_split.take() else {
            return;
        };
        std::mem::swap(&mut split.channels[split.current], &mut self.commands);

        let total_commands = split.channels.iter().map(Vec::len).sum();
        let mut merged = Vec::with_capacity(total_commands);
        for mut channel in split.channels {
            merged.append(&mut channel);
        }
        self.commands = merged;
    }

    fn for_each_command_with_channel<F>(&self, mut visit: F)
    where
        F: FnMut(Option<usize>, &DebugDrawCommand),
    {
        let Some(split) = self.channel_split.as_ref() else {
            for command in &self.commands {
                visit(None, command);
            }
            return;
        };

        for (channel, commands) in split.channels.iter().enumerate() {
            let commands = if channel == split.current {
                self.commands.as_slice()
            } else {
                commands.as_slice()
            };
            for command in commands {
                visit(Some(channel), command);
            }
        }
    }

    /// Return command summaries in the order the list would paint after channel merge.
    pub fn command_summaries(&self) -> Vec<DebugDrawCommandSummary> {
        let mut summaries = Vec::with_capacity(self.command_count());
        let mut clip_stack = Vec::new();
        self.for_each_command_with_channel(|channel, command| {
            summaries.push(command.summary_with_clip_state(channel, &mut clip_stack));
        });
        summaries
    }

    /// Return aggregate source-level metadata for recorded debug draw commands.
    pub fn list_summary(&self) -> DebugDrawListSummary {
        let mut summary = DebugDrawListSummary::default();
        let mut clip_stack = Vec::new();
        self.for_each_command_with_channel(|channel, command| {
            summary.include(command.summary_with_clip_state(channel, &mut clip_stack));
        });
        summary.final_clip_depth = clip_stack.len();
        summary
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

    pub fn add_concave_poly_filled<I>(&mut self, points: I, color: Color)
    where
        I: IntoIterator<Item = Point>,
    {
        let points: Arc<[Point]> = Arc::from(points.into_iter().collect::<Vec<_>>());
        self.commands
            .push(DebugDrawCommand::ConcavePolyFilled { points, color });
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

    pub fn add_rect_filled_multi_color(
        &mut self,
        rect: Rect,
        upper_left: Color,
        upper_right: Color,
        bottom_right: Color,
        bottom_left: Color,
    ) {
        self.commands.push(DebugDrawCommand::RectFilledMultiColor {
            rect,
            upper_left,
            upper_right,
            bottom_right,
            bottom_left,
        });
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

    pub fn add_triangle_list<V>(&mut self, vertices: V)
    where
        V: IntoIterator<Item = DebugDrawVertex>,
    {
        let vertices: Vec<_> = vertices.into_iter().collect();
        let indices = sequential_triangle_indices(vertices.len());
        self.commands.push(DebugDrawCommand::TriangleMesh {
            vertices: Arc::from(vertices),
            indices,
        });
    }

    pub fn add_triangle_mesh<V, I>(&mut self, vertices: V, indices: I)
    where
        V: IntoIterator<Item = DebugDrawVertex>,
        I: IntoIterator<Item = u32>,
    {
        self.commands.push(DebugDrawCommand::TriangleMesh {
            vertices: Arc::from(vertices.into_iter().collect::<Vec<_>>()),
            indices: Arc::from(indices.into_iter().collect::<Vec<_>>()),
        });
    }

    pub fn add_image_triangle_mesh<V, I>(&mut self, image: ImageId, vertices: V, indices: I)
    where
        V: IntoIterator<Item = DebugDrawVertex>,
        I: IntoIterator<Item = u32>,
    {
        self.add_image_triangle_mesh_with_options(
            image,
            vertices,
            indices,
            DebugDrawImageMeshOptions::default(),
        );
    }

    pub fn add_image_triangle_mesh_with_options<V, I>(
        &mut self,
        image: ImageId,
        vertices: V,
        indices: I,
        options: DebugDrawImageMeshOptions,
    ) where
        V: IntoIterator<Item = DebugDrawVertex>,
        I: IntoIterator<Item = u32>,
    {
        self.commands.push(DebugDrawCommand::ImageTriangleMesh {
            image,
            vertices: Arc::from(vertices.into_iter().collect::<Vec<_>>()),
            indices: Arc::from(indices.into_iter().collect::<Vec<_>>()),
            options,
        });
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

    pub fn add_image_quad(&mut self, image: ImageId, points: [Point; 4], uvs: [UvPoint; 4]) {
        self.add_image_quad_with_options(image, points, uvs, DebugDrawImageQuadOptions::default());
    }

    pub fn add_image_quad_with_options(
        &mut self,
        image: ImageId,
        points: [Point; 4],
        uvs: [UvPoint; 4],
        options: DebugDrawImageQuadOptions,
    ) {
        self.commands.push(DebugDrawCommand::ImageQuad {
            image,
            points,
            uvs,
            options,
        });
    }

    pub fn add_image_rounded(
        &mut self,
        rect: Rect,
        image: ImageId,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    ) {
        self.add_image_rounded_with_options(
            rect,
            image,
            DebugDrawImageOptions::default(),
            rounding,
            corners,
        );
    }

    pub fn add_image_rounded_with_options(
        &mut self,
        rect: Rect,
        image: ImageId,
        options: DebugDrawImageOptions,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    ) {
        self.commands.push(DebugDrawCommand::ImageRounded {
            rect,
            image,
            options,
            rounding,
            corners,
        });
    }

    pub fn add_image_region_rounded(
        &mut self,
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        options: DebugDrawImageOptions,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    ) {
        self.commands.push(DebugDrawCommand::ImageRegionRounded {
            rect,
            image,
            uv,
            options,
            rounding,
            corners,
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
        let split_count = self
            .channel_split
            .as_ref()
            .map(|split| split.channels.iter().map(Vec::len).sum())
            .unwrap_or(0);
        self.commands.len() + split_count
    }

    pub fn is_empty(&self) -> bool {
        self.command_count() == 0
    }
}

impl Default for ImUiDebugDrawList {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            channel_split: None,
        }
    }
}

pub(super) fn debug_draw_with_options<H, W, K, F>(
    ui: &mut W,
    id: K,
    options: DebugDrawOptions,
    draw: F,
) -> DebugDrawResponse
where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    K: Hash,
    F: FnOnce(&mut ImUiDebugDrawList),
{
    let mut list = ImUiDebugDrawList::default();
    draw(&mut list);
    list.channels_merge();
    let list_summary = list.list_summary();
    let command_summaries = Arc::from(list.command_summaries().into_boxed_slice());
    let commands: Arc<[DebugDrawCommand]> = Arc::from(list.commands.into_boxed_slice());
    let mut response = ResponseExt::default();
    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        cx.keyed(("fret-ui-kit.imui.debug_draw", id), |cx| {
            debug_draw_element(cx, commands, options, response)
        })
    });
    ui.add(element);
    DebugDrawResponse {
        response,
        list_summary,
        command_summaries,
    }
}

fn debug_draw_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    commands: Arc<[DebugDrawCommand]>,
    options: DebugDrawOptions,
    response: &mut ResponseExt,
) -> AnyElement {
    if options.interaction.enabled {
        return debug_draw_pressable_element(cx, commands, options, response);
    }

    debug_draw_canvas_element(
        cx,
        commands,
        options.layout,
        options.clip_to_bounds,
        options.test_id,
    )
}

fn debug_draw_pressable_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    commands: Arc<[DebugDrawCommand]>,
    options: DebugDrawOptions,
    response: &mut ResponseExt,
) -> AnyElement {
    let interaction = options.interaction.clone();
    let enabled = interaction.enabled && !super::imui_is_disabled(cx);
    let mut props = PressableProps {
        layout: options.layout,
        enabled,
        focusable: enabled && interaction.focusable,
        a11y: PressableA11y {
            label: interaction.a11y_label,
            ..Default::default()
        },
        ..Default::default()
    };
    props.focus_ring = None;

    let clip_to_bounds = options.clip_to_bounds;
    cx.pressable_with_id(props, move |cx, state, id| {
        let behavior = super::item_behavior::install_pressable_item_behavior_with_options(
            cx,
            id,
            super::item_behavior::PressableItemBehaviorOptions {
                report_pointer_click: true,
            },
        );
        let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

        cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
            if reason == ActivateReason::Keyboard {
                super::mark_lifecycle_instant_if_inactive(
                    host,
                    acx,
                    &lifecycle_model_for_activate,
                    false,
                );
            }
            host.record_transient_event(acx, super::KEY_CLICKED);
            host.notify(acx);
        }));

        let clicked = cx.take_transient_for(id, super::KEY_CLICKED);
        super::item_behavior::populate_pressable_item_response(
            cx,
            id,
            state,
            &behavior,
            super::item_behavior::PressableItemResponseInput {
                enabled,
                clicked,
                changed: false,
                lifecycle_edited: false,
            },
            response,
        );

        vec![debug_draw_canvas_element(
            cx,
            commands,
            debug_draw_fill_layout(),
            clip_to_bounds,
            options.test_id,
        )]
    })
}

fn debug_draw_fill_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn debug_draw_canvas_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    commands: Arc<[DebugDrawCommand]>,
    layout: LayoutStyle,
    clip_to_bounds: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut props = CanvasProps {
        layout,
        cache_policy: CanvasCachePolicy::smooth_default(),
    };
    props.cache_policy.shared_text.keep_frames = 30;
    props.cache_policy.path.keep_frames = 30;

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
    if let Some(test_id) = test_id {
        element = element.test_id(test_id);
    }
    element
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Corners, Size};
    use fret_ui::element::ElementKind;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        )
    }

    fn empty_commands() -> Arc<[DebugDrawCommand]> {
        Arc::from(Vec::<DebugDrawCommand>::new().into_boxed_slice())
    }

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
    fn debug_draw_default_element_stays_noninteractive_canvas() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "debug-draw.canvas", |cx| {
            let mut response = ResponseExt::default();
            let element = debug_draw_element(
                cx,
                empty_commands(),
                DebugDrawOptions {
                    test_id: Some(Arc::from("imui.debug_draw")),
                    ..Default::default()
                },
                &mut response,
            );

            assert!(matches!(element.kind, ElementKind::Canvas(_)));
            assert_eq!(
                element
                    .semantics_decoration
                    .as_ref()
                    .and_then(|decoration| decoration.test_id.as_deref()),
                Some("imui.debug_draw")
            );
            assert!(!response.enabled);
        });
    }

    #[test]
    fn debug_draw_interaction_wraps_canvas_in_pressable_response_surface() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "debug-draw.pressable",
            |cx| {
                let mut response = ResponseExt::default();
                let element = debug_draw_element(
                    cx,
                    empty_commands(),
                    DebugDrawOptions {
                        test_id: Some(Arc::from("imui.debug_draw.interactive")),
                        interaction: DebugDrawInteractionOptions::enabled()
                            .focusable(true)
                            .with_a11y_label("Debug draw canvas"),
                        ..Default::default()
                    },
                    &mut response,
                );

                let ElementKind::Pressable(props) = &element.kind else {
                    panic!("interactive debug draw should wrap the canvas in a pressable");
                };
                assert!(props.enabled);
                assert!(props.focusable);
                assert_eq!(props.a11y.label.as_deref(), Some("Debug draw canvas"));
                assert_eq!(props.a11y.test_id.as_deref(), None);
                assert_eq!(element.children.len(), 1);
                assert!(matches!(element.children[0].kind, ElementKind::Canvas(_)));
                assert_eq!(
                    element.children[0]
                        .semantics_decoration
                        .as_ref()
                        .and_then(|decoration| decoration.test_id.as_deref()),
                    Some("imui.debug_draw.interactive")
                );
                assert!(response.enabled);
            },
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
        list.add_rect_filled_multi_color(
            Rect::new(Point::new(Px(4.0), Px(1.0)), Size::new(Px(6.0), Px(5.0))),
            Color::from_srgb_hex_rgb(0xff_00_00),
            Color::from_srgb_hex_rgb(0x00_ff_00),
            Color::from_srgb_hex_rgb(0x00_00_ff),
            Color::from_srgb_hex_rgb(0xff_ff_00),
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

        assert_eq!(list.command_count(), 19);
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
        assert!(matches!(
            list.commands[5],
            DebugDrawCommand::RectFilledMultiColor { .. }
        ));
        assert!(matches!(list.commands[6], DebugDrawCommand::Quad { .. }));
        assert!(matches!(
            list.commands[7],
            DebugDrawCommand::QuadFilled { .. }
        ));
        assert!(matches!(
            list.commands[8],
            DebugDrawCommand::Triangle { .. }
        ));
        assert!(matches!(
            list.commands[9],
            DebugDrawCommand::TriangleFilled { .. }
        ));
        assert!(matches!(list.commands[10], DebugDrawCommand::Circle { .. }));
        assert!(matches!(
            list.commands[11],
            DebugDrawCommand::CircleFilled { .. }
        ));
        assert!(matches!(list.commands[12], DebugDrawCommand::Ngon { .. }));
        assert!(matches!(
            list.commands[13],
            DebugDrawCommand::NgonFilled { .. }
        ));
        assert!(matches!(
            list.commands[14],
            DebugDrawCommand::Ellipse { .. }
        ));
        assert!(matches!(
            list.commands[15],
            DebugDrawCommand::EllipseFilled { .. }
        ));
        assert!(matches!(
            list.commands[16],
            DebugDrawCommand::BezierQuadratic { .. }
        ));
        assert!(matches!(
            list.commands[17],
            DebugDrawCommand::BezierCubic { .. }
        ));
        assert!(matches!(list.commands[18], DebugDrawCommand::Text { .. }));
    }

    #[test]
    fn debug_draw_list_records_triangle_mesh_commands() {
        let mut list = ImUiDebugDrawList::default();
        let vertices = [
            DebugDrawVertex::colored(
                Point::new(Px(0.0), Px(0.0)),
                Color::from_srgb_hex_rgb(0xff_00_00),
            ),
            DebugDrawVertex::colored(
                Point::new(Px(8.0), Px(0.0)),
                Color::from_srgb_hex_rgb(0x00_ff_00),
            ),
            DebugDrawVertex::colored(
                Point::new(Px(4.0), Px(8.0)),
                Color::from_srgb_hex_rgb(0x00_00_ff),
            ),
        ];
        list.add_triangle_mesh(vertices, [0, 1, 2]);
        list.add_image_triangle_mesh_with_options(
            ImageId::default(),
            vertices.map(|vertex| {
                DebugDrawVertex::new(vertex.position, UvPoint::new(0.5, 0.25), vertex.color)
            }),
            [0, 1, 2],
            DebugDrawImageMeshOptions {
                sampling: ImageSamplingHint::Nearest,
                opacity: 0.5,
            },
        );

        assert_eq!(list.command_count(), 2);
        assert!(matches!(
            list.commands[0],
            DebugDrawCommand::TriangleMesh { .. }
        ));
        let DebugDrawCommand::ImageTriangleMesh { options, .. } = &list.commands[1] else {
            panic!("expected image triangle mesh command");
        };
        assert_eq!(options.sampling, ImageSamplingHint::Nearest);
        assert_eq!(options.opacity, 0.5);
    }

    #[test]
    fn debug_draw_list_reports_command_summaries_in_merge_order() {
        let mut list = ImUiDebugDrawList::default();
        let vertices = [
            DebugDrawVertex::colored(
                Point::new(Px(0.0), Px(0.0)),
                Color::from_srgb_hex_rgb(0xff_00_00),
            ),
            DebugDrawVertex::colored(
                Point::new(Px(8.0), Px(0.0)),
                Color::from_srgb_hex_rgb(0x00_ff_00),
            ),
            DebugDrawVertex::colored(
                Point::new(Px(4.0), Px(8.0)),
                Color::from_srgb_hex_rgb(0x00_00_ff),
            ),
        ];

        list.channels_split(3);
        list.add_rect_filled(
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(4.0), Px(4.0))),
            Color::from_srgb_hex_rgb(0xff_ff_ff),
        );
        list.channels_set_current(2);
        list.add_line(
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(8.0), Px(8.0)),
            Color::from_srgb_hex_rgb(0xff_00_00),
            Px(1.0),
        );
        list.channels_set_current(1);
        list.add_image_triangle_mesh(ImageId::default(), vertices, [0, 1, 2]);

        let summaries = list.command_summaries();
        assert_eq!(summaries.len(), 3);
        assert_eq!(
            summaries
                .iter()
                .map(|summary| (summary.channel, summary.kind))
                .collect::<Vec<_>>(),
            vec![
                (Some(0), DebugDrawCommandKind::RectFilled),
                (Some(1), DebugDrawCommandKind::ImageTriangleMesh),
                (Some(2), DebugDrawCommandKind::Line),
            ]
        );
        assert_eq!(summaries[1].image, Some(ImageId::default()));
        assert_eq!(summaries[1].vertex_count, 3);
        assert_eq!(summaries[1].index_count, 3);
        assert_eq!(summaries[1].triangle_count, 1);
    }

    #[test]
    fn debug_draw_list_summary_counts_visible_command_classes() {
        let mut list = ImUiDebugDrawList::default();
        list.push_clip_rect(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(16.0), Px(16.0)),
        ));
        list.add_image(
            Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(8.0), Px(8.0))),
            ImageId::default(),
        );
        list.add_svg_image(
            Rect::new(Point::new(Px(2.0), Px(2.0)), Size::new(Px(8.0), Px(8.0))),
            SvgSource::Static(b"<svg/>"),
        );
        list.add_rect_filled_multi_color(
            Rect::new(Point::new(Px(3.0), Px(3.0)), Size::new(Px(10.0), Px(10.0))),
            Color::from_srgb_hex_rgb(0xff_00_00),
            Color::from_srgb_hex_rgb(0x00_ff_00),
            Color::from_srgb_hex_rgb(0x00_00_ff),
            Color::from_srgb_hex_rgb(0xff_ff_ff),
        );
        list.add_text(
            Point::new(Px(4.0), Px(4.0)),
            "debug",
            Color::from_srgb_hex_rgb(0xff_ff_ff),
            Px(12.0),
        );
        list.pop_clip_rect();

        let summary = list.list_summary();
        assert_eq!(summary.command_count, 6);
        assert_eq!(summary.clip_push_count, 1);
        assert_eq!(summary.clip_pop_count, 1);
        assert_eq!(summary.image_command_count, 1);
        assert_eq!(summary.svg_command_count, 1);
        assert_eq!(summary.text_command_count, 1);
        assert_eq!(summary.vertex_count, 4);
        assert_eq!(summary.index_count, 6);
        assert_eq!(summary.triangle_count, 2);
    }

    #[test]
    fn debug_draw_command_summaries_track_effective_clip_stack() {
        let outer = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(32.0)));
        let inner = Rect::new(Point::new(Px(4.0), Px(4.0)), Size::new(Px(12.0), Px(12.0)));

        let mut list = ImUiDebugDrawList::default();
        list.add_line(
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(8.0), Px(8.0)),
            Color::from_srgb_hex_rgb(0xff_ff_ff),
            Px(1.0),
        );
        list.push_clip_rect(outer);
        list.add_rect_filled(
            Rect::new(Point::new(Px(2.0), Px(2.0)), Size::new(Px(6.0), Px(6.0))),
            Color::from_srgb_hex_rgb(0xff_00_00),
        );
        list.push_clip_rect(inner);
        list.add_text(
            Point::new(Px(6.0), Px(6.0)),
            "clipped",
            Color::from_srgb_hex_rgb(0xff_ff_ff),
            Px(12.0),
        );
        list.pop_clip_rect();
        list.add_image(
            Rect::new(Point::new(Px(8.0), Px(8.0)), Size::new(Px(6.0), Px(6.0))),
            ImageId::default(),
        );
        list.pop_clip_rect();

        let summaries = list.command_summaries();
        assert_eq!(summaries[0].clip_rect, None);
        assert_eq!(summaries[0].clip_depth, 0);
        assert_eq!(summaries[1].clip_rect, Some(outer));
        assert_eq!(summaries[1].clip_depth, 1);
        assert_eq!(summaries[2].clip_rect, Some(outer));
        assert_eq!(summaries[2].clip_depth, 1);
        assert_eq!(summaries[3].clip_rect, Some(inner));
        assert_eq!(summaries[3].clip_depth, 2);
        assert_eq!(summaries[4].clip_rect, Some(inner));
        assert_eq!(summaries[4].clip_depth, 2);
        assert_eq!(summaries[5].clip_rect, Some(outer));
        assert_eq!(summaries[5].clip_depth, 1);
        assert_eq!(summaries[6].clip_rect, Some(outer));
        assert_eq!(summaries[6].clip_depth, 1);
        assert_eq!(summaries[7].clip_rect, None);
        assert_eq!(summaries[7].clip_depth, 0);

        let summary = list.list_summary();
        assert_eq!(summary.max_clip_depth, 2);
        assert_eq!(summary.final_clip_depth, 0);
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
    fn debug_draw_channels_merge_in_channel_order() {
        let mut list = ImUiDebugDrawList::default();
        list.add_line(
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(1.0), Px(1.0)),
            Color::from_srgb_hex_rgb(0xff_00_00),
            Px(1.0),
        );

        list.channels_split(3);
        list.channels_set_current(2);
        list.add_text(
            Point::new(Px(8.0), Px(8.0)),
            "foreground",
            Color::from_srgb_hex_rgb(0xff_ff_ff),
            Px(12.0),
        );
        list.channels_set_current(1);
        list.add_rect_filled(
            Rect::new(Point::new(Px(2.0), Px(2.0)), Size::new(Px(4.0), Px(4.0))),
            Color::from_srgb_hex_rgb(0x00_ff_00),
        );
        list.channels_set_current(0);
        list.add_circle_filled(
            Point::new(Px(6.0), Px(6.0)),
            Px(2.0),
            Color::from_srgb_hex_rgb(0x00_00_ff),
        );

        assert_eq!(list.command_count(), 4);
        list.channels_merge();

        assert_eq!(list.command_count(), 4);
        assert!(matches!(list.commands[0], DebugDrawCommand::Line { .. }));
        assert!(matches!(
            list.commands[1],
            DebugDrawCommand::CircleFilled { .. }
        ));
        assert!(matches!(
            list.commands[2],
            DebugDrawCommand::RectFilled { .. }
        ));
        assert!(matches!(list.commands[3], DebugDrawCommand::Text { .. }));
    }

    #[test]
    fn debug_draw_channels_ignore_invalid_channel_switches() {
        let mut list = ImUiDebugDrawList::default();
        list.channels_split(2);
        list.channels_set_current(4);
        list.add_text(
            Point::new(Px(0.0), Px(0.0)),
            "still-channel-zero",
            Color::from_srgb_hex_rgb(0xff_ff_ff),
            Px(12.0),
        );
        list.channels_merge();

        assert_eq!(list.command_count(), 1);
        assert!(matches!(list.commands[0], DebugDrawCommand::Text { .. }));
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
        list.add_image_quad(
            image,
            [
                Point::new(Px(0.0), Px(0.0)),
                Point::new(Px(24.0), Px(0.0)),
                Point::new(Px(24.0), Px(16.0)),
                Point::new(Px(0.0), Px(16.0)),
            ],
            [
                UvPoint { u: 0.0, v: 0.0 },
                UvPoint { u: 1.0, v: 0.0 },
                UvPoint { u: 1.0, v: 1.0 },
                UvPoint { u: 0.0, v: 1.0 },
            ],
        );
        list.add_image_rounded(
            rect,
            image,
            Px(4.0),
            DebugDrawRoundCorners::TOP_LEFT | DebugDrawRoundCorners::BOTTOM_RIGHT,
        );
        list.add_image_region_rounded(
            rect,
            image,
            UvRect::FULL,
            image_options,
            Px(4.0),
            DebugDrawRoundCorners::ALL,
        );
        list.add_svg_image_with_options(rect, SvgSource::Static(b"<svg/>"), svg_options);
        list.add_svg_mask_icon_with_options(
            rect,
            SvgSource::Static(b"<svg/>"),
            Color::from_srgb_hex_rgb(0xff_ff_ff),
            svg_options,
        );

        assert_eq!(list.command_count(), 7);
        assert!(matches!(list.commands[0], DebugDrawCommand::Image { .. }));
        assert!(matches!(
            list.commands[1],
            DebugDrawCommand::ImageRegion { .. }
        ));
        assert!(matches!(
            list.commands[2],
            DebugDrawCommand::ImageQuad { .. }
        ));
        assert!(matches!(
            list.commands[3],
            DebugDrawCommand::ImageRounded { .. }
        ));
        assert!(matches!(
            list.commands[4],
            DebugDrawCommand::ImageRegionRounded { .. }
        ));
        assert!(matches!(
            list.commands[5],
            DebugDrawCommand::SvgImage { .. }
        ));
        assert!(matches!(
            list.commands[6],
            DebugDrawCommand::SvgMaskIcon { .. }
        ));
    }

    #[test]
    fn debug_draw_list_records_concave_poly_fill_command() {
        let mut list = ImUiDebugDrawList::default();
        let points = [
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(18.0), Px(0.0)),
            Point::new(Px(10.0), Px(8.0)),
            Point::new(Px(18.0), Px(16.0)),
            Point::new(Px(0.0), Px(16.0)),
        ];

        list.add_concave_poly_filled(points, Color::from_srgb_hex_rgb(0xff_ff_ff));

        let DebugDrawCommand::ConcavePolyFilled {
            points: recorded, ..
        } = &list.commands[0]
        else {
            panic!("concave polygon fill should record a dedicated command");
        };
        assert_eq!(&**recorded, &points);
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
    fn debug_draw_path_builder_records_concave_fill_command() {
        let mut list = ImUiDebugDrawList::default();
        let points = [
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(18.0), Px(0.0)),
            Point::new(Px(10.0), Px(8.0)),
            Point::new(Px(18.0), Px(16.0)),
            Point::new(Px(0.0), Px(16.0)),
        ];

        list.path(|path| {
            path.line_to(points[0])
                .line_to(points[1])
                .line_to(points[2])
                .line_to(points[3])
                .line_to(points[4]);
            path.fill_concave(Color::from_srgb_hex_rgb(0xff_ff_ff));
            assert!(path.is_empty());

            path.line_to(points[0]).line_to(points[1]);
            path.fill_concave(Color::from_srgb_hex_rgb(0xff_ff_ff));
            assert!(path.is_empty());
        });

        assert_eq!(list.command_count(), 1);
        let DebugDrawCommand::ConcavePolyFilled {
            points: recorded, ..
        } = &list.commands[0]
        else {
            panic!("path concave fill should record a dedicated command");
        };
        assert_eq!(&**recorded, &points);
    }

    #[test]
    fn debug_draw_path_builder_appends_rect_points() {
        let mut list = ImUiDebugDrawList::default();
        let rect = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(20.0), Px(10.0)),
        );

        list.path(|path| {
            path.rect(rect);
            assert_eq!(path.point_count(), 4);
            path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), true);
        });

        let DebugDrawCommand::Polyline { points, closed, .. } = &list.commands[0] else {
            panic!("path rect helper should record a closed polyline command");
        };
        assert!(*closed);
        assert_eq!(points.len(), 4);
        assert_eq!(
            &**points,
            &[
                Point::new(Px(10.0), Px(20.0)),
                Point::new(Px(30.0), Px(20.0)),
                Point::new(Px(30.0), Px(30.0)),
                Point::new(Px(10.0), Px(30.0)),
            ]
        );
    }

    #[test]
    fn debug_draw_path_builder_appends_rounded_rect_corner_samples() {
        let mut list = ImUiDebugDrawList::default();
        let rect = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(20.0), Px(10.0)),
        );

        list.path(|path| {
            path.rect_with_rounding(
                rect,
                Px(4.0),
                DebugDrawRoundCorners::TOP_LEFT | DebugDrawRoundCorners::BOTTOM_RIGHT,
            );
            assert_eq!(path.point_count(), 10);
            path.fill_convex(Color::from_srgb_hex_rgb(0xff_ff_ff));
        });

        let DebugDrawCommand::ConvexPolyFilled { points, .. } = &list.commands[0] else {
            panic!("rounded path rect helper should record sampled convex fill points");
        };
        assert_eq!(points.len(), 10);
        assert_point_near(points[0], Point::new(Px(10.0), Px(24.0)));
        assert_point_near(points[3], Point::new(Px(14.0), Px(20.0)));
        assert_point_near(points[4], Point::new(Px(30.0), Px(20.0)));
        assert_point_near(points[5], Point::new(Px(30.0), Px(26.0)));
        assert_point_near(points[8], Point::new(Px(26.0), Px(30.0)));
        assert_point_near(points[9], Point::new(Px(10.0), Px(30.0)));
    }

    #[test]
    fn debug_draw_path_builder_rect_rounding_clamps_and_handles_invalid_inputs() {
        let mut list = ImUiDebugDrawList::default();
        let rect = Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(12.0), Px(8.0)));

        list.path(|path| {
            path.rect_with_rounding(rect, Px(50.0), DebugDrawRoundCorners::ALL);
            assert_eq!(path.point_count(), 16);
            assert_point_near(path.points[0], Point::new(Px(10.0), Px(23.0)));
            path.clear();

            path.rect_with_rounding(rect, Px(4.0), DebugDrawRoundCorners::NONE);
            assert_eq!(path.point_count(), 4);
            assert_eq!(path.points[0], Point::new(Px(10.0), Px(20.0)));
            assert_eq!(path.points[2], Point::new(Px(22.0), Px(28.0)));
            path.clear();

            path.rect_with_rounding(
                Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(0.0), Px(8.0))),
                Px(4.0),
                DebugDrawRoundCorners::ALL,
            );
            path.rect_with_rounding(rect, Px(f32::NAN), DebugDrawRoundCorners::ALL);
            assert!(path.is_empty());
        });

        assert_eq!(list.command_count(), 0);
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
    fn rounded_image_helpers_follow_imgui_path_rect_corner_rules() {
        let rect = Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(12.0), Px(8.0)));

        let all = rounded_rect_corner_radii(rect, Px(50.0), DebugDrawRoundCorners::ALL);
        assert_eq!(all, Corners::all(Px(3.0)));
        assert!(corner_radii_are_visible(all));

        let diagonal = rounded_rect_corner_radii(
            rect,
            Px(50.0),
            DebugDrawRoundCorners::TOP_LEFT | DebugDrawRoundCorners::BOTTOM_RIGHT,
        );
        assert_eq!(diagonal.top_left, Px(7.0));
        assert_eq!(diagonal.top_right, Px(0.0));
        assert_eq!(diagonal.bottom_right, Px(7.0));
        assert_eq!(diagonal.bottom_left, Px(0.0));

        assert_eq!(
            rounded_rect_corner_radii(rect, Px(4.0), DebugDrawRoundCorners::NONE),
            Corners::all(Px(0.0))
        );
        assert_eq!(
            rounded_rect_corner_radii(rect, Px(f32::NAN), DebugDrawRoundCorners::ALL),
            Corners::all(Px(0.0))
        );
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
    fn concave_poly_fill_path_requires_three_points_and_closes() {
        assert!(concave_poly_fill_path(&[Point::new(Px(0.0), Px(0.0))]).is_none());
        assert!(
            concave_poly_fill_path(&[Point::new(Px(0.0), Px(0.0)), Point::new(Px(10.0), Px(0.0)),])
                .is_none()
        );

        let path = concave_poly_fill_path(&[
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(18.0), Px(0.0)),
            Point::new(Px(10.0), Px(8.0)),
            Point::new(Px(18.0), Px(16.0)),
            Point::new(Px(0.0), Px(16.0)),
        ])
        .unwrap();

        assert_eq!(
            path,
            vec![
                PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
                PathCommand::LineTo(Point::new(Px(18.0), Px(0.0))),
                PathCommand::LineTo(Point::new(Px(10.0), Px(8.0))),
                PathCommand::LineTo(Point::new(Px(18.0), Px(16.0))),
                PathCommand::LineTo(Point::new(Px(0.0), Px(16.0))),
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

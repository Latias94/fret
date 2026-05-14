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
        self.response.rect()
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
        prepaint: false,
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
mod tests;

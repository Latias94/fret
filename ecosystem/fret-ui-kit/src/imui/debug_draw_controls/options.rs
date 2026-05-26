use std::sync::Arc;

use fret_core::scene::{DashPatternV1, ImageSamplingHint};
use fret_core::{
    Color, PathStyle, Point, Px, SceneMeshVertex, StrokeCapV1, StrokeJoinV1, StrokeStyle,
    StrokeStyleV2, SvgFit, UvPoint, ViewportFit,
};
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

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

    pub(in crate::imui::debug_draw_controls) fn is_visible(self) -> bool {
        self.width.0 > 0.0
    }

    pub(in crate::imui::debug_draw_controls) fn path_style(self) -> PathStyle {
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

    pub(in crate::imui::debug_draw_controls) fn scene_vertex(self) -> SceneMeshVertex {
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

//! Immediate-mode debug draw helper backed by declarative `Canvas`.

#[cfg(test)]
use fret_core::scene::{DashPatternV1, ImageSamplingHint};
#[cfg(test)]
use fret_core::{
    Color, PathStyle, Point, Px, Rect, StrokeCapV1, StrokeJoinV1, StrokeStyle, SvgFit, UvPoint,
    ViewportFit,
};

use super::ResponseExt;

mod commands;
mod draw_list;
mod draw_list_shapes;
mod element;
mod facade;
mod geometry;
mod options;
mod paint;
mod paint_helpers;
mod paint_shapes;
mod path_builder;
mod paths;
mod response;
mod summaries;

use commands::{
    DebugDrawClipCommand, DebugDrawCommand, DebugDrawLinearCommand, DebugDrawMediaCommand,
    DebugDrawMeshCommand, DebugDrawRoundCommand,
};
pub use draw_list::ImUiDebugDrawList;
pub(super) use facade::debug_draw_with_options;
pub use options::{
    DebugDrawImageMeshOptions, DebugDrawImageOptions, DebugDrawImageQuadOptions,
    DebugDrawInteractionOptions, DebugDrawOptions, DebugDrawRoundCorners, DebugDrawStrokeStyle,
    DebugDrawSvgOptions, DebugDrawVertex,
};
pub use path_builder::ImUiDebugDrawPath;
pub use response::DebugDrawResponse;
pub use summaries::{DebugDrawCommandKind, DebugDrawCommandSummary, DebugDrawListSummary};

const DEFAULT_ELLIPSE_SEGMENTS: usize = 32;
const DEFAULT_PATH_ARC_SEGMENTS: usize = 12;
const DEFAULT_PATH_BEZIER_SEGMENTS: usize = 12;
const DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS: usize = 32;

#[cfg(test)]
mod tests;

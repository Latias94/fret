//! Immediate-mode debug draw helper backed by declarative `Canvas`.

use std::hash::Hash;
use std::sync::Arc;

#[cfg(test)]
use fret_core::scene::{DashPatternV1, ImageSamplingHint};
#[cfg(test)]
use fret_core::{
    Color, PathStyle, Point, Px, Rect, StrokeCapV1, StrokeJoinV1, StrokeStyle, SvgFit, UvPoint,
    ViewportFit,
};
use fret_ui::UiHost;

use super::{ResponseExt, UiWriterImUiFacadeExt};

mod commands;
mod draw_list;
mod draw_list_shapes;
mod element;
mod geometry;
mod options;
mod paint;
mod paint_helpers;
mod paint_shapes;
mod path_builder;
mod paths;
mod response;
mod summaries;

use commands::DebugDrawCommand;
use element::debug_draw_element;
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
    DebugDrawResponse::new(response, list_summary, command_summaries)
}

#[cfg(test)]
mod tests;

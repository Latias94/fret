//! Paint-root pass scene adapter contract.
//!
//! This module keeps immediate pass static scene routing behind named operations. Concrete retained
//! scene/service sinks live next to the retained paint root binding.

use fret_ui::UiHost;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) trait PaintRootPassSceneCx<H: UiHost> {
    fn paint_root_pass_groups_static<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        groups: &[(fret_core::Rect, std::sync::Arc<str>, bool)],
        zoom: f32,
    ) where
        M: NodeGraphCanvasMiddleware;

    fn paint_root_pass_groups_selected_overlay<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        groups: &[(fret_core::Rect, std::sync::Arc<str>, bool)],
        zoom: f32,
    ) where
        M: NodeGraphCanvasMiddleware;

    fn paint_root_pass_nodes_static<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        render: &RenderData,
        zoom: f32,
    ) where
        M: NodeGraphCanvasMiddleware;
}

pub(super) fn paint_root_pass_groups_static<H, M>(
    cx: &mut impl PaintRootPassSceneCx<H>,
    canvas: &mut NodeGraphCanvasWith<M>,
    groups: &[(fret_core::Rect, std::sync::Arc<str>, bool)],
    zoom: f32,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    cx.paint_root_pass_groups_static(canvas, groups, zoom);
}

pub(super) fn paint_root_pass_groups_selected_overlay<H, M>(
    cx: &mut impl PaintRootPassSceneCx<H>,
    canvas: &mut NodeGraphCanvasWith<M>,
    groups: &[(fret_core::Rect, std::sync::Arc<str>, bool)],
    zoom: f32,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    cx.paint_root_pass_groups_selected_overlay(canvas, groups, zoom);
}

pub(super) fn paint_root_pass_nodes_static<H, M>(
    cx: &mut impl PaintRootPassSceneCx<H>,
    canvas: &mut NodeGraphCanvasWith<M>,
    render: &RenderData,
    zoom: f32,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    cx.paint_root_pass_nodes_static(canvas, render, zoom);
}

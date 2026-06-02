// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::layout::split_tab_bar;
use super::prelude_core::*;

mod drag_ghost;
mod drop_hints;
mod drop_overlay;
mod floating_chrome;
mod split_handle;
mod tab_chrome;
mod tab_detail;
mod viewport_surface;

pub(super) use drag_ghost::{DockDragGhostPaint, paint_drag_payload_ghost};
pub(super) use drop_hints::paint_drop_hints;
pub(super) use drop_overlay::{
    ComplexDropOverlayPaintInput, complex_drop_overlay_paint_inputs,
    paint_complex_drop_overlay_inputs, paint_tab_insert_preview_title,
};
pub(super) use floating_chrome::{FloatingChromePaintInput, paint_floating_chrome_inputs};
pub(super) use split_handle::{
    SplitHandlePaintInput, paint_split_handle_inputs, split_handle_paint_inputs,
};
pub(super) use tab_chrome::{
    TabChromePaintInput, paint_tab_chrome_inputs, tab_chrome_paint_inputs,
};
pub(super) use tab_detail::{
    TabDetailPaintInput, paint_tab_detail_inputs, tab_detail_paint_inputs,
};
pub(super) use viewport_surface::{
    ViewportSurfacePaintInput, paint_viewport_surface_inputs, viewport_surface_paint_inputs,
};

pub(super) fn paint_basic_drop_overlay(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    window: fret_core::AppWindowId,
    bounds: Rect,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    drag_source_tabs: Option<DockNodeId>,
    scene: &mut Scene,
) {
    let Some(target) = target else {
        return;
    };

    let primary = theme.color_token("primary");
    let radius_sm = theme.metric_token("metric.radius.sm");
    let radius_md = theme.metric_token("metric.radius.md");
    let primary_alpha = |alpha: f32| Color {
        a: alpha,
        ..primary
    };

    let overlay_float_bg = theme
        .color_by_key("component.docking.drop_overlay.float.bg")
        .unwrap_or_else(|| primary_alpha(0.10));
    let overlay_float_border = theme
        .color_by_key("component.docking.drop_overlay.float.border")
        .unwrap_or_else(|| primary_alpha(0.85));
    let overlay_empty_bg = theme
        .color_by_key("component.docking.drop_overlay.empty.bg")
        .unwrap_or_else(|| primary_alpha(0.08));
    let overlay_empty_border = theme
        .color_by_key("component.docking.drop_overlay.empty.border")
        .unwrap_or_else(|| primary_alpha(0.75));
    let overlay_center_content_bg = theme
        .color_by_key("component.docking.drop_overlay.center.content.bg")
        .unwrap_or_else(|| primary_alpha(0.12));
    let overlay_center_content_border = theme
        .color_by_key("component.docking.drop_overlay.center.content.border")
        .unwrap_or_else(|| primary_alpha(0.65));
    let overlay_center_tab_bar_bg = theme
        .color_by_key("component.docking.drop_overlay.center.tab_bar.bg")
        .unwrap_or_else(|| primary_alpha(0.14));
    let overlay_center_tab_bar_border = theme
        .color_by_key("component.docking.drop_overlay.center.tab_bar.border")
        .unwrap_or_else(|| primary_alpha(0.45));

    match target {
        DockDropTarget::Float { window: w } => {
            if w != window {
                return;
            }
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: bounds,
                background: fret_core::Paint::Solid(overlay_float_bg).into(),
                border: Edges::all(Px(3.0)),
                border_paint: fret_core::Paint::Solid(overlay_float_border).into(),
                corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
            });
        }
        DockDropTarget::EmptyDockSpace { window: w } => {
            if w != window {
                return;
            }
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: bounds,
                background: fret_core::Paint::Solid(overlay_empty_bg).into(),
                border: Edges::all(Px(3.0)),
                border_paint: fret_core::Paint::Solid(overlay_empty_border).into(),
                corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
            });
        }
        DockDropTarget::Dock(target) if target.zone == DropZone::Center => {
            let Some(rect) = layout.get(&target.tabs).copied() else {
                return;
            };
            let same_tabs_reorder = drag_source_tabs.is_some_and(|src| src == target.tabs);
            if same_tabs_reorder {
                return;
            }
            let (tab_bar, content) = split_tab_bar(rect);
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(9_985),
                rect: content,
                background: fret_core::Paint::Solid(overlay_center_content_bg).into(),
                border: Edges::all(Px(2.0)),
                border_paint: fret_core::Paint::Solid(overlay_center_content_border).into(),
                corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
            });
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(9_990),
                rect: tab_bar,
                background: fret_core::Paint::Solid(overlay_center_tab_bar_bg).into(),
                border: Edges::all(Px(1.0)),
                border_paint: fret_core::Paint::Solid(overlay_center_tab_bar_border).into(),
                corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
            });
        }
        DockDropTarget::Dock(_) => {}
    }
}

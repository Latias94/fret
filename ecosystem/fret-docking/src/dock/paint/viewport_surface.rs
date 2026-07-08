use std::collections::HashMap;

use fret_core::{
    Color, DockNode, DockNodeId, Edges, PanelKey, Scene, SceneOp, ViewportMapping,
    geometry::{Px, Rect},
};

use super::super::layout::split_tab_bar;
use super::super::manager::DockManager;
use super::super::{DockPanel, DockViewportLayout, DockViewportOverlayHooks, ViewportPanel};

#[derive(Debug, Clone)]
pub(in crate::dock) struct ViewportSurfacePaintInput {
    panel: PanelKey,
    panel_color: Color,
    viewport: ViewportPanel,
    layout: DockViewportLayout,
}

fn viewport_surface_paint_input(
    dock: &DockManager,
    window: fret_core::AppWindowId,
    panel_key: &PanelKey,
    panel: &DockPanel,
    content: Rect,
) -> Option<ViewportSurfacePaintInput> {
    let viewport = panel.viewport?;
    let layout = dock
        .viewport_layout(window, viewport.target)
        .filter(|layout| layout.content_rect == content)
        .unwrap_or_else(|| {
            let mapping = ViewportMapping {
                content_rect: content,
                target_px_size: viewport.target_px_size,
                fit: viewport.fit,
            };
            DockViewportLayout {
                content_rect: content,
                mapping,
                draw_rect: mapping.map().draw_rect,
            }
        });

    Some(ViewportSurfacePaintInput {
        panel: panel_key.clone(),
        panel_color: panel.color,
        viewport,
        layout,
    })
}

pub(in crate::dock) fn viewport_surface_paint_inputs(
    dock: &DockManager,
    window: fret_core::AppWindowId,
    layout: &HashMap<DockNodeId, Rect>,
) -> Vec<ViewportSurfacePaintInput> {
    let mut inputs = Vec::new();
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = dock.workspace.graph.node(node_id) else {
            continue;
        };
        let Some(panel_key) = tabs.get(*active) else {
            continue;
        };
        let Some(panel) = dock.panel(panel_key) else {
            continue;
        };
        let (_tab_bar, content) = split_tab_bar(rect);
        if let Some(input) = viewport_surface_paint_input(dock, window, panel_key, panel, content) {
            inputs.push(input);
        }
    }
    inputs
}

fn paint_viewport_surface_input(
    theme: fret_ui::ThemeSnapshot,
    window: fret_core::AppWindowId,
    input: &ViewportSurfacePaintInput,
    overlay_hooks: Option<&dyn DockViewportOverlayHooks>,
    scene: &mut Scene,
) {
    let content = input.layout.content_rect;
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(3),
        rect: content,
        background: fret_core::Paint::Solid(input.panel_color).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: fret_core::Corners::all(theme.metric_token("metric.radius.sm")),
    });

    scene.push(SceneOp::PushClipRect { rect: content });
    scene.push(SceneOp::ViewportSurface {
        order: fret_core::DrawOrder(4),
        rect: input.layout.draw_rect,
        target: input.viewport.target,
        opacity: 1.0,
    });
    if let Some(hooks) = overlay_hooks {
        hooks.paint_with_layout(
            theme,
            window,
            &input.panel,
            input.viewport,
            input.layout,
            scene,
        );
    }
    scene.push(SceneOp::PopClip);
}

pub(in crate::dock) fn paint_viewport_surface_inputs(
    theme: fret_ui::ThemeSnapshot,
    window: fret_core::AppWindowId,
    inputs: &[ViewportSurfacePaintInput],
    overlay_hooks: Option<&dyn DockViewportOverlayHooks>,
    scene: &mut Scene,
) {
    for input in inputs {
        paint_viewport_surface_input(theme.clone(), window, input, overlay_hooks, scene);
    }
}

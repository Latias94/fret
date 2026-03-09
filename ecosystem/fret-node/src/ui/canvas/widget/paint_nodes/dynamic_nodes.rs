use crate::ui::canvas::widget::*;
use crate::ui::{NodeChromeHint, NodeGraphSkinRef};

use super::dynamic_from_geometry::paint_node_ring;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_dynamic_selected_nodes<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        skin: Option<&NodeGraphSkinRef>,
        focused_node: Option<GraphNodeId>,
        corner: Px,
        title_h: f32,
        zoom: f32,
    ) {
        for node in snapshot.selected_nodes.iter().copied() {
            let Some(node_geom) = geom.nodes.get(&node) else {
                continue;
            };
            let rect = node_geom.rect;
            let hint = self.resolve_dynamic_node_hint(cx.app, skin, node, true, focused_node);

            if let Some(ring) = hint.ring_selected {
                paint_node_ring(cx.scene, rect, corner, ring, zoom);
            }
            if focused_node == Some(node)
                && let Some(ring) = hint.ring_focused
            {
                paint_node_ring(cx.scene, rect, corner, ring, zoom);
            }

            self.paint_dynamic_selected_node_body(cx.scene, rect, hint, corner, title_h, zoom);
            self.paint_dynamic_selected_node_resize_handles(cx, node, rect, zoom);
        }
    }

    pub(super) fn paint_dynamic_focused_node_ring<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        skin: Option<&NodeGraphSkinRef>,
        focused_node: Option<GraphNodeId>,
        corner: Px,
        zoom: f32,
    ) {
        let Some(node) = focused_node else {
            return;
        };
        if snapshot
            .selected_nodes
            .iter()
            .any(|selected| *selected == node)
        {
            return;
        }
        let Some(node_geom) = geom.nodes.get(&node) else {
            return;
        };

        let hint = self.resolve_dynamic_node_hint(cx.app, skin, node, false, focused_node);
        if let Some(ring) = hint.ring_focused {
            paint_node_ring(cx.scene, node_geom.rect, corner, ring, zoom);
        }
    }

    fn resolve_dynamic_node_hint<H: UiHost>(
        &self,
        app: &H,
        skin: Option<&NodeGraphSkinRef>,
        node: GraphNodeId,
        is_selected: bool,
        focused_node: Option<GraphNodeId>,
    ) -> NodeChromeHint {
        if let Some(skin) = skin {
            self.graph
                .read_ref(app, |graph| {
                    skin.node_chrome_hint_with_state(
                        graph,
                        node,
                        &self.style,
                        is_selected,
                        focused_node == Some(node),
                    )
                })
                .ok()
                .unwrap_or_default()
        } else {
            NodeChromeHint::default()
        }
    }

    fn paint_dynamic_selected_node_body(
        &mut self,
        scene: &mut fret_core::Scene,
        rect: Rect,
        hint: NodeChromeHint,
        corner: Px,
        title_h: f32,
        zoom: f32,
    ) {
        let background = hint.background.unwrap_or(self.style.paint.node_background);
        let border_color = hint
            .border_selected
            .or(hint.border)
            .unwrap_or(self.style.paint.node_border_selected);

        scene.push(SceneOp::Quad {
            order: DrawOrder(3),
            rect,
            background: fret_core::Paint::Solid(background).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(corner),
        });

        if let Some(color) = hint.header_background {
            scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect: Rect::new(
                    rect.origin,
                    Size::new(rect.size.width, Px(title_h.min(rect.size.height.0))),
                ),
                background: fret_core::Paint::Solid(color).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: Corners {
                    top_left: corner,
                    top_right: corner,
                    bottom_right: Px(0.0),
                    bottom_left: Px(0.0),
                },
            });
        }

        scene.push(SceneOp::Quad {
            order: DrawOrder(3),
            rect,
            background: fret_core::Paint::TRANSPARENT.into(),
            border: Edges::all(Px(1.0 / zoom)),
            border_paint: fret_core::Paint::Solid(border_color).into(),
            corner_radii: Corners::all(corner),
        });
    }

    fn paint_dynamic_selected_node_resize_handles<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        node: GraphNodeId,
        rect: Rect,
        zoom: f32,
    ) {
        let show_resize_handle = self
            .interaction
            .node_resize
            .as_ref()
            .is_some_and(|resize| resize.node == node)
            || self
                .interaction
                .last_pos
                .is_some_and(|pos| Self::rect_contains(rect, pos));
        if !show_resize_handle {
            return;
        }

        let handles = self
            .graph
            .read_ref(cx.app, |graph| {
                self.presenter.node_resize_handles(graph, node, &self.style)
            })
            .ok()
            .unwrap_or_default();
        for handle in NodeResizeHandle::ALL {
            if !handles.contains(handle) {
                continue;
            }
            let rect = self.node_resize_handle_rect(rect, handle, zoom);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(5),
                rect,
                background: fret_core::Paint::Solid(self.style.paint.resize_handle_background)
                    .into(),
                border: Edges::all(Px(1.0 / zoom)),
                border_paint: fret_core::Paint::Solid(self.style.paint.resize_handle_border).into(),
                corner_radii: Corners::all(Px(2.0 / zoom)),
            });
        }
    }
}

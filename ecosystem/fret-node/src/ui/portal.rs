//! Canvas portal host for embedding `fret-ui` subtrees into a node graph canvas.
//!
//! This is the staged "Canvas Portal" implementation described in ADR 0135:
//! - Stage 1: canvas paints simple labels (current default).
//! - Stage 2: host node header/body subtrees as regular `fret-ui` elements (this module).
//! - Stage 3: per-port-row subtrees and richer semantics (future).

use std::collections::BTreeSet;
use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::declarative::RenderRootContext;
use fret_ui::element::AnyElement;
use fret_ui::elements::{ElementContext, GlobalElementId, bounds_for_element};
use fret_ui::{UiHost, retained_bridge::*};

use crate::core::{CanvasPoint, Graph, NodeId};
use crate::io::NodeGraphViewState;
use crate::ui::measured::MeasuredGeometryStore;
use crate::ui::style::NodeGraphStyle;

use super::canvas::{node_order, node_ports, node_size_default_px};

/// Layout information for a portal-rendered node subtree.
#[derive(Debug, Clone, Copy)]
pub struct NodeGraphPortalNodeLayout {
    /// Node id in the graph model.
    pub node: NodeId,
    /// Node bounds in window coordinates.
    pub node_window: Rect,
    /// Zoom factor for the canvas.
    pub zoom: f32,
}

/// A portal host widget that mounts per-node declarative `fret-ui` subtrees on top of the canvas.
///
/// This widget:
/// - computes node bounds from `Graph + NodeGraphViewState + MeasuredGeometryStore`,
/// - renders a declarative element tree each layout pass,
/// - feeds measured subtree sizes back into `MeasuredGeometryStore` as node size hints.
///
/// The portal subtree is rendered in screen-space (window coordinates) and does not participate in
/// the canvas pan/zoom transform, matching the ADR 0135 "escape hatch" model.
pub struct NodeGraphPortalHost<P> {
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    measured: Arc<MeasuredGeometryStore>,
    style: NodeGraphStyle,
    root_name: String,
    render: P,

    last_published_nodes: Vec<NodeId>,
}

impl<P> NodeGraphPortalHost<P> {
    pub fn new(
        graph: Model<Graph>,
        view_state: Model<NodeGraphViewState>,
        measured: Arc<MeasuredGeometryStore>,
        style: NodeGraphStyle,
        root_name: impl Into<String>,
        render: P,
    ) -> Self {
        Self {
            graph,
            view_state,
            measured,
            style,
            root_name: root_name.into(),
            render,
            last_published_nodes: Vec::new(),
        }
    }

    fn window_node_rect(
        bounds_origin: Point,
        pan: CanvasPoint,
        zoom: f32,
        node_pos: CanvasPoint,
        size_px: (f32, f32),
    ) -> Rect {
        let z = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let x = bounds_origin.x.0 + (node_pos.x + pan.x) * z;
        let y = bounds_origin.y.0 + (node_pos.y + pan.y) * z;
        Rect::new(
            Point::new(Px(x), Px(y)),
            Size::new(Px(size_px.0.max(0.0)), Px(size_px.1.max(0.0))),
        )
    }
}

impl<H: UiHost, P> Widget<H> for NodeGraphPortalHost<P>
where
    P: for<'a> FnMut(
        &mut ElementContext<'a, H>,
        &Graph,
        NodeGraphPortalNodeLayout,
    ) -> Vec<AnyElement>,
{
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        // Child hit-testing is driven by the declarative subtree root, not this wrapper.
        false
    }

    fn semantics_present(&self) -> bool {
        true
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.view_state, Invalidation::Layout);

        let Some(window) = cx.window else {
            cx.tree.set_children(cx.node, Vec::new());
            return cx.bounds.size;
        };

        let (pan, zoom, draw_order) = self
            .view_state
            .read_ref(cx.app, |s| (s.pan, s.zoom, s.draw_order.clone()))
            .ok()
            .unwrap_or_default();
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };

        let graph_snapshot = self
            .graph
            .read_ref(cx.app, |g| g.clone())
            .ok()
            .unwrap_or_default();

        let order = node_order(&graph_snapshot, &draw_order);
        let bounds_origin = cx.bounds.origin;

        let measured = self.measured.clone();
        let style = self.style.clone();
        let render = &mut self.render;
        let mut element_ids: Vec<(NodeId, GlobalElementId)> = Vec::new();

        let root_node =
            RenderRootContext::new(&mut *cx.tree, cx.app, cx.services, window, cx.bounds)
                .render_root(&self.root_name, |ecx| {
                    let mut out: Vec<AnyElement> = Vec::new();

                    for node_id in &order {
                        let Some(node) = graph_snapshot.nodes.get(node_id) else {
                            continue;
                        };

                        let size_px = node
                            .size
                            .map(|s| (s.width, s.height))
                            .or_else(|| measured.node_size_px(*node_id))
                            .unwrap_or_else(|| {
                                let (inputs, outputs) = node_ports(&graph_snapshot, *node_id);
                                node_size_default_px(inputs.len(), outputs.len(), &style)
                            });

                        let node_window =
                            Self::window_node_rect(bounds_origin, pan, zoom, node.pos, size_px);

                        let left = Px((node.pos.x + pan.x) * zoom);
                        let top = Px((node.pos.y + pan.y) * zoom);

                        let layout = NodeGraphPortalNodeLayout {
                            node: *node_id,
                            node_window,
                            zoom,
                        };

                        let children =
                            ecx.keyed(*node_id, |ecx| render(ecx, &graph_snapshot, layout));
                        if children.is_empty() {
                            continue;
                        }

                        let positioned = ecx.container(
                            fret_ui::element::ContainerProps {
                                layout: fret_ui::element::LayoutStyle {
                                    position: fret_ui::element::PositionStyle::Absolute,
                                    inset: fret_ui::element::InsetStyle {
                                        left: Some(left),
                                        top: Some(top),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |ecx| {
                                element_ids.push((*node_id, ecx.root_id()));
                                children
                            },
                        );
                        out.push(positioned);
                    }

                    out
                });

        cx.tree.set_children(cx.node, vec![root_node]);
        cx.layout_in(root_node, cx.bounds);

        // Publish measured node sizes for the node view containers we just rendered.
        //
        // Clamp against the canvas default node size so portal content cannot shrink nodes below
        // their port chrome minimum.
        let mut publish: Vec<(NodeId, (f32, f32))> = Vec::new();
        for (node_id, element) in &element_ids {
            if let Some(bounds) = bounds_for_element(cx.app, window, *element) {
                let (inputs, outputs) = node_ports(&graph_snapshot, *node_id);
                let min = node_size_default_px(inputs.len(), outputs.len(), &self.style);
                let measured_px = (bounds.size.width.0, bounds.size.height.0);
                publish.push((
                    *node_id,
                    (measured_px.0.max(min.0), measured_px.1.max(min.1)),
                ));
            }
        }

        let prev_published = self.last_published_nodes.clone();
        let published = self.measured.update_if_changed(|node_sizes, _anchors| {
            let mut changed = false;
            let mut keep: BTreeSet<NodeId> = BTreeSet::new();

            for (node_id, size) in &publish {
                let cur = node_sizes.get(node_id).copied();
                if cur != Some(*size) {
                    node_sizes.insert(*node_id, *size);
                    changed = true;
                }
                keep.insert(*node_id);
            }

            for old in &prev_published {
                if keep.contains(old) {
                    continue;
                }
                if node_sizes.remove(old).is_some() {
                    changed = true;
                }
            }

            changed
        });

        if published.is_some() {
            self.last_published_nodes = publish.iter().map(|(id, _)| *id).collect();
            cx.app.request_redraw(window);
        }

        cx.bounds.size
    }
}

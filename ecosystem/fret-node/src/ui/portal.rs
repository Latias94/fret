//! Canvas portal host for embedding `fret-ui` subtrees into a node graph canvas.
//!
//! This is the staged "Canvas Portal" implementation described in ADR 0135:
//! - Stage 1: canvas paints simple labels (current default).
//! - Stage 2: host node header/body subtrees as regular `fret-ui` elements (this module).
//! - Stage 3: per-port-row subtrees and richer semantics (future).

use std::collections::BTreeSet;
use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};
use fret_runtime::CommandId;
use fret_runtime::Model;
use fret_ui::declarative::RenderRootContext;
use fret_ui::element::AnyElement;
use fret_ui::elements::{ElementContext, GlobalElementId, bounds_for_element};
use fret_ui::{UiHost, retained_bridge::*};
use uuid::Uuid;

use crate::core::{CanvasPoint, Graph, NodeId};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::ui::edit_queue::NodeGraphEditQueue;
use crate::ui::measured::MeasuredGeometryStore;
use crate::ui::style::NodeGraphStyle;

use super::canvas::{node_order, node_ports, node_size_default_px};

pub const CMD_SUBMIT_TEXT_PREFIX: &str = "fret_node.portal.submit_text:";
pub const CMD_CANCEL_TEXT_PREFIX: &str = "fret_node.portal.cancel_text:";
pub const CMD_STEP_TEXT_PREFIX: &str = "fret_node.portal.step_text:";

pub fn portal_submit_text_command(node: NodeId) -> CommandId {
    CommandId::new(format!("{CMD_SUBMIT_TEXT_PREFIX}{}", node.0))
}

pub fn portal_cancel_text_command(node: NodeId) -> CommandId {
    CommandId::new(format!("{CMD_CANCEL_TEXT_PREFIX}{}", node.0))
}

pub fn portal_step_text_command(node: NodeId, delta: i32) -> CommandId {
    CommandId::new(format!(
        "{CMD_STEP_TEXT_PREFIX}{}:{delta}:{}",
        node.0,
        PortalTextStepMode::Normal.as_str()
    ))
}

pub fn portal_step_text_command_with_mode(
    node: NodeId,
    delta: i32,
    mode: PortalTextStepMode,
) -> CommandId {
    CommandId::new(format!(
        "{CMD_STEP_TEXT_PREFIX}{}:{delta}:{}",
        node.0,
        mode.as_str()
    ))
}

pub fn parse_portal_text_command(command: &CommandId) -> Option<PortalTextCommand> {
    let s = command.as_str();
    if let Some(rest) = s.strip_prefix(CMD_SUBMIT_TEXT_PREFIX) {
        let uuid = Uuid::parse_str(rest).ok()?;
        return Some(PortalTextCommand::Submit { node: NodeId(uuid) });
    }
    if let Some(rest) = s.strip_prefix(CMD_CANCEL_TEXT_PREFIX) {
        let uuid = Uuid::parse_str(rest).ok()?;
        return Some(PortalTextCommand::Cancel { node: NodeId(uuid) });
    }
    if let Some(rest) = s.strip_prefix(CMD_STEP_TEXT_PREFIX) {
        let mut parts = rest.split(':');
        let uuid_str = parts.next()?;
        let delta_str = parts.next()?;
        let mode_str = parts.next().unwrap_or("normal");
        let uuid = Uuid::parse_str(uuid_str).ok()?;
        let delta = delta_str.parse::<i32>().ok()?;
        let mode = PortalTextStepMode::parse(mode_str)?;
        return Some(PortalTextCommand::Step {
            node: NodeId(uuid),
            delta,
            mode,
        });
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalTextStepMode {
    Fine,
    Normal,
    Coarse,
}

impl PortalTextStepMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fine => "fine",
            Self::Normal => "normal",
            Self::Coarse => "coarse",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "fine" => Some(Self::Fine),
            "normal" => Some(Self::Normal),
            "coarse" => Some(Self::Coarse),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalTextCommand {
    Submit {
        node: NodeId,
    },
    Cancel {
        node: NodeId,
    },
    Step {
        node: NodeId,
        delta: i32,
        mode: PortalTextStepMode,
    },
}

#[derive(Debug, Clone)]
pub enum PortalCommandOutcome {
    NotHandled,
    Handled,
    Commit(GraphTransaction),
}

pub trait NodeGraphPortalCommandHandler<H: UiHost> {
    fn handle_portal_command(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        graph: &Graph,
        command: PortalTextCommand,
    ) -> PortalCommandOutcome;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PortalNoopCommandHandler;

impl<H: UiHost> NodeGraphPortalCommandHandler<H> for PortalNoopCommandHandler {
    fn handle_portal_command(
        &mut self,
        _cx: &mut CommandCx<'_, H>,
        _graph: &Graph,
        _command: PortalTextCommand,
    ) -> PortalCommandOutcome {
        PortalCommandOutcome::NotHandled
    }
}

#[derive(Debug, Clone)]
pub struct PortalCommandHandlerChain<A, B> {
    pub first: A,
    pub second: B,
}

impl<A, B> PortalCommandHandlerChain<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<H: UiHost, A, B> NodeGraphPortalCommandHandler<H> for PortalCommandHandlerChain<A, B>
where
    A: NodeGraphPortalCommandHandler<H>,
    B: NodeGraphPortalCommandHandler<H>,
{
    fn handle_portal_command(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        graph: &Graph,
        command: PortalTextCommand,
    ) -> PortalCommandOutcome {
        match self.first.handle_portal_command(cx, graph, command.clone()) {
            PortalCommandOutcome::NotHandled => {
                self.second.handle_portal_command(cx, graph, command)
            }
            other => other,
        }
    }
}

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
pub struct NodeGraphPortalHost<P, C = PortalNoopCommandHandler> {
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    measured: Arc<MeasuredGeometryStore>,
    style: NodeGraphStyle,
    root_name: String,
    render: P,

    edits: Option<Model<NodeGraphEditQueue>>,
    focus_canvas: Option<fret_core::NodeId>,
    command_handler: C,

    last_published_nodes: Vec<NodeId>,
}

impl<P> NodeGraphPortalHost<P, PortalNoopCommandHandler> {
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
            edits: None,
            focus_canvas: None,
            command_handler: PortalNoopCommandHandler,
            last_published_nodes: Vec::new(),
        }
    }
}

impl<P, C> NodeGraphPortalHost<P, C> {
    pub fn with_edit_queue(mut self, edits: Model<NodeGraphEditQueue>) -> Self {
        self.edits = Some(edits);
        self
    }

    pub fn with_canvas_focus_target(mut self, node: fret_core::NodeId) -> Self {
        self.focus_canvas = Some(node);
        self
    }

    pub fn with_command_handler<C2>(self, handler: C2) -> NodeGraphPortalHost<P, C2> {
        NodeGraphPortalHost {
            graph: self.graph,
            view_state: self.view_state,
            measured: self.measured,
            style: self.style,
            root_name: self.root_name,
            render: self.render,
            edits: self.edits,
            focus_canvas: self.focus_canvas,
            command_handler: handler,
            last_published_nodes: self.last_published_nodes,
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

impl<H: UiHost, P, C> Widget<H> for NodeGraphPortalHost<P, C>
where
    P: for<'a> FnMut(
        &mut ElementContext<'a, H>,
        &Graph,
        NodeGraphPortalNodeLayout,
    ) -> Vec<AnyElement>,
    C: NodeGraphPortalCommandHandler<H>,
{
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        // Child hit-testing is driven by the declarative subtree root, not this wrapper.
        false
    }

    fn semantics_present(&self) -> bool {
        true
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        let Some(cmd) = parse_portal_text_command(command) else {
            return false;
        };

        let graph_snapshot = self
            .graph
            .read_ref(cx.app, |g| g.clone())
            .ok()
            .unwrap_or_default();

        let outcome = self
            .command_handler
            .handle_portal_command(cx, &graph_snapshot, cmd);

        match outcome {
            PortalCommandOutcome::NotHandled => false,
            PortalCommandOutcome::Handled => {
                if let Some(canvas) = self.focus_canvas {
                    cx.request_focus(canvas);
                }
                cx.stop_propagation();
                cx.request_redraw();
                true
            }
            PortalCommandOutcome::Commit(tx) => {
                if let Some(edits) = &self.edits {
                    let _ = edits.update(cx.app, |q, _cx| {
                        q.push(tx);
                    });
                }
                if let Some(canvas) = self.focus_canvas {
                    cx.request_focus(canvas);
                }
                cx.stop_propagation();
                cx.request_redraw();
                true
            }
        }
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
                .render_dismissible_root_with_hooks(&self.root_name, |ecx| {
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

                        let positioned = ecx.semantics(
                            fret_ui::element::SemanticsProps {
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
        //
        // IMPORTANT: do not allow portal layout constraints (e.g. shrink-to-fit under absolute
        // positioning near the viewport edge) to cause node sizes to "wobble" during pan/zoom.
        // We treat portal measurements as a growth-only hint unless the node has an explicit size.
        let mut publish: Vec<(NodeId, (f32, f32))> = Vec::new();
        for (node_id, element) in &element_ids {
            let Some(node) = graph_snapshot.nodes.get(node_id) else {
                continue;
            };
            if node.size.is_some() {
                continue;
            }
            if let Some(bounds) = bounds_for_element(cx.app, window, *element) {
                let (inputs, outputs) = node_ports(&graph_snapshot, *node_id);
                let min = node_size_default_px(inputs.len(), outputs.len(), &self.style);
                let measured_px = (bounds.size.width.0, bounds.size.height.0);
                let prev_px = self.measured.node_size_px(*node_id).unwrap_or(min);
                publish.push((
                    *node_id,
                    (
                        measured_px.0.max(min.0).max(prev_px.0),
                        measured_px.1.max(min.1).max(prev_px.1),
                    ),
                ));
            }
        }

        let prev_published = self.last_published_nodes.clone();
        let keep: BTreeSet<NodeId> = publish.iter().map(|(id, _)| *id).collect();
        let remove_nodes: Vec<NodeId> = prev_published
            .iter()
            .copied()
            .filter(|id| !keep.contains(id))
            .collect();

        let published = self.measured.apply_batch_if_changed(
            crate::ui::measured::MeasuredGeometryBatch {
                node_sizes_px: publish.clone(),
                port_anchors_px: Vec::new(),
                remove_nodes,
                remove_ports: Vec::new(),
            },
            crate::ui::measured::MeasuredGeometryApplyOptions::default(),
        );

        if published.is_some() {
            self.last_published_nodes = publish.iter().map(|(id, _)| *id).collect();
            cx.app.request_redraw(window);
        }

        cx.bounds.size
    }
}

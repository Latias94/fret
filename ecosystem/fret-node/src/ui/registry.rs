//! B-layer view registries (ReactFlow-style `nodeTypes`/`edgeTypes`).
//!
//! This module provides a minimal, practical substrate for building editor-grade node graph UIs
//! without forking the canvas:
//!
//! - `NodeGraphNodeTypes` maps `NodeKindKey` -> portal renderer (per-node UI subtree).
//! - Each renderer is `FnMut`, allowing it to hold per-type state (cache, resources), similar to
//!   React component instances.
//!
//! The registry is intentionally UI-focused and lives behind the `fret-ui` feature. The headless
//! runtime equivalents live in `crate::runtime`.

use std::collections::BTreeMap;

use fret_ui::UiHost;
use fret_ui::element::Elements;
use fret_ui::elements::ElementContext;

use crate::core::{Graph, NodeKindKey};
use crate::ui::portal::NodeGraphPortalNodeLayout;

pub type NodeGraphNodeRenderer<H> =
    dyn for<'a> FnMut(&mut ElementContext<'a, H>, &Graph, NodeGraphPortalNodeLayout) -> Elements;

/// ReactFlow-style `nodeTypes` mapping (via the canvas portal escape hatch).
///
/// This registry provides a stable place to hang per-kind node view implementations that can be
/// composed into a `NodeGraphPortalHost`.
pub struct NodeGraphNodeTypes<H: UiHost> {
    node_types: BTreeMap<NodeKindKey, Box<NodeGraphNodeRenderer<H>>>,
    fallback: Option<Box<NodeGraphNodeRenderer<H>>>,
}

impl<H: UiHost> Default for NodeGraphNodeTypes<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> NodeGraphNodeTypes<H> {
    pub fn new() -> Self {
        Self {
            node_types: BTreeMap::new(),
            fallback: None,
        }
    }

    pub fn with_fallback<R>(
        mut self,
        mut renderer: impl for<'a> FnMut(
            &mut ElementContext<'a, H>,
            &Graph,
            NodeGraphPortalNodeLayout,
        ) -> R
        + 'static,
    ) -> Self
    where
        R: Into<Elements> + 'static,
    {
        self.fallback = Some(Box::new(move |cx, graph, layout| {
            renderer(cx, graph, layout).into()
        }));
        self
    }

    pub fn register<R>(
        mut self,
        kind: NodeKindKey,
        mut renderer: impl for<'a> FnMut(
            &mut ElementContext<'a, H>,
            &Graph,
            NodeGraphPortalNodeLayout,
        ) -> R
        + 'static,
    ) -> Self
    where
        R: Into<Elements> + 'static,
    {
        self.node_types.insert(
            kind,
            Box::new(move |cx, graph, layout| renderer(cx, graph, layout).into()),
        );
        self
    }

    /// Converts this registry into a portal renderer closure for [`crate::ui::portal::NodeGraphPortalHost`].
    pub fn into_portal_renderer(
        mut self,
    ) -> impl for<'a> FnMut(&mut ElementContext<'a, H>, &Graph, NodeGraphPortalNodeLayout) -> Elements
    {
        move |ecx, graph, layout| self.render(ecx, graph, layout)
    }

    pub fn render(
        &mut self,
        ecx: &mut ElementContext<'_, H>,
        graph: &Graph,
        layout: NodeGraphPortalNodeLayout,
    ) -> Elements {
        let Some(node) = graph.nodes.get(&layout.node) else {
            return Elements::default();
        };

        if let Some(renderer) = self.node_types.get_mut(&node.kind) {
            return renderer(ecx, graph, layout);
        }

        if let Some(fallback) = self.fallback.as_mut() {
            return fallback(ecx, graph, layout);
        }

        Elements::default()
    }
}

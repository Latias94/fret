use super::*;
use std::any::TypeId;

impl<H: UiHost> UiTree<H> {
    pub fn layout_all(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        bounds: Rect,
        scale_factor: f32,
    ) {
        let started = self.debug_enabled.then_some(Instant::now());
        if self.debug_enabled {
            self.debug_stats.frame_id = app.frame_id();
            self.debug_stats.layout_nodes_visited = 0;
            self.debug_stats.layout_nodes_performed = 0;
            self.debug_stats.focus = self.focus;
            self.debug_stats.captured = self.captured;
        }

        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        for root in roots {
            let _ = self.layout_in(app, services, root, bounds, scale_factor);
        }

        if self.semantics_requested {
            self.semantics_requested = false;
            self.refresh_semantics_snapshot(app);
        }

        if let Some(started) = started {
            self.debug_stats.layout_time = started.elapsed();
        }
    }

    pub fn layout(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        available: Size,
        scale_factor: f32,
    ) -> Size {
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            available,
        );
        self.layout_in(app, services, root, bounds, scale_factor)
    }

    pub fn layout_in(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        bounds: Rect,
        scale_factor: f32,
    ) -> Size {
        self.layout_node(app, services, root, bounds, scale_factor)
    }

    fn translate_subtree_bounds(&mut self, node: NodeId, delta: Point) {
        let mut stack = vec![node];
        while let Some(id) = stack.pop() {
            let Some(n) = self.nodes.get_mut(id) else {
                continue;
            };
            n.bounds.origin = Point::new(n.bounds.origin.x + delta.x, n.bounds.origin.y + delta.y);
            for &child in &n.children {
                stack.push(child);
            }
        }
    }

    fn layout_node(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        bounds: Rect,
        scale_factor: f32,
    ) -> Size {
        if self.debug_enabled {
            self.debug_stats.layout_nodes_visited =
                self.debug_stats.layout_nodes_visited.saturating_add(1);
        }

        let (prev_bounds, measured, invalidated) = match self.nodes.get(node) {
            Some(n) => (n.bounds, n.measured_size, n.invalidation.layout),
            None => return Size::default(),
        };

        if let Some(n) = self.nodes.get_mut(node) {
            n.bounds = bounds;
        }

        if !invalidated
            && prev_bounds.size == bounds.size
            && prev_bounds.origin != bounds.origin
            && measured != Size::default()
        {
            let delta = Point::new(
                bounds.origin.x - prev_bounds.origin.x,
                bounds.origin.y - prev_bounds.origin.y,
            );
            if (delta.x.0 != 0.0 || delta.y.0 != 0.0)
                && let Some(children) = self.nodes.get(node).map(|n| n.children.clone())
            {
                for child in children {
                    self.translate_subtree_bounds(child, delta);
                }
            }
            return measured;
        }

        let needs_layout = invalidated || prev_bounds != bounds;
        if !needs_layout {
            return measured;
        }
        if self.debug_enabled {
            self.debug_stats.layout_nodes_performed =
                self.debug_stats.layout_nodes_performed.saturating_add(1);
        }
        let sf = scale_factor;

        let mut observations: Vec<(ModelId, Invalidation)> = Vec::new();
        let mut observe_model = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };

        let mut global_observations: Vec<(TypeId, Invalidation)> = Vec::new();
        let mut observe_global = |id: TypeId, inv: Invalidation| {
            global_observations.push((id, inv));
        };
        // Theme changes can affect layout metrics across most of the tree; treat it as a default
        // dependency to ensure layout re-runs when the global theme is updated.
        observe_global(TypeId::of::<Theme>(), Invalidation::Layout);

        let size = self.with_widget_mut(node, |widget, tree| {
            let children: Vec<NodeId> = tree
                .nodes
                .get(node)
                .map(|n| n.children.clone())
                .unwrap_or_default();
            let mut cx = LayoutCx {
                app,
                node,
                window: tree.window,
                focus: tree.focus,
                children: &children,
                bounds,
                available: bounds.size,
                scale_factor: sf,
                services: &mut *services,
                observe_model: &mut observe_model,
                observe_global: &mut observe_global,
                tree,
            };
            widget.layout(&mut cx)
        });

        self.observed_in_layout.record(node, observations);
        self.observed_globals_in_layout
            .record(node, global_observations);
        if let Some(n) = self.nodes.get_mut(node) {
            n.measured_size = size;
            n.invalidation.layout = false;
        }

        size
    }
}

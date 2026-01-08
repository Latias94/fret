use super::*;
use std::any::TypeId;

use crate::layout_constraints::{AvailableSpace, LayoutConstraints};

impl<H: UiHost> UiTree<H> {
    pub fn layout_all(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        bounds: Rect,
        scale_factor: f32,
    ) {
        let started = self.debug_enabled.then(Instant::now);
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

        #[cfg(feature = "layout-engine-v2")]
        self.layout_engine.begin_frame(app.frame_id());
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

        #[cfg(feature = "layout-engine-v2")]
        self.layout_engine.end_frame();
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

    pub fn measure_in(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        constraints: LayoutConstraints,
        scale_factor: f32,
    ) -> Size {
        self.measure_node(app, services, node, constraints, scale_factor)
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
            if delta.x.0 != 0.0 || delta.y.0 != 0.0 {
                let window = self.window;
                let mut stack: Vec<NodeId> = Vec::new();
                let mut i = 0usize;
                loop {
                    let child = self
                        .nodes
                        .get(node)
                        .and_then(|n| n.children.get(i))
                        .copied();
                    let Some(child) = child else {
                        break;
                    };
                    stack.push(child);
                    i += 1;
                }

                while let Some(id) = stack.pop() {
                    let Some(n) = self.nodes.get_mut(id) else {
                        continue;
                    };
                    n.bounds.origin =
                        Point::new(n.bounds.origin.x + delta.x, n.bounds.origin.y + delta.y);
                    if let (Some(window), Some(element)) = (window, n.element) {
                        crate::elements::record_bounds_for_element(app, window, element, n.bounds);
                    }
                    for &child in &n.children {
                        stack.push(child);
                    }
                }
            }
            if let (Some(window), Some(element)) =
                (self.window, self.nodes.get(node).and_then(|n| n.element))
            {
                crate::elements::record_bounds_for_element(app, window, element, bounds);
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

        let mut observations = SmallCopyList::<(ModelId, Invalidation), 8>::default();
        let mut observe_model = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };

        let mut global_observations = SmallCopyList::<(TypeId, Invalidation), 8>::default();
        let mut observe_global = |id: TypeId, inv: Invalidation| {
            global_observations.push((id, inv));
        };
        // Theme changes can affect layout metrics across most of the tree; treat it as a default
        // dependency to ensure layout re-runs when the global theme is updated.
        observe_global(TypeId::of::<Theme>(), Invalidation::Layout);
        // Text shaping/metrics depend on the effective font stack. Track a single stable key so
        // changing font configuration or loading new fonts forces a relayout without directly
        // depending on backend configuration globals.
        observe_global(
            TypeId::of::<fret_runtime::TextFontStackKey>(),
            Invalidation::Layout,
        );

        let size = self.with_widget_mut(node, |widget, tree| {
            let mut children_buf = SmallNodeList::<32>::default();
            if let Some(children) = tree.nodes.get(node).map(|n| n.children.as_slice()) {
                children_buf.set(children);
            }
            let mut cx = LayoutCx {
                app,
                node,
                window: tree.window,
                focus: tree.focus,
                children: children_buf.as_slice(),
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

        self.observed_in_layout
            .record(node, observations.as_slice());
        self.observed_globals_in_layout
            .record(node, global_observations.as_slice());
        if let Some(n) = self.nodes.get_mut(node) {
            n.measured_size = size;
            n.invalidation.layout = false;
        }

        size
    }

    fn measure_node(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        constraints: LayoutConstraints,
        scale_factor: f32,
    ) -> Size {
        let key = MeasureStackKey {
            node,
            known_w_bits: constraints.known.width.map(|px| px.0.to_bits()),
            known_h_bits: constraints.known.height.map(|px| px.0.to_bits()),
            avail_w: available_space_key(constraints.available.width),
            avail_h: available_space_key(constraints.available.height),
            scale_bits: scale_factor.to_bits(),
        };
        if self.measure_stack.contains(&key) {
            if cfg!(debug_assertions) {
                panic!("measure_in re-entered for {node:?} under {constraints:?}");
            }
            return Size::default();
        }
        self.measure_stack.push(key);

        let sf = scale_factor;

        let mut observations = SmallCopyList::<(ModelId, Invalidation), 8>::default();
        let mut observe_model = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };

        let mut global_observations = SmallCopyList::<(TypeId, Invalidation), 8>::default();
        let mut observe_global = |id: TypeId, inv: Invalidation| {
            global_observations.push((id, inv));
        };
        observe_global(TypeId::of::<Theme>(), Invalidation::Layout);
        observe_global(
            TypeId::of::<fret_runtime::TextFontStackKey>(),
            Invalidation::Layout,
        );

        let size = self.with_widget_mut(node, |widget, tree| {
            let mut children_buf = SmallNodeList::<32>::default();
            if let Some(children) = tree.nodes.get(node).map(|n| n.children.as_slice()) {
                children_buf.set(children);
            }
            let mut cx = crate::widget::MeasureCx {
                app,
                node,
                window: tree.window,
                focus: tree.focus,
                children: children_buf.as_slice(),
                constraints,
                scale_factor: sf,
                services: &mut *services,
                observe_model: &mut observe_model,
                observe_global: &mut observe_global,
                tree,
            };
            widget.measure(&mut cx)
        });

        self.observed_in_layout
            .record(node, observations.as_slice());
        self.observed_globals_in_layout
            .record(node, global_observations.as_slice());

        let popped = self.measure_stack.pop();
        debug_assert_eq!(popped, Some(key));
        size
    }
}

fn available_space_key(avail: AvailableSpace) -> (u8, u32) {
    match avail {
        AvailableSpace::Definite(px) => (0, px.0.to_bits()),
        AvailableSpace::MinContent => (1, 0),
        AvailableSpace::MaxContent => (2, 0),
    }
}

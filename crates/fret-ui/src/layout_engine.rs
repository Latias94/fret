use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use fret_core::{FrameId, NodeId, Point, Px, Rect, Size};
use taffy::{TaffyTree, prelude::NodeId as TaffyNodeId};

use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

mod flow;
pub(crate) use flow::{
    ParentLayoutKind, build_flow_subtree, build_viewport_flow_subtree,
    layout_children_from_engine_if_solved,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeContext {
    node: NodeId,
    measured: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutId(TaffyNodeId);

pub struct TaffyLayoutEngine {
    tree: TaffyTree<NodeContext>,
    node_to_layout: HashMap<NodeId, LayoutId>,
    layout_to_node: HashMap<LayoutId, NodeId>,
    styles: HashMap<NodeId, taffy::Style>,
    children: HashMap<NodeId, Vec<NodeId>>,
    seen: HashSet<NodeId>,
    solve_generation: u64,
    node_solved_generation: HashMap<NodeId, u64>,
    solve_scale_factor: f32,
    frame_id: Option<FrameId>,
    last_solve_time: Duration,
}

impl Default for TaffyLayoutEngine {
    fn default() -> Self {
        let mut tree = TaffyTree::new();
        tree.enable_rounding();
        Self {
            tree,
            node_to_layout: HashMap::new(),
            layout_to_node: HashMap::new(),
            styles: HashMap::new(),
            children: HashMap::new(),
            seen: HashSet::new(),
            solve_generation: 0,
            node_solved_generation: HashMap::new(),
            solve_scale_factor: 1.0,
            frame_id: None,
            last_solve_time: Duration::default(),
        }
    }
}

impl TaffyLayoutEngine {
    pub fn begin_frame(&mut self, frame_id: FrameId) {
        if self.frame_id != Some(frame_id) {
            self.frame_id = Some(frame_id);
            self.seen.clear();
            self.solve_generation = 0;
            self.node_solved_generation.clear();
            self.solve_scale_factor = 1.0;
            self.last_solve_time = Duration::default();
        }
    }

    pub fn end_frame(&mut self) {
        let stale: Vec<NodeId> = self
            .node_to_layout
            .keys()
            .copied()
            .filter(|node| !self.seen.contains(node))
            .collect();

        for node in stale {
            let Some(layout_id) = self.node_to_layout.remove(&node) else {
                continue;
            };
            self.layout_to_node.remove(&layout_id);
            self.styles.remove(&node);
            self.children.remove(&node);
            self.node_solved_generation.remove(&node);
            let _ = self.tree.remove(layout_id.0);
        }
        self.seen.clear();
    }

    pub fn layout_id_for_node(&self, node: NodeId) -> Option<LayoutId> {
        self.node_to_layout.get(&node).copied()
    }

    pub fn node_for_layout_id(&self, id: LayoutId) -> Option<NodeId> {
        self.layout_to_node.get(&id).copied()
    }

    pub fn solve_count(&self) -> u64 {
        self.solve_generation
    }

    pub fn last_solve_time(&self) -> Duration {
        self.last_solve_time
    }

    pub fn child_layout_rect_if_solved(&self, parent: NodeId, child: NodeId) -> Option<Rect> {
        if self.solve_generation == 0 {
            return None;
        }
        let parent_gen = self.node_solved_generation.get(&parent).copied()?;
        let child_gen = self.node_solved_generation.get(&child).copied()?;
        if parent_gen == 0 || child_gen == 0 || parent_gen != child_gen {
            return None;
        }
        if !self.seen.contains(&parent) || !self.seen.contains(&child) {
            return None;
        }
        if !self
            .children
            .get(&parent)
            .is_some_and(|children| children.contains(&child))
        {
            return None;
        }
        self.layout_id_for_node(child)
            .map(|id| self.layout_rect(id))
    }

    pub fn request_layout_node(&mut self, node: NodeId) -> LayoutId {
        self.seen.insert(node);
        if let Some(id) = self.node_to_layout.get(&node).copied() {
            return id;
        }

        let taffy_id = self
            .tree
            .new_leaf_with_context(
                Default::default(),
                NodeContext {
                    node,
                    measured: false,
                },
            )
            .expect("taffy new_leaf");
        let id = LayoutId(taffy_id);
        self.node_to_layout.insert(node, id);
        self.layout_to_node.insert(id, node);
        id
    }

    pub fn set_measured(&mut self, node: NodeId, measured: bool) {
        let id = self.request_layout_node(node).0;
        let ctx = self
            .tree
            .get_node_context(id)
            .copied()
            .unwrap_or(NodeContext { node, measured });
        if ctx.node == node && ctx.measured == measured {
            return;
        }
        self.node_solved_generation.remove(&node);
        let _ = self
            .tree
            .set_node_context(id, Some(NodeContext { node, measured }));
        let _ = self.tree.mark_dirty(id);
    }

    pub fn set_style(&mut self, node: NodeId, style: taffy::Style) {
        let id = self.request_layout_node(node).0;
        if self.styles.get(&node) == Some(&style) {
            return;
        }
        self.node_solved_generation.remove(&node);
        if self.tree.set_style(id, style.clone()).is_ok() {
            self.styles.insert(node, style);
            let _ = self.tree.mark_dirty(id);
        }
    }

    pub fn set_children(&mut self, node: NodeId, children: &[NodeId]) {
        let parent = self.request_layout_node(node).0;

        let prev = self
            .children
            .get(&node)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        if prev == children {
            return;
        }
        self.node_solved_generation.remove(&node);

        let mut child_nodes: Vec<TaffyNodeId> = Vec::with_capacity(children.len());
        for &child in children {
            let child_id = self.request_layout_node(child).0;
            child_nodes.push(child_id);
        }

        if self.tree.set_children(parent, &child_nodes).is_ok() {
            self.children.insert(node, children.to_vec());
            let _ = self.tree.mark_dirty(parent);
        }
    }

    #[cfg_attr(feature = "layout-engine-v2", stacksafe::stacksafe)]
    pub fn compute_root_with_measure(
        &mut self,
        root: LayoutId,
        available: LayoutSize<AvailableSpace>,
        scale_factor: f32,
        mut measure: impl FnMut(NodeId, LayoutConstraints) -> Size,
    ) {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct MeasureKey {
            node: NodeId,
            known_w: Option<u32>,
            known_h: Option<u32>,
            avail_w: (u8, u32),
            avail_h: (u8, u32),
        }

        fn avail_key(avail: taffy::style::AvailableSpace) -> (u8, u32) {
            match avail {
                taffy::style::AvailableSpace::Definite(v) => (0, v.to_bits()),
                taffy::style::AvailableSpace::MinContent => (1, 0),
                taffy::style::AvailableSpace::MaxContent => (2, 0),
            }
        }

        let started = Instant::now();
        let sf = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        self.solve_scale_factor = sf;

        let available = taffy::geometry::Size {
            width: match available.width {
                AvailableSpace::Definite(px) => taffy::style::AvailableSpace::Definite(px.0 * sf),
                AvailableSpace::MinContent => taffy::style::AvailableSpace::MinContent,
                AvailableSpace::MaxContent => taffy::style::AvailableSpace::MaxContent,
            },
            height: match available.height {
                AvailableSpace::Definite(px) => taffy::style::AvailableSpace::Definite(px.0 * sf),
                AvailableSpace::MinContent => taffy::style::AvailableSpace::MinContent,
                AvailableSpace::MaxContent => taffy::style::AvailableSpace::MaxContent,
            },
        };

        let mut measure_cache: HashMap<MeasureKey, taffy::geometry::Size<f32>> = HashMap::new();
        self.tree
            .compute_layout_with_measure(root.0, available, |known, avail, _id, ctx, _style| {
                let Some(ctx) = ctx else {
                    return taffy::geometry::Size::default();
                };
                if !ctx.measured {
                    return taffy::geometry::Size::default();
                }

                let key = MeasureKey {
                    node: ctx.node,
                    known_w: known.width.map(|v| v.to_bits()),
                    known_h: known.height.map(|v| v.to_bits()),
                    avail_w: avail_key(avail.width),
                    avail_h: avail_key(avail.height),
                };
                if let Some(size) = measure_cache.get(&key) {
                    return *size;
                }

                let constraints = LayoutConstraints::new(
                    LayoutSize::new(
                        known.width.map(|w| Px(w / sf)),
                        known.height.map(|h| Px(h / sf)),
                    ),
                    LayoutSize::new(
                        match avail.width {
                            taffy::style::AvailableSpace::Definite(w) => {
                                AvailableSpace::Definite(Px(w / sf))
                            }
                            taffy::style::AvailableSpace::MinContent => AvailableSpace::MinContent,
                            taffy::style::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
                        },
                        match avail.height {
                            taffy::style::AvailableSpace::Definite(h) => {
                                AvailableSpace::Definite(Px(h / sf))
                            }
                            taffy::style::AvailableSpace::MinContent => AvailableSpace::MinContent,
                            taffy::style::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
                        },
                    ),
                );

                let s = measure(ctx.node, constraints);
                let out = taffy::geometry::Size {
                    width: s.width.0 * sf,
                    height: s.height.0 * sf,
                };
                measure_cache.insert(key, out);
                out
            })
            .ok();

        self.solve_generation = self.solve_generation.saturating_add(1);
        if let Some(root_node) = self.node_for_layout_id(root) {
            self.mark_solved_subtree(root_node);
        }
        self.last_solve_time += started.elapsed();
    }

    pub fn compute_root(
        &mut self,
        root: LayoutId,
        available: LayoutSize<AvailableSpace>,
        scale_factor: f32,
    ) {
        self.compute_root_with_measure(root, available, scale_factor, |_node, _constraints| {
            Size::default()
        });
    }

    pub fn layout_rect(&self, id: LayoutId) -> Rect {
        let Ok(layout) = self.tree.layout(id.0) else {
            return Rect::new(Point::new(Px(0.0), Px(0.0)), Size::default());
        };
        let sf = if self.solve_scale_factor.is_finite() && self.solve_scale_factor > 0.0 {
            self.solve_scale_factor
        } else {
            1.0
        };

        Rect::new(
            Point::new(Px(layout.location.x / sf), Px(layout.location.y / sf)),
            Size::new(Px(layout.size.width / sf), Px(layout.size.height / sf)),
        )
    }

    fn mark_solved_subtree(&mut self, root: NodeId) {
        let mut stack: Vec<NodeId> = vec![root];
        while let Some(node) = stack.pop() {
            self.node_solved_generation
                .insert(node, self.solve_generation);
            if let Some(children) = self.children.get(&node) {
                stack.extend(children.iter().copied());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::SlotMap;

    fn fresh_node_ids(count: usize) -> Vec<NodeId> {
        let mut map: SlotMap<NodeId, ()> = SlotMap::with_key();
        (0..count).map(|_| map.insert(())).collect()
    }

    #[test]
    fn multiple_roots_do_not_couple_layout_results() {
        let [root_a, root_b, child_a, child_b] = fresh_node_ids(4).try_into().unwrap();

        let mut engine = TaffyLayoutEngine::default();
        engine.begin_frame(FrameId(1));

        engine.set_children(root_a, &[child_a]);
        engine.set_children(root_b, &[child_b]);

        engine.set_style(
            root_a,
            taffy::Style {
                display: taffy::style::Display::Block,
                size: taffy::geometry::Size {
                    width: taffy::style::Dimension::length(100.0),
                    height: taffy::style::Dimension::length(10.0),
                },
                ..Default::default()
            },
        );
        engine.set_style(
            root_b,
            taffy::Style {
                display: taffy::style::Display::Block,
                size: taffy::geometry::Size {
                    width: taffy::style::Dimension::length(200.0),
                    height: taffy::style::Dimension::length(10.0),
                },
                ..Default::default()
            },
        );

        let fill = taffy::Style {
            display: taffy::style::Display::Block,
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::percent(1.0),
                height: taffy::style::Dimension::percent(1.0),
            },
            ..Default::default()
        };
        engine.set_style(child_a, fill.clone());
        engine.set_style(child_b, fill);

        let root_a_id = engine.layout_id_for_node(root_a).unwrap();
        let root_b_id = engine.layout_id_for_node(root_b).unwrap();

        engine.compute_root(
            root_a_id,
            LayoutSize::new(
                AvailableSpace::Definite(Px(100.0)),
                AvailableSpace::Definite(Px(10.0)),
            ),
            1.0,
        );

        let child_a_id = engine.layout_id_for_node(child_a).unwrap();
        let a_before = engine.layout_rect(child_a_id);
        assert!((a_before.size.width.0 - 100.0).abs() < 0.01);
        assert!((a_before.size.height.0 - 10.0).abs() < 0.01);

        engine.compute_root(
            root_b_id,
            LayoutSize::new(
                AvailableSpace::Definite(Px(200.0)),
                AvailableSpace::Definite(Px(10.0)),
            ),
            1.0,
        );

        let child_b_id = engine.layout_id_for_node(child_b).unwrap();
        let b = engine.layout_rect(child_b_id);
        assert!((b.size.width.0 - 200.0).abs() < 0.01);
        assert!((b.size.height.0 - 10.0).abs() < 0.01);

        let a_after = engine.layout_rect(child_a_id);
        assert_eq!(a_before, a_after);

        assert_eq!(
            engine.child_layout_rect_if_solved(root_a, child_a),
            Some(a_after),
            "solved subtree rects should remain readable after solving an unrelated root"
        );
    }
}

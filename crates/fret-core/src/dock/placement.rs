use std::collections::{HashMap, HashSet};

use crate::{AppWindowId, Axis, DockGraph, DockNodeId, PanelKey};

use super::layout::DockLayoutBuilder;

const DEFAULT_LEFT_RAIL_FRACTION: f32 = 0.26;
const DEFAULT_RIGHT_RAIL_FRACTION: f32 = 0.24;
const DEFAULT_BOTTOM_RAIL_FRACTION: f32 = 0.28;

/// Product-level target for placing a dock panel in a default layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockPanelPlacementTarget {
    Center,
    LeftRail,
    RightRail,
    BottomRail,
    Stack {
        anchor: PanelKey,
        insert_index: Option<usize>,
    },
}

impl DockPanelPlacementTarget {
    pub fn center() -> Self {
        Self::Center
    }

    pub fn left_rail() -> Self {
        Self::LeftRail
    }

    pub fn right_rail() -> Self {
        Self::RightRail
    }

    pub fn bottom_rail() -> Self {
        Self::BottomRail
    }

    pub fn stacked_with(anchor: impl Into<PanelKey>) -> Self {
        Self::Stack {
            anchor: anchor.into(),
            insert_index: None,
        }
    }

    pub fn stacked_with_at(anchor: impl Into<PanelKey>, insert_index: usize) -> Self {
        Self::Stack {
            anchor: anchor.into(),
            insert_index: Some(insert_index),
        }
    }
}

/// Product-level placement for one dock panel in a generated layout.
#[derive(Debug, Clone, PartialEq)]
pub struct DockPanelPlacement {
    panel: PanelKey,
    target: DockPanelPlacementTarget,
    selected: bool,
    fallback: Option<DockPanelPlacementTarget>,
    fraction: Option<f32>,
}

impl DockPanelPlacement {
    pub fn new(panel: impl Into<PanelKey>, target: DockPanelPlacementTarget) -> Self {
        Self {
            panel: panel.into(),
            target,
            selected: false,
            fallback: None,
            fraction: None,
        }
    }

    pub fn center(panel: impl Into<PanelKey>) -> Self {
        Self::new(panel, DockPanelPlacementTarget::center())
    }

    pub fn left_rail(panel: impl Into<PanelKey>) -> Self {
        Self::new(panel, DockPanelPlacementTarget::left_rail())
    }

    pub fn right_rail(panel: impl Into<PanelKey>) -> Self {
        Self::new(panel, DockPanelPlacementTarget::right_rail())
    }

    pub fn bottom_rail(panel: impl Into<PanelKey>) -> Self {
        Self::new(panel, DockPanelPlacementTarget::bottom_rail())
    }

    pub fn stacked_with(panel: impl Into<PanelKey>, anchor: impl Into<PanelKey>) -> Self {
        Self::new(panel, DockPanelPlacementTarget::stacked_with(anchor))
    }

    pub fn stacked_with_at(
        panel: impl Into<PanelKey>,
        anchor: impl Into<PanelKey>,
        insert_index: usize,
    ) -> Self {
        Self::new(
            panel,
            DockPanelPlacementTarget::stacked_with_at(anchor, insert_index),
        )
    }

    pub fn panel(&self) -> &PanelKey {
        &self.panel
    }

    pub fn target(&self) -> &DockPanelPlacementTarget {
        &self.target
    }

    pub fn fallback_target(&self) -> Option<&DockPanelPlacementTarget> {
        self.fallback.as_ref()
    }

    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }

    pub fn fraction(mut self, fraction: f32) -> Self {
        self.fraction = Some(fraction);
        self
    }

    pub fn fallback(mut self, fallback: DockPanelPlacementTarget) -> Self {
        self.fallback = Some(fallback);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum PlacementBucket {
    Center,
    Left,
    Right,
    Bottom,
}

#[derive(Debug)]
struct PlacementStack {
    panels: Vec<PanelKey>,
    selected: Option<PanelKey>,
    fraction: f32,
}

impl PlacementStack {
    fn new(fraction: f32) -> Self {
        Self {
            panels: Vec::new(),
            selected: None,
            fraction,
        }
    }

    fn insert(&mut self, placement: DockPanelPlacement, insert_index: Option<usize>) {
        let panel = placement.panel;
        let index = insert_index
            .unwrap_or(self.panels.len())
            .min(self.panels.len());
        self.panels.insert(index, panel.clone());
        if placement.selected {
            self.selected = Some(panel);
        }
        if let Some(fraction) = placement.fraction {
            self.fraction = sanitize_fraction(fraction, self.fraction);
        }
    }

    fn active_index(&self) -> usize {
        self.selected
            .as_ref()
            .and_then(|selected| self.panels.iter().position(|panel| panel == selected))
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlacementPosition {
    bucket: PlacementBucket,
    index: usize,
}

struct PlacementStacks {
    center: PlacementStack,
    left: PlacementStack,
    right: PlacementStack,
    bottom: PlacementStack,
    panel_positions: HashMap<PanelKey, PlacementPosition>,
    implicit_stack_tails: HashMap<PanelKey, PanelKey>,
}

impl Default for PlacementStacks {
    fn default() -> Self {
        Self {
            center: PlacementStack::new(0.0),
            left: PlacementStack::new(DEFAULT_LEFT_RAIL_FRACTION),
            right: PlacementStack::new(DEFAULT_RIGHT_RAIL_FRACTION),
            bottom: PlacementStack::new(DEFAULT_BOTTOM_RAIL_FRACTION),
            panel_positions: HashMap::new(),
            implicit_stack_tails: HashMap::new(),
        }
    }
}

impl PlacementStacks {
    fn from_placements(placements: impl IntoIterator<Item = DockPanelPlacement>) -> Self {
        let mut stacks = Self::default();
        let mut pending = Vec::new();

        for placement in last_unique_placements(placements) {
            if let Some(placement) = stacks.push_if_resolved(placement) {
                pending.push(placement);
            }
        }

        while !pending.is_empty() {
            let before = pending.len();
            let mut unresolved = Vec::new();
            for placement in pending {
                if let Some(placement) = stacks.push_if_resolved(placement) {
                    unresolved.push(placement);
                }
            }
            if unresolved.len() == before {
                pending = unresolved;
                break;
            }
            pending = unresolved;
        }

        for placement in pending {
            stacks.push_with_fallback(placement);
        }

        stacks
    }

    fn push_if_resolved(&mut self, placement: DockPanelPlacement) -> Option<DockPanelPlacement> {
        let Some(target) = placement.target().clone().or_resolved_against(self) else {
            return Some(placement);
        };
        self.push_resolved(placement, target);
        None
    }

    fn push_with_fallback(&mut self, placement: DockPanelPlacement) {
        let target = placement
            .fallback_target()
            .cloned()
            .and_then(|target| target.or_resolved_against(self))
            .unwrap_or(ResolvedPlacementTarget {
                bucket: PlacementBucket::Center,
                insert_index: None,
                implicit_stack_anchor: None,
            });
        self.push_resolved(placement, target);
    }

    fn push_resolved(&mut self, placement: DockPanelPlacement, target: ResolvedPlacementTarget) {
        let bucket = target.bucket;
        let panel = placement.panel.clone();
        let implicit_stack_anchor = target.implicit_stack_anchor;
        self.stack_mut(bucket)
            .insert(placement, target.insert_index);
        self.rebuild_positions_for_bucket(bucket);
        if let Some(anchor) = implicit_stack_anchor {
            self.implicit_stack_tails.insert(anchor, panel);
        }
    }

    fn stack_mut(&mut self, bucket: PlacementBucket) -> &mut PlacementStack {
        match bucket {
            PlacementBucket::Center => &mut self.center,
            PlacementBucket::Left => &mut self.left,
            PlacementBucket::Right => &mut self.right,
            PlacementBucket::Bottom => &mut self.bottom,
        }
    }

    fn stack(&self, bucket: PlacementBucket) -> &PlacementStack {
        match bucket {
            PlacementBucket::Center => &self.center,
            PlacementBucket::Left => &self.left,
            PlacementBucket::Right => &self.right,
            PlacementBucket::Bottom => &self.bottom,
        }
    }

    fn rebuild_positions_for_bucket(&mut self, bucket: PlacementBucket) {
        let positions: Vec<(PanelKey, usize)> = self
            .stack(bucket)
            .panels
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, panel)| (panel, index))
            .collect();
        for (panel, index) in positions {
            self.panel_positions
                .insert(panel, PlacementPosition { bucket, index });
        }
    }
}

fn last_unique_placements(
    placements: impl IntoIterator<Item = DockPanelPlacement>,
) -> Vec<DockPanelPlacement> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let placements: Vec<DockPanelPlacement> = placements.into_iter().collect();
    for placement in placements.into_iter().rev() {
        if seen.insert(placement.panel.clone()) {
            out.push(placement);
        }
    }
    out.reverse();
    out
}

#[derive(Clone)]
struct ResolvedPlacementTarget {
    bucket: PlacementBucket,
    insert_index: Option<usize>,
    implicit_stack_anchor: Option<PanelKey>,
}

impl DockPanelPlacementTarget {
    fn or_resolved_against(self, stacks: &PlacementStacks) -> Option<ResolvedPlacementTarget> {
        match self {
            Self::Center => Some(ResolvedPlacementTarget {
                bucket: PlacementBucket::Center,
                insert_index: None,
                implicit_stack_anchor: None,
            }),
            Self::LeftRail => Some(ResolvedPlacementTarget {
                bucket: PlacementBucket::Left,
                insert_index: None,
                implicit_stack_anchor: None,
            }),
            Self::RightRail => Some(ResolvedPlacementTarget {
                bucket: PlacementBucket::Right,
                insert_index: None,
                implicit_stack_anchor: None,
            }),
            Self::BottomRail => Some(ResolvedPlacementTarget {
                bucket: PlacementBucket::Bottom,
                insert_index: None,
                implicit_stack_anchor: None,
            }),
            Self::Stack {
                anchor,
                insert_index,
            } => {
                let position = stacks.panel_positions.get(&anchor)?;
                let Some(insert_index) = insert_index else {
                    let tail = stacks.implicit_stack_tails.get(&anchor).unwrap_or(&anchor);
                    let tail_position = stacks.panel_positions.get(tail).unwrap_or(position);
                    return Some(ResolvedPlacementTarget {
                        bucket: tail_position.bucket,
                        insert_index: Some(tail_position.index + 1),
                        implicit_stack_anchor: Some(anchor),
                    });
                };
                Some(ResolvedPlacementTarget {
                    bucket: position.bucket,
                    insert_index: Some(insert_index),
                    implicit_stack_anchor: None,
                })
            }
        }
    }
}

fn sanitize_fraction(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 && value < 1.0 {
        value
    } else {
        fallback
    }
}

impl DockGraph {
    pub fn from_panel_placements(
        window: AppWindowId,
        placements: impl IntoIterator<Item = DockPanelPlacement>,
    ) -> Self {
        let PlacementStacks {
            center,
            left,
            right,
            bottom,
            panel_positions: _,
            ..
        } = PlacementStacks::from_placements(placements);
        let left_fraction = left.fraction;
        let right_fraction = right.fraction;
        let bottom_fraction = bottom.fraction;

        let mut builder = DockLayoutBuilder::new();
        let center = build_stack_node(&mut builder, center);
        let left = build_stack_node(&mut builder, left);
        let right = build_stack_node(&mut builder, right);
        let bottom = build_stack_node(&mut builder, bottom);
        let work_area = build_work_area_node(
            &mut builder,
            left,
            center,
            right,
            left_fraction,
            right_fraction,
        );
        let root = match (work_area, bottom) {
            (Some(work_area), Some(bottom)) => {
                let bottom_fraction =
                    sanitize_fraction(bottom_fraction, DEFAULT_BOTTOM_RAIL_FRACTION);
                Some(builder.split(
                    Axis::Vertical,
                    vec![work_area, bottom],
                    vec![1.0 - bottom_fraction, bottom_fraction],
                ))
            }
            (Some(work_area), None) => Some(work_area),
            (None, Some(bottom)) => Some(bottom),
            (None, None) => None,
        };

        if let Some(root) = root {
            builder.set_window_root(window, root);
        }
        builder.into_graph()
    }
}

fn build_stack_node(builder: &mut DockLayoutBuilder, stack: PlacementStack) -> Option<DockNodeId> {
    if stack.panels.is_empty() {
        return None;
    }
    let active = stack.active_index();
    Some(builder.tabs(stack.panels, active))
}

fn build_work_area_node(
    builder: &mut DockLayoutBuilder,
    left: Option<DockNodeId>,
    center: Option<DockNodeId>,
    right: Option<DockNodeId>,
    left_fraction: f32,
    right_fraction: f32,
) -> Option<DockNodeId> {
    let mut children = Vec::new();
    let mut fractions = Vec::new();
    if let Some(left) = left {
        children.push(left);
        fractions.push(left_fraction);
    }
    if let Some(center) = center {
        children.push(center);
        let side_total = left.map(|_| left_fraction).unwrap_or(0.0)
            + right.map(|_| right_fraction).unwrap_or(0.0);
        fractions.push((1.0 - side_total).max(0.05));
    }
    if let Some(right) = right {
        children.push(right);
        fractions.push(right_fraction);
    }

    match children.len() {
        0 => None,
        1 => children.into_iter().next(),
        _ => {
            if center.is_none() {
                let share = 1.0 / children.len() as f32;
                fractions.fill(share);
            }
            Some(builder.split(Axis::Horizontal, children, fractions))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DockNode;

    fn collect_panels_in_node(graph: &DockGraph, node: DockNodeId) -> Vec<PanelKey> {
        match graph.node(node) {
            Some(DockNode::Tabs { tabs, .. }) => tabs.clone(),
            Some(DockNode::Split { children, .. }) => children
                .iter()
                .flat_map(|child| collect_panels_in_node(graph, *child))
                .collect(),
            Some(DockNode::Floating { child }) => collect_panels_in_node(graph, *child),
            None => Vec::new(),
        }
    }

    fn tab_order(graph: &DockGraph, tabs_node: DockNodeId) -> Vec<PanelKey> {
        match graph.node(tabs_node) {
            Some(DockNode::Tabs { tabs, .. }) => tabs.clone(),
            other => panic!("expected tabs node, got {other:?}"),
        }
    }

    #[test]
    fn panel_placements_build_editor_rails() {
        let window = AppWindowId::default();
        let hierarchy = PanelKey::new("core.hierarchy");
        let scene = PanelKey::new("core.scene");
        let inspector = PanelKey::new("core.inspector");
        let console = PanelKey::new("core.console");

        let graph = DockGraph::from_panel_placements(
            window,
            [
                DockPanelPlacement::left_rail(hierarchy.clone()).fraction(0.20),
                DockPanelPlacement::center(scene.clone()).selected(),
                DockPanelPlacement::right_rail(inspector.clone()).fraction(0.25),
                DockPanelPlacement::bottom_rail(console.clone()).fraction(0.30),
            ],
        );

        assert!(graph.window_root(window).is_some());
        assert_eq!(
            graph.collect_panels_in_window(window),
            vec![
                hierarchy.clone(),
                scene.clone(),
                inspector.clone(),
                console.clone()
            ]
        );
        assert!(graph.find_panel_in_window(window, &hierarchy).is_some());
        assert!(graph.find_panel_in_window(window, &scene).is_some());
        assert!(graph.find_panel_in_window(window, &inspector).is_some());
        assert!(graph.find_panel_in_window(window, &console).is_some());
    }

    #[test]
    fn panel_placements_stack_with_anchor_and_select_inserted_panel() {
        let window = AppWindowId::default();
        let editor = PanelKey::new("core.editor");
        let preview = PanelKey::new("core.preview");
        let inspector = PanelKey::new("core.inspector");

        let graph = DockGraph::from_panel_placements(
            window,
            [
                DockPanelPlacement::center(editor.clone()),
                DockPanelPlacement::right_rail(inspector),
                DockPanelPlacement::stacked_with(preview.clone(), editor.clone()).selected(),
            ],
        );

        let (editor_tabs, editor_index) = graph
            .find_panel_in_window(window, &editor)
            .expect("editor should be open");
        let (preview_tabs, preview_index) = graph
            .find_panel_in_window(window, &preview)
            .expect("preview should be open");

        assert_eq!(editor_tabs, preview_tabs);
        assert_eq!(editor_index, 0);
        assert_eq!(preview_index, 1);
        assert!(matches!(
            graph.node(preview_tabs),
            Some(DockNode::Tabs { active: 1, .. })
        ));
    }

    #[test]
    fn panel_placements_stack_with_anchor_declared_later() {
        let window = AppWindowId::default();
        let editor = PanelKey::new("core.editor");
        let preview = PanelKey::new("core.preview");

        let graph = DockGraph::from_panel_placements(
            window,
            [
                DockPanelPlacement::stacked_with(preview.clone(), editor.clone()).selected(),
                DockPanelPlacement::center(editor.clone()),
            ],
        );

        let (editor_tabs, editor_index) = graph
            .find_panel_in_window(window, &editor)
            .expect("editor should be open");
        let (preview_tabs, preview_index) = graph
            .find_panel_in_window(window, &preview)
            .expect("preview should be open");

        assert_eq!(editor_tabs, preview_tabs);
        assert_eq!(editor_index, 0);
        assert_eq!(preview_index, 1);
        assert!(matches!(
            graph.node(preview_tabs),
            Some(DockNode::Tabs { active: 1, .. })
        ));
    }

    #[test]
    fn panel_placements_duplicate_panel_uses_last_placement() {
        let window = AppWindowId::default();
        let center = PanelKey::new("core.center");
        let editor = PanelKey::new("core.editor");
        let preview = PanelKey::new("core.preview");

        let graph = DockGraph::from_panel_placements(
            window,
            [
                DockPanelPlacement::left_rail(editor.clone()),
                DockPanelPlacement::center(center.clone()),
                DockPanelPlacement::right_rail(editor.clone()),
                DockPanelPlacement::stacked_with(preview.clone(), editor.clone()),
            ],
        );

        assert_eq!(
            graph
                .collect_panels_in_window(window)
                .into_iter()
                .filter(|panel| panel == &editor)
                .count(),
            1
        );
        let (editor_tabs, editor_index) = graph
            .find_panel_in_window(window, &editor)
            .expect("editor should be open once");
        let (preview_tabs, preview_index) = graph
            .find_panel_in_window(window, &preview)
            .expect("preview should stack with the last editor placement");
        assert_eq!(editor_tabs, preview_tabs);
        assert_eq!(editor_index, 0);
        assert_eq!(preview_index, 1);
        assert_eq!(
            graph.collect_panels_in_window(window),
            vec![center, editor, preview],
            "last editor placement should win, putting editor and preview in the right rail"
        );
    }

    #[test]
    fn panel_placements_stack_with_explicit_insert_index() {
        let window = AppWindowId::default();
        let editor = PanelKey::new("core.editor");
        let preview = PanelKey::new("core.preview");
        let docs = PanelKey::new("core.docs");
        let terminal = PanelKey::new("core.terminal");

        let graph = DockGraph::from_panel_placements(
            window,
            [
                DockPanelPlacement::center(editor.clone()),
                DockPanelPlacement::stacked_with(preview.clone(), editor.clone()),
                DockPanelPlacement::stacked_with_at(docs.clone(), editor.clone(), 0).selected(),
                DockPanelPlacement::stacked_with_at(terminal.clone(), editor.clone(), 99),
            ],
        );

        let (editor_tabs, editor_index) = graph
            .find_panel_in_window(window, &editor)
            .expect("editor should be open");
        assert_eq!(
            tab_order(&graph, editor_tabs),
            vec![docs, editor.clone(), preview, terminal],
            "explicit insert_index should be absolute within the resolved stack and clamp at len"
        );
        assert_eq!(editor_index, 1);
        assert!(matches!(
            graph.node(editor_tabs),
            Some(DockNode::Tabs { active: 0, .. })
        ));
    }

    #[test]
    fn panel_placements_same_anchor_siblings_preserve_input_order() {
        let window = AppWindowId::default();
        let editor = PanelKey::new("core.editor");
        let preview = PanelKey::new("core.preview");
        let docs = PanelKey::new("core.docs");

        let graph = DockGraph::from_panel_placements(
            window,
            [
                DockPanelPlacement::center(editor.clone()),
                DockPanelPlacement::stacked_with(preview.clone(), editor.clone()),
                DockPanelPlacement::stacked_with(docs.clone(), editor.clone()),
            ],
        );

        let (editor_tabs, _) = graph
            .find_panel_in_window(window, &editor)
            .expect("editor should be open");
        assert_eq!(tab_order(&graph, editor_tabs), vec![editor, preview, docs]);
    }

    #[test]
    fn panel_placements_mutual_stack_cycle_uses_each_fallback() {
        let window = AppWindowId::default();
        let center = PanelKey::new("core.center");
        let left = PanelKey::new("core.left");
        let right = PanelKey::new("core.right");

        let graph = DockGraph::from_panel_placements(
            window,
            [
                DockPanelPlacement::center(center.clone()),
                DockPanelPlacement::stacked_with(left.clone(), right.clone())
                    .fallback(DockPanelPlacementTarget::left_rail()),
                DockPanelPlacement::stacked_with(right.clone(), left.clone())
                    .fallback(DockPanelPlacementTarget::right_rail()),
            ],
        );

        assert_eq!(
            graph.collect_panels_in_window(window),
            vec![left, center, right],
            "unresolved stack cycles should not let one fallbacked panel steal another panel's fallback"
        );
    }

    #[test]
    fn panel_placements_use_fallback_when_stack_anchor_is_missing() {
        let window = AppWindowId::default();
        let missing = PanelKey::new("missing.anchor");
        let scene = PanelKey::new("core.scene");
        let console = PanelKey::new("core.console");

        let graph = DockGraph::from_panel_placements(
            window,
            [
                DockPanelPlacement::center(scene.clone()),
                DockPanelPlacement::stacked_with(console.clone(), missing)
                    .fallback(DockPanelPlacementTarget::bottom_rail()),
            ],
        );

        let location = graph
            .panel_location_in_window(window, &console)
            .expect("fallback panel should be open");
        assert_eq!(location.tab_index, 0);
        assert_eq!(location.tab_count, 1);
        let root = graph.window_root(window).expect("root should exist");
        let Some(DockNode::Split {
            axis: Axis::Vertical,
            children,
            ..
        }) = graph.node(root)
        else {
            panic!("center plus bottom fallback should build a vertical root split");
        };
        assert_eq!(
            collect_panels_in_node(&graph, children[1]),
            vec![console],
            "missing-anchor fallback should land in the bottom rail"
        );
        assert_eq!(collect_panels_in_node(&graph, children[0]), vec![scene]);
    }

    #[test]
    fn panel_placements_default_missing_stack_anchor_to_center() {
        let window = AppWindowId::default();
        let missing = PanelKey::new("missing.anchor");
        let scene = PanelKey::new("core.scene");
        let console = PanelKey::new("core.console");

        let graph = DockGraph::from_panel_placements(
            window,
            [
                DockPanelPlacement::center(scene.clone()),
                DockPanelPlacement::stacked_with(console.clone(), missing),
            ],
        );

        let (scene_tabs, scene_index) = graph
            .find_panel_in_window(window, &scene)
            .expect("scene should be open");
        let (console_tabs, console_index) = graph
            .find_panel_in_window(window, &console)
            .expect("console should default to center");
        assert_eq!(scene_tabs, console_tabs);
        assert_eq!(scene_index, 0);
        assert_eq!(console_index, 1);
    }
}

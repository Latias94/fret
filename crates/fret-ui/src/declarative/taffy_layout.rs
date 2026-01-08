use super::prelude::*;

pub(crate) fn taffy_dimension(length: Length) -> Dimension {
    match length {
        Length::Auto => Dimension::auto(),
        Length::Fill => Dimension::percent(1.0),
        Length::Px(px) => Dimension::length(px.0),
    }
}

pub(crate) fn taffy_position(position: crate::element::PositionStyle) -> TaffyPosition {
    match position {
        crate::element::PositionStyle::Static | crate::element::PositionStyle::Relative => {
            TaffyPosition::Relative
        }
        crate::element::PositionStyle::Absolute => TaffyPosition::Absolute,
    }
}

fn taffy_lpa(px: Option<Px>) -> LengthPercentageAuto {
    match px {
        Some(px) => LengthPercentageAuto::length(px.0),
        None => LengthPercentageAuto::auto(),
    }
}

pub(crate) fn taffy_rect_lpa_from_inset(
    position: crate::element::PositionStyle,
    inset: crate::element::InsetStyle,
) -> TaffyRect<LengthPercentageAuto> {
    if position == crate::element::PositionStyle::Static {
        return TaffyRect {
            left: LengthPercentageAuto::auto(),
            right: LengthPercentageAuto::auto(),
            top: LengthPercentageAuto::auto(),
            bottom: LengthPercentageAuto::auto(),
        };
    }
    TaffyRect {
        left: taffy_lpa(inset.left),
        right: taffy_lpa(inset.right),
        top: taffy_lpa(inset.top),
        bottom: taffy_lpa(inset.bottom),
    }
}

fn taffy_lpa_margin_edge(edge: crate::element::MarginEdge) -> LengthPercentageAuto {
    match edge {
        crate::element::MarginEdge::Px(px) => LengthPercentageAuto::length(px.0),
        crate::element::MarginEdge::Auto => LengthPercentageAuto::auto(),
    }
}

pub(crate) fn taffy_rect_lpa_from_margin_edges(
    margin: crate::element::MarginEdges,
) -> TaffyRect<LengthPercentageAuto> {
    TaffyRect {
        left: taffy_lpa_margin_edge(margin.left),
        right: taffy_lpa_margin_edge(margin.right),
        top: taffy_lpa_margin_edge(margin.top),
        bottom: taffy_lpa_margin_edge(margin.bottom),
    }
}

pub(crate) fn taffy_grid_line(line: crate::element::GridLine) -> TaffyLine<GridPlacement> {
    let start = line
        .start
        .map(taffy::style_helpers::line::<GridPlacement>)
        .unwrap_or(GridPlacement::Auto);
    let end = line
        .span
        .map(GridPlacement::Span)
        .unwrap_or(GridPlacement::Auto);
    TaffyLine { start, end }
}

pub(crate) fn taffy_align_items(align: CrossAlign) -> TaffyAlignItems {
    match align {
        CrossAlign::Start => TaffyAlignItems::FlexStart,
        CrossAlign::Center => TaffyAlignItems::Center,
        CrossAlign::End => TaffyAlignItems::FlexEnd,
        CrossAlign::Stretch => TaffyAlignItems::Stretch,
    }
}

pub(crate) fn taffy_align_self(align: CrossAlign) -> TaffyAlignSelf {
    match align {
        CrossAlign::Start => TaffyAlignSelf::FlexStart,
        CrossAlign::Center => TaffyAlignSelf::Center,
        CrossAlign::End => TaffyAlignSelf::FlexEnd,
        CrossAlign::Stretch => TaffyAlignSelf::Stretch,
    }
}

pub(crate) fn taffy_justify(justify: MainAlign) -> JustifyContent {
    match justify {
        MainAlign::Start => JustifyContent::FlexStart,
        MainAlign::Center => JustifyContent::Center,
        MainAlign::End => JustifyContent::FlexEnd,
        MainAlign::SpaceBetween => JustifyContent::SpaceBetween,
        MainAlign::SpaceAround => JustifyContent::SpaceAround,
        MainAlign::SpaceEvenly => JustifyContent::SpaceEvenly,
    }
}

pub(super) struct TaffyContainerCache {
    pub(super) children: Vec<NodeId>,
    pub(super) taffy: TaffyTree<Option<NodeId>>,
    pub(super) root: TaffyNodeId,
    pub(super) child_nodes: Vec<TaffyNodeId>,
    pub(super) node_by_child: HashMap<NodeId, TaffyNodeId>,
    pub(super) root_style: Option<TaffyStyle>,
    pub(super) child_styles: HashMap<NodeId, TaffyStyle>,
    pub(super) measure_cache:
        std::collections::HashMap<TaffyMeasureKey, taffy::geometry::Size<f32>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct TaffyMeasureKey {
    pub(super) child: NodeId,
    pub(super) known_w: Option<u32>,
    pub(super) known_h: Option<u32>,
    pub(super) avail_w: (u8, u32),
    pub(super) avail_h: (u8, u32),
}

pub(crate) fn taffy_available_space_key(avail: TaffyAvailableSpace) -> (u8, u32) {
    match avail {
        TaffyAvailableSpace::Definite(v) => (0, v.to_bits()),
        TaffyAvailableSpace::MinContent => (1, 0),
        TaffyAvailableSpace::MaxContent => (2, 0),
    }
}

impl Default for TaffyContainerCache {
    fn default() -> Self {
        // Root stays stable across frames; children are updated incrementally.
        let mut taffy: TaffyTree<Option<NodeId>> = TaffyTree::new();
        let root = taffy.new_leaf(TaffyStyle::default()).expect("taffy root");
        Self {
            children: Vec::new(),
            taffy,
            root,
            child_nodes: Vec::new(),
            node_by_child: HashMap::new(),
            root_style: None,
            child_styles: HashMap::new(),
            measure_cache: std::collections::HashMap::new(),
        }
    }
}

impl TaffyContainerCache {
    pub(super) fn sync_root_style(&mut self, root_style: TaffyStyle) {
        if self.root_style.as_ref() == Some(&root_style) {
            return;
        }
        self.taffy
            .set_style(self.root, root_style.clone())
            .expect("taffy root style");
        self.root_style = Some(root_style);
    }

    pub(super) fn sync_children(
        &mut self,
        children: &[NodeId],
        mut style_for_child: impl FnMut(NodeId) -> TaffyStyle,
    ) {
        let children_changed = self.children != children;

        if children_changed {
            let keep: std::collections::HashSet<NodeId> = children.iter().copied().collect();
            let removed: Vec<NodeId> = self
                .node_by_child
                .keys()
                .copied()
                .filter(|child| !keep.contains(child))
                .collect();

            for child in removed {
                let Some(node) = self.node_by_child.remove(&child) else {
                    continue;
                };
                self.child_styles.remove(&child);
                self.taffy.remove(node).expect("taffy remove");
            }

            self.children = children.to_vec();
        }

        self.child_nodes.clear();
        self.child_nodes.reserve(children.len());
        for &child in children {
            let node = if let Some(&node) = self.node_by_child.get(&child) {
                node
            } else {
                let node = self
                    .taffy
                    .new_leaf_with_context(TaffyStyle::default(), Some(child))
                    .expect("taffy leaf");
                self.node_by_child.insert(child, node);
                node
            };
            self.child_nodes.push(node);

            let style = style_for_child(child);
            let style_changed = self.child_styles.get(&child) != Some(&style);
            if style_changed {
                self.taffy
                    .set_style(node, style.clone())
                    .expect("taffy child style");
                self.child_styles.insert(child, style);
            }
        }

        if children_changed {
            self.taffy
                .set_children(self.root, &self.child_nodes)
                .expect("taffy set children");
        }
    }
}

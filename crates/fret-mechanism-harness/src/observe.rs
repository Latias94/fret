use std::collections::{HashMap, HashSet};

use fret_core::{NodeId, Point, Rect, SemanticsRole, SemanticsSnapshot};
use fret_diag_protocol::UiSelectorV1;
use serde::{Deserialize, Serialize};
use slotmap::Key as _;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundsSpace {
    #[default]
    Layout,
    Visual,
    Hit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedTree {
    pub window_id: Option<u64>,
    pub window_bounds: Rect,
    #[serde(default)]
    pub roots: Vec<ObservedRoot>,
    #[serde(default)]
    pub nodes: Vec<ObservedNode>,
    #[serde(default)]
    pub hit_tests: Vec<ObservedHitTestSample>,
    #[serde(default)]
    pub overlays: Vec<ObservedOverlay>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<ObservedMechanismMetric>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barrier_root_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_barrier_root_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_node_id: Option<u64>,
}

impl ObservedTree {
    pub fn new(window_bounds: Rect) -> Self {
        Self {
            window_id: None,
            window_bounds,
            roots: Vec::new(),
            nodes: Vec::new(),
            hit_tests: Vec::new(),
            overlays: Vec::new(),
            metrics: Vec::new(),
            focus_node_id: None,
            barrier_root_node_id: None,
            focus_barrier_root_node_id: None,
            captured_node_id: None,
        }
    }

    pub fn from_semantics_snapshot(snapshot: &SemanticsSnapshot, window_bounds: Rect) -> Self {
        let mut children: HashMap<u64, Vec<u64>> = HashMap::new();
        for node in &snapshot.nodes {
            if let Some(parent) = node.parent {
                children
                    .entry(parent.data().as_ffi())
                    .or_default()
                    .push(node.id.data().as_ffi());
            }
        }

        let mut root_z: HashMap<u64, u32> = HashMap::new();
        let mut visible_ids: HashSet<u64> = HashSet::new();
        let roots = snapshot
            .roots
            .iter()
            .map(|root| {
                let node_id = root.root.data().as_ffi();
                root_z.insert(node_id, root.z_index);
                if root.visible {
                    collect_subtree_ids(node_id, &children, &mut visible_ids);
                }
                ObservedRoot {
                    node_id,
                    visible: root.visible,
                    blocks_underlay_input: root.blocks_underlay_input,
                    hit_testable: root.hit_testable,
                    z_index: root.z_index,
                }
            })
            .collect::<Vec<_>>();

        let all_visible = roots.is_empty();
        let mut tree = Self {
            window_id: Some(snapshot.window.data().as_ffi()),
            window_bounds,
            roots,
            nodes: Vec::with_capacity(snapshot.nodes.len()),
            hit_tests: Vec::new(),
            overlays: Vec::new(),
            metrics: Vec::new(),
            focus_node_id: snapshot.focus.map(|id| id.data().as_ffi()),
            barrier_root_node_id: snapshot.barrier_root.map(|id| id.data().as_ffi()),
            focus_barrier_root_node_id: snapshot.focus_barrier_root.map(|id| id.data().as_ffi()),
            captured_node_id: snapshot.captured.map(|id| id.data().as_ffi()),
        };

        let parent_by_id = snapshot
            .nodes
            .iter()
            .map(|node| {
                (
                    node.id.data().as_ffi(),
                    node.parent.map(|parent| parent.data().as_ffi()),
                )
            })
            .collect::<HashMap<_, _>>();

        for node in &snapshot.nodes {
            let node_id = node.id.data().as_ffi();
            tree.nodes.push(ObservedNode {
                node_id: Some(node_id),
                parent_node_id: node.parent.map(|id| id.data().as_ffi()),
                global_element_id: None,
                root_z_index: Some(root_z_for(node_id, &parent_by_id, &root_z)),
                test_id: node.test_id.clone(),
                role: Some(role_label(node.role).to_string()),
                label: node.label.clone(),
                bounds: node.bounds,
                visual_bounds: None,
                hit_bounds: None,
                visible: all_visible || visible_ids.contains(&node_id),
                hit_testable: true,
                focusable: None,
                active_descendant_node_id: node.active_descendant.map(|id| id.data().as_ffi()),
                labelled_by_node_ids: node
                    .labelled_by
                    .iter()
                    .map(|id| id.data().as_ffi())
                    .collect(),
                described_by_node_ids: node
                    .described_by
                    .iter()
                    .map(|id| id.data().as_ffi())
                    .collect(),
                controls_node_ids: node.controls.iter().map(|id| id.data().as_ffi()).collect(),
                disabled: Some(node.flags.disabled),
                hidden: Some(node.flags.hidden),
            });
        }

        tree
    }

    pub fn push_node(&mut self, node: ObservedNode) {
        self.nodes.push(node);
    }

    pub fn set_layout_bounds_for_node_id(&mut self, node_id: u64, bounds: Rect) -> bool {
        let Some(node) = self
            .nodes
            .iter_mut()
            .find(|node| node.node_id == Some(node_id))
        else {
            return false;
        };
        node.bounds = bounds;
        true
    }

    pub fn set_test_id_for_node_id(&mut self, node_id: u64, test_id: impl Into<String>) -> bool {
        let Some(node) = self
            .nodes
            .iter_mut()
            .find(|node| node.node_id == Some(node_id))
        else {
            return false;
        };
        node.test_id = Some(test_id.into());
        true
    }

    pub fn set_visual_bounds_for_test_id(&mut self, test_id: &str, bounds: Rect) -> bool {
        self.set_space_bounds_for_test_id(test_id, BoundsSpace::Visual, bounds)
    }

    pub fn set_hit_bounds_for_test_id(&mut self, test_id: &str, bounds: Rect) -> bool {
        self.set_space_bounds_for_test_id(test_id, BoundsSpace::Hit, bounds)
    }

    pub fn set_space_bounds_for_test_id(
        &mut self,
        test_id: &str,
        space: BoundsSpace,
        bounds: Rect,
    ) -> bool {
        let Some(node) = self
            .nodes
            .iter_mut()
            .find(|node| node.test_id.as_deref() == Some(test_id))
        else {
            return false;
        };
        match space {
            BoundsSpace::Layout => node.bounds = bounds,
            BoundsSpace::Visual => node.visual_bounds = Some(bounds),
            BoundsSpace::Hit => node.hit_bounds = Some(bounds),
        }
        true
    }

    pub fn push_hit_test_sample(&mut self, sample: ObservedHitTestSample) {
        self.hit_tests.push(sample);
    }

    pub fn set_metric(&mut self, id: impl Into<String>, value: f32) {
        let id = id.into();
        if let Some(metric) = self.metrics.iter_mut().find(|metric| metric.id == id) {
            metric.value = value;
        } else {
            self.metrics.push(ObservedMechanismMetric { id, value });
        }
    }

    pub fn metric_value(&self, id: &str) -> Result<f32, QueryError> {
        self.metrics
            .iter()
            .find(|metric| metric.id == id)
            .map(|metric| metric.value)
            .ok_or_else(|| QueryError::NoMetric { id: id.to_string() })
    }

    pub fn select<'a>(&'a self, selector: &UiSelectorV1) -> Vec<&'a ObservedNode> {
        self.select_with_filter(selector, true)
    }

    pub fn select_unfiltered<'a>(&'a self, selector: &UiSelectorV1) -> Vec<&'a ObservedNode> {
        self.select_with_filter(selector, false)
    }

    fn select_with_filter<'a>(
        &'a self,
        selector: &UiSelectorV1,
        apply_barrier_filter: bool,
    ) -> Vec<&'a ObservedNode> {
        let mut matches = self
            .nodes
            .iter()
            .filter(|node| !apply_barrier_filter || self.node_is_selectable(node))
            .filter(|node| self.matches_selector(node, selector))
            .collect::<Vec<_>>();
        matches.sort_by_key(|node| {
            (
                node.root_z_index.unwrap_or(0),
                self.depth_for(node.node_id.unwrap_or_default()),
            )
        });
        matches.reverse();
        matches
    }

    pub fn select_best<'a>(
        &'a self,
        selector: &UiSelectorV1,
    ) -> Result<&'a ObservedNode, QueryError> {
        self.select(selector)
            .into_iter()
            .next()
            .ok_or_else(|| QueryError::NoMatch {
                selector: format!("{selector:?}"),
            })
    }

    pub fn select_best_unfiltered<'a>(
        &'a self,
        selector: &UiSelectorV1,
    ) -> Result<&'a ObservedNode, QueryError> {
        self.select_unfiltered(selector)
            .into_iter()
            .next()
            .ok_or_else(|| QueryError::NoMatch {
                selector: format!("{selector:?}"),
            })
    }

    pub fn select_under<'a>(
        &'a self,
        scope: &UiSelectorV1,
        selector: &UiSelectorV1,
    ) -> Vec<&'a ObservedNode> {
        let Ok(scope) = self.select_best(scope) else {
            return Vec::new();
        };
        let Some(scope_id) = scope.node_id else {
            return Vec::new();
        };
        self.select(selector)
            .into_iter()
            .filter(|node| {
                node.node_id
                    .map(|id| self.is_descendant_of_or_self(id, scope_id))
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn bounds_for(
        &self,
        selector: &UiSelectorV1,
        space: BoundsSpace,
    ) -> Result<Rect, QueryError> {
        self.select_best(selector).map(|node| node.bounds_in(space))
    }

    pub fn hit_sample(&self, sample_id: &str) -> Option<&ObservedHitTestSample> {
        self.hit_tests
            .iter()
            .find(|sample| sample.id.as_str() == sample_id)
    }

    pub fn focus_node(&self) -> Option<&ObservedNode> {
        let focus = self.focus_node_id?;
        self.nodes.iter().find(|node| node.node_id == Some(focus))
    }

    pub fn overlay(&self, id: &str) -> Option<&ObservedOverlay> {
        self.overlays.iter().find(|overlay| overlay.id == id)
    }

    pub fn overlay_bounds_for(&self, id: &str) -> Result<Rect, QueryError> {
        let overlay = self
            .overlay(id)
            .ok_or_else(|| QueryError::NoOverlay { id: id.to_string() })?;
        overlay
            .bounds
            .ok_or_else(|| QueryError::OverlayMissingBounds { id: id.to_string() })
    }

    fn matches_selector(&self, node: &ObservedNode, selector: &UiSelectorV1) -> bool {
        if !matches_selector_root_z(node, selector) {
            return false;
        }

        match selector {
            UiSelectorV1::TestId { id, .. } => node.test_id.as_deref() == Some(id.as_str()),
            UiSelectorV1::RoleAndName { role, name, .. } => {
                node.role.as_deref() == Some(role.as_str())
                    && node.label.as_deref() == Some(name.as_str())
            }
            UiSelectorV1::RoleAndPath {
                role,
                name,
                ancestors,
                ..
            } => {
                node.role.as_deref() == Some(role.as_str())
                    && node.label.as_deref() == Some(name.as_str())
                    && node
                        .node_id
                        .map(|id| self.ancestors_match(id, ancestors))
                        .unwrap_or(false)
            }
            UiSelectorV1::NodeId { node: id, .. } => node.node_id == Some(*id),
            UiSelectorV1::GlobalElementId { element, .. } => {
                node.global_element_id == Some(*element)
            }
        }
    }

    fn ancestors_match(
        &self,
        node_id: u64,
        ancestors: &[fret_diag_protocol::UiRoleAndNameV1],
    ) -> bool {
        let mut cur = self
            .nodes
            .iter()
            .find(|node| node.node_id == Some(node_id))
            .and_then(|node| node.parent_node_id);

        for want in ancestors.iter().rev() {
            let mut found = false;
            while let Some(id) = cur {
                let Some(node) = self.nodes.iter().find(|node| node.node_id == Some(id)) else {
                    break;
                };
                if node.role.as_deref() == Some(want.role.as_str())
                    && node.label.as_deref() == Some(want.name.as_str())
                {
                    found = true;
                    cur = node.parent_node_id;
                    break;
                }
                cur = node.parent_node_id;
            }
            if !found {
                return false;
            }
        }

        true
    }

    fn node_is_selectable(&self, node: &ObservedNode) -> bool {
        if !node.visible {
            return false;
        }
        let Some(barrier) = self.barrier_root_node_id else {
            return true;
        };
        node.node_id
            .map(|id| self.is_descendant_of_or_self(id, barrier))
            .unwrap_or(false)
    }

    fn is_descendant_of_or_self(&self, mut id: u64, ancestor: u64) -> bool {
        if id == ancestor {
            return true;
        }
        while let Some(node) = self.nodes.iter().find(|node| node.node_id == Some(id)) {
            let Some(parent) = node.parent_node_id else {
                return false;
            };
            if parent == ancestor {
                return true;
            }
            id = parent;
        }
        false
    }

    fn depth_for(&self, mut id: u64) -> u32 {
        let mut depth = 0u32;
        while let Some(node) = self.nodes.iter().find(|node| node.node_id == Some(id)) {
            let Some(parent) = node.parent_node_id else {
                break;
            };
            depth = depth.saturating_add(1);
            id = parent;
        }
        depth
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedRoot {
    pub node_id: u64,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    pub z_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedNode {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_element_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_z_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub bounds: Rect,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_bounds: Option<Rect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_bounds: Option<Rect>,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_true")]
    pub hit_testable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focusable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_descendant_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labelled_by_node_ids: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub described_by_node_ids: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls_node_ids: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
}

impl ObservedNode {
    pub fn new(test_id: impl Into<String>, bounds: Rect) -> Self {
        Self {
            node_id: None,
            parent_node_id: None,
            global_element_id: None,
            root_z_index: None,
            test_id: Some(test_id.into()),
            role: None,
            label: None,
            bounds,
            visual_bounds: None,
            hit_bounds: None,
            visible: true,
            hit_testable: true,
            focusable: None,
            active_descendant_node_id: None,
            labelled_by_node_ids: Vec::new(),
            described_by_node_ids: Vec::new(),
            controls_node_ids: Vec::new(),
            disabled: None,
            hidden: None,
        }
    }

    pub fn bounds_in(&self, space: BoundsSpace) -> Rect {
        match space {
            BoundsSpace::Layout => self.bounds,
            BoundsSpace::Visual => self.visual_bounds.unwrap_or(self.bounds),
            BoundsSpace::Hit => self
                .hit_bounds
                .or(self.visual_bounds)
                .unwrap_or(self.bounds),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedSemanticsRelation {
    ActiveDescendant,
    LabelledBy,
    DescribedBy,
    Controls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedSemanticsFlag {
    Disabled,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedHitTestSample {
    pub id: String,
    pub point: Point,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barrier_root_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_layer_root_node_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedOverlay {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<UiSelectorV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub panel: Option<UiSelectorV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Rect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedMechanismMetric {
    pub id: String,
    pub value: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("selector did not match any observed node: {selector}")]
    NoMatch { selector: String },
    #[error("observed overlay did not exist: {id}")]
    NoOverlay { id: String },
    #[error("observed overlay did not include bounds: {id}")]
    OverlayMissingBounds { id: String },
    #[error("observed mechanism metric did not exist: {id}")]
    NoMetric { id: String },
}

pub fn role_label(role: SemanticsRole) -> &'static str {
    match role {
        SemanticsRole::Generic => "generic",
        SemanticsRole::Window => "window",
        SemanticsRole::Panel => "panel",
        SemanticsRole::Group => "group",
        SemanticsRole::Toolbar => "toolbar",
        SemanticsRole::Heading => "heading",
        SemanticsRole::Dialog => "dialog",
        SemanticsRole::AlertDialog => "alert_dialog",
        SemanticsRole::Alert => "alert",
        SemanticsRole::Status => "status",
        SemanticsRole::Log => "log",
        SemanticsRole::Button => "button",
        SemanticsRole::Link => "link",
        SemanticsRole::Image => "image",
        SemanticsRole::Checkbox => "checkbox",
        SemanticsRole::Switch => "switch",
        SemanticsRole::Slider => "slider",
        SemanticsRole::ProgressBar => "progress_bar",
        SemanticsRole::ScrollBar => "scroll_bar",
        SemanticsRole::ComboBox => "combo_box",
        SemanticsRole::RadioGroup => "radio_group",
        SemanticsRole::RadioButton => "radio_button",
        SemanticsRole::TabList => "tab_list",
        SemanticsRole::Tab => "tab",
        SemanticsRole::TabPanel => "tab_panel",
        SemanticsRole::MenuBar => "menu_bar",
        SemanticsRole::Menu => "menu",
        SemanticsRole::MenuItem => "menu_item",
        SemanticsRole::MenuItemCheckbox => "menu_item_checkbox",
        SemanticsRole::MenuItemRadio => "menu_item_radio",
        SemanticsRole::Tooltip => "tooltip",
        SemanticsRole::Text => "text",
        SemanticsRole::TextField => "text_field",
        SemanticsRole::List => "list",
        SemanticsRole::ListItem => "list_item",
        SemanticsRole::Separator => "separator",
        SemanticsRole::ListBox => "list_box",
        SemanticsRole::ListBoxOption => "list_box_option",
        SemanticsRole::TreeItem => "tree_item",
        SemanticsRole::Viewport => "viewport",
        _ => "unknown",
    }
}

fn matches_selector_root_z(node: &ObservedNode, selector: &UiSelectorV1) -> bool {
    let want = match selector {
        UiSelectorV1::RoleAndName { root_z_index, .. }
        | UiSelectorV1::RoleAndPath { root_z_index, .. }
        | UiSelectorV1::TestId { root_z_index, .. }
        | UiSelectorV1::GlobalElementId { root_z_index, .. }
        | UiSelectorV1::NodeId { root_z_index, .. } => *root_z_index,
    };
    want.map(|z| node.root_z_index.unwrap_or(0) == z)
        .unwrap_or(true)
}

fn root_z_for(
    id: u64,
    parent_by_id: &HashMap<u64, Option<u64>>,
    root_z: &HashMap<u64, u32>,
) -> u32 {
    let mut cur = Some(id);
    while let Some(node_id) = cur {
        if let Some(z) = root_z.get(&node_id).copied() {
            return z;
        }
        cur = parent_by_id.get(&node_id).and_then(|parent| *parent);
    }
    0
}

fn collect_subtree_ids(root: u64, children: &HashMap<u64, Vec<u64>>, out: &mut HashSet<u64>) {
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        if !out.insert(id) {
            continue;
        }
        if let Some(kids) = children.get(&id) {
            stack.extend(kids.iter().copied());
        }
    }
}

fn default_true() -> bool {
    true
}

#[allow(dead_code)]
fn node_id_to_u64(node: NodeId) -> u64 {
    node.data().as_ffi()
}

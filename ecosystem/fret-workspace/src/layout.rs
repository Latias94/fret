use std::sync::Arc;

use fret_core::Axis;
use fret_runtime::CommandId;

use crate::close_policy::WorkspaceDirtyClosePolicy;
use crate::tabs::WorkspaceApplyCommandOutcome;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const DEFAULT_PANE_RESIZE_STEP_FRACTION: f32 = 0.05;

/// A lightweight "workspace shell" layout that can be persisted by apps.
///
/// This intentionally does not embed docking internals:
/// - Dock layout persistence is covered by docking contracts (ADR 0013).
/// - Workspace layout focuses on editor chrome: document groups (tabs) and pane splits.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceLayout {
    pub windows: Vec<WorkspaceWindowLayout>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceWindowLayout {
    pub id: Arc<str>,
    pub pane_tree: WorkspacePaneTree,
    pub active_pane: Option<Arc<str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitSide {
    First,
    Second,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitMode {
    /// Create a new pane without cloning any tab state.
    Empty,
    /// Clone the active tab (if any) into the new pane.
    CloneActiveTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl Rect {
    fn unit() -> Self {
        Self {
            x0: 0.0,
            y0: 0.0,
            x1: 1.0,
            y1: 1.0,
        }
    }

    fn center_x(self) -> f32 {
        (self.x0 + self.x1) * 0.5
    }

    fn center_y(self) -> f32 {
        (self.y0 + self.y1) * 0.5
    }

    fn interval_gap(a0: f32, a1: f32, b0: f32, b1: f32) -> f32 {
        let left = a0 - b1;
        let right = b0 - a1;
        left.max(right).max(0.0)
    }

    fn score_neighbor(self, dir: PaneDirection, other: Rect) -> Option<(f32, f32, f32)> {
        match dir {
            PaneDirection::Left => {
                let dx = self.x0 - other.x1;
                if dx < 0.0 {
                    return None;
                }
                let gap = Rect::interval_gap(self.y0, self.y1, other.y0, other.y1);
                let align = (self.center_y() - other.center_y()).abs();
                Some((dx, gap, align))
            }
            PaneDirection::Right => {
                let dx = other.x0 - self.x1;
                if dx < 0.0 {
                    return None;
                }
                let gap = Rect::interval_gap(self.y0, self.y1, other.y0, other.y1);
                let align = (self.center_y() - other.center_y()).abs();
                Some((dx, gap, align))
            }
            PaneDirection::Up => {
                let dy = self.y0 - other.y1;
                if dy < 0.0 {
                    return None;
                }
                let gap = Rect::interval_gap(self.x0, self.x1, other.x0, other.x1);
                let align = (self.center_x() - other.center_x()).abs();
                Some((dy, gap, align))
            }
            PaneDirection::Down => {
                let dy = other.y0 - self.y1;
                if dy < 0.0 {
                    return None;
                }
                let gap = Rect::interval_gap(self.x0, self.x1, other.x0, other.x1);
                let align = (self.center_x() - other.center_x()).abs();
                Some((dy, gap, align))
            }
        }
    }
}

impl WorkspaceWindowLayout {
    pub fn new(id: impl Into<Arc<str>>, root_pane_id: impl Into<Arc<str>>) -> Self {
        let root_pane_id: Arc<str> = root_pane_id.into();
        Self {
            id: id.into(),
            pane_tree: WorkspacePaneTree::leaf(root_pane_id.clone()),
            active_pane: Some(root_pane_id),
        }
    }

    pub fn active_pane_id(&self) -> Option<&Arc<str>> {
        self.active_pane.as_ref()
    }

    pub fn activate_pane(&mut self, id: &str) -> bool {
        if self.pane_tree.find_pane(id).is_none() {
            return false;
        }
        self.active_pane = Some(Arc::<str>::from(id));
        true
    }

    pub fn split_active_pane(
        &mut self,
        axis: Axis,
        side: SplitSide,
        fraction: f32,
        new_pane_id: impl Into<Arc<str>>,
    ) -> bool {
        let Some(active) = self.active_pane.clone() else {
            return false;
        };
        let new_pane_id: Arc<str> = new_pane_id.into();
        if active.as_ref() == new_pane_id.as_ref() {
            return false;
        }

        if self.pane_tree.find_pane(new_pane_id.as_ref()).is_some() {
            return false;
        }

        let ok = self.pane_tree.split_leaf(
            active.as_ref(),
            axis,
            fraction,
            side,
            WorkspacePaneTree::leaf(new_pane_id.clone()),
        );
        if ok {
            self.active_pane = Some(new_pane_id);
        }
        ok
    }

    pub fn split_active_pane_auto(
        &mut self,
        axis: Axis,
        side: SplitSide,
        fraction: f32,
        mode: SplitMode,
    ) -> bool {
        if self.active_pane.is_none() {
            self.active_pane = self.pane_tree.first_leaf_id().cloned();
        }

        let Some(active) = self.active_pane.clone() else {
            return false;
        };

        let active_tab = self
            .pane_tree
            .find_pane(active.as_ref())
            .and_then(|p| p.tabs.active().cloned());

        let new_pane_id = self.generate_next_pane_id();
        let ok = self.split_active_pane(axis, side, fraction, new_pane_id.clone());
        if !ok {
            return false;
        }

        if mode == SplitMode::CloneActiveTab {
            if let Some(tab) = active_tab {
                if let Some(pane) = self.pane_tree.find_pane_mut(new_pane_id.as_ref()) {
                    pane.tabs.open_and_activate(tab);
                }
            }
        }

        true
    }

    pub fn generate_next_pane_id(&self) -> Arc<str> {
        let prefix = format!("{}.pane.", self.id);
        for i in 1u32.. {
            let candidate = format!("{prefix}{i}");
            if self.pane_tree.find_pane(&candidate).is_none() {
                return Arc::<str>::from(candidate);
            }
        }
        unreachable!("u32 pane id counter exhausted")
    }

    pub fn focus_next_pane(&mut self) -> bool {
        self.focus_adjacent_pane(true)
    }

    pub fn focus_prev_pane(&mut self) -> bool {
        self.focus_adjacent_pane(false)
    }

    pub fn focus_pane_in_direction(&mut self, dir: PaneDirection) -> bool {
        if self.active_pane.is_none() {
            self.active_pane = self.pane_tree.first_leaf_id().cloned();
        }
        let Some(active) = self.active_pane.clone() else {
            return false;
        };

        let Some(target) = self.pane_tree.find_neighbor_leaf(active.as_ref(), dir) else {
            return false;
        };
        if target.as_ref() == active.as_ref() {
            return false;
        }
        self.active_pane = Some(target);
        true
    }

    pub fn move_active_tab_to_direction(&mut self, dir: PaneDirection) -> bool {
        if self.active_pane.is_none() {
            self.active_pane = self.pane_tree.first_leaf_id().cloned();
        }
        let Some(active) = self.active_pane.clone() else {
            return false;
        };

        let Some(target) = self.pane_tree.find_neighbor_leaf(active.as_ref(), dir) else {
            return false;
        };
        if target.as_ref() == active.as_ref() {
            return false;
        }
        self.move_active_tab_to_pane(target.as_ref())
    }

    fn focus_adjacent_pane(&mut self, forward: bool) -> bool {
        let mut ids: Vec<Arc<str>> = Vec::new();
        self.pane_tree.collect_leaf_ids(&mut ids);
        if ids.is_empty() {
            return false;
        }

        let active = self.active_pane.clone();
        let next = match active {
            None => ids[0].clone(),
            Some(active) => {
                let index = ids
                    .iter()
                    .position(|id| id.as_ref() == active.as_ref())
                    .unwrap_or(0);
                let next = if forward {
                    ids[(index + 1) % ids.len()].clone()
                } else {
                    ids[(index + ids.len() - 1) % ids.len()].clone()
                };
                next
            }
        };

        self.active_pane = Some(next);
        true
    }

    pub fn move_active_tab_to_pane(&mut self, target_pane_id: &str) -> bool {
        let Some(source_id) = self.active_pane.clone() else {
            return false;
        };
        if source_id.as_ref() == target_pane_id {
            return false;
        }

        let Some(active_tab) = self
            .pane_tree
            .find_pane(source_id.as_ref())
            .and_then(|p| p.tabs.active().cloned())
        else {
            return false;
        };

        let Some(source) = self.pane_tree.find_pane_mut(source_id.as_ref()) else {
            return false;
        };
        if !source.tabs.close(active_tab.as_ref()) {
            return false;
        }

        let Some(target) = self.pane_tree.find_pane_mut(target_pane_id) else {
            return false;
        };
        target.tabs.open_and_activate(active_tab);
        self.active_pane = Some(target.id.clone());
        true
    }

    pub fn move_active_tab_to_next_pane(&mut self) -> bool {
        self.move_active_tab_to_adjacent_pane(true)
    }

    pub fn move_active_tab_to_prev_pane(&mut self) -> bool {
        self.move_active_tab_to_adjacent_pane(false)
    }

    fn move_active_tab_to_adjacent_pane(&mut self, forward: bool) -> bool {
        if self.active_pane.is_none() {
            self.active_pane = self.pane_tree.first_leaf_id().cloned();
        }

        let Some(active) = self.active_pane.clone() else {
            return false;
        };

        let mut ids: Vec<Arc<str>> = Vec::new();
        self.pane_tree.collect_leaf_ids(&mut ids);
        if ids.len() < 2 {
            return false;
        }

        let index = ids
            .iter()
            .position(|id| id.as_ref() == active.as_ref())
            .unwrap_or(0);
        let target = if forward {
            ids[(index + 1) % ids.len()].clone()
        } else {
            ids[(index + ids.len() - 1) % ids.len()].clone()
        };

        self.move_active_tab_to_pane(target.as_ref())
    }

    pub fn resize_active_pane(&mut self, axis: Axis, delta_fraction: f32) -> bool {
        if self.active_pane.is_none() {
            self.active_pane = self.pane_tree.first_leaf_id().cloned();
        }
        let Some(active) = self.active_pane.clone() else {
            return false;
        };

        self.pane_tree
            .resize_leaf_nearest_split(active.as_ref(), axis, delta_fraction)
    }

    pub fn active_pane_mut(&mut self) -> Option<&mut WorkspacePaneLayout> {
        let active = self.active_pane.clone()?;
        self.pane_tree.find_pane_mut(active.as_ref())
    }

    pub fn apply_command_with_close_policy(
        &mut self,
        command: &CommandId,
        policy: Option<&mut dyn WorkspaceDirtyClosePolicy>,
    ) -> WorkspaceApplyCommandOutcome {
        match command.as_str() {
            crate::commands::CMD_WORKSPACE_PANE_NEXT => {
                return WorkspaceApplyCommandOutcome::applied(self.focus_next_pane());
            }
            crate::commands::CMD_WORKSPACE_PANE_PREV => {
                return WorkspaceApplyCommandOutcome::applied(self.focus_prev_pane());
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_NEXT => {
                return WorkspaceApplyCommandOutcome::applied(self.move_active_tab_to_next_pane());
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_PREV => {
                return WorkspaceApplyCommandOutcome::applied(self.move_active_tab_to_prev_pane());
            }
            crate::commands::CMD_WORKSPACE_PANE_RESIZE_RIGHT => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.resize_active_pane(Axis::Horizontal, DEFAULT_PANE_RESIZE_STEP_FRACTION),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_RESIZE_LEFT => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.resize_active_pane(Axis::Horizontal, -DEFAULT_PANE_RESIZE_STEP_FRACTION),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_RESIZE_UP => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.resize_active_pane(Axis::Vertical, DEFAULT_PANE_RESIZE_STEP_FRACTION),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_RESIZE_DOWN => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.resize_active_pane(Axis::Vertical, -DEFAULT_PANE_RESIZE_STEP_FRACTION),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_RIGHT => {
                return WorkspaceApplyCommandOutcome::applied(self.split_active_pane_auto(
                    Axis::Horizontal,
                    SplitSide::Second,
                    0.5,
                    SplitMode::CloneActiveTab,
                ));
            }
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_LEFT => {
                return WorkspaceApplyCommandOutcome::applied(self.split_active_pane_auto(
                    Axis::Horizontal,
                    SplitSide::First,
                    0.5,
                    SplitMode::CloneActiveTab,
                ));
            }
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_UP => {
                return WorkspaceApplyCommandOutcome::applied(self.split_active_pane_auto(
                    Axis::Vertical,
                    SplitSide::First,
                    0.5,
                    SplitMode::CloneActiveTab,
                ));
            }
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_DOWN => {
                return WorkspaceApplyCommandOutcome::applied(self.split_active_pane_auto(
                    Axis::Vertical,
                    SplitSide::Second,
                    0.5,
                    SplitMode::CloneActiveTab,
                ));
            }
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_LEFT => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.focus_pane_in_direction(PaneDirection::Left),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_RIGHT => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.focus_pane_in_direction(PaneDirection::Right),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_UP => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.focus_pane_in_direction(PaneDirection::Up),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_DOWN => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.focus_pane_in_direction(PaneDirection::Down),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_LEFT => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.move_active_tab_to_direction(PaneDirection::Left),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_RIGHT => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.move_active_tab_to_direction(PaneDirection::Right),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_UP => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.move_active_tab_to_direction(PaneDirection::Up),
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_DOWN => {
                return WorkspaceApplyCommandOutcome::applied(
                    self.move_active_tab_to_direction(PaneDirection::Down),
                );
            }
            _ => {}
        }

        if let Some(id) = command
            .as_str()
            .strip_prefix(crate::commands::CMD_WORKSPACE_PANE_ACTIVATE_PREFIX)
        {
            let id = id.trim();
            if id.is_empty() {
                return WorkspaceApplyCommandOutcome::applied(false);
            }
            return WorkspaceApplyCommandOutcome::applied(self.activate_pane(id));
        }

        if let Some(pane_id) = command
            .as_str()
            .strip_prefix(crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_TO_PREFIX)
        {
            let pane_id = pane_id.trim();
            if pane_id.is_empty() {
                return WorkspaceApplyCommandOutcome::applied(false);
            }
            return WorkspaceApplyCommandOutcome::applied(self.move_active_tab_to_pane(pane_id));
        }

        if let Some(rest) = command
            .as_str()
            .strip_prefix(crate::commands::CMD_WORKSPACE_PANE_SPLIT_PREFIX)
        {
            let rest = rest.trim();
            let mut parts = rest.splitn(3, '.');
            let axis = parts.next().unwrap_or_default();
            let side = parts.next().unwrap_or_default();
            let new_pane_id = parts.next().unwrap_or_default().trim();
            if new_pane_id.is_empty() {
                return WorkspaceApplyCommandOutcome::applied(false);
            }

            let axis = match axis {
                "horizontal" => Axis::Horizontal,
                "vertical" => Axis::Vertical,
                _ => return WorkspaceApplyCommandOutcome::applied(false),
            };
            let side = match side {
                "first" => SplitSide::First,
                "second" => SplitSide::Second,
                _ => return WorkspaceApplyCommandOutcome::applied(false),
            };

            return WorkspaceApplyCommandOutcome::applied(self.split_active_pane(
                axis,
                side,
                0.5,
                Arc::<str>::from(new_pane_id),
            ));
        }

        if self.active_pane.is_none() {
            self.active_pane = self.pane_tree.first_leaf_id().cloned();
        }

        match self.active_pane_mut() {
            Some(pane) => pane.tabs.apply_command_with_close_policy(command, policy),
            None => WorkspaceApplyCommandOutcome::applied(false),
        }
    }

    pub fn apply_command(&mut self, command: &CommandId) -> bool {
        match command.as_str() {
            crate::commands::CMD_WORKSPACE_PANE_NEXT => return self.focus_next_pane(),
            crate::commands::CMD_WORKSPACE_PANE_PREV => return self.focus_prev_pane(),
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_NEXT => {
                return self.move_active_tab_to_next_pane();
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_PREV => {
                return self.move_active_tab_to_prev_pane();
            }
            crate::commands::CMD_WORKSPACE_PANE_RESIZE_RIGHT => {
                return self
                    .resize_active_pane(Axis::Horizontal, DEFAULT_PANE_RESIZE_STEP_FRACTION);
            }
            crate::commands::CMD_WORKSPACE_PANE_RESIZE_LEFT => {
                return self
                    .resize_active_pane(Axis::Horizontal, -DEFAULT_PANE_RESIZE_STEP_FRACTION);
            }
            crate::commands::CMD_WORKSPACE_PANE_RESIZE_UP => {
                return self.resize_active_pane(Axis::Vertical, DEFAULT_PANE_RESIZE_STEP_FRACTION);
            }
            crate::commands::CMD_WORKSPACE_PANE_RESIZE_DOWN => {
                return self.resize_active_pane(Axis::Vertical, -DEFAULT_PANE_RESIZE_STEP_FRACTION);
            }
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_RIGHT => {
                return self.split_active_pane_auto(
                    Axis::Horizontal,
                    SplitSide::Second,
                    0.5,
                    SplitMode::CloneActiveTab,
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_LEFT => {
                return self.split_active_pane_auto(
                    Axis::Horizontal,
                    SplitSide::First,
                    0.5,
                    SplitMode::CloneActiveTab,
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_UP => {
                return self.split_active_pane_auto(
                    Axis::Vertical,
                    SplitSide::First,
                    0.5,
                    SplitMode::CloneActiveTab,
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_DOWN => {
                return self.split_active_pane_auto(
                    Axis::Vertical,
                    SplitSide::Second,
                    0.5,
                    SplitMode::CloneActiveTab,
                );
            }
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_LEFT => {
                return self.focus_pane_in_direction(PaneDirection::Left);
            }
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_RIGHT => {
                return self.focus_pane_in_direction(PaneDirection::Right);
            }
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_UP => {
                return self.focus_pane_in_direction(PaneDirection::Up);
            }
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_DOWN => {
                return self.focus_pane_in_direction(PaneDirection::Down);
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_LEFT => {
                return self.move_active_tab_to_direction(PaneDirection::Left);
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_RIGHT => {
                return self.move_active_tab_to_direction(PaneDirection::Right);
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_UP => {
                return self.move_active_tab_to_direction(PaneDirection::Up);
            }
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_DOWN => {
                return self.move_active_tab_to_direction(PaneDirection::Down);
            }
            _ => {}
        }

        if let Some(id) = command
            .as_str()
            .strip_prefix(crate::commands::CMD_WORKSPACE_PANE_ACTIVATE_PREFIX)
        {
            let id = id.trim();
            if id.is_empty() {
                return false;
            }
            return self.activate_pane(id);
        }

        if let Some(pane_id) = command
            .as_str()
            .strip_prefix(crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_TO_PREFIX)
        {
            let pane_id = pane_id.trim();
            if pane_id.is_empty() {
                return false;
            }
            return self.move_active_tab_to_pane(pane_id);
        }

        if let Some(rest) = command
            .as_str()
            .strip_prefix(crate::commands::CMD_WORKSPACE_PANE_SPLIT_PREFIX)
        {
            let rest = rest.trim();
            let mut parts = rest.splitn(3, '.');
            let axis = parts.next().unwrap_or_default();
            let side = parts.next().unwrap_or_default();
            let new_pane_id = parts.next().unwrap_or_default().trim();
            if new_pane_id.is_empty() {
                return false;
            }

            let axis = match axis {
                "horizontal" => Axis::Horizontal,
                "vertical" => Axis::Vertical,
                _ => return false,
            };
            let side = match side {
                "first" => SplitSide::First,
                "second" => SplitSide::Second,
                _ => return false,
            };

            return self.split_active_pane(axis, side, 0.5, Arc::<str>::from(new_pane_id));
        }

        if self.active_pane.is_none() {
            self.active_pane = self.pane_tree.first_leaf_id().cloned();
        }

        self.active_pane_mut()
            .is_some_and(|pane| pane.tabs.apply_command(command))
    }
}

#[derive(Debug, Clone)]
pub struct WorkspacePaneLayout {
    pub id: Arc<str>,
    pub tabs: crate::tabs::WorkspaceTabs,
}

impl WorkspacePaneLayout {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            tabs: crate::tabs::WorkspaceTabs::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorkspacePaneTree {
    Leaf(WorkspacePaneLayout),
    Split {
        axis: Axis,
        /// Fraction of the available space given to `a`.
        fraction: f32,
        a: Box<WorkspacePaneTree>,
        b: Box<WorkspacePaneTree>,
    },
}

impl WorkspacePaneTree {
    pub fn leaf(id: impl Into<Arc<str>>) -> Self {
        Self::Leaf(WorkspacePaneLayout::new(id))
    }

    pub fn split(axis: Axis, fraction: f32, a: WorkspacePaneTree, b: WorkspacePaneTree) -> Self {
        Self::Split {
            axis,
            fraction: clamp_fraction(fraction),
            a: Box::new(a),
            b: Box::new(b),
        }
    }

    pub fn find_pane(&self, id: &str) -> Option<&WorkspacePaneLayout> {
        match self {
            WorkspacePaneTree::Leaf(pane) => (pane.id.as_ref() == id).then_some(pane),
            WorkspacePaneTree::Split { a, b, .. } => a.find_pane(id).or_else(|| b.find_pane(id)),
        }
    }

    pub fn find_pane_mut(&mut self, id: &str) -> Option<&mut WorkspacePaneLayout> {
        match self {
            WorkspacePaneTree::Leaf(pane) => (pane.id.as_ref() == id).then_some(pane),
            WorkspacePaneTree::Split { a, b, .. } => {
                a.find_pane_mut(id).or_else(|| b.find_pane_mut(id))
            }
        }
    }

    pub fn first_leaf_id(&self) -> Option<&Arc<str>> {
        match self {
            WorkspacePaneTree::Leaf(pane) => Some(&pane.id),
            WorkspacePaneTree::Split { a, .. } => a.first_leaf_id(),
        }
    }

    pub fn collect_leaf_ids(&self, into: &mut Vec<Arc<str>>) {
        match self {
            WorkspacePaneTree::Leaf(pane) => into.push(pane.id.clone()),
            WorkspacePaneTree::Split { a, b, .. } => {
                a.collect_leaf_ids(into);
                b.collect_leaf_ids(into);
            }
        }
    }

    pub fn find_neighbor_leaf(&self, pane_id: &str, dir: PaneDirection) -> Option<Arc<str>> {
        let mut rects: Vec<(Arc<str>, Rect)> = Vec::new();
        self.collect_leaf_rects(Rect::unit(), &mut rects);

        let active_rect = rects
            .iter()
            .find(|(id, _)| id.as_ref() == pane_id)
            .map(|(_, rect)| *rect)?;

        let mut best: Option<(Arc<str>, (f32, f32, f32))> = None;
        for (id, rect) in rects {
            if id.as_ref() == pane_id {
                continue;
            }

            if let Some((primary, secondary, tertiary)) = active_rect.score_neighbor(dir, rect) {
                let candidate = (primary, secondary, tertiary);
                match &best {
                    None => best = Some((id, candidate)),
                    Some((_, current)) => {
                        if candidate < *current {
                            best = Some((id, candidate));
                        }
                    }
                }
            }
        }

        best.map(|(id, _)| id)
    }

    fn collect_leaf_rects(&self, rect: Rect, out: &mut Vec<(Arc<str>, Rect)>) {
        match self {
            WorkspacePaneTree::Leaf(pane) => out.push((pane.id.clone(), rect)),
            WorkspacePaneTree::Split {
                axis,
                fraction,
                a,
                b,
            } => match axis {
                Axis::Horizontal => {
                    let w = rect.x1 - rect.x0;
                    let ax1 = rect.x0 + w * clamp_fraction(*fraction);
                    let a_rect = Rect {
                        x0: rect.x0,
                        y0: rect.y0,
                        x1: ax1,
                        y1: rect.y1,
                    };
                    let b_rect = Rect {
                        x0: ax1,
                        y0: rect.y0,
                        x1: rect.x1,
                        y1: rect.y1,
                    };
                    a.collect_leaf_rects(a_rect, out);
                    b.collect_leaf_rects(b_rect, out);
                }
                Axis::Vertical => {
                    let h = rect.y1 - rect.y0;
                    let ay1 = rect.y0 + h * clamp_fraction(*fraction);
                    let a_rect = Rect {
                        x0: rect.x0,
                        y0: rect.y0,
                        x1: rect.x1,
                        y1: ay1,
                    };
                    let b_rect = Rect {
                        x0: rect.x0,
                        y0: ay1,
                        x1: rect.x1,
                        y1: rect.y1,
                    };
                    a.collect_leaf_rects(a_rect, out);
                    b.collect_leaf_rects(b_rect, out);
                }
            },
        }
    }

    pub fn split_leaf(
        &mut self,
        pane_id: &str,
        axis: Axis,
        fraction: f32,
        side: SplitSide,
        new_subtree: WorkspacePaneTree,
    ) -> bool {
        match self {
            WorkspacePaneTree::Leaf(pane) => {
                if pane.id.as_ref() != pane_id {
                    return false;
                }
                let existing = std::mem::replace(self, WorkspacePaneTree::leaf(""));
                let (a, b) = match side {
                    SplitSide::First => (new_subtree, existing),
                    SplitSide::Second => (existing, new_subtree),
                };
                *self = WorkspacePaneTree::Split {
                    axis,
                    fraction: clamp_fraction(fraction),
                    a: Box::new(a),
                    b: Box::new(b),
                };
                true
            }
            WorkspacePaneTree::Split { a, b, .. } => {
                a.split_leaf(pane_id, axis, fraction, side, new_subtree.clone())
                    || b.split_leaf(pane_id, axis, fraction, side, new_subtree)
            }
        }
    }

    pub fn resize_leaf_nearest_split(
        &mut self,
        pane_id: &str,
        axis: Axis,
        delta_fraction: f32,
    ) -> bool {
        let (_contains, resized) =
            self.resize_leaf_nearest_split_impl(pane_id, axis, delta_fraction);
        resized
    }

    fn resize_leaf_nearest_split_impl(
        &mut self,
        pane_id: &str,
        axis: Axis,
        delta_fraction: f32,
    ) -> (bool, bool) {
        match self {
            WorkspacePaneTree::Leaf(pane) => (pane.id.as_ref() == pane_id, false),
            WorkspacePaneTree::Split {
                axis: node_axis,
                fraction,
                a,
                b,
            } => {
                let (contains_a, resized_a) =
                    a.resize_leaf_nearest_split_impl(pane_id, axis, delta_fraction);
                if contains_a {
                    if resized_a {
                        return (true, true);
                    }
                    if *node_axis == axis {
                        *fraction = clamp_fraction(*fraction + delta_fraction);
                        return (true, true);
                    }
                    return (true, false);
                }

                let (contains_b, resized_b) =
                    b.resize_leaf_nearest_split_impl(pane_id, axis, delta_fraction);
                if contains_b {
                    if resized_b {
                        return (true, true);
                    }
                    if *node_axis == axis {
                        *fraction = clamp_fraction(*fraction - delta_fraction);
                        return (true, true);
                    }
                    return (true, false);
                }

                (false, false)
            }
        }
    }
}

fn clamp_fraction(fraction: f32) -> f32 {
    fraction.clamp(0.05, 0.95)
}

pub const WORKSPACE_LAYOUT_VERSION_V1: u32 = 1;

/// Persistable snapshot of `WorkspaceLayout` (V1).
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkspaceLayoutV1 {
    pub layout_version: u32,
    pub windows: Vec<WorkspaceWindowLayoutV1>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkspaceWindowLayoutV1 {
    pub id: Arc<str>,
    pub pane_tree: WorkspacePaneTreeV1,
    pub active_pane: Option<Arc<str>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkspacePaneLayoutV1 {
    pub id: Arc<str>,
    pub tabs: crate::tabs::WorkspaceTabsV1,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WorkspacePaneTreeV1 {
    Leaf(WorkspacePaneLayoutV1),
    Split {
        axis: Axis,
        fraction: f32,
        a: Box<WorkspacePaneTreeV1>,
        b: Box<WorkspacePaneTreeV1>,
    },
}

impl From<&WorkspaceLayout> for WorkspaceLayoutV1 {
    fn from(value: &WorkspaceLayout) -> Self {
        Self {
            layout_version: WORKSPACE_LAYOUT_VERSION_V1,
            windows: value
                .windows
                .iter()
                .map(|w| WorkspaceWindowLayoutV1 {
                    id: w.id.clone(),
                    pane_tree: WorkspacePaneTreeV1::from(&w.pane_tree),
                    active_pane: w.active_pane.clone(),
                })
                .collect(),
        }
    }
}

impl From<&WorkspacePaneTree> for WorkspacePaneTreeV1 {
    fn from(value: &WorkspacePaneTree) -> Self {
        match value {
            WorkspacePaneTree::Leaf(pane) => WorkspacePaneTreeV1::Leaf(WorkspacePaneLayoutV1 {
                id: pane.id.clone(),
                tabs: pane.tabs.snapshot_v1(),
            }),
            WorkspacePaneTree::Split {
                axis,
                fraction,
                a,
                b,
            } => WorkspacePaneTreeV1::Split {
                axis: *axis,
                fraction: clamp_fraction(*fraction),
                a: Box::new(WorkspacePaneTreeV1::from(a.as_ref())),
                b: Box::new(WorkspacePaneTreeV1::from(b.as_ref())),
            },
        }
    }
}

impl WorkspaceLayoutV1 {
    pub fn into_layout(self) -> WorkspaceLayout {
        let mut windows: Vec<WorkspaceWindowLayout> = self
            .windows
            .into_iter()
            .map(|w| {
                let pane_tree = w.pane_tree.into_tree();
                let mut window = WorkspaceWindowLayout {
                    id: w.id,
                    pane_tree,
                    active_pane: w.active_pane,
                };

                if let Some(active) = window.active_pane.clone() {
                    if window.pane_tree.find_pane(active.as_ref()).is_none() {
                        window.active_pane =
                            window.pane_tree.first_leaf_id().cloned().or(Some(active));
                    }
                } else {
                    window.active_pane = window.pane_tree.first_leaf_id().cloned();
                }

                window
            })
            .collect();

        windows.retain(|w| w.pane_tree.first_leaf_id().is_some());

        WorkspaceLayout { windows }
    }
}

impl WorkspacePaneTreeV1 {
    pub fn into_tree(self) -> WorkspacePaneTree {
        match self {
            WorkspacePaneTreeV1::Leaf(pane) => WorkspacePaneTree::Leaf(WorkspacePaneLayout {
                id: pane.id,
                tabs: crate::tabs::WorkspaceTabs::from_snapshot_v1(pane.tabs),
            }),
            WorkspacePaneTreeV1::Split {
                axis,
                fraction,
                a,
                b,
            } => WorkspacePaneTree::Split {
                axis,
                fraction: clamp_fraction(fraction),
                a: Box::new(a.into_tree()),
                b: Box::new(b.into_tree()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_command_routes_to_active_pane_tabs() {
        let mut window = WorkspaceWindowLayout::new("main", "p1");
        window.pane_tree = WorkspacePaneTree::split(
            Axis::Horizontal,
            0.5,
            WorkspacePaneTree::leaf("p1"),
            WorkspacePaneTree::leaf("p2"),
        );
        window.active_pane = Some(Arc::<str>::from("p2"));

        window
            .pane_tree
            .find_pane_mut("p2")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("a"));
        window
            .pane_tree
            .find_pane_mut("p2")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("b"));

        assert!(window.apply_command(&CommandId::from(crate::commands::CMD_WORKSPACE_TAB_PREV)));
        assert_eq!(
            window
                .pane_tree
                .find_pane("p2")
                .unwrap()
                .tabs
                .active()
                .unwrap()
                .as_ref(),
            "a"
        );
    }

    #[test]
    fn apply_command_handles_activate_pane_prefix() {
        let mut window = WorkspaceWindowLayout::new("main", "p1");
        window.pane_tree = WorkspacePaneTree::split(
            Axis::Horizontal,
            0.5,
            WorkspacePaneTree::leaf("p1"),
            WorkspacePaneTree::leaf("p2"),
        );

        let cmd = crate::commands::pane_activate_command("p2").unwrap();
        assert!(window.apply_command(&cmd));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p2");
    }

    #[test]
    fn split_active_pane_creates_new_pane_and_sets_active() {
        let mut window = WorkspaceWindowLayout::new("main", "p1");
        window
            .pane_tree
            .find_pane_mut("p1")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("a"));

        assert!(window.split_active_pane(Axis::Horizontal, SplitSide::Second, 0.6, "p2"));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p2");
        assert!(window.pane_tree.find_pane("p1").is_some());
        assert!(window.pane_tree.find_pane("p2").is_some());
    }

    #[test]
    fn generate_next_pane_id_skips_existing_leaf_ids() {
        let window = WorkspaceWindowLayout::new("w", "w.pane.1");
        assert_eq!(window.generate_next_pane_id().as_ref(), "w.pane.2");
    }

    #[test]
    fn drag_split_then_move_active_tab_via_commands() {
        let mut window = WorkspaceWindowLayout::new("main", "p1");
        window.pane_tree = WorkspacePaneTree::split(
            Axis::Horizontal,
            0.5,
            WorkspacePaneTree::leaf("p1"),
            WorkspacePaneTree::leaf("p2"),
        );
        window.active_pane = Some(Arc::<str>::from("p1"));

        window
            .pane_tree
            .find_pane_mut("p1")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("a"));

        // Split `p2` into a new pane, then move the active tab from `p1` into that new pane.
        assert!(window.apply_command(&crate::commands::pane_activate_command("p2").unwrap()));
        assert!(
            window.apply_command(
                &crate::commands::pane_split_command(Axis::Horizontal, SplitSide::Second, "p3")
                    .unwrap()
            )
        );
        assert!(window.pane_tree.find_pane("p3").is_some());

        assert!(window.apply_command(&crate::commands::pane_activate_command("p1").unwrap()));
        assert!(
            window.apply_command(&crate::commands::pane_move_active_tab_to_command("p3").unwrap())
        );

        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p3");
        assert!(
            window
                .pane_tree
                .find_pane("p3")
                .unwrap()
                .tabs
                .tabs()
                .iter()
                .any(|t| t.as_ref() == "a")
        );
        assert!(
            window
                .pane_tree
                .find_pane("p1")
                .unwrap()
                .tabs
                .tabs()
                .is_empty()
        );
    }

    #[test]
    fn split_leaf_rejects_unknown_leaf() {
        let mut tree = WorkspacePaneTree::leaf("p1");
        assert!(!tree.split_leaf(
            "missing",
            Axis::Vertical,
            0.5,
            SplitSide::Second,
            WorkspacePaneTree::leaf("p2")
        ));
    }

    #[test]
    fn focus_next_prev_cycles_leaf_order() {
        let mut window = WorkspaceWindowLayout::new("main", "p1");
        window.pane_tree = WorkspacePaneTree::split(
            Axis::Horizontal,
            0.5,
            WorkspacePaneTree::leaf("p1"),
            WorkspacePaneTree::leaf("p2"),
        );
        window.active_pane = Some(Arc::<str>::from("p1"));

        assert!(window.focus_next_pane());
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p2");
        assert!(window.focus_next_pane());
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p1");

        assert!(window.focus_prev_pane());
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p2");
    }

    #[test]
    fn move_active_tab_to_pane_moves_tab_and_focuses_target() {
        let mut window = WorkspaceWindowLayout::new("main", "p1");
        window.pane_tree = WorkspacePaneTree::split(
            Axis::Horizontal,
            0.5,
            WorkspacePaneTree::leaf("p1"),
            WorkspacePaneTree::leaf("p2"),
        );
        window.active_pane = Some(Arc::<str>::from("p1"));

        window
            .pane_tree
            .find_pane_mut("p1")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("a"));
        window
            .pane_tree
            .find_pane_mut("p2")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("b"));

        assert!(window.move_active_tab_to_pane("p2"));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p2");
        assert!(window.pane_tree.find_pane("p2").unwrap().tabs.is_dirty("a") == false);
        assert!(
            window
                .pane_tree
                .find_pane("p1")
                .unwrap()
                .tabs
                .tabs()
                .is_empty()
        );
        assert!(
            window
                .pane_tree
                .find_pane("p2")
                .unwrap()
                .tabs
                .tabs()
                .iter()
                .any(|t| t.as_ref() == "a")
        );
    }

    #[test]
    fn move_active_tab_to_next_prev_pane_wraps() {
        let mut window = WorkspaceWindowLayout::new("main", "p1");
        window.pane_tree = WorkspacePaneTree::split(
            Axis::Horizontal,
            0.5,
            WorkspacePaneTree::leaf("p1"),
            WorkspacePaneTree::leaf("p2"),
        );
        window.active_pane = Some(Arc::<str>::from("p1"));

        window
            .pane_tree
            .find_pane_mut("p1")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("a"));

        assert!(window.move_active_tab_to_next_pane());
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p2");

        assert!(window.move_active_tab_to_next_pane());
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p1");

        window
            .pane_tree
            .find_pane_mut("p1")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("b"));

        assert!(window.move_active_tab_to_prev_pane());
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p2");
    }

    #[test]
    fn resize_active_pane_adjusts_nearest_matching_split_fraction() {
        let mut window = WorkspaceWindowLayout::new("main", "p1");
        window.pane_tree = WorkspacePaneTree::split(
            Axis::Vertical,
            0.5,
            WorkspacePaneTree::split(
                Axis::Horizontal,
                0.6,
                WorkspacePaneTree::leaf("p1"),
                WorkspacePaneTree::leaf("p2"),
            ),
            WorkspacePaneTree::leaf("p3"),
        );

        window.active_pane = Some(Arc::<str>::from("p1"));
        assert!(window.resize_active_pane(Axis::Horizontal, 0.1));
        let WorkspacePaneTree::Split { a, fraction, .. } = &window.pane_tree else {
            panic!("expected root split");
        };
        assert!(
            (*fraction - 0.5).abs() < 1e-6,
            "vertical split should remain unchanged"
        );

        let WorkspacePaneTree::Split { fraction, .. } = a.as_ref() else {
            panic!("expected nested split");
        };
        assert!((*fraction - 0.7).abs() < 1e-6);

        window.active_pane = Some(Arc::<str>::from("p2"));
        assert!(window.resize_active_pane(Axis::Horizontal, 0.1));
        let WorkspacePaneTree::Split { a, .. } = &window.pane_tree else {
            panic!("expected root split");
        };
        let WorkspacePaneTree::Split { fraction, .. } = a.as_ref() else {
            panic!("expected nested split");
        };
        assert!((*fraction - 0.6).abs() < 1e-6);
    }

    #[test]
    fn split_pane_allocates_id_and_clones_active_tab() {
        let mut window = WorkspaceWindowLayout::new("w", "p1");
        window.active_pane = Some(Arc::<str>::from("p1"));
        window
            .pane_tree
            .find_pane_mut("p1")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("a"));

        assert!(window.apply_command(&CommandId::from(
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_RIGHT
        )));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "w.pane.1");

        let new_pane = window.pane_tree.find_pane("w.pane.1").unwrap();
        assert_eq!(new_pane.tabs.active().unwrap().as_ref(), "a");

        let old_pane = window.pane_tree.find_pane("p1").unwrap();
        assert!(old_pane.tabs.tabs().iter().any(|t| t.as_ref() == "a"));
    }

    #[test]
    fn split_pane_auto_id_increments() {
        let mut window = WorkspaceWindowLayout::new("w", "p1");
        window.active_pane = Some(Arc::<str>::from("p1"));
        window
            .pane_tree
            .find_pane_mut("p1")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("a"));

        assert!(window.apply_command(&CommandId::from(
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_DOWN
        )));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "w.pane.1");

        assert!(window.apply_command(&CommandId::from(
            crate::commands::CMD_WORKSPACE_PANE_SPLIT_DOWN
        )));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "w.pane.2");
    }

    #[test]
    fn focus_pane_in_direction_uses_geometry_best_effort() {
        let mut window = WorkspaceWindowLayout::new("w", "p1");
        window.pane_tree = WorkspacePaneTree::split(
            Axis::Vertical,
            0.5,
            WorkspacePaneTree::split(
                Axis::Horizontal,
                0.5,
                WorkspacePaneTree::leaf("p1"),
                WorkspacePaneTree::leaf("p2"),
            ),
            WorkspacePaneTree::split(
                Axis::Horizontal,
                0.5,
                WorkspacePaneTree::leaf("p3"),
                WorkspacePaneTree::leaf("p4"),
            ),
        );

        window.active_pane = Some(Arc::<str>::from("p1"));
        assert!(window.apply_command(&CommandId::from(
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_RIGHT
        )));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p2");

        assert!(window.apply_command(&CommandId::from(
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_DOWN
        )));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p4");

        assert!(window.apply_command(&CommandId::from(
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_LEFT
        )));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p3");

        assert!(window.apply_command(&CommandId::from(
            crate::commands::CMD_WORKSPACE_PANE_FOCUS_UP
        )));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p1");
    }

    #[test]
    fn move_active_tab_to_direction_moves_to_neighbor_pane() {
        let mut window = WorkspaceWindowLayout::new("w", "p1");
        window.pane_tree = WorkspacePaneTree::split(
            Axis::Horizontal,
            0.5,
            WorkspacePaneTree::leaf("p1"),
            WorkspacePaneTree::leaf("p2"),
        );
        window.active_pane = Some(Arc::<str>::from("p2"));

        window
            .pane_tree
            .find_pane_mut("p2")
            .unwrap()
            .tabs
            .open_and_activate(Arc::<str>::from("a"));

        assert!(window.apply_command(&CommandId::from(
            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_LEFT
        )));
        assert_eq!(window.active_pane_id().unwrap().as_ref(), "p1");
        assert!(
            window
                .pane_tree
                .find_pane("p1")
                .unwrap()
                .tabs
                .tabs()
                .iter()
                .any(|t| t.as_ref() == "a")
        );
    }
}

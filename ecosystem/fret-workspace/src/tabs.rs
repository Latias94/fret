use std::collections::HashSet;
use std::sync::Arc;

use fret_runtime::CommandId;

use crate::commands::{
    CMD_WORKSPACE_TAB_ACTIVATE_PREFIX, CMD_WORKSPACE_TAB_CLOSE, CMD_WORKSPACE_TAB_CLOSE_LEFT,
    CMD_WORKSPACE_TAB_CLOSE_OTHERS, CMD_WORKSPACE_TAB_CLOSE_PREFIX, CMD_WORKSPACE_TAB_CLOSE_RIGHT,
    CMD_WORKSPACE_TAB_MOVE_AFTER_PREFIX, CMD_WORKSPACE_TAB_MOVE_BEFORE_PREFIX,
    CMD_WORKSPACE_TAB_MOVE_LEFT, CMD_WORKSPACE_TAB_MOVE_RIGHT, CMD_WORKSPACE_TAB_NEXT,
    CMD_WORKSPACE_TAB_PREV,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TabCycleMode {
    /// Cycle tabs in the current visible order.
    InOrder,
    /// Cycle tabs by most-recently-used order (editor default).
    Mru,
}

impl Default for TabCycleMode {
    fn default() -> Self {
        Self::Mru
    }
}

/// A small in-memory model for "editor tabs" (documents).
///
/// Notes:
/// - This is intentionally policy-oriented and does not depend on `fret-app::App`.
/// - IDs are app-defined (e.g. document path, buffer id, page id).
#[derive(Debug, Clone)]
pub struct WorkspaceTabs {
    tabs: Vec<Arc<str>>,
    active: Option<Arc<str>>,
    mru: Vec<Arc<str>>,
    dirty: HashSet<Arc<str>>,
    cycle_mode: TabCycleMode,
}

impl Default for WorkspaceTabs {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active: None,
            mru: Vec::new(),
            dirty: HashSet::new(),
            cycle_mode: TabCycleMode::default(),
        }
    }
}

impl WorkspaceTabs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cycle_mode(mut self, mode: TabCycleMode) -> Self {
        self.cycle_mode = mode;
        self
    }

    pub fn tabs(&self) -> &[Arc<str>] {
        &self.tabs
    }

    pub fn mru(&self) -> &[Arc<str>] {
        &self.mru
    }

    pub fn active(&self) -> Option<&Arc<str>> {
        self.active.as_ref()
    }

    pub fn cycle_mode(&self) -> TabCycleMode {
        self.cycle_mode
    }

    pub fn is_dirty(&self, id: &str) -> bool {
        self.dirty.iter().any(|t| t.as_ref() == id)
    }

    pub fn dirty_in_tab_order(&self) -> Vec<Arc<str>> {
        self.tabs
            .iter()
            .filter(|t| self.dirty.contains(*t))
            .cloned()
            .collect()
    }

    pub fn open_and_activate(&mut self, id: Arc<str>) {
        if !self.tabs.iter().any(|t| t.as_ref() == id.as_ref()) {
            self.tabs.push(id.clone());
        }
        self.activate(id);
    }

    pub fn snapshot_v1(&self) -> WorkspaceTabsV1 {
        WorkspaceTabsV1 {
            tabs: self.tabs.clone(),
            active: self.active.clone(),
            mru: self.mru.clone(),
            dirty: self.dirty_in_tab_order(),
            cycle_mode: self.cycle_mode,
        }
    }

    pub fn from_snapshot_v1(snapshot: WorkspaceTabsV1) -> Self {
        let mut state = WorkspaceTabs::new().with_cycle_mode(snapshot.cycle_mode);

        for id in snapshot.tabs {
            if !state.tabs.iter().any(|t| t.as_ref() == id.as_ref()) {
                state.tabs.push(id);
            }
        }

        if let Some(active) = snapshot.active {
            let _ = state.activate(active);
        } else if let Some(first) = state.tabs.first().cloned() {
            let _ = state.activate(first);
        }

        for id in snapshot.dirty {
            state.set_dirty(id, true);
        }

        // Restore MRU order best-effort: filter to known tabs and ensure active is first.
        let mut mru: Vec<Arc<str>> = Vec::new();
        for id in snapshot.mru {
            if state.tabs.iter().any(|t| t.as_ref() == id.as_ref())
                && !mru.iter().any(|t| t.as_ref() == id.as_ref())
            {
                mru.push(id);
            }
        }

        if let Some(active) = state.active.clone() {
            mru.retain(|t| t.as_ref() != active.as_ref());
            mru.insert(0, active);
        }

        state.mru = mru;
        state
    }

    pub fn activate(&mut self, id: Arc<str>) -> bool {
        if !self.tabs.iter().any(|t| t.as_ref() == id.as_ref()) {
            return false;
        }
        self.active = Some(id.clone());
        self.touch_mru(id);
        true
    }

    pub fn activate_str(&mut self, id: &str) -> bool {
        let Some(existing) = self.tabs.iter().find(|t| t.as_ref() == id).cloned() else {
            return false;
        };
        self.activate(existing)
    }

    pub fn set_dirty(&mut self, id: Arc<str>, dirty: bool) {
        if dirty && !self.tabs.iter().any(|t| t.as_ref() == id.as_ref()) {
            return;
        }
        if dirty {
            self.dirty.insert(id);
        } else {
            self.dirty.retain(|t| t.as_ref() != id.as_ref());
        }
    }

    pub fn close(&mut self, id: &str) -> bool {
        let Some(index) = self.tabs.iter().position(|t| t.as_ref() == id) else {
            return false;
        };

        let removed = self.tabs.remove(index);
        self.dirty.remove(&removed);
        self.mru.retain(|t| t.as_ref() != removed.as_ref());

        if self.active.as_deref() == Some(removed.as_ref()) {
            self.active = None;
            if let Some(next) = self.mru.first().cloned() {
                self.active = Some(next);
            } else if let Some(fallback) = self
                .tabs
                .get(index.min(self.tabs.len().saturating_sub(1)))
                .cloned()
            {
                let _ = self.activate(fallback);
            }
        }

        true
    }

    pub fn close_others(&mut self) -> bool {
        let Some(active) = self.active.clone() else {
            return false;
        };

        if self.tabs.len() <= 1 {
            return false;
        }

        let before = self.tabs.len();
        self.tabs.retain(|t| t.as_ref() == active.as_ref());
        self.dirty.retain(|t| t.as_ref() == active.as_ref());
        self.mru.retain(|t| t.as_ref() == active.as_ref());
        self.mru = vec![active.clone()];
        self.active = Some(active);
        self.tabs.len() != before
    }

    pub fn close_left_of_active(&mut self) -> bool {
        let Some(active) = self.active.clone() else {
            return false;
        };
        let Some(index) = self.tabs.iter().position(|t| t.as_ref() == active.as_ref()) else {
            return false;
        };
        if index == 0 {
            return false;
        }

        let keep_from = index;
        let removed: Vec<Arc<str>> = self.tabs[..keep_from].to_vec();
        self.tabs = self.tabs[keep_from..].to_vec();

        for r in &removed {
            self.dirty.remove(r);
        }
        self.mru
            .retain(|t| !removed.iter().any(|r| r.as_ref() == t.as_ref()));
        self.active = Some(active.clone());
        if self
            .mru
            .first()
            .is_some_and(|t| t.as_ref() == active.as_ref())
        {
            // ok
        } else {
            self.mru.retain(|t| t.as_ref() != active.as_ref());
            self.mru.insert(0, active);
        }

        !removed.is_empty()
    }

    pub fn close_right_of_active(&mut self) -> bool {
        let Some(active) = self.active.clone() else {
            return false;
        };
        let Some(index) = self.tabs.iter().position(|t| t.as_ref() == active.as_ref()) else {
            return false;
        };
        if index + 1 >= self.tabs.len() {
            return false;
        }

        let keep_to = index + 1;
        let removed: Vec<Arc<str>> = self.tabs[keep_to..].to_vec();
        self.tabs.truncate(keep_to);

        for r in &removed {
            self.dirty.remove(r);
        }
        self.mru
            .retain(|t| !removed.iter().any(|r| r.as_ref() == t.as_ref()));
        self.active = Some(active.clone());
        if self
            .mru
            .first()
            .is_some_and(|t| t.as_ref() == active.as_ref())
        {
            // ok
        } else {
            self.mru.retain(|t| t.as_ref() != active.as_ref());
            self.mru.insert(0, active);
        }

        !removed.is_empty()
    }

    pub fn next(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }

        if self.active.is_none() {
            if let Some(first) = self.tabs.first().cloned() {
                return self.activate(first);
            }
        }

        let Some(active) = self.active.clone() else {
            return false;
        };

        match self.cycle_mode {
            TabCycleMode::InOrder => {
                let Some(index) = self.tabs.iter().position(|t| t.as_ref() == active.as_ref())
                else {
                    return false;
                };
                let next = self.tabs[(index + 1) % self.tabs.len()].clone();
                self.activate(next)
            }
            TabCycleMode::Mru => {
                if self.mru.len() <= 1 {
                    return false;
                }
                let next = self.mru[1].clone();
                self.activate(next)
            }
        }
    }

    pub fn prev(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }

        if self.active.is_none() {
            if let Some(first) = self.tabs.first().cloned() {
                return self.activate(first);
            }
        }

        let Some(active) = self.active.clone() else {
            return false;
        };

        match self.cycle_mode {
            TabCycleMode::InOrder => {
                let Some(index) = self.tabs.iter().position(|t| t.as_ref() == active.as_ref())
                else {
                    return false;
                };
                let prev = self.tabs[(index + self.tabs.len() - 1) % self.tabs.len()].clone();
                self.activate(prev)
            }
            TabCycleMode::Mru => {
                if self.mru.len() <= 1 {
                    return false;
                }
                let prev = self
                    .mru
                    .last()
                    .cloned()
                    .unwrap_or_else(|| self.mru[0].clone());
                self.activate(prev)
            }
        }
    }

    pub fn apply_command(&mut self, command: &CommandId) -> bool {
        match command.as_str() {
            CMD_WORKSPACE_TAB_NEXT => return self.next(),
            CMD_WORKSPACE_TAB_PREV => return self.prev(),
            CMD_WORKSPACE_TAB_CLOSE => {
                let Some(active) = self.active.clone() else {
                    return false;
                };
                return self.close(active.as_ref());
            }
            CMD_WORKSPACE_TAB_CLOSE_OTHERS => return self.close_others(),
            CMD_WORKSPACE_TAB_CLOSE_LEFT => return self.close_left_of_active(),
            CMD_WORKSPACE_TAB_CLOSE_RIGHT => return self.close_right_of_active(),
            CMD_WORKSPACE_TAB_MOVE_LEFT => return self.move_active_by(-1),
            CMD_WORKSPACE_TAB_MOVE_RIGHT => return self.move_active_by(1),
            _ => {}
        }

        if let Some(id) = command
            .as_str()
            .strip_prefix(CMD_WORKSPACE_TAB_MOVE_BEFORE_PREFIX)
        {
            let id = id.trim();
            if id.is_empty() {
                return false;
            }
            return self.move_active_relative_to(id, false);
        }

        if let Some(id) = command
            .as_str()
            .strip_prefix(CMD_WORKSPACE_TAB_MOVE_AFTER_PREFIX)
        {
            let id = id.trim();
            if id.is_empty() {
                return false;
            }
            return self.move_active_relative_to(id, true);
        }

        if let Some(id) = command
            .as_str()
            .strip_prefix(CMD_WORKSPACE_TAB_ACTIVATE_PREFIX)
        {
            let id = id.trim();
            if id.is_empty() {
                return false;
            }
            return self.activate_str(id);
        }

        if let Some(id) = command
            .as_str()
            .strip_prefix(CMD_WORKSPACE_TAB_CLOSE_PREFIX)
        {
            let id = id.trim();
            if id.is_empty() {
                return false;
            }
            return self.close(id);
        }

        false
    }

    fn move_active_relative_to(&mut self, target_id: &str, after: bool) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }

        if self.active.is_none() {
            if let Some(first) = self.tabs.first().cloned() {
                return self.activate(first);
            }
        }

        let Some(active) = self.active.clone() else {
            return false;
        };

        if active.as_ref() == target_id {
            return false;
        }

        let Some(active_index) = self.tabs.iter().position(|t| t.as_ref() == active.as_ref())
        else {
            return false;
        };

        if !self.tabs.iter().any(|t| t.as_ref() == target_id) {
            return false;
        }

        let item = self.tabs.remove(active_index);

        // Recompute after removal to avoid index adjustment edge cases.
        let Some(mut target_index) = self.tabs.iter().position(|t| t.as_ref() == target_id) else {
            return false;
        };
        if after {
            target_index += 1;
        }
        target_index = target_index.min(self.tabs.len());
        self.tabs.insert(target_index, item);
        true
    }

    fn move_active_by(&mut self, delta: isize) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }

        if self.active.is_none() {
            if let Some(first) = self.tabs.first().cloned() {
                return self.activate(first);
            }
        }

        let Some(active) = self.active.clone() else {
            return false;
        };

        let Some(index) = self.tabs.iter().position(|t| t.as_ref() == active.as_ref()) else {
            return false;
        };

        let new_index_i = index as isize + delta;
        let new_index = new_index_i.clamp(0, (self.tabs.len() - 1) as isize) as usize;
        if new_index == index {
            return false;
        }

        let item = self.tabs.remove(index);
        self.tabs.insert(new_index, item);
        true
    }

    fn touch_mru(&mut self, id: Arc<str>) {
        self.mru.retain(|t| t.as_ref() != id.as_ref());
        self.mru.insert(0, id);
    }
}

/// Persistable snapshot of `WorkspaceTabs` (V1).
///
/// This uses stable, JSON-friendly shapes (vecs + strings) and avoids hash-based structures to
/// preserve deterministic output.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkspaceTabsV1 {
    pub tabs: Vec<Arc<str>>,
    pub active: Option<Arc<str>>,
    pub mru: Vec<Arc<str>>,
    pub dirty: Vec<Arc<str>>,
    pub cycle_mode: TabCycleMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tabs(ids: &[&str]) -> Vec<Arc<str>> {
        ids.iter().map(|s| Arc::<str>::from(*s)).collect()
    }

    #[test]
    fn mru_next_toggles_between_two_most_recent() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c"]) {
            state.open_and_activate(id);
        }
        assert_eq!(state.active().unwrap().as_ref(), "c");

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_PREV)));
        assert_eq!(state.active().unwrap().as_ref(), "a");

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_NEXT)));
        assert_eq!(state.active().unwrap().as_ref(), "c");

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_NEXT)));
        assert_eq!(state.active().unwrap().as_ref(), "a");
    }

    #[test]
    fn close_active_picks_mru_fallback() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c"]) {
            state.open_and_activate(id);
        }

        assert!(state.activate(Arc::<str>::from("a")));
        assert!(state.activate(Arc::<str>::from("c")));
        assert_eq!(state.active().unwrap().as_ref(), "c");

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_CLOSE)));
        assert_eq!(state.active().unwrap().as_ref(), "a");
        assert_eq!(state.tabs().len(), 2);
    }

    #[test]
    fn close_last_clears_active() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        state.open_and_activate(Arc::<str>::from("only"));

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_CLOSE)));
        assert!(state.active().is_none());
        assert!(state.tabs().is_empty());
    }

    #[test]
    fn snapshot_round_trip_preserves_active_dirty_and_mru() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c"]) {
            state.open_and_activate(id);
        }
        assert!(state.activate(Arc::<str>::from("a")));
        state.set_dirty(Arc::<str>::from("b"), true);

        let snap = state.snapshot_v1();
        let restored = WorkspaceTabs::from_snapshot_v1(snap);

        assert_eq!(restored.active().unwrap().as_ref(), "a");
        assert_eq!(restored.tabs().len(), 3);
        assert!(restored.is_dirty("b"));
        assert_eq!(restored.mru().first().unwrap().as_ref(), "a");
    }

    #[test]
    fn move_active_left_right_reorders_tab_list_without_changing_active() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c"]) {
            state.open_and_activate(id);
        }
        assert_eq!(state.tabs()[0].as_ref(), "a");
        assert_eq!(state.tabs()[1].as_ref(), "b");
        assert_eq!(state.tabs()[2].as_ref(), "c");
        assert_eq!(state.active().unwrap().as_ref(), "c");

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_MOVE_LEFT)));
        assert_eq!(state.active().unwrap().as_ref(), "c");
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["a", "c", "b"]
        );

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_MOVE_RIGHT)));
        assert_eq!(state.active().unwrap().as_ref(), "c");
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn move_active_before_after_reorders_relative_to_target() {
        use crate::commands::{tab_move_active_after_command, tab_move_active_before_command};

        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c", "d"]) {
            state.open_and_activate(id);
        }
        assert_eq!(state.active().unwrap().as_ref(), "d");

        let cmd = tab_move_active_before_command("b").unwrap();
        assert!(state.apply_command(&cmd));
        assert_eq!(state.active().unwrap().as_ref(), "d");
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["a", "d", "b", "c"]
        );

        let cmd = tab_move_active_after_command("c").unwrap();
        assert!(state.apply_command(&cmd));
        assert_eq!(state.active().unwrap().as_ref(), "d");
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["a", "b", "c", "d"]
        );
    }
}

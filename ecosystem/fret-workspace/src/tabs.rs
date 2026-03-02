use std::collections::HashSet;
use std::sync::Arc;

use fret_runtime::CommandId;

use crate::commands::{
    CMD_WORKSPACE_TAB_ACTIVATE_PREFIX, CMD_WORKSPACE_TAB_CLOSE, CMD_WORKSPACE_TAB_CLOSE_LEFT,
    CMD_WORKSPACE_TAB_CLOSE_OTHERS, CMD_WORKSPACE_TAB_CLOSE_PREFIX, CMD_WORKSPACE_TAB_CLOSE_RIGHT,
    CMD_WORKSPACE_TAB_COMMIT_PREVIEW, CMD_WORKSPACE_TAB_MOVE_AFTER_PREFIX,
    CMD_WORKSPACE_TAB_MOVE_BEFORE_PREFIX, CMD_WORKSPACE_TAB_MOVE_LEFT,
    CMD_WORKSPACE_TAB_MOVE_RIGHT, CMD_WORKSPACE_TAB_NEXT, CMD_WORKSPACE_TAB_OPEN_PREVIEW_PREFIX,
    CMD_WORKSPACE_TAB_PIN_PREFIX, CMD_WORKSPACE_TAB_PREV, CMD_WORKSPACE_TAB_TOGGLE_PIN,
    CMD_WORKSPACE_TAB_UNPIN_PREFIX,
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
    pinned_tab_count: usize,
    preview_tab_id: Option<Arc<str>>,
    preview_enabled: bool,
    cycle_mode: TabCycleMode,
}

impl Default for WorkspaceTabs {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active: None,
            mru: Vec::new(),
            dirty: HashSet::new(),
            pinned_tab_count: 0,
            preview_tab_id: None,
            preview_enabled: true,
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

    pub fn pinned_count(&self) -> usize {
        self.pinned_tab_count.min(self.tabs.len())
    }

    pub fn preview_tab_id(&self) -> Option<&Arc<str>> {
        self.preview_tab_id.as_ref()
    }

    pub fn preview_enabled(&self) -> bool {
        self.preview_enabled
    }

    pub fn set_preview_enabled(&mut self, enabled: bool) {
        self.preview_enabled = enabled;
        if !enabled {
            self.preview_tab_id = None;
        }
    }

    pub fn is_tab_preview(&self, id: &str) -> bool {
        self.preview_tab_id.as_deref().is_some_and(|t| t == id)
    }

    pub fn set_pinned_count(&mut self, count: usize) {
        self.pinned_tab_count = count.min(self.tabs.len());
    }

    pub fn is_tab_pinned(&self, id: &str) -> bool {
        let pinned_count = self.pinned_count();
        self.tabs
            .iter()
            .take(pinned_count)
            .any(|t| t.as_ref() == id)
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
        if self.is_tab_preview(id.as_ref()) {
            self.preview_tab_id = None;
        }
        self.activate(id);
    }

    pub fn open_preview_and_activate(&mut self, id: Arc<str>) -> bool {
        if !self.preview_enabled {
            let before_active = self.active.clone();
            self.open_and_activate(id);
            return before_active != self.active;
        }

        // Never demote an already-open normal tab into preview.
        if self.tabs.iter().any(|t| t.as_ref() == id.as_ref()) && !self.is_tab_preview(id.as_ref())
        {
            return self.activate(id);
        }

        // Replace the existing preview tab when possible.
        if let Some(existing_preview) = self.preview_tab_id.clone()
            && existing_preview.as_ref() != id.as_ref()
        {
            let existing_is_dirty = self.is_dirty(existing_preview.as_ref());
            let existing_is_pinned = self.is_tab_pinned(existing_preview.as_ref());
            if !existing_is_dirty && !existing_is_pinned {
                if let Some(ix) = self
                    .tabs
                    .iter()
                    .position(|t| t.as_ref() == existing_preview.as_ref())
                {
                    self.tabs.remove(ix);
                    self.dirty.remove(&existing_preview);
                    self.mru.retain(|t| t.as_ref() != existing_preview.as_ref());
                    self.pinned_tab_count = self.pinned_tab_count.min(self.tabs.len());

                    let insert_at = ix.min(self.tabs.len());
                    self.tabs.insert(insert_at, id.clone());
                    self.preview_tab_id = Some(id.clone());
                    return self.activate(id);
                }
            }
            // If the existing preview cannot be safely replaced, treat it as committed.
            self.preview_tab_id = None;
        }

        if !self.tabs.iter().any(|t| t.as_ref() == id.as_ref()) {
            let pinned_count = self.pinned_count();
            let insert_at = self
                .active
                .as_ref()
                .and_then(|active| {
                    self.tabs
                        .iter()
                        .position(|t| t.as_ref() == active.as_ref())
                        .map(|ix| {
                            if ix < pinned_count {
                                pinned_count
                            } else {
                                (ix + 1).min(self.tabs.len())
                            }
                        })
                })
                .unwrap_or(self.tabs.len())
                .max(pinned_count)
                .min(self.tabs.len());
            self.tabs.insert(insert_at, id.clone());
        }

        self.preview_tab_id = Some(id.clone());
        self.activate(id)
    }

    pub fn commit_preview(&mut self, id: &str) -> bool {
        if !self.is_tab_preview(id) {
            return false;
        }
        self.preview_tab_id = None;
        true
    }

    pub fn pin_tab(&mut self, id: &str) -> bool {
        let id = id.trim();
        if id.is_empty() {
            return false;
        }
        if self.is_tab_preview(id) {
            self.preview_tab_id = None;
        }
        let pinned_count = self.pinned_count();
        let Some(index) = self.tabs.iter().position(|t| t.as_ref() == id) else {
            return false;
        };
        if index < pinned_count {
            return false;
        }

        let item = self.tabs.remove(index);
        let insert_at = pinned_count.min(self.tabs.len());
        self.tabs.insert(insert_at, item);
        self.pinned_tab_count = (pinned_count + 1).min(self.tabs.len());
        true
    }

    pub fn unpin_tab(&mut self, id: &str) -> bool {
        let id = id.trim();
        if id.is_empty() {
            return false;
        }
        if self.is_tab_preview(id) {
            self.preview_tab_id = None;
        }
        let pinned_count = self.pinned_count();
        let Some(index) = self.tabs.iter().position(|t| t.as_ref() == id) else {
            return false;
        };
        if index >= pinned_count {
            return false;
        }

        let item = self.tabs.remove(index);
        let next_pinned_count = pinned_count.saturating_sub(1).min(self.tabs.len());
        self.pinned_tab_count = next_pinned_count;
        let insert_at = next_pinned_count.min(self.tabs.len());
        self.tabs.insert(insert_at, item);
        true
    }

    pub fn pin_active(&mut self) -> bool {
        let Some(active) = self.active.clone() else {
            return false;
        };
        self.pin_tab(active.as_ref())
    }

    pub fn unpin_active(&mut self) -> bool {
        let Some(active) = self.active.clone() else {
            return false;
        };
        self.unpin_tab(active.as_ref())
    }

    pub fn snapshot_v1(&self) -> WorkspaceTabsV1 {
        WorkspaceTabsV1 {
            tabs: self.tabs.clone(),
            active: self.active.clone(),
            mru: self.mru.clone(),
            dirty: self.dirty_in_tab_order(),
            pinned_tab_count: self.pinned_count(),
            preview: self.preview_tab_id.clone(),
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

        state.set_pinned_count(snapshot.pinned_tab_count);

        if let Some(active) = snapshot.active {
            let _ = state.activate(active);
        } else if let Some(first) = state.tabs.first().cloned() {
            let _ = state.activate(first);
        }

        for id in snapshot.dirty {
            state.set_dirty(id, true);
        }

        if let Some(preview) = snapshot.preview {
            if state.tabs.iter().any(|t| t.as_ref() == preview.as_ref()) {
                state.preview_tab_id = Some(preview);
            }
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
        if dirty && self.is_tab_preview(id.as_ref()) {
            self.preview_tab_id = None;
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
        if self.is_tab_preview(removed.as_ref()) {
            self.preview_tab_id = None;
        }
        let pinned_count = self.pinned_count();
        if index < pinned_count {
            self.pinned_tab_count = pinned_count.saturating_sub(1);
        }
        self.pinned_tab_count = self.pinned_tab_count.min(self.tabs.len());
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

        let active_was_preview = self.is_tab_preview(active.as_ref());
        let active_was_pinned = self.is_tab_pinned(active.as_ref());
        let before = self.tabs.len();
        self.tabs.retain(|t| t.as_ref() == active.as_ref());
        self.dirty.retain(|t| t.as_ref() == active.as_ref());
        self.mru.retain(|t| t.as_ref() == active.as_ref());
        self.mru = vec![active.clone()];
        self.active = Some(active);
        self.pinned_tab_count = if active_was_pinned { 1 } else { 0 };
        self.preview_tab_id = if active_was_preview {
            self.active.clone()
        } else {
            None
        };
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
        let pinned_count = self.pinned_count();
        let removed_pinned = keep_from.min(pinned_count);
        self.pinned_tab_count = pinned_count
            .saturating_sub(removed_pinned)
            .min(self.tabs.len());

        for r in &removed {
            self.dirty.remove(r);
        }
        self.mru
            .retain(|t| !removed.iter().any(|r| r.as_ref() == t.as_ref()));
        self.active = Some(active.clone());
        if let Some(preview) = self.preview_tab_id.clone()
            && !self.tabs.iter().any(|t| t.as_ref() == preview.as_ref())
        {
            self.preview_tab_id = None;
        }
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
        self.pinned_tab_count = self.pinned_tab_count.min(self.tabs.len());

        for r in &removed {
            self.dirty.remove(r);
        }
        self.mru
            .retain(|t| !removed.iter().any(|r| r.as_ref() == t.as_ref()));
        self.active = Some(active.clone());
        if let Some(preview) = self.preview_tab_id.clone()
            && !self.tabs.iter().any(|t| t.as_ref() == preview.as_ref())
        {
            self.preview_tab_id = None;
        }
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
            CMD_WORKSPACE_TAB_COMMIT_PREVIEW => {
                let Some(active) = self.active.clone() else {
                    return false;
                };
                return self.commit_preview(active.as_ref());
            }
            CMD_WORKSPACE_TAB_TOGGLE_PIN => {
                let Some(active) = self.active.clone() else {
                    return false;
                };
                if self.is_tab_pinned(active.as_ref()) {
                    return self.unpin_tab(active.as_ref());
                }
                return self.pin_tab(active.as_ref());
            }
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
            .strip_prefix(CMD_WORKSPACE_TAB_OPEN_PREVIEW_PREFIX)
        {
            let id = id.trim();
            if id.is_empty() {
                return false;
            }
            return self.open_preview_and_activate(Arc::<str>::from(id));
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

        if let Some(id) = command.as_str().strip_prefix(CMD_WORKSPACE_TAB_PIN_PREFIX) {
            let id = id.trim();
            if id.is_empty() {
                return false;
            }
            return self.pin_tab(id);
        }

        if let Some(id) = command
            .as_str()
            .strip_prefix(CMD_WORKSPACE_TAB_UNPIN_PREFIX)
        {
            let id = id.trim();
            if id.is_empty() {
                return false;
            }
            return self.unpin_tab(id);
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

        let pinned_count = self.pinned_count();
        let active_is_pinned = active_index < pinned_count;
        let target_is_pinned = self
            .tabs
            .iter()
            .take(pinned_count)
            .any(|t| t.as_ref() == target_id);
        if active_is_pinned != target_is_pinned {
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

        let pinned_count = self.pinned_count();
        let (min_index, max_index) = if index < pinned_count {
            (0, pinned_count.saturating_sub(1))
        } else {
            (
                pinned_count.min(self.tabs.len().saturating_sub(1)),
                self.tabs.len() - 1,
            )
        };
        let new_index_i = index as isize + delta;
        let new_index = new_index_i.clamp(min_index as isize, max_index as isize) as usize;
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
    #[cfg_attr(feature = "serde", serde(default))]
    pub pinned_tab_count: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub preview: Option<Arc<str>>,
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
    fn close_others_keeps_only_active_tab_and_state() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c"]) {
            state.open_and_activate(id);
        }

        assert!(state.activate(Arc::<str>::from("b")));
        state.set_dirty(Arc::<str>::from("a"), true);
        state.set_dirty(Arc::<str>::from("b"), true);
        state.set_dirty(Arc::<str>::from("c"), true);

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_CLOSE_OTHERS)));
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["b"]
        );
        assert_eq!(state.active().unwrap().as_ref(), "b");
        assert_eq!(
            state.mru().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["b"]
        );
        assert!(!state.is_dirty("a"));
        assert!(state.is_dirty("b"));
        assert!(!state.is_dirty("c"));

        assert!(!state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_CLOSE_OTHERS)));
    }

    #[test]
    fn close_left_of_active_removes_tabs_and_dirty_left_of_active() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c", "d"]) {
            state.open_and_activate(id);
        }

        assert!(state.activate(Arc::<str>::from("c")));
        state.set_dirty(Arc::<str>::from("a"), true);
        state.set_dirty(Arc::<str>::from("c"), true);
        state.set_dirty(Arc::<str>::from("d"), true);

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_CLOSE_LEFT)));
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["c", "d"]
        );
        assert_eq!(state.active().unwrap().as_ref(), "c");
        assert_eq!(
            state.mru().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["c", "d"]
        );
        assert!(!state.is_dirty("a"));
        assert!(state.is_dirty("c"));
        assert!(state.is_dirty("d"));
    }

    #[test]
    fn close_right_of_active_removes_tabs_and_dirty_right_of_active() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c", "d"]) {
            state.open_and_activate(id);
        }

        assert!(state.activate(Arc::<str>::from("b")));
        state.set_dirty(Arc::<str>::from("b"), true);
        state.set_dirty(Arc::<str>::from("d"), true);

        assert!(state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_CLOSE_RIGHT)));
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        assert_eq!(state.active().unwrap().as_ref(), "b");
        assert_eq!(
            state.mru().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["b", "a"]
        );
        assert!(state.is_dirty("b"));
        assert!(!state.is_dirty("d"));
    }

    #[test]
    fn snapshot_round_trip_preserves_active_dirty_and_mru() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c"]) {
            state.open_and_activate(id);
        }
        assert!(state.open_preview_and_activate(Arc::<str>::from("p")));
        assert!(state.activate(Arc::<str>::from("a")));
        state.set_dirty(Arc::<str>::from("b"), true);
        state.set_pinned_count(1);

        let snap = state.snapshot_v1();
        let restored = WorkspaceTabs::from_snapshot_v1(snap);

        assert_eq!(restored.active().unwrap().as_ref(), "a");
        assert_eq!(restored.tabs().len(), 4);
        assert!(restored.is_dirty("b"));
        assert_eq!(restored.pinned_count(), 1);
        assert_eq!(restored.preview_tab_id().unwrap().as_ref(), "p");
        assert_eq!(restored.mru().first().unwrap().as_ref(), "a");
    }

    #[test]
    fn preview_open_replaces_existing_preview() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::InOrder);
        state.open_and_activate(Arc::<str>::from("a"));
        assert!(state.open_preview_and_activate(Arc::<str>::from("b")));
        assert_eq!(state.preview_tab_id().unwrap().as_ref(), "b");
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["a", "b"]
        );

        assert!(state.open_preview_and_activate(Arc::<str>::from("c")));
        assert_eq!(state.preview_tab_id().unwrap().as_ref(), "c");
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["a", "c"]
        );
    }

    #[test]
    fn preview_commit_prevents_replacement() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::InOrder);
        assert!(state.open_preview_and_activate(Arc::<str>::from("a")));
        assert!(state.commit_preview("a"));
        assert!(state.open_preview_and_activate(Arc::<str>::from("b")));
        assert_eq!(state.preview_tab_id().unwrap().as_ref(), "b");
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["a", "b"]
        );
    }

    #[test]
    fn dirty_commits_preview() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::InOrder);
        assert!(state.open_preview_and_activate(Arc::<str>::from("a")));
        assert!(state.is_tab_preview("a"));

        state.set_dirty(Arc::<str>::from("a"), true);
        assert!(
            !state.is_tab_preview("a"),
            "dirty preview should be committed"
        );

        assert!(state.open_preview_and_activate(Arc::<str>::from("b")));
        assert_eq!(state.preview_tab_id().unwrap().as_ref(), "b");
        assert!(state.tabs().iter().any(|t| t.as_ref() == "a"));
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

    #[test]
    fn pin_unpin_active_updates_pinned_count_and_order() {
        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c", "d"]) {
            state.open_and_activate(id);
        }
        assert_eq!(state.active().unwrap().as_ref(), "d");
        assert_eq!(state.pinned_count(), 0);

        assert!(state.pin_active());
        assert_eq!(state.pinned_count(), 1);
        assert!(state.is_tab_pinned("d"));
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["d", "a", "b", "c"]
        );

        assert!(state.activate(Arc::<str>::from("b")));
        assert!(state.pin_active());
        assert_eq!(state.pinned_count(), 2);
        assert!(state.is_tab_pinned("b"));
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["d", "b", "a", "c"]
        );

        assert!(state.unpin_active());
        assert_eq!(state.pinned_count(), 1);
        assert!(!state.is_tab_pinned("b"));
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["d", "b", "a", "c"]
        );
    }

    #[test]
    fn move_commands_do_not_cross_pinned_boundary() {
        use crate::commands::tab_move_active_before_command;

        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c", "d"]) {
            state.open_and_activate(id);
        }
        state.set_pinned_count(2);
        assert!(state.is_tab_pinned("a"));
        assert!(state.is_tab_pinned("b"));

        assert!(state.activate(Arc::<str>::from("d")));
        assert!(
            !state.apply_command(&tab_move_active_before_command("a").unwrap()),
            "expected cross-boundary move to be rejected"
        );
        assert!(state.apply_command(&tab_move_active_before_command("c").unwrap()));
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["a", "b", "d", "c"]
        );

        assert!(state.activate(Arc::<str>::from("b")));
        assert!(
            !state.apply_command(&CommandId::from(CMD_WORKSPACE_TAB_MOVE_RIGHT)),
            "expected pinned move to not cross boundary"
        );
    }

    #[test]
    fn pin_unpin_prefix_commands_reorder_without_changing_active() {
        use crate::commands::{tab_pin_command, tab_unpin_command};

        let mut state = WorkspaceTabs::new().with_cycle_mode(TabCycleMode::Mru);
        for id in tabs(&["a", "b", "c"]) {
            state.open_and_activate(id);
        }
        assert_eq!(state.active().unwrap().as_ref(), "c");
        assert_eq!(state.pinned_count(), 0);

        let cmd = tab_pin_command("c").unwrap();
        assert!(state.apply_command(&cmd));
        assert_eq!(state.pinned_count(), 1);
        assert_eq!(state.active().unwrap().as_ref(), "c");
        assert_eq!(
            state.tabs().iter().map(|t| t.as_ref()).collect::<Vec<_>>(),
            vec!["c", "a", "b"]
        );

        let cmd = tab_unpin_command("c").unwrap();
        assert!(state.apply_command(&cmd));
        assert_eq!(state.pinned_count(), 0);
        assert_eq!(state.active().unwrap().as_ref(), "c");
    }
}

//! Layout probing helpers.
//!
//! Fret's declarative pass can read the last-known bounds for any `GlobalElementId`, but it does not
//! currently provide a "subcompose" API to measure children within the same frame. Components that
//! need measurement-driven painting (e.g. `Tabs` indicator width/position) can register element IDs
//! into a list-shaped probe and let the parent read the bounds on the next frame.

use fret_ui::elements::GlobalElementId;

#[derive(Debug, Default, Clone)]
pub struct LayoutProbeList {
    ids: Vec<GlobalElementId>,
}

impl LayoutProbeList {
    pub fn ensure_len(&mut self, len: usize) {
        if self.ids.len() != len {
            self.ids.resize(len, GlobalElementId(0));
        }
    }

    pub fn set(&mut self, idx: usize, id: GlobalElementId) {
        if idx >= self.ids.len() {
            self.ids.resize(idx + 1, GlobalElementId(0));
        }
        self.ids[idx] = id;
    }

    pub fn get(&self, idx: usize) -> Option<GlobalElementId> {
        let id = self.ids.get(idx).copied()?;
        (id.0 != 0).then_some(id)
    }
}

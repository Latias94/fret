use std::collections::HashSet;

use fret_core::AppWindowId;

/// Coalesced animation-frame requests that should wake exactly once per window until drained.
#[derive(Default, Debug, Clone)]
pub(crate) struct AnimationFrameRequests {
    windows: HashSet<AppWindowId>,
}

impl AnimationFrameRequests {
    pub(crate) fn request(&mut self, window: AppWindowId) -> bool {
        self.windows.insert(window)
    }

    pub(crate) fn clear(&mut self) {
        self.windows.clear();
    }

    pub(crate) fn has_pending(&self) -> bool {
        !self.windows.is_empty()
    }

    pub(crate) fn drain(&mut self) -> impl Iterator<Item = AppWindowId> {
        self.windows.drain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    #[test]
    fn request_coalesces_duplicate_windows() {
        let window = AppWindowId::default();
        let mut requests = AnimationFrameRequests::default();

        assert!(requests.request(window));
        assert!(!requests.request(window));

        let drained: Vec<_> = requests.drain().collect();
        assert_eq!(drained, vec![window]);
        assert!(!requests.has_pending());
    }

    #[test]
    fn drain_clears_pending_state() {
        let mut requests = AnimationFrameRequests::default();
        let w1 = AppWindowId::from(KeyData::from_ffi(1));
        let w2 = AppWindowId::from(KeyData::from_ffi(2));

        requests.request(w1);
        requests.request(w2);
        assert!(requests.has_pending());

        let drained: HashSet<_> = requests.drain().collect();
        assert_eq!(drained, HashSet::from([w1, w2]));
        assert!(!requests.has_pending());
    }
}

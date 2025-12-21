use fret_core::AppWindowId;
use fret_editor::AssetGuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedContinuation {
    NewScene,
    OpenScene { guid: AssetGuid },
    CloseWindow { window: AppWindowId },
}

#[derive(Debug, Default)]
pub struct UnsavedChangesService {
    pending: Option<(AppWindowId, UnsavedContinuation)>,
    revision: u64,
}

impl UnsavedChangesService {
    pub fn pending(&self) -> Option<(AppWindowId, UnsavedContinuation)> {
        self.pending
    }

    pub fn set_pending(&mut self, window: AppWindowId, action: UnsavedContinuation) {
        self.pending = Some((window, action));
        self.revision = self.revision.saturating_add(1);
    }

    pub fn clear(&mut self) {
        if self.pending.take().is_some() {
            self.revision = self.revision.saturating_add(1);
        }
    }
}

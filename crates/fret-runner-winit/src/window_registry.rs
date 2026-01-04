use std::collections::HashMap;

use fret_core::AppWindowId;
use winit::window::WindowId;

/// Maps native winit `WindowId` values to Fret's stable `AppWindowId`.
///
/// This keeps window bookkeeping in the winit glue layer, so runners/backends don't need to
/// duplicate the same mapping logic.
#[derive(Debug, Default)]
pub struct WinitWindowRegistry {
    winit_to_app: HashMap<WindowId, AppWindowId>,
}

impl WinitWindowRegistry {
    pub fn insert(&mut self, winit_id: WindowId, app: AppWindowId) {
        self.winit_to_app.insert(winit_id, app);
    }

    pub fn remove(&mut self, winit_id: WindowId) -> Option<AppWindowId> {
        self.winit_to_app.remove(&winit_id)
    }

    pub fn get(&self, winit_id: WindowId) -> Option<AppWindowId> {
        self.winit_to_app.get(&winit_id).copied()
    }
}

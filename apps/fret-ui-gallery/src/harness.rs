pub(crate) const UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER: &str = "hello_soft_wrap_marker";

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub(crate) struct UiGalleryCodeEditorHandlesStore {
    pub per_window: HashMap<AppWindowId, CodeEditorHandle>,
}

use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use fret_core::AppWindowId;

#[cfg(not(target_arch = "wasm32"))]
use fret_code_editor::CodeEditorHandle;

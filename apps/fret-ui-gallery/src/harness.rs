#[cfg(feature = "gallery-dev")]
pub(crate) const UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER: &str = "hello_soft_wrap_marker";

#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
#[derive(Default)]
pub(crate) struct UiGalleryCodeEditorHandlesStore {
    pub per_window: HashMap<AppWindowId, CodeEditorHandle>,
}

#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
#[derive(Default)]
pub(crate) struct UiGalleryMarkdownEditorHandlesStore {
    pub per_window: HashMap<AppWindowId, CodeEditorHandle>,
}

#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
use std::collections::HashMap;

#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
use fret_core::AppWindowId;

#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
use fret_code_editor::CodeEditorHandle;

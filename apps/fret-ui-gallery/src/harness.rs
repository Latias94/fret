#[cfg(feature = "gallery-dev")]
pub(crate) const UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER: &str = "hello_soft_wrap_marker";

#[cfg(feature = "gallery-dev")]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[cfg(feature = "gallery-dev")]
use fret_core::AppWindowId;
#[cfg(feature = "gallery-dev")]
use fret_runtime::Model;

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

#[cfg(feature = "gallery-dev")]
#[derive(Clone)]
pub(crate) struct UiGalleryChartTortureOutputHandle {
    pub output: Model<fret_chart::ChartCanvasOutput>,
    pub shared_engine: Rc<RefCell<delinea::engine::ChartEngine>>,
}

#[cfg(feature = "gallery-dev")]
#[derive(Default)]
pub(crate) struct UiGalleryChartTortureOutputStore {
    pub per_window: HashMap<AppWindowId, UiGalleryChartTortureOutputHandle>,
}

#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
use fret_code_editor::CodeEditorHandle;

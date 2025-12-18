use crate::{AppWindowId, DockNodeId, DropZone, PanelKey, WindowAnchor};

/// High-level docking operations emitted by the UI layer and applied by the app layer.
///
/// This is the transaction vocabulary that enables persistence, undo/redo, and plugins
/// without letting UI widgets mutate the dock graph ad-hoc.
#[derive(Debug, Clone, PartialEq)]
pub enum DockOp {
    SetActiveTab {
        tabs: DockNodeId,
        active: usize,
    },

    MovePanel {
        source_window: AppWindowId,
        panel: PanelKey,
        target_window: AppWindowId,
        target_tabs: DockNodeId,
        zone: DropZone,
        insert_index: Option<usize>,
    },

    FloatPanelToWindow {
        source_window: AppWindowId,
        panel: PanelKey,
        new_window: AppWindowId,
    },

    /// Request creating a new floating OS window and moving the panel into it.
    ///
    /// This is interpreted by the app/runner layer, because `fret-core` does not own window creation.
    RequestFloatPanelToNewWindow {
        source_window: AppWindowId,
        panel: PanelKey,
        anchor: Option<WindowAnchor>,
    },

    /// Merge all panels from `source_window` into `target_tabs` in `target_window`, then remove
    /// the dock root for `source_window`.
    ///
    /// Recommended default behavior when a floating window is closed is to merge its panels back
    /// into the main window rather than discarding them.
    MergeWindowInto {
        source_window: AppWindowId,
        target_window: AppWindowId,
        target_tabs: DockNodeId,
    },

    SetSplitFractionTwo {
        split: DockNodeId,
        first_fraction: f32,
    },
}

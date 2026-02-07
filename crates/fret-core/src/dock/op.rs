use crate::{AppWindowId, DockNodeId, DropZone, PanelKey, Rect, WindowAnchor};

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

    ClosePanel {
        window: AppWindowId,
        panel: PanelKey,
    },

    MovePanel {
        source_window: AppWindowId,
        panel: PanelKey,
        target_window: AppWindowId,
        target_tabs: DockNodeId,
        zone: DropZone,
        insert_index: Option<usize>,
    },

    /// Move an entire tab stack ("dock node") as a group.
    ///
    /// This is used for editor-grade interactions like dragging the tab bar empty space to
    /// undock/move the whole group, rather than a single tab.
    MoveTabs {
        source_window: AppWindowId,
        source_tabs: DockNodeId,
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

    /// Float a panel into an in-window floating dock container (ImGui docking, viewports disabled).
    ///
    /// This does not create a new OS window; the floating container is rendered within
    /// `target_window`'s dock host.
    FloatPanelInWindow {
        source_window: AppWindowId,
        panel: PanelKey,
        target_window: AppWindowId,
        rect: Rect,
    },

    /// Float a whole tab stack into an in-window floating dock container.
    FloatTabsInWindow {
        source_window: AppWindowId,
        source_tabs: DockNodeId,
        target_window: AppWindowId,
        rect: Rect,
    },

    /// Update the bounds of an in-window floating dock container.
    SetFloatingRect {
        window: AppWindowId,
        floating: DockNodeId,
        rect: Rect,
    },

    /// Raise an in-window floating dock container above other floating containers in the window.
    RaiseFloating {
        window: AppWindowId,
        floating: DockNodeId,
    },

    /// Merge an in-window floating dock container back into an existing tab stack.
    MergeFloatingInto {
        window: AppWindowId,
        floating: DockNodeId,
        target_tabs: DockNodeId,
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

    /// Update a split node's normalized `fractions` (length must match `children.len()`).
    SetSplitFractions {
        split: DockNodeId,
        fractions: Vec<f32>,
    },

    /// Atomically update multiple split nodes' normalized `fractions`.
    ///
    /// This is intended for editor-grade splitter drags where a single pointer interaction may
    /// need to update nested same-axis splits to avoid oscillation.
    SetSplitFractionsMany {
        updates: Vec<SplitFractionsUpdate>,
    },

    SetSplitFractionTwo {
        split: DockNodeId,
        first_fraction: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplitFractionsUpdate {
    pub split: DockNodeId,
    pub fractions: Vec<f32>,
}

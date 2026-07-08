use crate::{AppWindowId, DockNodeId, DropZone, PanelKey, Rect};

/// High-level docking operations emitted by the UI layer and applied by the app layer.
///
/// This is the transaction vocabulary that enables persistence, undo/redo, and plugins
/// without letting UI widgets mutate the dock graph ad-hoc.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DockOp {
    SetActiveTab {
        tabs: DockNodeId,
        active: usize,
    },

    /// Ensure `panel` is visible and active in exactly one dock window.
    ///
    /// If the panel is already open anywhere, this selects the existing owner instead of duplicating
    /// the panel. `preferred_window` is used only when the panel is not already present.
    EnsurePanelVisible {
        preferred_window: AppWindowId,
        panel: PanelKey,
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

    /// Move a panel into a window that currently has no dock root tabs.
    ///
    /// This creates the initial root tab stack for `target_window` and inserts `panel` into it.
    MovePanelToEmptyDockSpace {
        source_window: AppWindowId,
        panel: PanelKey,
        target_window: AppWindowId,
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

    /// Move an entire tab stack ("dock node") into a window that currently has no dock root tabs.
    ///
    /// This creates the initial root tab stack for `target_window` and moves the whole group.
    MoveTabsToEmptyDockSpace {
        source_window: AppWindowId,
        source_tabs: DockNodeId,
        target_window: AppWindowId,
    },

    /// Move an entire dock window forest into a window that currently has no dock root.
    ///
    /// This preserves both the source root and any in-window floating dock containers owned by
    /// `source_window`. It is used when an OS floating dock window is closing and the configured
    /// merge target exists but has no tabs yet.
    MoveWindowToEmptyDockSpace {
        source_window: AppWindowId,
        target_window: AppWindowId,
    },

    FloatPanelToWindow {
        source_window: AppWindowId,
        panel: PanelKey,
        new_window: AppWindowId,
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

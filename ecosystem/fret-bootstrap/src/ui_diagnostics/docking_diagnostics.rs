#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockingInteractionSnapshotV1 {
    #[serde(default)]
    pub dock_drag: Option<UiDockDragDiagnosticsV1>,
    #[serde(default)]
    pub dock_drop_resolve: Option<UiDockDropResolveDiagnosticsV1>,
    #[serde(default)]
    pub viewport_capture: Option<UiViewportCaptureDiagnosticsV1>,
    #[serde(default)]
    pub tab_strip_active_visibility: Option<UiDockTabStripActiveVisibilityDiagnosticsV1>,
    #[serde(default)]
    pub dock_graph_stats: Option<UiDockGraphStatsDiagnosticsV1>,
    #[serde(default)]
    pub dock_graph_signature: Option<UiDockGraphSignatureDiagnosticsV1>,
}

impl UiDockingInteractionSnapshotV1 {
    fn from_snapshot(snapshot: &fret_runtime::DockingInteractionDiagnostics) -> Self {
        Self {
            dock_drag: snapshot
                .dock_drag
                .map(UiDockDragDiagnosticsV1::from_snapshot),
            dock_drop_resolve: snapshot
                .dock_drop_resolve
                .as_ref()
                .map(UiDockDropResolveDiagnosticsV1::from_snapshot),
            viewport_capture: snapshot
                .viewport_capture
                .map(UiViewportCaptureDiagnosticsV1::from_snapshot),
            tab_strip_active_visibility: snapshot
                .tab_strip_active_visibility
                .map(UiDockTabStripActiveVisibilityDiagnosticsV1::from_snapshot),
            dock_graph_stats: snapshot
                .dock_graph_stats
                .map(UiDockGraphStatsDiagnosticsV1::from_snapshot),
            dock_graph_signature: snapshot
                .dock_graph_signature
                .as_ref()
                .map(UiDockGraphSignatureDiagnosticsV1::from_snapshot),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiDockTabStripActiveVisibilityDiagnosticsV1 {
    #[serde(default)]
    pub status: UiDockTabStripActiveVisibilityStatusV1,
    #[serde(default)]
    pub tabs_node: Option<u64>,
    pub overflow: bool,
    pub tab_count: u32,
    pub active: u32,
    pub scroll_px: f32,
    pub max_scroll_px: f32,
    pub active_visible: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDockTabStripActiveVisibilityStatusV1 {
    Ok,
    MissingWindowRoot,
    NoTabsFound,
    MissingLayoutRect,
    MissingTabsNode,
}

impl Default for UiDockTabStripActiveVisibilityStatusV1 {
    fn default() -> Self {
        Self::Ok
    }
}

impl UiDockTabStripActiveVisibilityDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockTabStripActiveVisibilityDiagnostics) -> Self {
        use slotmap::Key as _;
        Self {
            status: match snapshot.status {
                fret_runtime::DockTabStripActiveVisibilityStatusDiagnostics::Ok => {
                    UiDockTabStripActiveVisibilityStatusV1::Ok
                }
                fret_runtime::DockTabStripActiveVisibilityStatusDiagnostics::MissingWindowRoot => {
                    UiDockTabStripActiveVisibilityStatusV1::MissingWindowRoot
                }
                fret_runtime::DockTabStripActiveVisibilityStatusDiagnostics::NoTabsFound => {
                    UiDockTabStripActiveVisibilityStatusV1::NoTabsFound
                }
                fret_runtime::DockTabStripActiveVisibilityStatusDiagnostics::MissingLayoutRect => {
                    UiDockTabStripActiveVisibilityStatusV1::MissingLayoutRect
                }
                fret_runtime::DockTabStripActiveVisibilityStatusDiagnostics::MissingTabsNode => {
                    UiDockTabStripActiveVisibilityStatusV1::MissingTabsNode
                }
            },
            tabs_node: snapshot.tabs_node.map(|id| id.data().as_ffi()),
            overflow: snapshot.overflow,
            tab_count: snapshot.tab_count.min(u32::MAX as usize) as u32,
            active: snapshot.active.min(u32::MAX as usize) as u32,
            scroll_px: snapshot.scroll.0,
            max_scroll_px: snapshot.max_scroll.0,
            active_visible: snapshot.active_visible,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockGraphSignatureDiagnosticsV1 {
    pub signature: String,
    pub fingerprint64: u64,
}

impl UiDockGraphSignatureDiagnosticsV1 {
    fn from_snapshot(snapshot: &fret_runtime::DockGraphSignatureDiagnostics) -> Self {
        Self {
            signature: snapshot.signature.clone(),
            fingerprint64: snapshot.fingerprint64,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiDockGraphStatsDiagnosticsV1 {
    pub node_count: u32,
    pub tabs_count: u32,
    pub split_count: u32,
    pub floating_count: u32,
    pub max_depth: u32,
    pub max_split_depth: u32,
    pub canonical_ok: bool,
    pub has_nested_same_axis_splits: bool,
}

impl UiDockGraphStatsDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockGraphStatsDiagnostics) -> Self {
        Self {
            node_count: snapshot.node_count,
            tabs_count: snapshot.tabs_count,
            split_count: snapshot.split_count,
            floating_count: snapshot.floating_count,
            max_depth: snapshot.max_depth,
            max_split_depth: snapshot.max_split_depth,
            canonical_ok: snapshot.canonical_ok,
            has_nested_same_axis_splits: snapshot.has_nested_same_axis_splits,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDockDropResolveSourceV1 {
    InvertDocking,
    OutsideWindow,
    FloatZone,
    EmptyDockSpace,
    LayoutBoundsMiss,
    LatchedPreviousHover,
    TabBar,
    FloatingTitleBar,
    OuterHintRect,
    InnerHintRect,
    None,
}

impl UiDockDropResolveSourceV1 {
    fn from_source(source: fret_runtime::DockDropResolveSource) -> Self {
        match source {
            fret_runtime::DockDropResolveSource::InvertDocking => Self::InvertDocking,
            fret_runtime::DockDropResolveSource::OutsideWindow => Self::OutsideWindow,
            fret_runtime::DockDropResolveSource::FloatZone => Self::FloatZone,
            fret_runtime::DockDropResolveSource::EmptyDockSpace => Self::EmptyDockSpace,
            fret_runtime::DockDropResolveSource::LayoutBoundsMiss => Self::LayoutBoundsMiss,
            fret_runtime::DockDropResolveSource::LatchedPreviousHover => Self::LatchedPreviousHover,
            fret_runtime::DockDropResolveSource::TabBar => Self::TabBar,
            fret_runtime::DockDropResolveSource::FloatingTitleBar => Self::FloatingTitleBar,
            fret_runtime::DockDropResolveSource::OuterHintRect => Self::OuterHintRect,
            fret_runtime::DockDropResolveSource::InnerHintRect => Self::InnerHintRect,
            fret_runtime::DockDropResolveSource::None => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDockDropCandidateRectKindV1 {
    WindowBounds,
    DockBounds,
    FloatZone,
    LayoutBounds,
    RootRect,
    LeafTabsRect,
    TabBarRect,
    InnerHintRect,
    OuterHintRect,
}

impl UiDockDropCandidateRectKindV1 {
    fn from_kind(kind: fret_runtime::DockDropCandidateRectKind) -> Self {
        match kind {
            fret_runtime::DockDropCandidateRectKind::WindowBounds => Self::WindowBounds,
            fret_runtime::DockDropCandidateRectKind::DockBounds => Self::DockBounds,
            fret_runtime::DockDropCandidateRectKind::FloatZone => Self::FloatZone,
            fret_runtime::DockDropCandidateRectKind::LayoutBounds => Self::LayoutBounds,
            fret_runtime::DockDropCandidateRectKind::RootRect => Self::RootRect,
            fret_runtime::DockDropCandidateRectKind::LeafTabsRect => Self::LeafTabsRect,
            fret_runtime::DockDropCandidateRectKind::TabBarRect => Self::TabBarRect,
            fret_runtime::DockDropCandidateRectKind::InnerHintRect => Self::InnerHintRect,
            fret_runtime::DockDropCandidateRectKind::OuterHintRect => Self::OuterHintRect,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiDockDropCandidateRectDiagnosticsV1 {
    pub kind: UiDockDropCandidateRectKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<UiDropZoneV1>,
    pub rect: RectV1,
}

impl UiDockDropCandidateRectDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockDropCandidateRectDiagnostics) -> Self {
        Self {
            kind: UiDockDropCandidateRectKindV1::from_kind(snapshot.kind),
            zone: snapshot.zone.map(UiDropZoneV1::from_zone),
            rect: RectV1::from(snapshot.rect),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDropZoneV1 {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

impl UiDropZoneV1 {
    fn from_zone(zone: fret_core::DropZone) -> Self {
        match zone {
            fret_core::DropZone::Center => Self::Center,
            fret_core::DropZone::Left => Self::Left,
            fret_core::DropZone::Right => Self::Right,
            fret_core::DropZone::Top => Self::Top,
            fret_core::DropZone::Bottom => Self::Bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiDockDropTargetDiagnosticsV1 {
    pub layout_root: u64,
    pub tabs: u64,
    pub zone: UiDropZoneV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insert_index: Option<u64>,
    pub outer: bool,
}

impl UiDockDropTargetDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockDropTargetDiagnostics) -> Self {
        Self {
            layout_root: snapshot.layout_root.data().as_ffi(),
            tabs: snapshot.tabs.data().as_ffi(),
            zone: UiDropZoneV1::from_zone(snapshot.zone),
            insert_index: snapshot.insert_index.map(|v| v as u64),
            outer: snapshot.outer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiDockDropPreviewKindDiagnosticsV1 {
    WrapBinary,
    InsertIntoSplit {
        axis: String,
        split: u64,
        insert_index: u64,
    },
}

impl UiDockDropPreviewKindDiagnosticsV1 {
    fn from_kind(kind: fret_runtime::DockDropPreviewKindDiagnostics) -> Self {
        match kind {
            fret_runtime::DockDropPreviewKindDiagnostics::WrapBinary => Self::WrapBinary,
            fret_runtime::DockDropPreviewKindDiagnostics::InsertIntoSplit {
                axis,
                split,
                insert_index,
            } => Self::InsertIntoSplit {
                axis: match axis {
                    fret_core::Axis::Horizontal => "horizontal",
                    fret_core::Axis::Vertical => "vertical",
                }
                .to_string(),
                split: split.data().as_ffi(),
                insert_index: insert_index as u64,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockDropPreviewDiagnosticsV1 {
    pub kind: UiDockDropPreviewKindDiagnosticsV1,
}

impl UiDockDropPreviewDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockDropPreviewDiagnostics) -> Self {
        Self {
            kind: UiDockDropPreviewKindDiagnosticsV1::from_kind(snapshot.kind),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockDropResolveDiagnosticsV1 {
    pub pointer_id: u64,
    pub position: PointV1,
    pub window_bounds: RectV1,
    pub dock_bounds: RectV1,
    pub source: UiDockDropResolveSourceV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<UiDockDropTargetDiagnosticsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<UiDockDropPreviewDiagnosticsV1>,
    #[serde(default)]
    pub candidates: Vec<UiDockDropCandidateRectDiagnosticsV1>,
}

impl UiDockDropResolveDiagnosticsV1 {
    fn from_snapshot(snapshot: &fret_runtime::DockDropResolveDiagnostics) -> Self {
        Self {
            pointer_id: snapshot.pointer_id.0,
            position: PointV1::from(snapshot.position),
            window_bounds: RectV1::from(snapshot.window_bounds),
            dock_bounds: RectV1::from(snapshot.dock_bounds),
            source: UiDockDropResolveSourceV1::from_source(snapshot.source),
            resolved: snapshot
                .resolved
                .map(UiDockDropTargetDiagnosticsV1::from_snapshot),
            preview: snapshot
                .preview
                .map(UiDockDropPreviewDiagnosticsV1::from_snapshot),
            candidates: snapshot
                .candidates
                .iter()
                .copied()
                .map(UiDockDropCandidateRectDiagnosticsV1::from_snapshot)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockDragDiagnosticsV1 {
    pub pointer_id: u64,
    pub source_window: u64,
    pub current_window: u64,
    #[serde(default)]
    pub kind: String,
    pub dragging: bool,
    pub cross_window_hover: bool,
    #[serde(default)]
    pub transparent_payload_applied: bool,
    #[serde(default)]
    pub transparent_payload_mouse_passthrough_applied: bool,
    #[serde(default)]
    pub window_under_cursor_source: String,
    #[serde(default)]
    pub moving_window: Option<u64>,
    #[serde(default)]
    pub window_under_moving_window: Option<u64>,
    #[serde(default)]
    pub window_under_moving_window_source: String,
}

impl UiDockDragDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockDragDiagnostics) -> Self {
        Self {
            pointer_id: snapshot.pointer_id.0,
            source_window: snapshot.source_window.data().as_ffi(),
            current_window: snapshot.current_window.data().as_ffi(),
            kind: dock_drag_kind_label(snapshot.kind).to_string(),
            dragging: snapshot.dragging,
            cross_window_hover: snapshot.cross_window_hover,
            transparent_payload_applied: snapshot.transparent_payload_applied,
            transparent_payload_mouse_passthrough_applied: snapshot
                .transparent_payload_mouse_passthrough_applied,
            window_under_cursor_source: dock_drag_window_under_cursor_source_label(
                snapshot.window_under_cursor_source,
            )
            .to_string(),
            moving_window: snapshot.moving_window.map(|w| w.data().as_ffi()),
            window_under_moving_window: snapshot.window_under_moving_window.map(|w| w.data().as_ffi()),
            window_under_moving_window_source: dock_drag_window_under_cursor_source_label(
                snapshot.window_under_moving_window_source,
            )
            .to_string(),
        }
    }
}

fn dock_drag_kind_label(kind: fret_runtime::DragKindId) -> &'static str {
    if kind == fret_runtime::DRAG_KIND_DOCK_PANEL {
        return "dock_panel";
    }
    if kind == fret_runtime::DRAG_KIND_DOCK_TABS {
        return "dock_tabs";
    }
    "unknown"
}

fn dock_drag_window_under_cursor_source_label(
    source: fret_runtime::WindowUnderCursorSource,
) -> &'static str {
    use fret_runtime::WindowUnderCursorSource as Src;
    match source {
        Src::Unknown => "unknown",
        Src::PlatformWin32 => "platform_win32",
        Src::PlatformMacos => "platform_macos",
        Src::Latched => "latched",
        Src::HeuristicZOrder => "heuristic_z_order",
        Src::HeuristicRects => "heuristic_rects",
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiViewportCaptureDiagnosticsV1 {
    pub pointer_id: u64,
    pub target: u64,
}

impl UiViewportCaptureDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::ViewportCaptureDiagnostics) -> Self {
        Self {
            pointer_id: snapshot.pointer_id.0,
            target: snapshot.target.data().as_ffi(),
        }
    }
}

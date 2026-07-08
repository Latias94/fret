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
#[derive(Default)]
pub enum UiDockTabStripActiveVisibilityStatusV1 {
    #[default]
    Ok,
    MissingWindowRoot,
    NoTabsFound,
    MissingLayoutRect,
    MissingTabsNode,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiDockDropPolicyDecisionV1 {
    #[default]
    NotApplicable,
    Allowed,
    DeniedDockingPolicy,
}

impl UiDockDropPolicyDecisionV1 {
    fn from_policy(policy: fret_runtime::DockDropPolicyDecisionDiagnostics) -> Self {
        match policy {
            fret_runtime::DockDropPolicyDecisionDiagnostics::NotApplicable => Self::NotApplicable,
            fret_runtime::DockDropPolicyDecisionDiagnostics::Allowed => Self::Allowed,
            fret_runtime::DockDropPolicyDecisionDiagnostics::DeniedDockingPolicy => {
                Self::DeniedDockingPolicy
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiDockDropCommandKindV1 {
    #[default]
    None,
    MovePanel,
    MovePanelToEmptyDockSpace,
    MoveTabs,
    MoveTabsToEmptyDockSpace,
    FloatPanelInWindow,
    FloatTabsInWindow,
    RequestFloatPanelToNewWindow,
    RequestFloatTabsToNewWindow,
}

impl UiDockDropCommandKindV1 {
    fn from_command(command: fret_runtime::DockDropCommandKindDiagnostics) -> Self {
        match command {
            fret_runtime::DockDropCommandKindDiagnostics::None => Self::None,
            fret_runtime::DockDropCommandKindDiagnostics::MovePanel => Self::MovePanel,
            fret_runtime::DockDropCommandKindDiagnostics::MovePanelToEmptyDockSpace => {
                Self::MovePanelToEmptyDockSpace
            }
            fret_runtime::DockDropCommandKindDiagnostics::MoveTabs => Self::MoveTabs,
            fret_runtime::DockDropCommandKindDiagnostics::MoveTabsToEmptyDockSpace => {
                Self::MoveTabsToEmptyDockSpace
            }
            fret_runtime::DockDropCommandKindDiagnostics::FloatPanelInWindow => {
                Self::FloatPanelInWindow
            }
            fret_runtime::DockDropCommandKindDiagnostics::FloatTabsInWindow => {
                Self::FloatTabsInWindow
            }
            fret_runtime::DockDropCommandKindDiagnostics::RequestFloatPanelToNewWindow => {
                Self::RequestFloatPanelToNewWindow
            }
            fret_runtime::DockDropCommandKindDiagnostics::RequestFloatTabsToNewWindow => {
                Self::RequestFloatTabsToNewWindow
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDockDropPayloadKindV1 {
    Panel,
    Tabs,
}

impl UiDockDropPayloadKindV1 {
    fn from_payload_kind(kind: fret_runtime::DockDropPayloadKindDiagnostics) -> Self {
        match kind {
            fret_runtime::DockDropPayloadKindDiagnostics::Panel => Self::Panel,
            fret_runtime::DockDropPayloadKindDiagnostics::Tabs => Self::Tabs,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiDockDropRejectionReasonV1 {
    #[default]
    None,
    NoResolvedTarget,
    DeniedByPolicy,
    NoCommitIntent,
    InvalidCommitTarget,
}

impl UiDockDropRejectionReasonV1 {
    fn from_rejection(reason: fret_runtime::DockDropRejectionReasonDiagnostics) -> Self {
        match reason {
            fret_runtime::DockDropRejectionReasonDiagnostics::None => Self::None,
            fret_runtime::DockDropRejectionReasonDiagnostics::NoResolvedTarget => {
                Self::NoResolvedTarget
            }
            fret_runtime::DockDropRejectionReasonDiagnostics::DeniedByPolicy => {
                Self::DeniedByPolicy
            }
            fret_runtime::DockDropRejectionReasonDiagnostics::NoCommitIntent => {
                Self::NoCommitIntent
            }
            fret_runtime::DockDropRejectionReasonDiagnostics::InvalidCommitTarget => {
                Self::InvalidCommitTarget
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiDockDropCleanupReasonV1 {
    #[default]
    None,
    ClearHoverOnly,
    ClearHoverAndInvalidateLayout,
}

impl UiDockDropCleanupReasonV1 {
    fn from_cleanup(reason: fret_runtime::DockDropCleanupReasonDiagnostics) -> Self {
        match reason {
            fret_runtime::DockDropCleanupReasonDiagnostics::None => Self::None,
            fret_runtime::DockDropCleanupReasonDiagnostics::ClearHoverOnly => {
                Self::ClearHoverOnly
            }
            fret_runtime::DockDropCleanupReasonDiagnostics::ClearHoverAndInvalidateLayout => {
                Self::ClearHoverAndInvalidateLayout
            }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_kind: Option<UiDockDropPayloadKindV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_window: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_window: Option<u64>,
    pub source: UiDockDropResolveSourceV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<UiDockDropTargetDiagnosticsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denied: Option<UiDockDropTargetDiagnosticsV1>,
    #[serde(default)]
    pub policy: UiDockDropPolicyDecisionV1,
    #[serde(default)]
    pub command: UiDockDropCommandKindV1,
    #[serde(default)]
    pub rejection_reason: UiDockDropRejectionReasonV1,
    #[serde(default)]
    pub commit_capable: bool,
    #[serde(default)]
    pub cleanup_reason: UiDockDropCleanupReasonV1,
    #[serde(default)]
    pub clears_hover: bool,
    #[serde(default)]
    pub invalidates_layout: bool,
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
            payload_kind: snapshot
                .payload_kind
                .map(UiDockDropPayloadKindV1::from_payload_kind),
            source_window: snapshot.source_window.map(|window| window.data().as_ffi()),
            target_window: snapshot.target_window.map(|window| window.data().as_ffi()),
            source: UiDockDropResolveSourceV1::from_source(snapshot.source),
            resolved: snapshot
                .resolved
                .map(UiDockDropTargetDiagnosticsV1::from_snapshot),
            denied: snapshot
                .denied
                .map(UiDockDropTargetDiagnosticsV1::from_snapshot),
            policy: UiDockDropPolicyDecisionV1::from_policy(snapshot.policy),
            command: UiDockDropCommandKindV1::from_command(snapshot.command),
            rejection_reason: UiDockDropRejectionReasonV1::from_rejection(
                snapshot.rejection_reason,
            ),
            commit_capable: snapshot.commit_capable,
            cleanup_reason: UiDockDropCleanupReasonV1::from_cleanup(snapshot.cleanup_reason),
            clears_hover: snapshot.clears_hover,
            invalidates_layout: snapshot.invalidates_layout,
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

#[cfg(test)]
mod dock_drop_resolve_diagnostics_tests {
    use super::*;
    use slotmap::KeyData;

    fn dock_node_id(id: u64) -> fret_core::DockNodeId {
        fret_core::DockNodeId::from(KeyData::from_ffi(id))
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> fret_core::geometry::Rect {
        fret_core::geometry::Rect::new(
            fret_core::geometry::Point::new(
                fret_core::geometry::Px(x),
                fret_core::geometry::Px(y),
            ),
            fret_core::geometry::Size::new(
                fret_core::geometry::Px(width),
                fret_core::geometry::Px(height),
            ),
        )
    }

    #[test]
    fn drop_resolve_diagnostics_serializes_resolved_transaction_fields() {
        let target = fret_runtime::DockDropTargetDiagnostics {
            layout_root: dock_node_id(11),
            tabs: dock_node_id(12),
            zone: fret_core::DropZone::Left,
            insert_index: Some(2),
            outer: true,
        };
        let snapshot = fret_runtime::DockDropResolveDiagnostics {
            pointer_id: fret_core::PointerId(7),
            position: fret_core::geometry::Point::new(
                fret_core::geometry::Px(10.0),
                fret_core::geometry::Px(20.0),
            ),
            window_bounds: rect(0.0, 0.0, 800.0, 600.0),
            dock_bounds: rect(4.0, 8.0, 700.0, 500.0),
            payload_kind: Some(fret_runtime::DockDropPayloadKindDiagnostics::Tabs),
            source_window: None,
            target_window: None,
            source: fret_runtime::DockDropResolveSource::InnerHintRect,
            resolved: None,
            denied: Some(target),
            preview: None,
            policy: fret_runtime::DockDropPolicyDecisionDiagnostics::DeniedDockingPolicy,
            command: fret_runtime::DockDropCommandKindDiagnostics::RequestFloatTabsToNewWindow,
            rejection_reason: fret_runtime::DockDropRejectionReasonDiagnostics::DeniedByPolicy,
            commit_capable: false,
            cleanup_reason: fret_runtime::DockDropCleanupReasonDiagnostics::ClearHoverAndInvalidateLayout,
            clears_hover: true,
            invalidates_layout: true,
            candidates: Vec::new(),
        };

        let value = serde_json::to_value(UiDockDropResolveDiagnosticsV1::from_snapshot(&snapshot))
            .expect("serialize dock drop resolve diagnostics");

        assert_eq!(value["pointer_id"], serde_json::json!(7));
        assert_eq!(value["payload_kind"], serde_json::json!("tabs"));
        assert_eq!(value["source"], serde_json::json!("inner_hint_rect"));
        assert_eq!(
            value["rejection_reason"],
            serde_json::json!("denied_by_policy")
        );
        assert_eq!(
            value["cleanup_reason"],
            serde_json::json!("clear_hover_and_invalidate_layout")
        );
        assert_eq!(
            value["denied"]["layout_root"],
            serde_json::json!(target.layout_root.data().as_ffi())
        );
        assert_eq!(
            value["denied"]["tabs"],
            serde_json::json!(target.tabs.data().as_ffi())
        );
        assert_eq!(value["denied"]["zone"], serde_json::json!("left"));
        assert_eq!(value["denied"]["insert_index"], serde_json::json!(2));
        assert_eq!(value["denied"]["outer"], serde_json::json!(true));
        assert_eq!(
            value["policy"],
            serde_json::json!("denied_docking_policy")
        );
        assert_eq!(
            value["command"],
            serde_json::json!("request_float_tabs_to_new_window")
        );
        assert_eq!(value["commit_capable"], serde_json::json!(false));
        assert_eq!(value["clears_hover"], serde_json::json!(true));
        assert_eq!(value["invalidates_layout"], serde_json::json!(true));
        assert!(value.get("resolved").is_none());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockDragDiagnosticsV1 {
    pub pointer_id: u64,
    pub source_window: u64,
    pub current_window: u64,
    /// Window-local logical cursor position at the time the snapshot was published.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<PointV1>,
    /// Window-local logical cursor position when the drag session started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_position: Option<PointV1>,
    /// Cursor grab offset in window-local logical coordinates (ImGui-style multi-viewport anchor).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_grab_offset: Option<PointV1>,
    /// The OS window requested to follow the cursor for this drag session (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_window: Option<u64>,
    /// Raw cursor position in screen-space physical pixels, as observed by the runner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_screen_pos_raw_physical_px: Option<PointV1>,
    /// Cursor position in screen-space physical pixels used for local position conversion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_screen_pos_used_physical_px: Option<PointV1>,
    #[serde(default)]
    pub cursor_screen_pos_was_clamped: bool,
    #[serde(default)]
    pub cursor_override_active: bool,
    /// Outer position of `current_window` in screen-space physical pixels when routing was computed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_window_outer_pos_physical_px: Option<PointV1>,
    /// Decoration offset (client origin relative to outer origin) in physical pixels for `current_window`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_window_decoration_offset_physical_px: Option<PointV1>,
    /// Computed client origin (screen-space physical px) for `current_window`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_window_client_origin_screen_physical_px: Option<PointV1>,
    #[serde(default)]
    pub current_window_client_origin_source_platform: bool,
    /// Scale factor used by the runner when converting screen physical px into window-local logical px.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_window_scale_factor_x1000_from_runner: Option<u32>,
    /// Local position derived from screen cursor + client origin + scale factor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_window_local_pos_from_screen_logical_px: Option<PointV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_window_scale_factor_x1000: Option<u32>,
    #[serde(default)]
    pub kind: String,
    pub dragging: bool,
    pub cross_window_hover: bool,
    #[serde(default)]
    pub payload_ghost_visible: bool,
    #[serde(default)]
    pub transparent_payload_applied: bool,
    #[serde(default)]
    pub transparent_payload_hit_test_passthrough_applied: bool,
    #[serde(default)]
    pub window_under_cursor_source: String,
    #[serde(default)]
    pub moving_window: Option<u64>,
    /// Outer position of `moving_window` in screen-space physical pixels when routing was computed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moving_window_outer_pos_physical_px: Option<PointV1>,
    /// Decoration offset (client origin relative to outer origin) in physical pixels for `moving_window`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moving_window_decoration_offset_physical_px: Option<PointV1>,
    /// Computed client origin (screen-space physical px) for `moving_window`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moving_window_client_origin_screen_physical_px: Option<PointV1>,
    #[serde(default)]
    pub moving_window_client_origin_source_platform: bool,
    /// Scale factor used by the runner when converting screen physical px into moving-window-local logical px.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moving_window_scale_factor_x1000_from_runner: Option<u32>,
    /// Local position derived from screen cursor + moving-window client origin + scale factor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moving_window_local_pos_from_screen_logical_px: Option<PointV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moving_window_scale_factor_x1000: Option<u32>,
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
            position: Some(PointV1::from(snapshot.position)),
            start_position: Some(PointV1::from(snapshot.start_position)),
            cursor_grab_offset: snapshot.cursor_grab_offset.map(PointV1::from),
            follow_window: snapshot.follow_window.map(|w| w.data().as_ffi()),
            cursor_screen_pos_raw_physical_px: snapshot
                .cursor_screen_pos_raw_physical_px
                .map(PointV1::from),
            cursor_screen_pos_used_physical_px: snapshot
                .cursor_screen_pos_used_physical_px
                .map(PointV1::from),
            cursor_screen_pos_was_clamped: snapshot.cursor_screen_pos_was_clamped,
            cursor_override_active: snapshot.cursor_override_active,
            current_window_outer_pos_physical_px: snapshot
                .current_window_outer_pos_physical_px
                .map(PointV1::from),
            current_window_decoration_offset_physical_px: snapshot
                .current_window_decoration_offset_physical_px
                .map(PointV1::from),
            current_window_client_origin_screen_physical_px: snapshot
                .current_window_client_origin_screen_physical_px
                .map(PointV1::from),
            current_window_client_origin_source_platform: snapshot
                .current_window_client_origin_source_platform,
            current_window_scale_factor_x1000_from_runner: snapshot
                .current_window_scale_factor_x1000_from_runner,
            current_window_local_pos_from_screen_logical_px: snapshot
                .current_window_local_pos_from_screen_logical_px
                .map(PointV1::from),
            current_window_scale_factor_x1000: snapshot.current_window_scale_factor_x1000,
            kind: dock_drag_kind_label(snapshot.kind).to_string(),
            dragging: snapshot.dragging,
            cross_window_hover: snapshot.cross_window_hover,
            payload_ghost_visible: snapshot.payload_ghost_visible,
            transparent_payload_applied: snapshot.transparent_payload_applied,
            transparent_payload_hit_test_passthrough_applied: snapshot
                .transparent_payload_hit_test_passthrough_applied,
            window_under_cursor_source: dock_drag_window_under_cursor_source_label(
                snapshot.window_under_cursor_source,
            )
            .to_string(),
            moving_window: snapshot.moving_window.map(|w| w.data().as_ffi()),
            moving_window_outer_pos_physical_px: snapshot
                .moving_window_outer_pos_physical_px
                .map(PointV1::from),
            moving_window_decoration_offset_physical_px: snapshot
                .moving_window_decoration_offset_physical_px
                .map(PointV1::from),
            moving_window_client_origin_screen_physical_px: snapshot
                .moving_window_client_origin_screen_physical_px
                .map(PointV1::from),
            moving_window_client_origin_source_platform: snapshot
                .moving_window_client_origin_source_platform,
            moving_window_scale_factor_x1000_from_runner: snapshot
                .moving_window_scale_factor_x1000_from_runner,
            moving_window_local_pos_from_screen_logical_px: snapshot
                .moving_window_local_pos_from_screen_logical_px
                .map(PointV1::from),
            moving_window_scale_factor_x1000: snapshot.moving_window_scale_factor_x1000,
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

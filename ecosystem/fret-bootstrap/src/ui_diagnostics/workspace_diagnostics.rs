#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWorkspaceInteractionSnapshotV1 {
    #[serde(default)]
    pub tab_strip_active_visibility: Vec<UiWorkspaceTabStripActiveVisibilityDiagnosticsV1>,
}

impl UiWorkspaceInteractionSnapshotV1 {
    pub fn from_snapshot(snapshot: &fret_runtime::WorkspaceInteractionDiagnostics) -> Self {
        Self {
            tab_strip_active_visibility: snapshot
                .tab_strip_active_visibility
                .iter()
                .cloned()
                .map(UiWorkspaceTabStripActiveVisibilityDiagnosticsV1::from_snapshot)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWorkspaceTabStripActiveVisibilityStatusV1 {
    Ok,
    NoActiveTab,
    MissingScrollViewportRect,
    MissingActiveTabRect,
}

impl Default for UiWorkspaceTabStripActiveVisibilityStatusV1 {
    fn default() -> Self {
        Self::Ok
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWorkspaceTabStripActiveVisibilityDiagnosticsV1 {
    #[serde(default)]
    pub status: UiWorkspaceTabStripActiveVisibilityStatusV1,
    #[serde(default)]
    pub pane_id: Option<String>,
    #[serde(default)]
    pub active_tab_id: Option<String>,
    pub tab_count: u32,
    pub overflow: bool,
    pub scroll_x_px: f32,
    pub max_scroll_x_px: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_viewport_rect: Option<RectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_tab_rect: Option<RectV1>,
    pub active_visible: bool,
}

impl UiWorkspaceTabStripActiveVisibilityDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::WorkspaceTabStripActiveVisibilityDiagnostics) -> Self {
        Self {
            status: match snapshot.status {
                fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok => {
                    UiWorkspaceTabStripActiveVisibilityStatusV1::Ok
                }
                fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::NoActiveTab => {
                    UiWorkspaceTabStripActiveVisibilityStatusV1::NoActiveTab
                }
                fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::MissingScrollViewportRect => {
                    UiWorkspaceTabStripActiveVisibilityStatusV1::MissingScrollViewportRect
                }
                fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::MissingActiveTabRect => {
                    UiWorkspaceTabStripActiveVisibilityStatusV1::MissingActiveTabRect
                }
            },
            pane_id: snapshot.pane_id.map(|s| s.as_ref().to_string()),
            active_tab_id: snapshot.active_tab_id.map(|s| s.as_ref().to_string()),
            tab_count: snapshot.tab_count.min(u32::MAX as usize) as u32,
            overflow: snapshot.overflow,
            scroll_x_px: snapshot.scroll_x.0,
            max_scroll_x_px: snapshot.max_scroll_x.0,
            scroll_viewport_rect: snapshot.scroll_viewport_rect.map(RectV1::from),
            active_tab_rect: snapshot.active_tab_rect.map(RectV1::from),
            active_visible: snapshot.active_visible,
        }
    }
}

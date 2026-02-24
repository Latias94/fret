#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDiagnosticsSnapshotV1 {
    pub focused_element: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_element_path: Option<String>,
    pub focused_element_node: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_element_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_element_visual_bounds: Option<RectV1>,
    pub active_text_selection: Option<(u64, u64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_text_selection_path: Option<(String, String)>,
    pub hovered_pressable: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_pressable_path: Option<String>,
    pub hovered_pressable_node: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_pressable_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_pressable_visual_bounds: Option<RectV1>,
    pub pressed_pressable: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed_pressable_path: Option<String>,
    pub pressed_pressable_node: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed_pressable_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed_pressable_visual_bounds: Option<RectV1>,
    pub hovered_hover_region: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_hover_region_path: Option<String>,
    pub wants_continuous_frames: bool,
    pub observed_models: Vec<ElementObservedModelsV1>,
    pub observed_globals: Vec<ElementObservedGlobalsV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_layout_queries: Vec<ElementObservedLayoutQueriesV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_query_regions: Vec<ElementLayoutQueryRegionV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<ElementEnvironmentSnapshotV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_environment: Vec<ElementObservedEnvironmentV1>,
    #[serde(default)]
    pub view_cache_reuse_roots: Vec<u64>,
    #[serde(default)]
    pub view_cache_reuse_root_element_counts: Vec<(u64, u32)>,
    #[serde(default)]
    pub view_cache_reuse_root_element_samples: Vec<ElementViewCacheReuseRootElementsSampleV1>,
    #[serde(default)]
    pub retained_keep_alive_roots_len: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_keep_alive_roots_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_keep_alive_roots_tail: Vec<u64>,
    #[serde(default)]
    pub node_entry_root_overwrites: Vec<ElementNodeEntryRootOverwriteV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementViewCacheReuseRootElementsSampleV1 {
    pub root: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<u64>,
    pub elements_len: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements_tail: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementNodeEntryRootOverwriteV1 {
    pub frame_id: u64,
    pub element: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub old_root: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_root_path: Option<String>,
    pub new_root: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_root_path: Option<String>,
    pub old_node: u64,
    pub new_node: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<UiSourceLocationV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedModelsV1 {
    pub element: u64,
    pub models: Vec<(u64, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedGlobalsV1 {
    pub element: u64,
    pub globals: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedLayoutQueriesV1 {
    pub element: u64,
    pub deps_fingerprint: u64,
    pub regions: Vec<ElementObservedLayoutQueryRegionV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedLayoutQueryRegionV1 {
    pub region: u64,
    pub invalidation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_name: Option<String>,
    pub region_revision: u64,
    pub region_changed_this_frame: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_committed_bounds: Option<RectV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementLayoutQueryRegionV1 {
    pub region: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub revision: u64,
    pub changed_this_frame: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committed_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_bounds: Option<RectV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementEnvironmentSnapshotV1 {
    pub viewport_bounds: RectV1,
    pub scale_factor: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefers_reduced_motion: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_scale_factor: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefers_reduced_transparency: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<fret_core::Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contrast_preference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forced_colors_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_pointer_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_area_insets: Option<UiEdgesV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occlusion_insets: Option<UiEdgesV1>,
}

impl ElementEnvironmentSnapshotV1 {
    fn from_diagnostics_snapshot(
        snapshot: &fret_ui::elements::EnvironmentQueryDiagnosticsSnapshot,
    ) -> Self {
        let edges_to_protocol = |value: fret_core::Edges| UiEdgesV1 {
            top_px: value.top.0,
            right_px: value.right.0,
            bottom_px: value.bottom.0,
            left_px: value.left.0,
        };
        Self {
            viewport_bounds: RectV1::from(snapshot.viewport_bounds),
            scale_factor: snapshot.scale_factor,
            color_scheme: snapshot
                .color_scheme
                .map(|s| color_scheme_label(s).to_string()),
            prefers_reduced_motion: snapshot.prefers_reduced_motion,
            text_scale_factor: snapshot.text_scale_factor,
            prefers_reduced_transparency: snapshot.prefers_reduced_transparency,
            accent_color: snapshot.accent_color,
            contrast_preference: snapshot
                .contrast_preference
                .map(|c| contrast_preference_label(c).to_string()),
            forced_colors_mode: snapshot
                .forced_colors_mode
                .map(|m| forced_colors_mode_label(m).to_string()),
            primary_pointer_type: Some(
                viewport_pointer_type_label(snapshot.primary_pointer_type).to_string(),
            ),
            safe_area_insets: snapshot.safe_area_insets.map(edges_to_protocol),
            occlusion_insets: snapshot.occlusion_insets.map(edges_to_protocol),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedEnvironmentV1 {
    pub element: u64,
    pub deps_fingerprint: u64,
    pub keys: Vec<ElementObservedEnvironmentKeyV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedEnvironmentKeyV1 {
    pub key: String,
    pub invalidation: String,
    pub key_revision: u64,
    pub key_changed_this_frame: bool,
}


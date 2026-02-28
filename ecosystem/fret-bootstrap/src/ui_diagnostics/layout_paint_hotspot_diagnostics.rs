#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineSolveV1 {
    pub root_node: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default)]
    pub root_element_kind: Option<String>,
    #[serde(default)]
    pub root_element_path: Option<String>,
    pub solve_time_us: u64,
    pub measure_calls: u64,
    pub measure_cache_hits: u64,
    #[serde(default)]
    pub measure_time_us: u64,
    #[serde(default)]
    pub top_measures: Vec<UiLayoutEngineMeasureHotspotV1>,
}

impl UiLayoutEngineSolveV1 {
    fn from_solve(s: &fret_ui::tree::UiDebugLayoutEngineSolve) -> Self {
        Self {
            root_node: s.root.data().as_ffi(),
            root_element: s.root_element.map(|id| id.0),
            root_element_kind: s.root_element_kind.map(|s| s.to_string()),
            root_element_path: s.root_element_path.clone(),
            solve_time_us: s.solve_time.as_micros().min(u64::MAX as u128) as u64,
            measure_calls: s.measure_calls,
            measure_cache_hits: s.measure_cache_hits,
            measure_time_us: s.measure_time.as_micros().min(u64::MAX as u128) as u64,
            top_measures: s
                .top_measures
                .iter()
                .map(|m| UiLayoutEngineMeasureHotspotV1 {
                    node: m.node.data().as_ffi(),
                    measure_time_us: m.measure_time.as_micros().min(u64::MAX as u128) as u64,
                    calls: m.calls,
                    cache_hits: m.cache_hits,
                    element: m.element.map(|id| id.0),
                    element_kind: m.element_kind.map(|s| s.to_string()),
                    top_children: m
                        .top_children
                        .iter()
                        .map(|c| UiLayoutEngineMeasureChildHotspotV1 {
                            child: c.child.data().as_ffi(),
                            measure_time_us: c.measure_time.as_micros().min(u64::MAX as u128)
                                as u64,
                            calls: c.calls,
                            element: c.element.map(|id| id.0),
                            element_kind: c.element_kind.map(|s| s.to_string()),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub widget_type: String,
    pub layout_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
}

impl UiLayoutHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugLayoutHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            element_path: h.element_path.clone(),
            widget_type: h.widget_type.to_string(),
            layout_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWidgetMeasureHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub widget_type: String,
    pub measure_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
}

impl UiWidgetMeasureHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugWidgetMeasureHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            element_path: h.element_path.clone(),
            widget_type: h.widget_type.to_string(),
            measure_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPaintWidgetHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    pub widget_type: String,
    pub paint_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
    #[serde(default)]
    pub inclusive_scene_ops_delta: u32,
    #[serde(default)]
    pub exclusive_scene_ops_delta: u32,
}

impl UiPaintWidgetHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugPaintWidgetHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            widget_type: h.widget_type.to_string(),
            paint_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_scene_ops_delta: h.inclusive_scene_ops_delta,
            exclusive_scene_ops_delta: h.exclusive_scene_ops_delta,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPaintTextPrepareHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    pub prepare_time_us: u64,
    pub text_len: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_factor: Option<f32>,
    #[serde(default)]
    pub reasons_mask: u16,
}

impl UiPaintTextPrepareHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugPaintTextPrepareHotspot) -> Self {
        fn wrap_as_str(wrap: fret_core::TextWrap) -> &'static str {
            match wrap {
                fret_core::TextWrap::None => "none",
                fret_core::TextWrap::Word => "word",
                fret_core::TextWrap::Balance => "balance",
                fret_core::TextWrap::WordBreak => "word_break",
                fret_core::TextWrap::Grapheme => "grapheme",
            }
        }

        fn overflow_as_str(overflow: fret_core::TextOverflow) -> &'static str {
            match overflow {
                fret_core::TextOverflow::Clip => "clip",
                fret_core::TextOverflow::Ellipsis => "ellipsis",
            }
        }

        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: Some(h.element_kind.to_string()),
            prepare_time_us: h.prepare_time.as_micros().min(u64::MAX as u128) as u64,
            text_len: h.text_len,
            max_width: h.constraints.max_width.map(|v| v.0),
            wrap: Some(wrap_as_str(h.constraints.wrap).to_string()),
            overflow: Some(overflow_as_str(h.constraints.overflow).to_string()),
            scale_factor: Some(h.constraints.scale_factor),
            reasons_mask: h.reasons_mask,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineMeasureHotspotV1 {
    pub node: u64,
    pub measure_time_us: u64,
    pub calls: u64,
    pub cache_hits: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    #[serde(default)]
    pub top_children: Vec<UiLayoutEngineMeasureChildHotspotV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineMeasureChildHotspotV1 {
    pub child: u64,
    pub measure_time_us: u64,
    pub calls: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
}

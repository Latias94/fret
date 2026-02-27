//! Built-in node graph style preset families (UI-only).
//!
//! Presets are paint-only and are applied via `NodeGraphSkin` so they can be switched without
//! rebuilding derived geometry.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use fret_core::scene::DashPatternV1;
use fret_core::{Color, Px};
use serde::Deserialize;

use crate::core::{EdgeId, EdgeKind, Graph, NodeId, PortId, PortKind};

use super::presenter::EdgeRenderHint;
use super::skin::{
    CanvasChromeHint, InteractionChromeHint, NodeChromeHint, NodeGraphSkin, NodeRingHint,
    PortChromeHint, PortShapeHint,
};
use super::style::NodeGraphStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphPresetFamily {
    WorkflowClean,
    SchematicContrast,
    GraphDark,
}

impl NodeGraphPresetFamily {
    pub fn all() -> [Self; 3] {
        [
            Self::WorkflowClean,
            Self::SchematicContrast,
            Self::GraphDark,
        ]
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::WorkflowClean => "WorkflowClean",
            Self::SchematicContrast => "SchematicContrast",
            Self::GraphDark => "GraphDark",
        }
    }

    fn preset_id(self) -> &'static str {
        match self {
            Self::WorkflowClean => "workflow_clean",
            Self::SchematicContrast => "schematic_contrast",
            Self::GraphDark => "graph_dark",
        }
    }
}

#[derive(Debug)]
pub struct NodeGraphPresetSkinV1 {
    rev: AtomicU64,
    index: AtomicUsize,
    presets: Arc<NodeGraphThemePresetsV1>,
    id_to_index: HashMap<String, usize>,
}

impl NodeGraphPresetSkinV1 {
    pub fn new_builtin(initial: NodeGraphPresetFamily) -> Arc<Self> {
        let presets = Arc::new(builtin_presets().clone());
        let mut id_to_index: HashMap<String, usize> = HashMap::new();
        for (i, p) in presets.presets.iter().enumerate() {
            id_to_index.insert(p.id.clone(), i);
        }
        let index = id_to_index.get(initial.preset_id()).copied().unwrap_or(0);
        Arc::new(Self {
            rev: AtomicU64::new(1),
            index: AtomicUsize::new(index),
            presets,
            id_to_index,
        })
    }

    pub fn preset_family(&self) -> NodeGraphPresetFamily {
        let idx = self.index.load(Ordering::Relaxed);
        let id = self
            .presets
            .presets
            .get(idx)
            .map(|p| p.id.as_str())
            .unwrap_or("workflow_clean");
        match id {
            "schematic_contrast" => NodeGraphPresetFamily::SchematicContrast,
            "graph_dark" => NodeGraphPresetFamily::GraphDark,
            _ => NodeGraphPresetFamily::WorkflowClean,
        }
    }

    pub fn set_preset_family(&self, family: NodeGraphPresetFamily) {
        if let Some(idx) = self.id_to_index.get(family.preset_id()).copied() {
            self.index.store(idx, Ordering::Relaxed);
            self.rev.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn cycle(&self) -> NodeGraphPresetFamily {
        let families = NodeGraphPresetFamily::all();
        let current = self.preset_family();
        let next = families
            .iter()
            .copied()
            .cycle()
            .skip_while(|v| *v != current)
            .nth(1)
            .unwrap_or(NodeGraphPresetFamily::WorkflowClean);
        self.set_preset_family(next);
        next
    }

    fn preset(&self) -> &NodeGraphThemePresetV1 {
        let idx = self.index.load(Ordering::Relaxed);
        self.presets
            .presets
            .get(idx)
            .unwrap_or_else(|| &self.presets.presets[0])
    }

    fn node_header_color(&self, graph: &Graph, node: NodeId) -> Color {
        let preset = self.preset();
        let kind = graph
            .nodes
            .get(&node)
            .map(|n| n.kind.0.as_str())
            .unwrap_or("");

        let node_tokens = &preset.paint_only_tokens.node;
        if let Some(v) = node_tokens.header_by_kind.get(kind) {
            return (*v).into();
        }
        if let Some((_a, b)) = kind.rsplit_once('.') {
            if let Some(v) = node_tokens.header_by_kind.get(b) {
                return (*v).into();
            }
        }
        node_tokens.header_background_default.into()
    }

    fn port_tokens_for(&self, graph: &Graph, port: PortId) -> &PortTokensV1 {
        let preset = self.preset();
        let kind = graph
            .ports
            .get(&port)
            .map(|p| p.kind)
            .unwrap_or(PortKind::Data);
        match kind {
            PortKind::Exec => &preset.paint_only_tokens.port.by_port_kind.exec,
            PortKind::Data => &preset.paint_only_tokens.port.by_port_kind.data,
        }
    }
}

impl NodeGraphSkin for NodeGraphPresetSkinV1 {
    fn revision(&self) -> u64 {
        self.rev.load(Ordering::Relaxed)
    }

    fn canvas_chrome_hint(&self, _graph: &Graph, _style: &NodeGraphStyle) -> CanvasChromeHint {
        let tokens = &self.preset().paint_only_tokens;
        CanvasChromeHint {
            background: Some(tokens.canvas.background.into()),
            grid_minor: Some(tokens.grid.minor_color.into()),
            grid_major: Some(tokens.grid.major_color.into()),
            grid_line_width_px: self
                .preset()
                .layout_tokens
                .as_ref()
                .and_then(|t| t.grid_minor_width_px),
        }
    }

    fn interaction_chrome_hint(
        &self,
        _graph: &Graph,
        _style: &NodeGraphStyle,
    ) -> InteractionChromeHint {
        let tokens = &self.preset().paint_only_tokens;
        InteractionChromeHint {
            hover: Some(tokens.states.hover.color.into()),
            invalid: Some(tokens.states.invalid.color.into()),
            convertible: Some(tokens.states.convertible.color.into()),
            preview_wire: Some(tokens.wire.preview_color.into()),
            dash_preview: Some(tokens.wire.dash_preview.into_dash()),
            dash_invalid: Some(tokens.wire.dash_invalid.into_dash()),
            dash_emphasis: Some(tokens.wire.dash_emphasis.into_dash()),
        }
    }

    fn node_chrome_hint(
        &self,
        graph: &Graph,
        node: NodeId,
        _style: &NodeGraphStyle,
        selected: bool,
    ) -> NodeChromeHint {
        let tokens = &self.preset().paint_only_tokens;
        NodeChromeHint {
            background: Some(tokens.node.body_background.into()),
            border: Some(tokens.node.border.into()),
            border_selected: Some(tokens.node.border_selected.into()),
            header_background: Some(self.node_header_color(graph, node)),
            title_text: Some(tokens.node.title_text.into()),
            ring_selected: selected.then_some(tokens.node.ring_selected.into_ring()),
            ..NodeChromeHint::default()
        }
    }

    fn node_chrome_hint_with_state(
        &self,
        graph: &Graph,
        node: NodeId,
        style: &NodeGraphStyle,
        selected: bool,
        focused: bool,
    ) -> NodeChromeHint {
        let mut hint = self.node_chrome_hint(graph, node, style, selected);
        if focused {
            hint.ring_focused = Some(
                self.preset()
                    .paint_only_tokens
                    .node
                    .ring_focused
                    .into_ring(),
            );
        }
        hint
    }

    fn port_chrome_hint(
        &self,
        graph: &Graph,
        port: PortId,
        _style: &NodeGraphStyle,
        _base_fill: Color,
    ) -> PortChromeHint {
        let tokens = self.port_tokens_for(graph, port);
        PortChromeHint {
            fill: Some(tokens.fill.into()),
            stroke: Some(tokens.stroke.into()),
            stroke_width: Some(tokens.stroke_width_px),
            inner_scale: Some(tokens.inner_scale),
            shape: Some(tokens.shape),
        }
    }

    fn edge_render_hint(
        &self,
        graph: &Graph,
        edge: EdgeId,
        _style: &NodeGraphStyle,
        base: &EdgeRenderHint,
        _selected: bool,
        _hovered: bool,
    ) -> EdgeRenderHint {
        let tokens = &self.preset().paint_only_tokens.wire;
        let kind = graph
            .edges
            .get(&edge)
            .map(|e| e.kind)
            .unwrap_or(EdgeKind::Data);
        let mut out = base.clone();
        out.color = Some(match kind {
            EdgeKind::Data => tokens.data_color.into(),
            EdgeKind::Exec => tokens.exec_color.into(),
        });
        out
    }
}

fn builtin_presets() -> &'static NodeGraphThemePresetsV1 {
    static PRESETS: OnceLock<NodeGraphThemePresetsV1> = OnceLock::new();
    PRESETS.get_or_init(|| {
        let raw = include_str!("../../../../themes/node-graph-presets.v1.json");
        serde_json::from_str(raw).expect("builtin node graph presets must parse")
    })
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct RgbaV1 {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl From<RgbaV1> for Color {
    fn from(v: RgbaV1) -> Self {
        Color {
            r: v.r,
            g: v.g,
            b: v.b,
            a: v.a,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct DashPatternTokensV1 {
    dash_px: f32,
    gap_px: f32,
    phase_px: f32,
}

impl DashPatternTokensV1 {
    fn into_dash(self) -> DashPatternV1 {
        DashPatternV1::new(Px(self.dash_px), Px(self.gap_px), Px(self.phase_px))
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct NodeRingTokensV1 {
    color: RgbaV1,
    width_px: f32,
    pad_px: f32,
}

impl NodeRingTokensV1 {
    fn into_ring(self) -> NodeRingHint {
        NodeRingHint {
            color: self.color.into(),
            width: self.width_px,
            pad: self.pad_px,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct NodeGraphThemePresetsV1 {
    #[allow(dead_code)]
    schema_version: String,
    #[allow(dead_code)]
    notes: String,
    presets: Vec<NodeGraphThemePresetV1>,
}

#[derive(Debug, Clone, Deserialize)]
struct NodeGraphThemePresetV1 {
    id: String,
    #[allow(dead_code)]
    display_name: String,
    #[allow(dead_code)]
    intent: String,
    paint_only_tokens: PaintOnlyTokensV1,
    #[serde(default)]
    layout_tokens: Option<LayoutTokensV1>,
    #[serde(default)]
    #[allow(dead_code)]
    interaction_state_matrix: serde_json::Value,
    #[serde(default)]
    #[allow(dead_code)]
    example_compositions: serde_json::Value,
    #[serde(default)]
    #[allow(dead_code)]
    a11y_notes: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutTokensV1 {
    #[allow(dead_code)]
    optional: bool,
    #[serde(default)]
    grid_minor_width_px: Option<f32>,
    #[serde(default)]
    #[allow(dead_code)]
    grid_major_width_px: Option<f32>,
    #[allow(dead_code)]
    node_corner_radius_px: Option<f32>,
    #[allow(dead_code)]
    node_header_height_px: Option<f32>,
    #[allow(dead_code)]
    pin_radius_px: Option<f32>,
    #[allow(dead_code)]
    wire_width_px: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct PaintOnlyTokensV1 {
    canvas: CanvasTokensV1,
    grid: GridTokensV1,
    #[allow(dead_code)]
    text: TextTokensV1,
    node: NodeTokensV1,
    port: PortThemeTokensV1,
    wire: WireTokensV1,
    states: StateTokensV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct CanvasTokensV1 {
    background: RgbaV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct GridTokensV1 {
    minor_color: RgbaV1,
    major_color: RgbaV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct TextTokensV1 {
    #[allow(dead_code)]
    primary: RgbaV1,
    #[allow(dead_code)]
    muted: RgbaV1,
}

#[derive(Debug, Clone, Deserialize)]
struct NodeTokensV1 {
    body_background: RgbaV1,
    border: RgbaV1,
    border_selected: RgbaV1,
    header_background_default: RgbaV1,
    header_by_kind: HashMap<String, RgbaV1>,
    title_text: RgbaV1,
    ring_selected: NodeRingTokensV1,
    ring_focused: NodeRingTokensV1,
}

#[derive(Debug, Clone, Deserialize)]
struct PortThemeTokensV1 {
    by_port_kind: PortKindTokensV1,
}

#[derive(Debug, Clone, Deserialize)]
struct PortKindTokensV1 {
    data: PortTokensV1,
    exec: PortTokensV1,
    #[allow(dead_code)]
    preview: PortTokensV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct PortTokensV1 {
    fill: RgbaV1,
    stroke: RgbaV1,
    stroke_width_px: f32,
    inner_scale: f32,
    shape: PortShapeHint,
}

#[derive(Debug, Clone, Deserialize)]
struct WireTokensV1 {
    data_color: RgbaV1,
    exec_color: RgbaV1,
    preview_color: RgbaV1,
    dash_preview: DashPatternTokensV1,
    dash_invalid: DashPatternTokensV1,
    dash_emphasis: DashPatternTokensV1,
}

#[derive(Debug, Clone, Deserialize)]
struct StateTokensV1 {
    hover: StateColorV1,
    invalid: StateColorV1,
    convertible: StateColorV1,
    #[allow(dead_code)]
    disabled: DisabledStateV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct StateColorV1 {
    color: RgbaV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct DisabledStateV1 {
    #[allow(dead_code)]
    alpha_mul: f32,
}

// Satisfy `serde` for `PortShapeHint` without exposing a serde dependency from the type itself.
impl<'de> Deserialize<'de> for PortShapeHint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Circle" => Ok(PortShapeHint::Circle),
            "Diamond" => Ok(PortShapeHint::Diamond),
            "Triangle" => Ok(PortShapeHint::Triangle),
            _ => Ok(PortShapeHint::Circle),
        }
    }
}

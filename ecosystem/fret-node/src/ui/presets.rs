//! Built-in node graph style preset families (UI-only).
//!
//! Presets are paint-only and are applied via `NodeGraphSkin` so they can be switched without
//! rebuilding derived geometry.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use fret_core::Color;
use fret_ui::ThemeSnapshot;
use fret_ui_kit::node_graph::presets::{
    EdgeMarkerKindTokensV1, EdgeMarkerTokensV1, NodeGraphThemePresetV1, NodeGraphThemePresetsV1,
    NodeRingTokensV1, PortShapeKindV1, PortTokensV1, WireHighlightTokensV1,
};

use crate::core::{EdgeId, EdgeKind, Graph, NodeId, PortId, PortKind};

use super::presenter::EdgeRenderHint;
use super::presenter::{EdgeMarker, EdgeMarkerKind};
use super::skin::{
    CanvasChromeHint, InteractionChromeHint, NodeChromeHint, NodeGraphSkin, NodeRingHint,
    NodeShadowHint, PortChromeHint, PortShapeHint, WireGlowHint, WireHighlightHint,
    WireOutlineHint,
};
use super::style::NodeGraphStyle;

pub use fret_ui_kit::node_graph::presets::NodeGraphPresetFamily;

fn node_ring_tokens_into_hint(tokens: NodeRingTokensV1) -> NodeRingHint {
    NodeRingHint {
        color: tokens.color.into(),
        width: tokens.width_px,
        pad: tokens.pad_px,
    }
}

fn port_shape_kind_into_hint(shape: PortShapeKindV1) -> PortShapeHint {
    match shape {
        PortShapeKindV1::Circle => PortShapeHint::Circle,
        PortShapeKindV1::Diamond => PortShapeHint::Diamond,
        PortShapeKindV1::Triangle => PortShapeHint::Triangle,
    }
}

fn edge_marker_tokens_into_marker(tokens: EdgeMarkerTokensV1) -> EdgeMarker {
    EdgeMarker {
        kind: match tokens.kind {
            EdgeMarkerKindTokensV1::Arrow => EdgeMarkerKind::Arrow,
        },
        size: tokens.size_px,
    }
}

fn wire_highlight_tokens_into_hint(tokens: WireHighlightTokensV1) -> WireHighlightHint {
    WireHighlightHint {
        width_mul: tokens.width_mul,
        alpha_mul: tokens.alpha_mul,
        color: tokens.color.map(|c| c.into()),
    }
}

#[derive(Debug)]
pub struct NodeGraphPresetSkinV1 {
    rev: AtomicU64,
    index: AtomicUsize,
    wire_glow_enabled: AtomicBool,
    wire_highlight_enabled: AtomicBool,
    edge_markers_enabled: AtomicBool,
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
            wire_glow_enabled: AtomicBool::new(true),
            wire_highlight_enabled: AtomicBool::new(true),
            edge_markers_enabled: AtomicBool::new(false),
            presets,
            id_to_index,
        })
    }

    pub fn new_from_snapshot(theme: ThemeSnapshot, initial: NodeGraphPresetFamily) -> Arc<Self> {
        let presets = Arc::new(fret_ui_kit::node_graph::presets::theme_derived_presets(
            &theme,
        ));
        let mut id_to_index: HashMap<String, usize> = HashMap::new();
        for (i, p) in presets.presets.iter().enumerate() {
            id_to_index.insert(p.id.clone(), i);
        }
        let index = id_to_index.get(initial.preset_id()).copied().unwrap_or(0);
        Arc::new(Self {
            rev: AtomicU64::new(theme.revision.max(1)),
            index: AtomicUsize::new(index),
            wire_glow_enabled: AtomicBool::new(true),
            wire_highlight_enabled: AtomicBool::new(true),
            edge_markers_enabled: AtomicBool::new(false),
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

    pub fn wire_glow_enabled(&self) -> bool {
        self.wire_glow_enabled.load(Ordering::Relaxed)
    }

    pub fn toggle_wire_glow(&self) -> bool {
        let next = !self.wire_glow_enabled();
        self.wire_glow_enabled.store(next, Ordering::Relaxed);
        self.rev.fetch_add(1, Ordering::Relaxed);
        next
    }

    pub fn wire_highlight_enabled(&self) -> bool {
        self.wire_highlight_enabled.load(Ordering::Relaxed)
    }

    pub fn toggle_wire_highlight(&self) -> bool {
        let next = !self.wire_highlight_enabled();
        self.wire_highlight_enabled.store(next, Ordering::Relaxed);
        self.rev.fetch_add(1, Ordering::Relaxed);
        next
    }

    pub fn edge_markers_enabled(&self) -> bool {
        self.edge_markers_enabled.load(Ordering::Relaxed)
    }

    pub fn toggle_edge_markers(&self) -> bool {
        let next = !self.edge_markers_enabled();
        self.edge_markers_enabled.store(next, Ordering::Relaxed);
        self.rev.fetch_add(1, Ordering::Relaxed);
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
        let wire_glow_enabled = self.wire_glow_enabled.load(Ordering::Relaxed);
        let wire_highlight_enabled = self.wire_highlight_enabled.load(Ordering::Relaxed);
        let tokens = &self.preset().paint_only_tokens;
        let preset_id = self.preset().id.as_str();
        let (wire_glow_selected, wire_glow_preview) = if wire_glow_enabled {
            match preset_id {
                "schematic_contrast" => (
                    Some(WireGlowHint {
                        blur_radius_px: 4.0,
                        downsample: 1,
                        alpha_mul: 0.55,
                    }),
                    Some(WireGlowHint {
                        blur_radius_px: 4.0,
                        downsample: 1,
                        alpha_mul: 0.45,
                    }),
                ),
                "graph_dark" => (
                    Some(WireGlowHint {
                        blur_radius_px: 10.0,
                        downsample: 2,
                        alpha_mul: 0.70,
                    }),
                    Some(WireGlowHint {
                        blur_radius_px: 8.0,
                        downsample: 2,
                        alpha_mul: 0.60,
                    }),
                ),
                _ => (
                    Some(WireGlowHint {
                        blur_radius_px: 6.0,
                        downsample: 2,
                        alpha_mul: 0.45,
                    }),
                    Some(WireGlowHint {
                        blur_radius_px: 6.0,
                        downsample: 2,
                        alpha_mul: 0.35,
                    }),
                ),
            }
        } else {
            (None, None)
        };

        let (wire_highlight_selected, wire_highlight_hovered) = (
            wire_highlight_enabled
                .then(|| {
                    tokens
                        .wire
                        .highlight_selected
                        .map(wire_highlight_tokens_into_hint)
                })
                .flatten(),
            wire_highlight_enabled
                .then(|| {
                    tokens
                        .wire
                        .highlight_hovered
                        .map(wire_highlight_tokens_into_hint)
                })
                .flatten(),
        );

        let outline_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: match preset_id {
                "schematic_contrast" => 0.35,
                "graph_dark" => 0.45,
                _ => 0.25,
            },
        };
        let outline_base_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: match preset_id {
                "schematic_contrast" => 0.18,
                "graph_dark" => 0.24,
                _ => 0.14,
            },
        };
        let (wire_outline_selected, wire_outline_preview, wire_outline_base) = (
            Some(WireOutlineHint {
                width_mul: 1.8,
                color: outline_color,
            }),
            Some(WireOutlineHint {
                width_mul: 1.8,
                color: outline_color,
            }),
            Some(WireOutlineHint {
                width_mul: 1.35,
                color: outline_base_color,
            }),
        );
        InteractionChromeHint {
            hover: Some(tokens.states.hover.color.into()),
            invalid: Some(tokens.states.invalid.color.into()),
            convertible: Some(tokens.states.convertible.color.into()),
            preview_wire: Some(tokens.wire.preview_color.into()),
            dash_preview: Some(tokens.wire.dash_preview.into_dash()),
            dash_invalid: Some(tokens.wire.dash_invalid.into_dash()),
            dash_emphasis: Some(tokens.wire.dash_emphasis.into_dash()),
            wire_glow_selected,
            wire_glow_preview,
            wire_highlight_selected,
            wire_highlight_hovered,
            wire_outline_selected,
            wire_outline_preview,
            wire_outline_base,
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
        let preset_id = self.preset().id.as_str();
        let shadow = selected.then(|| {
            let mut black = Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };
            match preset_id {
                "schematic_contrast" => {
                    black.a = 0.28;
                    NodeShadowHint {
                        offset_x_px: 0.0,
                        offset_y_px: 1.0,
                        blur_radius_px: 4.0,
                        downsample: 1,
                        color: black,
                    }
                }
                "graph_dark" => {
                    let mut glow: Color = tokens.node.border_selected.into();
                    glow.a = (glow.a * 0.55).clamp(0.0, 1.0);
                    NodeShadowHint {
                        offset_x_px: 0.0,
                        offset_y_px: 0.0,
                        blur_radius_px: 10.0,
                        downsample: 2,
                        color: glow,
                    }
                }
                _ => {
                    black.a = 0.18;
                    NodeShadowHint {
                        offset_x_px: 0.0,
                        offset_y_px: 1.0,
                        blur_radius_px: 6.0,
                        downsample: 2,
                        color: black,
                    }
                }
            }
        });
        NodeChromeHint {
            background: Some(tokens.node.body_background.into()),
            border: Some(tokens.node.border.into()),
            border_selected: Some(tokens.node.border_selected.into()),
            header_background: Some(self.node_header_color(graph, node)),
            title_text: Some(tokens.node.title_text.into()),
            ring_selected: selected
                .then_some(node_ring_tokens_into_hint(tokens.node.ring_selected)),
            shadow,
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
            hint.ring_focused = Some(node_ring_tokens_into_hint(
                self.preset().paint_only_tokens.node.ring_focused,
            ));
            if hint.shadow.is_none() {
                hint.shadow = Some(NodeShadowHint {
                    offset_x_px: 0.0,
                    offset_y_px: 0.0,
                    blur_radius_px: 10.0,
                    downsample: 2,
                    color: self.preset().paint_only_tokens.node.border_selected.into(),
                });
            }
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
            shape: Some(port_shape_kind_into_hint(tokens.shape)),
        }
    }

    fn edge_render_hint(
        &self,
        graph: &Graph,
        edge: EdgeId,
        _style: &NodeGraphStyle,
        base: &EdgeRenderHint,
        selected: bool,
        hovered: bool,
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
        if self.edge_markers_enabled.load(Ordering::Relaxed) {
            let selected_mul = tokens
                .marker_size_mul_selected
                .unwrap_or(1.0)
                .clamp(0.0, 4.0);
            let hovered_mul = tokens
                .marker_size_mul_hovered
                .unwrap_or(1.0)
                .clamp(0.0, 4.0);
            let state_mul = if hovered {
                hovered_mul
            } else if selected {
                selected_mul
            } else {
                1.0
            };

            match kind {
                EdgeKind::Exec => {
                    if out.start_marker.is_none()
                        && let Some(marker) = tokens.marker_exec_start
                    {
                        let mut marker = edge_marker_tokens_into_marker(marker);
                        marker.size *= state_mul;
                        out.start_marker = Some(marker);
                    }
                    if out.end_marker.is_none()
                        && let Some(marker) = tokens.marker_exec_end
                    {
                        let mut marker = edge_marker_tokens_into_marker(marker);
                        marker.size *= state_mul;
                        out.end_marker = Some(marker);
                    }
                }
                EdgeKind::Data => {
                    if out.start_marker.is_none()
                        && let Some(marker) = tokens.marker_data_start
                    {
                        let mut marker = edge_marker_tokens_into_marker(marker);
                        marker.size *= state_mul;
                        out.start_marker = Some(marker);
                    }
                    if out.end_marker.is_none()
                        && let Some(marker) = tokens.marker_data_end
                    {
                        let mut marker = edge_marker_tokens_into_marker(marker);
                        marker.size *= state_mul;
                        out.end_marker = Some(marker);
                    }
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui_kit::node_graph::presets::RgbaV1;

    fn assert_close(a: f32, b: f32) {
        assert!(
            (a - b).abs() <= 1.0e-6,
            "expected {a:?} to be close to {b:?}"
        );
    }

    fn assert_rgba_close(a: RgbaV1, b: RgbaV1) {
        assert_close(a.r, b.r);
        assert_close(a.g, b.g);
        assert_close(a.b, b.b);
        assert_close(a.a, b.a);
    }

    #[test]
    fn builtin_presets_wire_highlight_tokens_are_present() {
        let presets = builtin_presets();
        let cases = [
            ("workflow_clean", (0.65, 0.80, None), (0.70, 0.95, None)),
            ("schematic_contrast", (0.70, 0.90, None), (0.75, 1.0, None)),
            (
                "graph_dark",
                (
                    0.65,
                    0.85,
                    Some(RgbaV1 {
                        r: 0.22,
                        g: 0.74,
                        b: 0.97,
                        a: 1.0,
                    }),
                ),
                (
                    0.70,
                    0.95,
                    Some(RgbaV1 {
                        r: 0.4,
                        g: 0.91,
                        b: 0.98,
                        a: 1.0,
                    }),
                ),
            ),
        ];

        for (id, selected, hovered) in cases {
            let preset = presets
                .presets
                .iter()
                .find(|p| p.id == id)
                .unwrap_or_else(|| panic!("expected preset {id:?}"));
            let sel = preset
                .paint_only_tokens
                .wire
                .highlight_selected
                .unwrap_or_else(|| panic!("expected wire.highlight_selected for {id:?}"));
            let hov = preset
                .paint_only_tokens
                .wire
                .highlight_hovered
                .unwrap_or_else(|| panic!("expected wire.highlight_hovered for {id:?}"));

            assert_close(sel.width_mul, selected.0);
            assert_close(sel.alpha_mul, selected.1);
            match (sel.color, selected.2) {
                (None, None) => {}
                (Some(a), Some(b)) => assert_rgba_close(a, b),
                _ => panic!("selected highlight color mismatch for {id:?}"),
            }

            assert_close(hov.width_mul, hovered.0);
            assert_close(hov.alpha_mul, hovered.1);
            match (hov.color, hovered.2) {
                (None, None) => {}
                (Some(a), Some(b)) => assert_rgba_close(a, b),
                _ => panic!("hovered highlight color mismatch for {id:?}"),
            }
        }
    }

    #[test]
    fn builtin_presets_wire_marker_tokens_are_present() {
        let presets = builtin_presets();
        for id in ["workflow_clean", "schematic_contrast", "graph_dark"] {
            let preset = presets
                .presets
                .iter()
                .find(|p| p.id == id)
                .unwrap_or_else(|| panic!("expected preset {id:?}"));
            let marker = preset
                .paint_only_tokens
                .wire
                .marker_exec_start
                .unwrap_or_else(|| panic!("expected wire.marker_exec_start for {id:?}"));
            assert_eq!(marker.kind, EdgeMarkerKindTokensV1::Arrow);
            assert_close(marker.size_px, 8.0);

            let marker = preset
                .paint_only_tokens
                .wire
                .marker_exec_end
                .unwrap_or_else(|| panic!("expected wire.marker_exec_end for {id:?}"));
            assert_eq!(marker.kind, EdgeMarkerKindTokensV1::Arrow);
            assert_close(marker.size_px, 12.0);

            let selected_mul = preset
                .paint_only_tokens
                .wire
                .marker_size_mul_selected
                .unwrap_or_else(|| panic!("expected wire.marker_size_mul_selected for {id:?}"));
            assert_close(selected_mul, 1.15);

            let hovered_mul = preset
                .paint_only_tokens
                .wire
                .marker_size_mul_hovered
                .unwrap_or_else(|| panic!("expected wire.marker_size_mul_hovered for {id:?}"));
            assert_close(hovered_mul, 1.25);
        }
    }
}

//! Built-in node graph style preset families (UI-only).
//!
//! Presets are paint-only and are applied via `NodeGraphSkin` so they can be switched without
//! rebuilding derived geometry.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use fret_core::scene::DashPatternV1;
use fret_core::window::ColorScheme;
use fret_core::{Color, Px};
use fret_ui::ThemeSnapshot;
use serde::Deserialize;

use crate::core::{EdgeId, EdgeKind, Graph, NodeId, PortId, PortKind};

use super::presenter::EdgeRenderHint;
use super::presenter::{EdgeMarker, EdgeMarkerKind};
use super::skin::{
    CanvasChromeHint, InteractionChromeHint, NodeChromeHint, NodeGraphSkin, NodeRingHint,
    NodeShadowHint, PortChromeHint, PortShapeHint, WireGlowHint, WireHighlightHint,
    WireOutlineHint,
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
        let presets = Arc::new(theme_derived_presets(&theme));
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
                        .map(WireHighlightTokensV1::into_hint)
                })
                .flatten(),
            wire_highlight_enabled
                .then(|| {
                    tokens
                        .wire
                        .highlight_hovered
                        .map(WireHighlightTokensV1::into_hint)
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
                    black.a = 0.14;
                    NodeShadowHint {
                        offset_x_px: 0.0,
                        offset_y_px: 1.5,
                        blur_radius_px: 8.0,
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
            ring_selected: selected.then_some(tokens.node.ring_selected.into_ring()),
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
            hint.ring_focused = Some(
                self.preset()
                    .paint_only_tokens
                    .node
                    .ring_focused
                    .into_ring(),
            );
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
        if self.edge_markers_enabled.load(Ordering::Relaxed) && kind == EdgeKind::Exec {
            if out.end_marker.is_none()
                && let Some(marker) = tokens.marker_exec_end
            {
                out.end_marker = Some(marker.into_marker());
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

fn theme_derived_presets(theme: &ThemeSnapshot) -> NodeGraphThemePresetsV1 {
    NodeGraphThemePresetsV1 {
        schema_version: "node_graph_theme_presets.v1".to_string(),
        notes: "derived from ThemeSnapshot (with opt-out for GraphDark on light themes)"
            .to_string(),
        presets: vec![
            theme_derived_preset(theme, NodeGraphPresetFamily::WorkflowClean),
            theme_derived_preset(theme, NodeGraphPresetFamily::SchematicContrast),
            theme_derived_preset(theme, NodeGraphPresetFamily::GraphDark),
        ],
    }
}

fn theme_derived_preset(
    theme: &ThemeSnapshot,
    family: NodeGraphPresetFamily,
) -> NodeGraphThemePresetV1 {
    fn alpha(mut c: Color, a: f32) -> Color {
        c.a = a;
        c
    }

    fn mix(a: Color, b: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: a.r + (b.r - a.r) * t,
            g: a.g + (b.g - a.g) * t,
            b: a.b + (b.b - a.b) * t,
            a: a.a + (b.a - a.a) * t,
        }
    }

    fn tint(base: Color, accent: Color, amount: f32) -> Color {
        let mut out = mix(base, accent, amount);
        out.a = 1.0;
        out
    }

    let scheme_is_dark = theme.color_scheme == Some(ColorScheme::Dark);

    let background = theme.color_token("background");
    let foreground = theme.color_token("foreground");
    let border = theme.color_token("border");
    let ring = theme.color_token("ring");
    let card = theme.color_token("card");
    let card_foreground = theme.color_token("card-foreground");
    let muted_foreground = theme.color_token("muted-foreground");
    let accent = theme.color_token("accent");
    let primary = theme.color_token("primary");
    let destructive = theme.color_token("destructive");

    let chart_1 = theme.color_token("chart-1");
    let chart_2 = theme.color_token("chart-2");
    let chart_3 = theme.color_token("chart-3");
    let chart_4 = theme.color_token("chart-4");
    let chart_5 = theme.color_token("chart-5");

    let kind_colors = [
        ("source", chart_1),
        ("compute", chart_2),
        ("condition", chart_3),
        ("output", chart_4),
        ("utility", chart_5),
        ("preview", destructive),
    ];

    let (canvas_bg, grid_minor, grid_major, node_bg, node_border, node_border_selected, title_text) =
        match family {
            NodeGraphPresetFamily::WorkflowClean => (
                background,
                alpha(border, 0.50),
                alpha(border, 0.80),
                card,
                alpha(border, 1.0),
                alpha(ring, 1.0),
                card_foreground,
            ),
            NodeGraphPresetFamily::SchematicContrast => (
                background,
                alpha(border, 0.90),
                alpha(border, 1.0),
                card,
                alpha(foreground, 1.0),
                alpha(foreground, 1.0),
                theme.color_token("primary-foreground"),
            ),
            NodeGraphPresetFamily::GraphDark => {
                if scheme_is_dark {
                    (
                        background,
                        alpha(border, 0.80),
                        alpha(border, 1.0),
                        alpha(card, 0.95),
                        alpha(border, 1.0),
                        alpha(ring, 1.0),
                        foreground,
                    )
                } else {
                    // Opt-out baseline for a guaranteed dark canvas on light themes.
                    (
                        Color {
                            r: 0.07,
                            g: 0.09,
                            b: 0.15,
                            a: 1.0,
                        },
                        Color {
                            r: 0.12,
                            g: 0.16,
                            b: 0.22,
                            a: 1.0,
                        },
                        Color {
                            r: 0.22,
                            g: 0.25,
                            b: 0.32,
                            a: 1.0,
                        },
                        Color {
                            r: 0.12,
                            g: 0.16,
                            b: 0.22,
                            a: 0.95,
                        },
                        Color {
                            r: 0.29,
                            g: 0.33,
                            b: 0.39,
                            a: 1.0,
                        },
                        alpha(ring, 1.0),
                        Color {
                            r: 0.95,
                            g: 0.96,
                            b: 0.96,
                            a: 1.0,
                        },
                    )
                }
            }
        };

    let header_default = match family {
        NodeGraphPresetFamily::WorkflowClean => tint(card, border, 0.10),
        NodeGraphPresetFamily::SchematicContrast => alpha(theme.color_token("secondary"), 1.0),
        NodeGraphPresetFamily::GraphDark => tint(node_bg, border, 0.20),
    };

    let header_by_kind: HashMap<String, RgbaV1> = kind_colors
        .into_iter()
        .map(|(k, c)| {
            let c = match family {
                NodeGraphPresetFamily::WorkflowClean => tint(card, c, 0.22),
                NodeGraphPresetFamily::SchematicContrast => alpha(c, 1.0),
                NodeGraphPresetFamily::GraphDark => {
                    if scheme_is_dark {
                        tint(node_bg, c, 0.35)
                    } else {
                        alpha(c, 1.0)
                    }
                }
            };
            (k.to_string(), c.into())
        })
        .collect();

    let (ring_selected, ring_focused) = match family {
        NodeGraphPresetFamily::WorkflowClean => (
            NodeRingTokensV1 {
                color: alpha(ring, 0.40).into(),
                width_px: 3.0,
                pad_px: 2.0,
            },
            NodeRingTokensV1 {
                color: alpha(ring, 0.60).into(),
                width_px: 2.0,
                pad_px: 1.0,
            },
        ),
        NodeGraphPresetFamily::SchematicContrast => (
            NodeRingTokensV1 {
                color: theme.color_token("chart-4").into(),
                width_px: 4.0,
                pad_px: 0.0,
            },
            NodeRingTokensV1 {
                color: theme.color_token("chart-1").into(),
                width_px: 4.0,
                pad_px: 0.0,
            },
        ),
        NodeGraphPresetFamily::GraphDark => (
            NodeRingTokensV1 {
                color: alpha(ring, 1.0).into(),
                width_px: 3.0,
                pad_px: 2.0,
            },
            NodeRingTokensV1 {
                color: alpha(theme.color_token("chart-1"), 1.0).into(),
                width_px: 3.0,
                pad_px: 2.0,
            },
        ),
    };

    let (port_data, port_exec, port_preview) = match family {
        NodeGraphPresetFamily::WorkflowClean => (
            PortTokensV1 {
                fill: tint(card, primary, 0.35).into(),
                stroke: alpha(muted_foreground, 0.90).into(),
                stroke_width_px: 1.0,
                inner_scale: 1.0,
                shape: PortShapeHint::Circle,
            },
            PortTokensV1 {
                fill: card.into(),
                stroke: alpha(foreground, 0.80).into(),
                stroke_width_px: 1.5,
                inner_scale: 0.0,
                shape: PortShapeHint::Triangle,
            },
            PortTokensV1 {
                fill: tint(card, accent, 0.35).into(),
                stroke: alpha(muted_foreground, 0.90).into(),
                stroke_width_px: 1.0,
                inner_scale: 0.5,
                shape: PortShapeHint::Diamond,
            },
        ),
        NodeGraphPresetFamily::SchematicContrast => (
            PortTokensV1 {
                fill: alpha(theme.color_token("chart-2"), 1.0).into(),
                stroke: alpha(foreground, 1.0).into(),
                stroke_width_px: 2.0,
                inner_scale: 1.0,
                shape: PortShapeHint::Circle,
            },
            PortTokensV1 {
                fill: alpha(theme.color_token("chart-3"), 1.0).into(),
                stroke: alpha(foreground, 1.0).into(),
                stroke_width_px: 2.0,
                inner_scale: 0.0,
                shape: PortShapeHint::Triangle,
            },
            PortTokensV1 {
                fill: alpha(theme.color_token("chart-4"), 1.0).into(),
                stroke: alpha(foreground, 1.0).into(),
                stroke_width_px: 2.0,
                inner_scale: 1.0,
                shape: PortShapeHint::Diamond,
            },
        ),
        NodeGraphPresetFamily::GraphDark => (
            PortTokensV1 {
                fill: alpha(theme.color_token("chart-2"), 0.65).into(),
                stroke: alpha(theme.color_token("chart-2"), 1.0).into(),
                stroke_width_px: 1.5,
                inner_scale: 1.0,
                shape: PortShapeHint::Circle,
            },
            PortTokensV1 {
                fill: alpha(destructive, 0.65).into(),
                stroke: alpha(destructive, 1.0).into(),
                stroke_width_px: 1.5,
                inner_scale: 0.0,
                shape: PortShapeHint::Triangle,
            },
            PortTokensV1 {
                fill: alpha(theme.color_token("chart-4"), 0.65).into(),
                stroke: alpha(theme.color_token("chart-4"), 1.0).into(),
                stroke_width_px: 1.5,
                inner_scale: 0.5,
                shape: PortShapeHint::Diamond,
            },
        ),
    };

    let (wire_data, wire_exec, wire_preview, dash_preview, dash_invalid, dash_emphasis) =
        match family {
            NodeGraphPresetFamily::WorkflowClean => (
                alpha(muted_foreground, 1.0),
                alpha(foreground, 1.0),
                alpha(border, 1.0),
                DashPatternTokensV1 {
                    dash_px: 4.0,
                    gap_px: 4.0,
                    phase_px: 0.0,
                },
                DashPatternTokensV1 {
                    dash_px: 6.0,
                    gap_px: 3.0,
                    phase_px: 0.0,
                },
                DashPatternTokensV1 {
                    dash_px: 2.0,
                    gap_px: 2.0,
                    phase_px: 0.0,
                },
            ),
            NodeGraphPresetFamily::SchematicContrast => (
                alpha(theme.color_token("chart-2"), 1.0),
                alpha(theme.color_token("chart-3"), 1.0),
                alpha(theme.color_token("chart-4"), 1.0),
                DashPatternTokensV1 {
                    dash_px: 8.0,
                    gap_px: 4.0,
                    phase_px: 0.0,
                },
                DashPatternTokensV1 {
                    dash_px: 6.0,
                    gap_px: 3.0,
                    phase_px: 0.0,
                },
                DashPatternTokensV1 {
                    dash_px: 2.0,
                    gap_px: 2.0,
                    phase_px: 0.0,
                },
            ),
            NodeGraphPresetFamily::GraphDark => (
                alpha(theme.color_token("chart-2"), 1.0),
                alpha(destructive, 1.0),
                alpha(theme.color_token("chart-4"), 1.0),
                DashPatternTokensV1 {
                    dash_px: 6.0,
                    gap_px: 6.0,
                    phase_px: 0.0,
                },
                DashPatternTokensV1 {
                    dash_px: 6.0,
                    gap_px: 3.0,
                    phase_px: 0.0,
                },
                DashPatternTokensV1 {
                    dash_px: 10.0,
                    gap_px: 2.0,
                    phase_px: 0.0,
                },
            ),
        };

    let (hover, invalid, convertible) = match family {
        NodeGraphPresetFamily::WorkflowClean => {
            (alpha(ring, 1.0), alpha(destructive, 1.0), primary)
        }
        NodeGraphPresetFamily::SchematicContrast => (
            alpha(foreground, 1.0),
            alpha(destructive, 1.0),
            alpha(theme.color_token("chart-2"), 1.0),
        ),
        NodeGraphPresetFamily::GraphDark => (
            alpha(theme.color_token("chart-1"), 1.0),
            alpha(destructive, 1.0),
            alpha(theme.color_token("chart-2"), 1.0),
        ),
    };

    NodeGraphThemePresetV1 {
        id: family.preset_id().to_string(),
        display_name: family.display_name().to_string(),
        intent: match family {
            NodeGraphPresetFamily::WorkflowClean => "theme-derived, clean, minimal".to_string(),
            NodeGraphPresetFamily::SchematicContrast => "theme-derived, high contrast".to_string(),
            NodeGraphPresetFamily::GraphDark => {
                "theme-derived (or opt-out), dark editor chrome".to_string()
            }
        },
        paint_only_tokens: PaintOnlyTokensV1 {
            canvas: CanvasTokensV1 {
                background: canvas_bg.into(),
            },
            grid: GridTokensV1 {
                minor_color: grid_minor.into(),
                major_color: grid_major.into(),
            },
            text: TextTokensV1 {
                primary: foreground.into(),
                muted: muted_foreground.into(),
            },
            node: NodeTokensV1 {
                body_background: node_bg.into(),
                border: node_border.into(),
                border_selected: node_border_selected.into(),
                header_background_default: header_default.into(),
                header_by_kind,
                title_text: title_text.into(),
                ring_selected,
                ring_focused,
            },
            port: PortThemeTokensV1 {
                by_port_kind: PortKindTokensV1 {
                    data: port_data,
                    exec: port_exec,
                    preview: port_preview,
                },
            },
            wire: WireTokensV1 {
                data_color: wire_data.into(),
                exec_color: wire_exec.into(),
                preview_color: wire_preview.into(),
                dash_preview,
                dash_invalid,
                dash_emphasis,
                highlight_selected: Some(match family {
                    NodeGraphPresetFamily::WorkflowClean => WireHighlightTokensV1 {
                        width_mul: 0.65,
                        alpha_mul: 0.80,
                        color: None,
                    },
                    NodeGraphPresetFamily::SchematicContrast => WireHighlightTokensV1 {
                        width_mul: 0.70,
                        alpha_mul: 0.90,
                        color: None,
                    },
                    NodeGraphPresetFamily::GraphDark => WireHighlightTokensV1 {
                        width_mul: 0.65,
                        alpha_mul: 0.85,
                        color: None,
                    },
                }),
                highlight_hovered: Some(match family {
                    NodeGraphPresetFamily::WorkflowClean => WireHighlightTokensV1 {
                        width_mul: 0.70,
                        alpha_mul: 0.95,
                        color: None,
                    },
                    NodeGraphPresetFamily::SchematicContrast => WireHighlightTokensV1 {
                        width_mul: 0.75,
                        alpha_mul: 1.0,
                        color: None,
                    },
                    NodeGraphPresetFamily::GraphDark => WireHighlightTokensV1 {
                        width_mul: 0.70,
                        alpha_mul: 0.95,
                        color: None,
                    },
                }),
                marker_exec_end: Some(EdgeMarkerTokensV1 {
                    kind: EdgeMarkerKindTokensV1::Arrow,
                    size_px: 12.0,
                }),
            },
            states: StateTokensV1 {
                hover: StateColorV1 {
                    color: hover.into(),
                },
                invalid: StateColorV1 {
                    color: invalid.into(),
                },
                convertible: StateColorV1 {
                    color: convertible.into(),
                },
                disabled: DisabledStateV1 { alpha_mul: 0.5 },
            },
        },
        layout_tokens: None,
        interaction_state_matrix: serde_json::Value::Null,
        example_compositions: serde_json::Value::Null,
        a11y_notes: serde_json::Value::Null,
    }
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

impl From<Color> for RgbaV1 {
    fn from(v: Color) -> Self {
        RgbaV1 {
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
    #[serde(default)]
    highlight_selected: Option<WireHighlightTokensV1>,
    #[serde(default)]
    highlight_hovered: Option<WireHighlightTokensV1>,
    #[serde(default)]
    marker_exec_end: Option<EdgeMarkerTokensV1>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct EdgeMarkerTokensV1 {
    kind: EdgeMarkerKindTokensV1,
    size_px: f32,
}

impl EdgeMarkerTokensV1 {
    fn into_marker(self) -> EdgeMarker {
        EdgeMarker {
            kind: match self.kind {
                EdgeMarkerKindTokensV1::Arrow => EdgeMarkerKind::Arrow,
            },
            size: self.size_px,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeMarkerKindTokensV1 {
    Arrow,
}

impl<'de> Deserialize<'de> for EdgeMarkerKindTokensV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Arrow" => Ok(EdgeMarkerKindTokensV1::Arrow),
            _ => Ok(EdgeMarkerKindTokensV1::Arrow),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WireHighlightTokensV1 {
    width_mul: f32,
    alpha_mul: f32,
    #[serde(default)]
    color: Option<RgbaV1>,
}

impl WireHighlightTokensV1 {
    fn into_hint(self) -> WireHighlightHint {
        WireHighlightHint {
            width_mul: self.width_mul,
            alpha_mul: self.alpha_mul,
            color: self.color.map(Into::into),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
                .marker_exec_end
                .unwrap_or_else(|| panic!("expected wire.marker_exec_end for {id:?}"));
            assert_eq!(marker.kind, EdgeMarkerKindTokensV1::Arrow);
            assert_close(marker.size_px, 12.0);
        }
    }
}

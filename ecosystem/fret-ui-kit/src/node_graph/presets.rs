//! Node-graph theme preset families (v1).
//!
//! The schema is designed to be JSON-serializable and paint-only: switching presets should not
//! require rebuilding derived geometry in the node graph widget.

use std::collections::HashMap;

use fret_core::scene::DashPatternV1;
use fret_core::window::ColorScheme;
use fret_core::{Color, Px};
use fret_ui::ThemeSnapshot;
use serde::Deserialize;

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

    pub fn preset_id(self) -> &'static str {
        match self {
            Self::WorkflowClean => "workflow_clean",
            Self::SchematicContrast => "schematic_contrast",
            Self::GraphDark => "graph_dark",
        }
    }
}

/// Derive in-tree preset families from an application theme snapshot.
///
/// This function is pure and intended to be used by higher-level editors to build paint-only
/// presets without hard-coding palette values into the node-graph integration crate.
pub fn theme_derived_presets(theme: &ThemeSnapshot) -> NodeGraphThemePresetsV1 {
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
                        alpha(border, 0.35),
                        alpha(border, 0.70),
                        tint(card, border, 0.20),
                        alpha(border, 1.0),
                        alpha(ring, 1.0),
                        card_foreground,
                    )
                } else {
                    // Opt-out: keep GraphDark as an explicit style family, but avoid forcing dark
                    // palette on a light theme snapshot.
                    (
                        background,
                        alpha(border, 0.50),
                        alpha(border, 0.80),
                        card,
                        alpha(border, 1.0),
                        alpha(ring, 1.0),
                        card_foreground,
                    )
                }
            }
        };

    let header_default = match family {
        NodeGraphPresetFamily::WorkflowClean => tint(card, border, 0.10),
        NodeGraphPresetFamily::SchematicContrast => alpha(theme.color_token("secondary"), 1.0),
        NodeGraphPresetFamily::GraphDark => tint(node_bg, border, 0.20),
    };

    let mut header_by_kind: HashMap<String, RgbaV1> = HashMap::new();
    for (k, c) in kind_colors {
        let header = match family {
            NodeGraphPresetFamily::WorkflowClean => tint(card, c, 0.22),
            NodeGraphPresetFamily::SchematicContrast => alpha(c, 1.0),
            NodeGraphPresetFamily::GraphDark => {
                if scheme_is_dark {
                    tint(node_bg, c, 0.35)
                } else {
                    tint(card, c, 0.22)
                }
            }
        };
        header_by_kind.insert(k.to_string(), header.into());
    }

    let (ring_sel, ring_focus) = match family {
        NodeGraphPresetFamily::WorkflowClean => (
            NodeRingTokensV1 {
                color: alpha(primary, 0.40).into(),
                width_px: 3.0,
                pad_px: 2.0,
            },
            NodeRingTokensV1 {
                color: alpha(primary, 0.60).into(),
                width_px: 2.0,
                pad_px: 1.0,
            },
        ),
        NodeGraphPresetFamily::SchematicContrast => (
            NodeRingTokensV1 {
                color: alpha(theme.color_token("chart-4"), 1.0).into(),
                width_px: 4.0,
                pad_px: 0.0,
            },
            NodeRingTokensV1 {
                color: alpha(theme.color_token("chart-5"), 1.0).into(),
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
                color: alpha(accent, 1.0).into(),
                width_px: 3.0,
                pad_px: 2.0,
            },
        ),
    };

    let (hover, invalid, convertible) = match family {
        NodeGraphPresetFamily::WorkflowClean => (
            alpha(theme.color_token("chart-1"), 1.0),
            alpha(destructive, 1.0),
            alpha(theme.color_token("chart-1"), 1.0),
        ),
        NodeGraphPresetFamily::SchematicContrast => (
            alpha(foreground, 1.0),
            alpha(destructive, 1.0),
            alpha(theme.color_token("chart-1"), 1.0),
        ),
        NodeGraphPresetFamily::GraphDark => (
            alpha(theme.color_token("chart-1"), 1.0),
            alpha(destructive, 1.0),
            alpha(theme.color_token("chart-1"), 1.0),
        ),
    };

    let port_data = match family {
        NodeGraphPresetFamily::WorkflowClean => PortTokensV1 {
            fill: alpha(muted_foreground, 0.85).into(),
            stroke: alpha(muted_foreground, 1.0).into(),
            stroke_width_px: 1.0,
            inner_scale: 1.0,
            shape: PortShapeKindV1::Circle,
        },
        NodeGraphPresetFamily::SchematicContrast => PortTokensV1 {
            fill: alpha(theme.color_token("chart-1"), 1.0).into(),
            stroke: alpha(foreground, 1.0).into(),
            stroke_width_px: 2.0,
            inner_scale: 1.0,
            shape: PortShapeKindV1::Circle,
        },
        NodeGraphPresetFamily::GraphDark => PortTokensV1 {
            fill: alpha(theme.color_token("chart-2"), 1.0).into(),
            stroke: alpha(theme.color_token("chart-2"), 1.0).into(),
            stroke_width_px: 1.5,
            inner_scale: 1.0,
            shape: PortShapeKindV1::Circle,
        },
    };

    let port_exec = match family {
        NodeGraphPresetFamily::WorkflowClean => PortTokensV1 {
            fill: alpha(card, 1.0).into(),
            stroke: alpha(foreground, 0.85).into(),
            stroke_width_px: 1.5,
            inner_scale: 0.0,
            shape: PortShapeKindV1::Circle,
        },
        NodeGraphPresetFamily::SchematicContrast => PortTokensV1 {
            fill: alpha(theme.color_token("chart-3"), 1.0).into(),
            stroke: alpha(foreground, 1.0).into(),
            stroke_width_px: 2.0,
            inner_scale: 0.0,
            shape: PortShapeKindV1::Circle,
        },
        NodeGraphPresetFamily::GraphDark => PortTokensV1 {
            fill: alpha(destructive, 0.65).into(),
            stroke: alpha(destructive, 1.0).into(),
            stroke_width_px: 1.5,
            inner_scale: 0.0,
            shape: PortShapeKindV1::Circle,
        },
    };

    let port_preview = PortTokensV1 {
        fill: header_by_kind
            .get("preview")
            .copied()
            .unwrap_or_else(|| alpha(destructive, 1.0).into()),
        stroke: alpha(destructive, 1.0).into(),
        stroke_width_px: 1.0,
        inner_scale: 0.5,
        shape: PortShapeKindV1::Circle,
    };

    let (wire_data, wire_exec, wire_preview) = match family {
        NodeGraphPresetFamily::WorkflowClean => (
            alpha(muted_foreground, 1.0),
            alpha(theme.color_token("secondary-foreground"), 1.0),
            alpha(border, 1.0),
        ),
        NodeGraphPresetFamily::SchematicContrast => (
            alpha(theme.color_token("chart-1"), 1.0),
            alpha(theme.color_token("chart-3"), 1.0),
            alpha(theme.color_token("chart-4"), 1.0),
        ),
        NodeGraphPresetFamily::GraphDark => (
            alpha(theme.color_token("chart-2"), 1.0),
            alpha(destructive, 1.0),
            alpha(theme.color_token("chart-4"), 1.0),
        ),
    };

    let (highlight_sel, highlight_hov) = match family {
        NodeGraphPresetFamily::WorkflowClean => (
            WireHighlightTokensV1 {
                width_mul: 0.65,
                alpha_mul: 0.80,
                color: None,
            },
            WireHighlightTokensV1 {
                width_mul: 0.70,
                alpha_mul: 0.95,
                color: None,
            },
        ),
        NodeGraphPresetFamily::SchematicContrast => (
            WireHighlightTokensV1 {
                width_mul: 0.70,
                alpha_mul: 0.90,
                color: None,
            },
            WireHighlightTokensV1 {
                width_mul: 0.75,
                alpha_mul: 1.0,
                color: None,
            },
        ),
        NodeGraphPresetFamily::GraphDark => (
            WireHighlightTokensV1 {
                width_mul: 0.65,
                alpha_mul: 0.85,
                color: Some(alpha(convertible, 1.0).into()),
            },
            WireHighlightTokensV1 {
                width_mul: 0.70,
                alpha_mul: 0.95,
                color: Some(alpha(hover, 1.0).into()),
            },
        ),
    };

    NodeGraphThemePresetV1 {
        id: family.preset_id().to_string(),
        display_name: family.display_name().to_string(),
        intent: match family {
            NodeGraphPresetFamily::WorkflowClean => "theme-derived, clean, minimal".to_string(),
            NodeGraphPresetFamily::SchematicContrast => "theme-derived, high contrast".to_string(),
            NodeGraphPresetFamily::GraphDark => {
                if scheme_is_dark {
                    "theme-derived, dark with neon accents".to_string()
                } else {
                    "theme-derived, dark family (opted-out on light theme)".to_string()
                }
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
                primary: title_text.into(),
                muted: muted_foreground.into(),
            },
            node: NodeTokensV1 {
                body_background: node_bg.into(),
                border: node_border.into(),
                border_selected: node_border_selected.into(),
                header_background_default: header_default.into(),
                header_by_kind,
                title_text: title_text.into(),
                ring_selected: ring_sel,
                ring_focused: ring_focus,
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
                dash_preview: DashPatternTokensV1 {
                    dash_px: 4.0,
                    gap_px: 4.0,
                    phase_px: 0.0,
                },
                dash_invalid: DashPatternTokensV1 {
                    dash_px: 6.0,
                    gap_px: 3.0,
                    phase_px: 0.0,
                },
                dash_emphasis: DashPatternTokensV1 {
                    dash_px: 2.0,
                    gap_px: 2.0,
                    phase_px: 0.0,
                },
                highlight_selected: Some(highlight_sel),
                highlight_hovered: Some(highlight_hov),
                marker_exec_end: Some(EdgeMarkerTokensV1 {
                    kind: EdgeMarkerKindTokensV1::Arrow,
                    size_px: 12.0,
                }),
                marker_exec_start: Some(EdgeMarkerTokensV1 {
                    kind: EdgeMarkerKindTokensV1::Arrow,
                    size_px: 8.0,
                }),
                marker_data_end: None,
                marker_data_start: None,
                marker_size_mul_selected: Some(1.15),
                marker_size_mul_hovered: Some(1.25),
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
pub struct RgbaV1 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
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
pub struct DashPatternTokensV1 {
    pub dash_px: f32,
    pub gap_px: f32,
    pub phase_px: f32,
}

impl DashPatternTokensV1 {
    pub fn into_dash(self) -> DashPatternV1 {
        DashPatternV1::new(Px(self.dash_px), Px(self.gap_px), Px(self.phase_px))
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct NodeRingTokensV1 {
    pub color: RgbaV1,
    pub width_px: f32,
    pub pad_px: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeGraphThemePresetsV1 {
    #[allow(dead_code)]
    pub schema_version: String,
    #[allow(dead_code)]
    pub notes: String,
    pub presets: Vec<NodeGraphThemePresetV1>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeGraphThemePresetV1 {
    pub id: String,
    #[allow(dead_code)]
    pub display_name: String,
    #[allow(dead_code)]
    pub intent: String,
    pub paint_only_tokens: PaintOnlyTokensV1,
    #[serde(default)]
    pub layout_tokens: Option<LayoutTokensV1>,
    #[serde(default)]
    #[allow(dead_code)]
    pub interaction_state_matrix: serde_json::Value,
    #[serde(default)]
    #[allow(dead_code)]
    pub example_compositions: serde_json::Value,
    #[serde(default)]
    #[allow(dead_code)]
    pub a11y_notes: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LayoutTokensV1 {
    #[allow(dead_code)]
    pub optional: bool,
    #[serde(default)]
    pub grid_minor_width_px: Option<f32>,
    #[serde(default)]
    #[allow(dead_code)]
    pub grid_major_width_px: Option<f32>,
    #[allow(dead_code)]
    pub node_corner_radius_px: Option<f32>,
    #[allow(dead_code)]
    pub node_header_height_px: Option<f32>,
    #[allow(dead_code)]
    pub pin_radius_px: Option<f32>,
    #[allow(dead_code)]
    pub wire_width_px: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaintOnlyTokensV1 {
    pub canvas: CanvasTokensV1,
    pub grid: GridTokensV1,
    #[allow(dead_code)]
    pub text: TextTokensV1,
    pub node: NodeTokensV1,
    pub port: PortThemeTokensV1,
    pub wire: WireTokensV1,
    pub states: StateTokensV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CanvasTokensV1 {
    pub background: RgbaV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct GridTokensV1 {
    pub minor_color: RgbaV1,
    pub major_color: RgbaV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct TextTokensV1 {
    #[allow(dead_code)]
    pub primary: RgbaV1,
    #[allow(dead_code)]
    pub muted: RgbaV1,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeTokensV1 {
    pub body_background: RgbaV1,
    pub border: RgbaV1,
    pub border_selected: RgbaV1,
    pub header_background_default: RgbaV1,
    pub header_by_kind: HashMap<String, RgbaV1>,
    pub title_text: RgbaV1,
    pub ring_selected: NodeRingTokensV1,
    pub ring_focused: NodeRingTokensV1,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PortThemeTokensV1 {
    pub by_port_kind: PortKindTokensV1,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PortKindTokensV1 {
    pub data: PortTokensV1,
    pub exec: PortTokensV1,
    #[allow(dead_code)]
    pub preview: PortTokensV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PortTokensV1 {
    pub fill: RgbaV1,
    pub stroke: RgbaV1,
    pub stroke_width_px: f32,
    pub inner_scale: f32,
    pub shape: PortShapeKindV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortShapeKindV1 {
    Circle,
    Diamond,
    Triangle,
}

impl<'de> Deserialize<'de> for PortShapeKindV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Circle" => Ok(PortShapeKindV1::Circle),
            "Diamond" => Ok(PortShapeKindV1::Diamond),
            "Triangle" => Ok(PortShapeKindV1::Triangle),
            _ => Ok(PortShapeKindV1::Circle),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WireTokensV1 {
    pub data_color: RgbaV1,
    pub exec_color: RgbaV1,
    pub preview_color: RgbaV1,
    pub dash_preview: DashPatternTokensV1,
    pub dash_invalid: DashPatternTokensV1,
    pub dash_emphasis: DashPatternTokensV1,
    #[serde(default)]
    pub highlight_selected: Option<WireHighlightTokensV1>,
    #[serde(default)]
    pub highlight_hovered: Option<WireHighlightTokensV1>,
    #[serde(default)]
    pub marker_exec_end: Option<EdgeMarkerTokensV1>,
    #[serde(default)]
    pub marker_exec_start: Option<EdgeMarkerTokensV1>,
    #[serde(default)]
    pub marker_data_end: Option<EdgeMarkerTokensV1>,
    #[serde(default)]
    pub marker_data_start: Option<EdgeMarkerTokensV1>,
    #[serde(default)]
    pub marker_size_mul_selected: Option<f32>,
    #[serde(default)]
    pub marker_size_mul_hovered: Option<f32>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct EdgeMarkerTokensV1 {
    pub kind: EdgeMarkerKindTokensV1,
    pub size_px: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeMarkerKindTokensV1 {
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
pub struct WireHighlightTokensV1 {
    pub width_mul: f32,
    pub alpha_mul: f32,
    #[serde(default)]
    pub color: Option<RgbaV1>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateTokensV1 {
    pub hover: StateColorV1,
    pub invalid: StateColorV1,
    pub convertible: StateColorV1,
    #[allow(dead_code)]
    pub disabled: DisabledStateV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct StateColorV1 {
    pub color: RgbaV1,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct DisabledStateV1 {
    #[allow(dead_code)]
    pub alpha_mul: f32,
}

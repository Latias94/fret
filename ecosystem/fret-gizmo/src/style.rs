use fret_core::Color;

use crate::gizmo::GizmoConfig;
use crate::view_gizmo::ViewGizmoConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoVisualPreset {
    Classic,
    Muted,
    HighContrast,
}

impl GizmoVisualPreset {
    pub const ALL: [Self; 3] = [Self::Classic, Self::Muted, Self::HighContrast];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Muted => "Muted",
            Self::HighContrast => "HighContrast",
        }
    }

    pub fn apply_to_config(self, cfg: &mut GizmoConfig) {
        match self {
            Self::Classic => {
                // Match `GizmoConfig::default()` visuals without changing behavior knobs.
                cfg.x_color = Color {
                    r: 1.0,
                    g: 0.2,
                    b: 0.4,
                    a: 1.0,
                };
                cfg.y_color = Color {
                    r: 0.2,
                    g: 1.0,
                    b: 0.4,
                    a: 1.0,
                };
                cfg.z_color = Color {
                    r: 0.2,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                };
                cfg.hover_color = Color {
                    r: 1.0,
                    g: 0.85,
                    b: 0.2,
                    a: 1.0,
                };
            }
            Self::Muted => {
                cfg.line_thickness_px = cfg.line_thickness_px.max(7.0);
                cfg.pick_radius_px = cfg.pick_radius_px.max(12.0);
                cfg.occluded_alpha = cfg.occluded_alpha.clamp(0.15, 0.35);

                cfg.x_color = Color {
                    r: 0.95,
                    g: 0.35,
                    b: 0.45,
                    a: 0.95,
                };
                cfg.y_color = Color {
                    r: 0.35,
                    g: 0.95,
                    b: 0.55,
                    a: 0.95,
                };
                cfg.z_color = Color {
                    r: 0.35,
                    g: 0.60,
                    b: 0.98,
                    a: 0.95,
                };
                cfg.hover_color = Color {
                    r: 1.0,
                    g: 0.92,
                    b: 0.35,
                    a: 1.0,
                };
            }
            Self::HighContrast => {
                cfg.line_thickness_px = cfg.line_thickness_px.max(9.0);
                cfg.pick_radius_px = cfg.pick_radius_px.max(14.0);
                cfg.occluded_alpha = cfg.occluded_alpha.clamp(0.25, 0.45);

                cfg.x_color = Color {
                    r: 1.0,
                    g: 0.15,
                    b: 0.25,
                    a: 1.0,
                };
                cfg.y_color = Color {
                    r: 0.15,
                    g: 1.0,
                    b: 0.35,
                    a: 1.0,
                };
                cfg.z_color = Color {
                    r: 0.15,
                    g: 0.55,
                    b: 1.0,
                    a: 1.0,
                };
                cfg.hover_color = Color {
                    r: 1.0,
                    g: 0.95,
                    b: 0.25,
                    a: 1.0,
                };
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewGizmoVisualPreset {
    Classic,
    Muted,
    HighContrast,
}

impl ViewGizmoVisualPreset {
    pub const ALL: [Self; 3] = [Self::Classic, Self::Muted, Self::HighContrast];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Muted => "Muted",
            Self::HighContrast => "HighContrast",
        }
    }

    pub fn apply_to_config(self, cfg: &mut ViewGizmoConfig) {
        match self {
            Self::Classic => {
                // Match `ViewGizmoConfig::default()` visuals without changing behavior knobs.
                cfg.face_color = Color {
                    r: 0.22,
                    g: 0.22,
                    b: 0.24,
                    a: 0.35,
                };
                cfg.edge_color = Color {
                    r: 0.95,
                    g: 0.95,
                    b: 0.98,
                    a: 0.9,
                };
                cfg.hover_color = Color {
                    r: 1.0,
                    g: 0.85,
                    b: 0.3,
                    a: 0.55,
                };
                cfg.x_color = Color {
                    r: 1.0,
                    g: 0.2,
                    b: 0.4,
                    a: 1.0,
                };
                cfg.y_color = Color {
                    r: 0.2,
                    g: 1.0,
                    b: 0.4,
                    a: 1.0,
                };
                cfg.z_color = Color {
                    r: 0.2,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                };
            }
            Self::Muted => {
                cfg.size_px = cfg.size_px.max(72.0);
                cfg.pick_padding_px = cfg.pick_padding_px.max(4.0);

                cfg.face_color = Color {
                    r: 0.15,
                    g: 0.15,
                    b: 0.18,
                    a: 0.85,
                };
                cfg.edge_color = Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.55,
                };
                cfg.hover_color = Color {
                    r: 1.0,
                    g: 0.9,
                    b: 0.35,
                    a: 0.75,
                };
            }
            Self::HighContrast => {
                cfg.size_px = cfg.size_px.max(80.0);
                cfg.pick_padding_px = cfg.pick_padding_px.max(4.0);

                cfg.face_color = Color {
                    r: 0.08,
                    g: 0.08,
                    b: 0.10,
                    a: 0.92,
                };
                cfg.edge_color = Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.75,
                };
                cfg.hover_color = Color {
                    r: 1.0,
                    g: 0.95,
                    b: 0.25,
                    a: 0.85,
                };
            }
        }
    }
}

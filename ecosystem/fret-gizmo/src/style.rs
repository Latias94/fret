use fret_core::Color;
use glam::Vec2;

use crate::gizmo::Gizmo;
use crate::gizmo::GizmoConfig;
use crate::view_gizmo::ViewGizmoConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoVisuals {
    pub size_px: f32,
    pub pick_radius_px: f32,
    pub line_thickness_px: f32,
    pub bounds_handle_size_px: f32,
    pub show_occluded: bool,
    pub occluded_alpha: f32,
    pub x_color: Color,
    pub y_color: Color,
    pub z_color: Color,
    pub hover_color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoPartVisuals {
    pub translate_head_length_fraction: f32,
    pub translate_head_radius_fraction: f32,
    pub translate_shaft_min_fraction: f32,
    pub translate_plane_offset_fraction: f32,
    pub translate_plane_size_fraction: f32,
    pub translate_plane_fill_alpha: f32,
    pub translate_plane_fill_hover_alpha: f32,
    pub translate_center_half_fraction: f32,
    pub translate_depth_ring_radius_fraction: f32,
    pub translate_depth_ring_radius_min_fraction: f32,
    pub scale_axis_end_box_half_fraction: f32,
    pub scale_plane_offset_fraction: f32,
    pub scale_plane_size_fraction: f32,
    pub scale_plane_fill_alpha: f32,
    pub scale_plane_fill_hover_alpha: f32,
    pub scale_uniform_half_fraction: f32,
    pub rotate_feedback_thickness_scale: f32,
}

impl Default for GizmoPartVisuals {
    fn default() -> Self {
        Self::classic()
    }
}

impl GizmoPartVisuals {
    pub fn classic() -> Self {
        Self {
            translate_head_length_fraction: 0.18,
            translate_head_radius_fraction: 0.07,
            translate_shaft_min_fraction: 0.20,
            translate_plane_offset_fraction: 0.15,
            translate_plane_size_fraction: 0.25,
            translate_plane_fill_alpha: 0.30,
            translate_plane_fill_hover_alpha: 0.55,
            translate_center_half_fraction: 0.08,
            translate_depth_ring_radius_fraction: 0.14,
            translate_depth_ring_radius_min_fraction: 0.08,
            scale_axis_end_box_half_fraction: 0.06,
            scale_plane_offset_fraction: 0.15,
            scale_plane_size_fraction: 0.25,
            scale_plane_fill_alpha: 0.22,
            scale_plane_fill_hover_alpha: 0.45,
            scale_uniform_half_fraction: 0.08,
            rotate_feedback_thickness_scale: 1.0,
        }
    }
}

impl Default for GizmoVisuals {
    fn default() -> Self {
        Self::classic()
    }
}

impl GizmoVisuals {
    pub fn classic() -> Self {
        Self {
            size_px: 96.0,
            pick_radius_px: 10.0,
            line_thickness_px: 6.0,
            bounds_handle_size_px: 12.0,
            show_occluded: true,
            occluded_alpha: 0.25,
            x_color: Color {
                r: 1.0,
                g: 0.2,
                b: 0.4,
                a: 1.0,
            },
            y_color: Color {
                r: 0.2,
                g: 1.0,
                b: 0.4,
                a: 1.0,
            },
            z_color: Color {
                r: 0.2,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
            hover_color: Color {
                r: 1.0,
                g: 0.85,
                b: 0.2,
                a: 1.0,
            },
        }
    }

    pub fn apply_to_config(self, cfg: &mut GizmoConfig) {
        cfg.size_px = self.size_px;
        cfg.pick_radius_px = self.pick_radius_px;
        cfg.line_thickness_px = self.line_thickness_px;
        cfg.bounds_handle_size_px = self.bounds_handle_size_px;
        cfg.show_occluded = self.show_occluded;
        cfg.occluded_alpha = self.occluded_alpha;
        cfg.x_color = self.x_color;
        cfg.y_color = self.y_color;
        cfg.z_color = self.z_color;
        cfg.hover_color = self.hover_color;
    }

    pub fn apply_minimum_to_config(self, cfg: &mut GizmoConfig) {
        cfg.size_px = cfg.size_px.max(self.size_px);
        cfg.pick_radius_px = cfg.pick_radius_px.max(self.pick_radius_px);
        cfg.line_thickness_px = cfg.line_thickness_px.max(self.line_thickness_px);
        cfg.bounds_handle_size_px = cfg.bounds_handle_size_px.max(self.bounds_handle_size_px);
        cfg.show_occluded |= self.show_occluded;
        cfg.occluded_alpha = cfg.occluded_alpha.max(self.occluded_alpha);

        cfg.x_color = self.x_color;
        cfg.y_color = self.y_color;
        cfg.z_color = self.z_color;
        cfg.hover_color = self.hover_color;
    }
}

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

    pub fn visuals(self) -> GizmoVisuals {
        match self {
            Self::Classic => GizmoVisuals::classic(),
            Self::Muted => GizmoVisuals {
                size_px: 104.0,
                pick_radius_px: 12.0,
                line_thickness_px: 7.0,
                bounds_handle_size_px: 13.0,
                show_occluded: true,
                occluded_alpha: 0.25,
                x_color: Color {
                    r: 0.95,
                    g: 0.35,
                    b: 0.45,
                    a: 0.95,
                },
                y_color: Color {
                    r: 0.35,
                    g: 0.95,
                    b: 0.55,
                    a: 0.95,
                },
                z_color: Color {
                    r: 0.35,
                    g: 0.60,
                    b: 0.98,
                    a: 0.95,
                },
                hover_color: Color {
                    r: 1.0,
                    g: 0.92,
                    b: 0.35,
                    a: 1.0,
                },
            },
            Self::HighContrast => GizmoVisuals {
                size_px: 112.0,
                pick_radius_px: 14.0,
                line_thickness_px: 9.0,
                bounds_handle_size_px: 14.0,
                show_occluded: true,
                occluded_alpha: 0.35,
                x_color: Color {
                    r: 1.0,
                    g: 0.15,
                    b: 0.25,
                    a: 1.0,
                },
                y_color: Color {
                    r: 0.15,
                    g: 1.0,
                    b: 0.35,
                    a: 1.0,
                },
                z_color: Color {
                    r: 0.15,
                    g: 0.55,
                    b: 1.0,
                    a: 1.0,
                },
                hover_color: Color {
                    r: 1.0,
                    g: 0.95,
                    b: 0.25,
                    a: 1.0,
                },
            },
        }
    }

    pub fn part_visuals(self) -> GizmoPartVisuals {
        match self {
            Self::Classic => GizmoPartVisuals::classic(),
            Self::Muted => GizmoPartVisuals {
                translate_head_length_fraction: 0.19,
                translate_head_radius_fraction: 0.075,
                translate_plane_size_fraction: 0.28,
                translate_plane_fill_alpha: 0.35,
                translate_plane_fill_hover_alpha: 0.60,
                scale_plane_fill_alpha: 0.25,
                scale_plane_fill_hover_alpha: 0.50,
                rotate_feedback_thickness_scale: 1.10,
                ..GizmoPartVisuals::classic()
            },
            Self::HighContrast => GizmoPartVisuals {
                translate_head_length_fraction: 0.20,
                translate_head_radius_fraction: 0.080,
                translate_plane_size_fraction: 0.30,
                translate_plane_fill_alpha: 0.40,
                translate_plane_fill_hover_alpha: 0.70,
                scale_plane_fill_alpha: 0.30,
                scale_plane_fill_hover_alpha: 0.60,
                rotate_feedback_thickness_scale: 1.25,
                ..GizmoPartVisuals::classic()
            },
        }
    }

    pub fn apply_to_config(self, cfg: &mut GizmoConfig) {
        match self {
            Self::Classic => {
                self.visuals().apply_to_config(cfg);
            }
            Self::Muted => {
                self.visuals().apply_to_config(cfg);
            }
            Self::HighContrast => {
                self.visuals().apply_to_config(cfg);
            }
        }
    }

    pub fn apply_to_gizmo(self, gizmo: &mut Gizmo) {
        self.visuals().apply_to_config(&mut gizmo.config);
        gizmo.set_part_visuals(self.part_visuals());
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewGizmoVisuals {
    pub margin_px: Vec2,
    pub size_px: f32,
    pub pick_padding_px: f32,
    pub center_button_radius_px: f32,
    pub face_color: Color,
    pub edge_color: Color,
    pub hover_color: Color,
    pub x_color: Color,
    pub y_color: Color,
    pub z_color: Color,
}

impl Default for ViewGizmoVisuals {
    fn default() -> Self {
        Self::classic()
    }
}

impl ViewGizmoVisuals {
    pub fn classic() -> Self {
        Self {
            margin_px: Vec2::new(16.0, 16.0),
            size_px: 84.0,
            pick_padding_px: 6.0,
            center_button_radius_px: 12.0,
            face_color: Color {
                r: 0.22,
                g: 0.22,
                b: 0.24,
                a: 0.35,
            },
            edge_color: Color {
                r: 0.95,
                g: 0.95,
                b: 0.98,
                a: 0.9,
            },
            hover_color: Color {
                r: 1.0,
                g: 0.85,
                b: 0.3,
                a: 0.55,
            },
            x_color: Color {
                r: 1.0,
                g: 0.2,
                b: 0.4,
                a: 1.0,
            },
            y_color: Color {
                r: 0.2,
                g: 1.0,
                b: 0.4,
                a: 1.0,
            },
            z_color: Color {
                r: 0.2,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
        }
    }

    pub fn apply_to_config(self, cfg: &mut ViewGizmoConfig) {
        cfg.margin_px = self.margin_px;
        cfg.size_px = self.size_px;
        cfg.pick_padding_px = self.pick_padding_px;
        cfg.center_button_radius_px = self.center_button_radius_px;
        cfg.face_color = self.face_color;
        cfg.edge_color = self.edge_color;
        cfg.hover_color = self.hover_color;
        cfg.x_color = self.x_color;
        cfg.y_color = self.y_color;
        cfg.z_color = self.z_color;
    }

    pub fn apply_minimum_to_config(self, cfg: &mut ViewGizmoConfig) {
        cfg.margin_px = Vec2::new(
            cfg.margin_px.x.max(self.margin_px.x),
            cfg.margin_px.y.max(self.margin_px.y),
        );
        cfg.size_px = cfg.size_px.max(self.size_px);
        cfg.pick_padding_px = cfg.pick_padding_px.max(self.pick_padding_px);
        cfg.center_button_radius_px = cfg
            .center_button_radius_px
            .max(self.center_button_radius_px);

        cfg.face_color = self.face_color;
        cfg.edge_color = self.edge_color;
        cfg.hover_color = self.hover_color;
        cfg.x_color = self.x_color;
        cfg.y_color = self.y_color;
        cfg.z_color = self.z_color;
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

    pub fn visuals(self) -> ViewGizmoVisuals {
        match self {
            Self::Classic => ViewGizmoVisuals::classic(),
            Self::Muted => ViewGizmoVisuals {
                margin_px: Vec2::new(16.0, 16.0),
                size_px: 72.0,
                pick_padding_px: 4.0,
                center_button_radius_px: 12.0,
                face_color: Color {
                    r: 0.15,
                    g: 0.15,
                    b: 0.18,
                    a: 0.85,
                },
                edge_color: Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.55,
                },
                hover_color: Color {
                    r: 1.0,
                    g: 0.9,
                    b: 0.35,
                    a: 0.75,
                },
                x_color: Color {
                    r: 1.0,
                    g: 0.2,
                    b: 0.4,
                    a: 1.0,
                },
                y_color: Color {
                    r: 0.2,
                    g: 1.0,
                    b: 0.4,
                    a: 1.0,
                },
                z_color: Color {
                    r: 0.2,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                },
            },
            Self::HighContrast => ViewGizmoVisuals {
                margin_px: Vec2::new(16.0, 16.0),
                size_px: 80.0,
                pick_padding_px: 4.0,
                center_button_radius_px: 12.0,
                face_color: Color {
                    r: 0.08,
                    g: 0.08,
                    b: 0.10,
                    a: 0.92,
                },
                edge_color: Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.75,
                },
                hover_color: Color {
                    r: 1.0,
                    g: 0.95,
                    b: 0.25,
                    a: 0.85,
                },
                x_color: Color {
                    r: 1.0,
                    g: 0.2,
                    b: 0.4,
                    a: 1.0,
                },
                y_color: Color {
                    r: 0.2,
                    g: 1.0,
                    b: 0.4,
                    a: 1.0,
                },
                z_color: Color {
                    r: 0.2,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                },
            },
        }
    }

    pub fn apply_to_config(self, cfg: &mut ViewGizmoConfig) {
        match self {
            Self::Classic => {
                self.visuals().apply_to_config(cfg);
            }
            Self::Muted => {
                self.visuals().apply_to_config(cfg);
            }
            Self::HighContrast => {
                self.visuals().apply_to_config(cfg);
            }
        }
    }
}

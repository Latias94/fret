use fret_core::Color;
use glam::Vec3;

use crate::gizmo::{DepthMode, GizmoDrawList3d, Line3d};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid3dConfig {
    pub origin: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub cell_size: f32,
    pub half_extent: f32,
    pub major_every: u32,
    pub minor_color: Color,
    pub major_color: Color,
    pub axis_u_color: Color,
    pub axis_v_color: Color,
    pub depth: DepthMode,
}

impl Default for Grid3dConfig {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            u: Vec3::X,
            v: Vec3::Z,
            cell_size: 0.5,
            half_extent: 10.0,
            major_every: 5,
            minor_color: Color {
                a: 0.28,
                ..Color::from_srgb_hex_rgb(0x4d_4d_57)
            },
            major_color: Color {
                a: 0.42,
                ..Color::from_srgb_hex_rgb(0x73_73_80)
            },
            axis_u_color: Color {
                a: 0.80,
                ..Color::from_srgb_hex_rgb(0xf2_59_52)
            },
            axis_v_color: Color {
                a: 0.80,
                ..Color::from_srgb_hex_rgb(0x52_8c_ff)
            },
            depth: DepthMode::Test,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid3d {
    pub config: Grid3dConfig,
}

impl Default for Grid3d {
    fn default() -> Self {
        Self::new(Grid3dConfig::default())
    }
}

impl Grid3d {
    pub fn new(config: Grid3dConfig) -> Self {
        Self { config }
    }

    pub fn draw(&self) -> GizmoDrawList3d {
        let mut out = GizmoDrawList3d::default();

        let cell = self.config.cell_size;
        let half = self.config.half_extent;
        if !cell.is_finite() || cell <= 1e-6 || !half.is_finite() || half <= 1e-6 {
            return out;
        }

        let u = self.config.u.normalize_or_zero();
        let v = self.config.v.normalize_or_zero();
        if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
            return out;
        }

        let count = (half / cell).floor() as i32;
        let count = count.clamp(1, 4096);
        let major_every = self.config.major_every.max(1) as i32;

        for i in -count..=count {
            let t = i as f32 * cell;
            let offset_v = v * t;
            let offset_u = u * t;

            let is_axis = i == 0;
            let is_major = (i.abs() % major_every) == 0;
            let color_u = if is_axis {
                self.config.axis_u_color
            } else if is_major {
                self.config.major_color
            } else {
                self.config.minor_color
            };
            let color_v = if is_axis {
                self.config.axis_v_color
            } else if is_major {
                self.config.major_color
            } else {
                self.config.minor_color
            };

            let a_u = self.config.origin + offset_v - u * half;
            let b_u = self.config.origin + offset_v + u * half;
            let a_v = self.config.origin + offset_u - v * half;
            let b_v = self.config.origin + offset_u + v * half;

            out.lines.push(Line3d {
                a: a_u,
                b: b_u,
                color: color_u,
                depth: self.config.depth,
            });
            out.lines.push(Line3d {
                a: a_v,
                b: b_v,
                color: color_v,
                depth: self.config.depth,
            });
        }

        out
    }
}

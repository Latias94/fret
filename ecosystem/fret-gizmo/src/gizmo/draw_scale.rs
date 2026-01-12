use super::*;

impl Gizmo {
    pub(super) fn draw_scale_feedback(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
    ) -> GizmoDrawList3d {
        if self.state.drag_mode != GizmoMode::Scale {
            return GizmoDrawList3d::default();
        }
        if self.state.active.is_none() || !self.state.drag_has_started || !self.state.drag_snap {
            return GizmoDrawList3d::default();
        }
        if self.state.drag_scale_is_bounds {
            return GizmoDrawList3d::default();
        }

        let Some(step) = self
            .config
            .scale_snap_step
            .filter(|s| s.is_finite() && *s > 0.0)
        else {
            return GizmoDrawList3d::default();
        };

        let length_world = self.state.drag_size_length_world.max(1e-6);
        let tick_len_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            10.0,
        )
        .unwrap_or(length_world * 0.06);
        let half = (tick_len_world * 0.5).max(1e-6);

        let minor = Color {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 0.18,
        };
        let major = Color {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 0.26,
        };
        let highlight = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.75,
        };

        let mut out = GizmoDrawList3d::default();

        let mut draw_ticks = |dir: Vec3, current_factor: f32| {
            let dir = dir.normalize_or_zero();
            if dir.length_squared() == 0.0 {
                return;
            }

            let tick_dir = Self::tick_perp_dir(
                view_projection,
                viewport,
                origin,
                self.config.depth_range,
                dir,
            );
            if tick_dir.length_squared() == 0.0 {
                return;
            }

            let k_cur = ((current_factor - 1.0) / step).round() as i32;
            let radius = (k_cur.abs() + 6).clamp(6, 24);

            for k in -radius..=radius {
                let factor = 1.0 + (k as f32) * step;
                if !factor.is_finite() || factor <= 0.01 {
                    continue;
                }
                let pos = origin + dir * (length_world * factor);
                let is_major = k % 5 == 0;
                let len = if is_major { half * 1.65 } else { half };
                let c = if is_major { major } else { minor };
                self.push_line(
                    &mut out.lines,
                    pos - tick_dir * len,
                    pos + tick_dir * len,
                    c,
                    DepthMode::Always,
                );
            }

            let pos = origin + dir * (length_world * current_factor.max(0.01));
            self.push_line(
                &mut out.lines,
                pos - tick_dir * (half * 2.2),
                pos + tick_dir * (half * 2.2),
                highlight,
                DepthMode::Always,
            );
        };

        if self.state.drag_scale_plane_axes.is_some() {
            let u = self.state.drag_scale_plane_u;
            let v = self.state.drag_scale_plane_v;
            let factors = self.state.drag_total_scale_plane_applied;
            draw_ticks(u, factors.x);
            draw_ticks(v, factors.y);
        } else {
            draw_ticks(
                self.state.drag_axis_dir,
                self.state.drag_total_scale_applied,
            );
        }

        out
    }

    pub(super) fn draw_scale_handles(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
        size_length_world: f32,
        include_axes: bool,
        include_uniform: bool,
        include_planes: bool,
    ) -> Vec<Line3d> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;

        let (u, v) = view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            .map(plane_basis)
            .unwrap_or((Vec3::X, Vec3::Y));

        let mut out = Vec::new();

        if include_axes {
            for &(((axis_dir, color), handle), axis_index) in &[
                (
                    ((axes[0], self.config.x_color), ScaleHandle::AxisX.id()),
                    0usize,
                ),
                (
                    ((axes[1], self.config.y_color), ScaleHandle::AxisY.id()),
                    1usize,
                ),
                (
                    ((axes[2], self.config.z_color), ScaleHandle::AxisZ.id()),
                    2usize,
                ),
            ] {
                if self.axis_is_masked(axis_index) {
                    continue;
                }
                let alpha = self.axis_visibility_alpha(
                    view_projection,
                    viewport,
                    origin,
                    axis_dir,
                    length_world,
                );
                if alpha <= 0.01 {
                    continue;
                }
                let c = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
                    self.config.hover_color
                } else {
                    color
                };
                let c = mix_alpha(c, alpha);

                let end = origin + axis_dir * length_world;
                if pv.occlusion.handles {
                    self.push_line(&mut out, origin, end, c, self.config.depth_mode);
                } else {
                    self.push_line_no_ghost(&mut out, origin, end, c, self.config.depth_mode);
                }

                // End box, screen-facing.
                let half = length_world * pv.scale_axis_end_box_half_fraction.max(0.0);
                let p0 = end + (-u - v) * half;
                let p1 = end + (u - v) * half;
                let p2 = end + (u + v) * half;
                let p3 = end + (-u + v) * half;
                for (a, b) in [(p0, p1), (p1, p2), (p2, p3), (p3, p0)] {
                    if pv.occlusion.handles {
                        self.push_line(&mut out, a, b, c, self.config.depth_mode);
                    } else {
                        self.push_line_no_ghost(&mut out, a, b, c, self.config.depth_mode);
                    }
                }
            }
        }

        if include_planes {
            let off = length_world * pv.scale_plane_offset_fraction.max(0.0);
            let size = length_world * pv.scale_plane_size_fraction.max(0.0);
            for &(u, v, base_color, handle) in &[
                (
                    axes[0],
                    axes[1],
                    mix_alpha(self.config.z_color, 0.55),
                    ScaleHandle::PlaneXY,
                ), // XY
                (
                    axes[0],
                    axes[2],
                    mix_alpha(self.config.y_color, 0.55),
                    ScaleHandle::PlaneXZ,
                ), // XZ
                (
                    axes[1],
                    axes[2],
                    mix_alpha(self.config.x_color, 0.55),
                    ScaleHandle::PlaneYZ,
                ), // YZ
            ] {
                let handle_id = handle.id();
                let plane_axes = match handle {
                    ScaleHandle::PlaneXY => (0usize, 1usize),
                    ScaleHandle::PlaneXZ => (0usize, 2usize),
                    ScaleHandle::PlaneYZ => (1usize, 2usize),
                    _ => continue,
                };
                if !self.plane_allowed_by_mask(plane_axes) {
                    continue;
                }
                let color = if self.is_handle_highlighted(GizmoMode::Scale, handle_id) {
                    mix_alpha(self.config.hover_color, 0.85)
                } else {
                    base_color
                };

                let quad = translate_plane_quad_world(origin, u, v, off, size);
                let alpha = self.plane_visibility_alpha(view_projection, viewport, quad);
                if alpha <= 0.01 {
                    continue;
                }
                self.push_quad_outline(
                    &mut out,
                    quad,
                    mix_alpha(color, alpha),
                    self.config.depth_mode,
                    pv.occlusion.handles,
                );
            }
        }

        if include_uniform {
            // Uniform scale box at the origin (screen-facing).
            let handle = ScaleHandle::Uniform.id();
            let base = mix_alpha(self.config.hover_color, 0.65);
            let c = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
                self.config.hover_color
            } else {
                base
            };
            let half = length_world * pv.scale_uniform_half_fraction.max(0.0);
            let p0 = origin + (-u - v) * half;
            let p1 = origin + (u - v) * half;
            let p2 = origin + (u + v) * half;
            let p3 = origin + (-u + v) * half;
            for (a, b) in [(p0, p1), (p1, p2), (p2, p3), (p3, p0)] {
                self.push_line(&mut out, a, b, c, DepthMode::Always);
            }
        }

        out
    }

    pub(super) fn draw_scale_solids(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
        size_length_world: f32,
        include_axes: bool,
        include_uniform: bool,
        include_planes: bool,
    ) -> Vec<Triangle3d> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;

        let (u, v) = view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            .map(plane_basis)
            .unwrap_or((Vec3::X, Vec3::Y));

        let mut out = Vec::new();

        // Axis end boxes (screen-facing filled quads).
        if include_axes {
            for &(((axis_dir, color), handle), axis_index) in &[
                (
                    ((axes[0], self.config.x_color), ScaleHandle::AxisX.id()),
                    0usize,
                ),
                (
                    ((axes[1], self.config.y_color), ScaleHandle::AxisY.id()),
                    1usize,
                ),
                (
                    ((axes[2], self.config.z_color), ScaleHandle::AxisZ.id()),
                    2usize,
                ),
            ] {
                if self.axis_is_masked(axis_index) {
                    continue;
                }
                let alpha = self.axis_visibility_alpha(
                    view_projection,
                    viewport,
                    origin,
                    axis_dir,
                    length_world,
                );
                if alpha <= 0.01 {
                    continue;
                }
                let outline = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
                    self.config.hover_color
                } else {
                    color
                };
                let fill = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
                    mix_alpha(outline, 0.70)
                } else {
                    mix_alpha(outline, 0.45)
                };
                let fill = mix_alpha(fill, alpha);

                let end = origin + axis_dir * length_world;
                let half = length_world * pv.scale_axis_end_box_half_fraction.max(0.0);
                let p0 = end + (-u - v) * half;
                let p1 = end + (u - v) * half;
                let p2 = end + (u + v) * half;
                let p3 = end + (-u + v) * half;
                if pv.occlusion.handles {
                    self.push_tri(&mut out, p0, p1, p2, fill, self.config.depth_mode);
                    self.push_tri(&mut out, p0, p2, p3, fill, self.config.depth_mode);
                } else {
                    self.push_tri_no_ghost(&mut out, p0, p1, p2, fill, self.config.depth_mode);
                    self.push_tri_no_ghost(&mut out, p0, p2, p3, fill, self.config.depth_mode);
                }
            }
        }

        if include_planes {
            let off = length_world * pv.scale_plane_offset_fraction.max(0.0);
            let size = length_world * pv.scale_plane_size_fraction.max(0.0);
            for &(u, v, base_color, handle) in &[
                (
                    axes[0],
                    axes[1],
                    mix_alpha(self.config.z_color, 0.55),
                    ScaleHandle::PlaneXY,
                ), // XY
                (
                    axes[0],
                    axes[2],
                    mix_alpha(self.config.y_color, 0.55),
                    ScaleHandle::PlaneXZ,
                ), // XZ
                (
                    axes[1],
                    axes[2],
                    mix_alpha(self.config.x_color, 0.55),
                    ScaleHandle::PlaneYZ,
                ), // YZ
            ] {
                let handle_id = handle.id();
                let plane_axes = match handle {
                    ScaleHandle::PlaneXY => (0usize, 1usize),
                    ScaleHandle::PlaneXZ => (0usize, 2usize),
                    ScaleHandle::PlaneYZ => (1usize, 2usize),
                    _ => continue,
                };
                if !self.plane_allowed_by_mask(plane_axes) {
                    continue;
                }
                let outline = if self.is_handle_highlighted(GizmoMode::Scale, handle_id) {
                    mix_alpha(self.config.hover_color, 0.85)
                } else {
                    base_color
                };
                let fill = if self.is_handle_highlighted(GizmoMode::Scale, handle_id) {
                    mix_alpha(outline, pv.scale_plane_fill_hover_alpha.clamp(0.0, 1.0))
                } else {
                    mix_alpha(outline, pv.scale_plane_fill_alpha.clamp(0.0, 1.0))
                };

                let quad = translate_plane_quad_world(origin, u, v, off, size);
                let alpha = self.plane_visibility_alpha(view_projection, viewport, quad);
                if alpha <= 0.01 {
                    continue;
                }
                let fill = mix_alpha(fill, alpha);
                self.push_quad_fill(
                    &mut out,
                    quad,
                    fill,
                    self.config.depth_mode,
                    pv.occlusion.scale_plane_fill,
                );
            }
        }

        if include_uniform {
            // Uniform scale at the origin (screen-facing filled quad), always-on-top.
            let handle = ScaleHandle::Uniform.id();
            let outline = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
                self.config.hover_color
            } else {
                mix_alpha(self.config.hover_color, 0.65)
            };
            let fill = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
                mix_alpha(outline, 0.75)
            } else {
                mix_alpha(outline, 0.40)
            };

            let half = length_world * pv.scale_uniform_half_fraction.max(0.0);
            let p0 = origin + (-u - v) * half;
            let p1 = origin + (u - v) * half;
            let p2 = origin + (u + v) * half;
            let p3 = origin + (-u + v) * half;
            self.push_tri(&mut out, p0, p1, p2, fill, DepthMode::Always);
            self.push_tri(&mut out, p0, p2, p3, fill, DepthMode::Always);
        }

        out
    }
}

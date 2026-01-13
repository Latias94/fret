use super::*;

impl Gizmo {
    pub(super) fn draw_rotate_rings(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> GizmoDrawList3d {
        let (include_axis, include_view, include_arcball) =
            if let Some(mask) = self.config.operation_mask {
                (
                    mask.contains(GizmoOps::rotate_axis()),
                    mask.contains(GizmoOps::rotate_view()),
                    mask.contains(GizmoOps::rotate_arcball()),
                )
            } else {
                let (view, arcball) = match self.config.mode {
                    GizmoMode::Universal => (
                        self.config.universal_includes_rotate_view_ring,
                        self.config.universal_includes_arcball,
                    ),
                    _ => (self.config.show_view_axis_ring, self.config.show_arcball),
                };
                (true, view, arcball)
            };

        let radius_world = size_length_world;
        let pv = self.state.part_visuals;

        let segments: usize = 64;
        let mut out = GizmoDrawList3d {
            lines: Vec::with_capacity(segments * 3),
            triangles: Vec::with_capacity(segments * 3 * 2),
        };

        let thickness_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            (self.config.line_thickness_px * pv.rotate_ring_thickness_scale).max(0.0),
        )
        .unwrap_or(radius_world * 0.04);

        let mut push_ring_band = |u: Vec3,
                                  v: Vec3,
                                  radius_world: f32,
                                  color: Color,
                                  depth: DepthMode,
                                  allow_ghost: bool| {
            let half = (thickness_world * 0.55)
                .clamp(radius_world * 0.010, radius_world * 0.075)
                .max(1e-6);
            let inner_r = (radius_world - half).max(radius_world * 0.2).max(1e-6);
            let outer_r = radius_world + half;

            let fill = mix_alpha(color, pv.rotate_ring_fill_alpha.clamp(0.0, 1.0));
            let edge = mix_alpha(color, pv.rotate_ring_edge_alpha.clamp(0.0, 1.0));
            self.push_ring_band(
                &mut out,
                origin,
                u,
                v,
                inner_r,
                outer_r,
                fill,
                edge,
                depth,
                allow_ghost,
                segments,
            );
        };

        if include_axis {
            for &(((axis_dir, color), handle), axis_index) in &[
                (
                    ((axes[0], self.config.x_color), RotateHandle::AxisX.id()),
                    0usize,
                ),
                (
                    ((axes[1], self.config.y_color), RotateHandle::AxisY.id()),
                    1usize,
                ),
                (
                    ((axes[2], self.config.z_color), RotateHandle::AxisZ.id()),
                    2usize,
                ),
            ] {
                if self.axis_is_masked(axis_index) {
                    continue;
                }
                let axis_dir = axis_dir.normalize_or_zero();
                if axis_dir.length_squared() == 0.0 {
                    continue;
                }
                let (u, v) = plane_basis(axis_dir);
                let alpha =
                    self.rotate_ring_visibility_alpha(view_projection, viewport, origin, axis_dir);
                if alpha <= 0.01 {
                    continue;
                }
                let c = if self.is_handle_highlighted(GizmoMode::Rotate, handle) {
                    self.config.hover_color
                } else {
                    color
                };
                let c = mix_alpha(c, alpha);

                push_ring_band(
                    u,
                    v,
                    radius_world,
                    c,
                    self.config.depth_mode,
                    pv.occlusion.rotate_axis_rings,
                );
            }
        }

        if include_view
            && let Some(view_dir) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        {
            let axis_dir = view_dir.normalize_or_zero();
            if axis_dir.length_squared() > 0.0 {
                let (u, v) = plane_basis(axis_dir);
                let handle = RotateHandle::View.id();
                let r = (radius_world * self.config.view_axis_ring_radius_scale).max(1e-6);
                let base = Color {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 0.8,
                };
                let c = if self.is_handle_highlighted(GizmoMode::Rotate, handle) {
                    self.config.hover_color
                } else {
                    base
                };

                push_ring_band(u, v, r, c, DepthMode::Always, pv.occlusion.rotate_view_ring);
            }
        }

        if include_arcball
            && let Some(view_dir) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        {
            let axis_dir = (-view_dir).normalize_or_zero();
            if axis_dir.length_squared() > 0.0 {
                let (u, v) = plane_basis(axis_dir);
                let r = (radius_world * self.config.arcball_radius_scale).max(1e-6);

                let handle = RotateHandle::Arcball.id();
                let base = Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.12,
                };
                let c = if self.is_handle_highlighted(GizmoMode::Rotate, handle) {
                    mix_alpha(self.config.hover_color, 0.55)
                } else {
                    base
                };
                push_ring_band(
                    u,
                    v,
                    r,
                    c,
                    DepthMode::Always,
                    pv.occlusion.rotate_arcball_ring,
                );
            }
        }

        out
    }

    pub(super) fn draw_rotate_feedback(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        size_length_world: f32,
    ) -> GizmoDrawList3d {
        let pv = self.state.part_visuals;
        let feedback_allow_ghost = pv.occlusion.feedback;
        if self.state.drag_mode != GizmoMode::Rotate {
            return GizmoDrawList3d::default();
        }
        let Some(active) = self.state.active else {
            return GizmoDrawList3d::default();
        };
        if !self.state.drag_has_started {
            return GizmoDrawList3d::default();
        }

        if self.state.drag_rotate_is_arcball {
            let u = self.state.drag_basis_u.normalize_or_zero();
            let v = self.state.drag_basis_v.normalize_or_zero();
            if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
                return GizmoDrawList3d::default();
            }

            let radius_world = (size_length_world * self.config.arcball_radius_scale).max(1e-6);

            let outline = mix_alpha(self.config.hover_color, 0.65);
            let fill = mix_alpha(self.config.hover_color, 0.10);
            let segments: usize = 48;
            let mut out = GizmoDrawList3d::default();

            let mut prev = origin + u * radius_world;
            for i in 1..=segments {
                let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                let p = origin + (u * t.cos() + v * t.sin()) * radius_world;
                if feedback_allow_ghost {
                    self.push_line(&mut out.lines, prev, p, outline, DepthMode::Always);
                } else {
                    self.push_line_no_ghost(&mut out.lines, prev, p, outline, DepthMode::Always);
                }
                self.push_tri_no_ghost(
                    &mut out.triangles,
                    origin,
                    prev,
                    p,
                    fill,
                    DepthMode::Always,
                );
                prev = p;
            }
            return out;
        }

        let axis_dir = self.state.drag_axis_dir.normalize_or_zero();
        if axis_dir.length_squared() == 0.0 {
            return GizmoDrawList3d::default();
        }

        let base_radius_world = size_length_world.max(1e-6);
        let radius_world = if active == Self::ROTATE_VIEW_HANDLE {
            (base_radius_world * self.config.view_axis_ring_radius_scale).max(1e-6)
        } else {
            base_radius_world
        };

        let thickness_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            (self.config.line_thickness_px * pv.rotate_feedback_thickness_scale).max(0.0),
        )
        .unwrap_or(radius_world * 0.04);

        let half = (thickness_world * 1.15)
            .clamp(radius_world * 0.015, radius_world * 0.12)
            .max(1e-6);
        let inner_r = (radius_world - half).max(radius_world * 0.2);
        let outer_r = radius_world + half;

        let base = match active.0 {
            1 => self.config.x_color,
            2 => self.config.y_color,
            3 => self.config.z_color,
            8 => Color {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 0.8,
            },
            9 => Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.25,
            },
            _ => self.config.hover_color,
        };
        let color = if self.state.active == Some(active) {
            self.config.hover_color
        } else {
            base
        };

        let total = self.state.drag_total_angle_applied;
        let start = self.state.drag_start_angle;
        let end = start + total;

        let u = self.state.drag_basis_u.normalize_or_zero();
        let v = self.state.drag_basis_v.normalize_or_zero();
        if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
            return GizmoDrawList3d::default();
        }

        let mut out = GizmoDrawList3d::default();

        if total.abs() > 1e-6 {
            let segs = ((total.abs() / std::f32::consts::TAU) * 96.0)
                .ceil()
                .clamp(12.0, 192.0) as usize;
            let step = total / (segs as f32);

            let fill_alpha = if self.state.drag_snap { 0.30 } else { 0.22 };
            let fill = mix_alpha(color, fill_alpha);
            let edge = mix_alpha(color, 0.95);

            let point =
                |theta: f32, r: f32| -> Vec3 { origin + (u * theta.cos() + v * theta.sin()) * r };

            let mut prev_outer = point(start, outer_r);
            for i in 0..segs {
                let t0 = start + step * (i as f32);
                let t1 = start + step * ((i + 1) as f32);
                let o0 = point(t0, outer_r);
                let i0 = point(t0, inner_r);
                let o1 = point(t1, outer_r);
                let i1 = point(t1, inner_r);

                self.push_tri_no_ghost(&mut out.triangles, o0, i0, i1, fill, DepthMode::Always);
                self.push_tri_no_ghost(&mut out.triangles, o0, i1, o1, fill, DepthMode::Always);

                if feedback_allow_ghost {
                    self.push_line(&mut out.lines, prev_outer, o1, edge, DepthMode::Always);
                } else {
                    self.push_line_no_ghost(
                        &mut out.lines,
                        prev_outer,
                        o1,
                        edge,
                        DepthMode::Always,
                    );
                }
                prev_outer = o1;
            }

            let start_dir = (u * start.cos() + v * start.sin()).normalize_or_zero();
            let end_dir = (u * end.cos() + v * end.sin()).normalize_or_zero();
            if start_dir.length_squared() > 0.0 {
                let a = origin;
                let b = origin + start_dir * radius_world;
                let c = mix_alpha(color, 0.35);
                if feedback_allow_ghost {
                    self.push_line(&mut out.lines, a, b, c, DepthMode::Always);
                } else {
                    self.push_line_no_ghost(&mut out.lines, a, b, c, DepthMode::Always);
                }
            }
            if end_dir.length_squared() > 0.0 {
                let a0 = origin;
                let b0 = origin + end_dir * radius_world;
                let c0 = mix_alpha(color, 0.75);
                if feedback_allow_ghost {
                    self.push_line(&mut out.lines, a0, b0, c0, DepthMode::Always);
                } else {
                    self.push_line_no_ghost(&mut out.lines, a0, b0, c0, DepthMode::Always);
                }

                let a1 = origin + end_dir * inner_r;
                let b1 = origin + end_dir * (outer_r + half * 0.8);
                if feedback_allow_ghost {
                    self.push_line(&mut out.lines, a1, b1, edge, DepthMode::Always);
                } else {
                    self.push_line_no_ghost(&mut out.lines, a1, b1, edge, DepthMode::Always);
                }
            }
        }

        if self.state.drag_snap
            && let Some(step) = self
                .config
                .rotate_snap_step_radians
                .filter(|s| s.is_finite() && *s > 0.0)
        {
            let ticks = (std::f32::consts::TAU / step).round() as usize;
            if (4..=128).contains(&ticks) {
                let tick_color = Color {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 0.35,
                };
                for k in 0..ticks {
                    let t = (k as f32) * step;
                    let dir = (u * t.cos() + v * t.sin()).normalize_or_zero();
                    if dir.length_squared() == 0.0 {
                        continue;
                    }
                    let a = origin + dir * (outer_r + half * 0.8);
                    let b = origin + dir * (outer_r + half * 2.2);
                    if feedback_allow_ghost {
                        self.push_line(&mut out.lines, a, b, tick_color, DepthMode::Always);
                    } else {
                        self.push_line_no_ghost(
                            &mut out.lines,
                            a,
                            b,
                            tick_color,
                            DepthMode::Always,
                        );
                    }
                }
            }
        }

        out
    }
}

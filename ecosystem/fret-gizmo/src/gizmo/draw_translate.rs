use super::*;

impl Gizmo {
    pub(super) fn draw_translate_axes(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Vec<Line3d> {
        let mut out = Vec::new();
        let pv = self.state.part_visuals;
        let length_world = size_length_world;
        let axis_tip_len = length_world * self.translate_axis_tip_scale();
        let head_len = length_world * pv.translate_head_length_fraction.max(0.0);
        let shaft_len =
            (length_world - head_len).max(length_world * pv.translate_shaft_min_fraction.max(0.0));
        for &(((axis_dir, color), handle), axis_index) in &[
            (((axes[0], self.config.x_color), HandleId(1)), 0usize),
            (((axes[1], self.config.y_color), HandleId(2)), 1usize),
            (((axes[2], self.config.z_color), HandleId(3)), 2usize),
        ] {
            if self.axis_is_masked(axis_index) {
                continue;
            }
            let alpha = self.axis_visibility_alpha(
                view_projection,
                viewport,
                origin,
                axis_dir,
                axis_tip_len,
            );
            if alpha <= 0.01 {
                continue;
            }
            let c = if self.is_handle_highlighted(GizmoMode::Translate, handle) {
                self.config.hover_color
            } else {
                color
            };
            let c = mix_alpha(c, alpha);
            if pv.occlusion.handles {
                self.push_line(
                    &mut out,
                    origin,
                    origin + axis_dir * shaft_len,
                    c,
                    self.config.depth_mode,
                );
            } else {
                self.push_line_no_ghost(
                    &mut out,
                    origin,
                    origin + axis_dir * shaft_len,
                    c,
                    self.config.depth_mode,
                );
            }
        }
        out
    }

    pub(super) fn draw_translate_planes(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Vec<Line3d> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;

        let off = length_world * pv.translate_plane_offset_fraction.max(0.0);
        let size = length_world * pv.translate_plane_size_fraction.max(0.0);

        let mut out = Vec::new();
        for &(u, v, base_color, handle) in &[
            (
                axes[0],
                axes[1],
                mix_alpha(self.config.z_color, 0.55),
                TranslateHandle::PlaneXY,
            ), // XY
            (
                axes[0],
                axes[2],
                mix_alpha(self.config.y_color, 0.55),
                TranslateHandle::PlaneXZ,
            ), // XZ
            (
                axes[1],
                axes[2],
                mix_alpha(self.config.x_color, 0.55),
                TranslateHandle::PlaneYZ,
            ), // YZ
        ] {
            let plane_axes = match handle {
                TranslateHandle::PlaneXY => (0usize, 1usize),
                TranslateHandle::PlaneXZ => (0usize, 2usize),
                TranslateHandle::PlaneYZ => (1usize, 2usize),
                _ => continue,
            };
            if !self.plane_allowed_by_mask(plane_axes) {
                continue;
            }
            let handle_id = handle.id();
            let color = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
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
        out
    }

    pub(super) fn draw_translate_screen(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        size_length_world: f32,
    ) -> Vec<Line3d> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;

        let Some(view_dir) =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        else {
            return Vec::new();
        };
        let (u, v) = plane_basis(view_dir);
        let half = length_world * pv.translate_center_half_fraction.max(0.0);
        let base = mix_alpha(self.config.hover_color, 0.65);
        let handle_id = TranslateHandle::Screen.id();
        let color = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
            mix_alpha(self.config.hover_color, 1.0)
        } else {
            base
        };

        let p0 = origin + (-u - v) * half;
        let p1 = origin + (u - v) * half;
        let p2 = origin + (u + v) * half;
        let p3 = origin + (-u + v) * half;

        let mut out = Vec::new();
        for (a, b) in [(p0, p1), (p1, p2), (p2, p3), (p3, p0)] {
            self.push_line(&mut out, a, b, color, DepthMode::Always);
        }
        out
    }

    pub(super) fn draw_translate_depth(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        size_length_world: f32,
    ) -> Vec<Line3d> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;

        let Some(view_dir) =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        else {
            return Vec::new();
        };
        let axis_dir = view_dir.normalize_or_zero();
        if axis_dir.length_squared() == 0.0 {
            return Vec::new();
        }
        let (u, v) = plane_basis(axis_dir);

        // A small ring around the center handle that controls translation along the view direction
        // (a "dolly" translate handle). The ring is rendered always-on-top so it remains usable in
        // dense scenes.
        let r = (length_world * pv.translate_depth_ring_radius_fraction.max(0.0))
            .max(length_world * pv.translate_depth_ring_radius_min_fraction.max(0.0));
        let handle_id = TranslateHandle::Depth.id();
        let base = mix_alpha(self.config.hover_color, 0.35);
        let color = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
            mix_alpha(self.config.hover_color, 0.95)
        } else {
            base
        };

        let segments: usize = 36;
        let mut out = Vec::with_capacity(segments);
        let mut prev = origin + u * r;
        for i in 1..=segments {
            let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let p = origin + (u * t.cos() + v * t.sin()) * r;
            self.push_line(&mut out, prev, p, color, DepthMode::Always);
            prev = p;
        }
        out
    }

    pub(super) fn draw_translate_solids(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
        size_length_world: f32,
        include_axes: bool,
        include_planes: bool,
        include_screen: bool,
    ) -> Vec<Triangle3d> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;

        let head_len = length_world * pv.translate_head_length_fraction.max(0.0);
        let head_radius = length_world * pv.translate_head_radius_fraction.max(0.0);
        let axis_tip_len = length_world * self.translate_axis_tip_scale();

        let mut out = Vec::new();
        let push_handle_tri = |out: &mut Vec<Triangle3d>,
                               a: Vec3,
                               b: Vec3,
                               c: Vec3,
                               color: Color,
                               depth: DepthMode| {
            if pv.occlusion.handles {
                self.push_tri(out, a, b, c, color, depth);
            } else {
                self.push_tri_no_ghost(out, a, b, c, color, depth);
            }
        };

        if include_axes {
            for &(((axis_dir, color), handle), axis_index) in &[
                (((axes[0], self.config.x_color), HandleId(1)), 0usize),
                (((axes[1], self.config.y_color), HandleId(2)), 1usize),
                (((axes[2], self.config.z_color), HandleId(3)), 2usize),
            ] {
                if self.axis_is_masked(axis_index) {
                    continue;
                }
                let axis_dir = axis_dir.normalize_or_zero();
                if axis_dir.length_squared() == 0.0 {
                    continue;
                }
                let c = if self.is_handle_highlighted(GizmoMode::Translate, handle) {
                    self.config.hover_color
                } else {
                    color
                };
                let alpha = self.axis_visibility_alpha(
                    view_projection,
                    viewport,
                    origin,
                    axis_dir,
                    axis_tip_len,
                );
                if alpha <= 0.01 {
                    continue;
                }

                let tip = origin + axis_dir * axis_tip_len;
                let base = tip - axis_dir * head_len;
                let (u, v) = plane_basis(axis_dir);
                let s = head_radius * 0.70710677;
                let c0 = base + (u + v) * s;
                let c1 = base + (-u + v) * s;
                let c2 = base + (-u - v) * s;
                let c3 = base + (u - v) * s;

                let c = mix_alpha(c, alpha);
                push_handle_tri(&mut out, tip, c0, c1, c, self.config.depth_mode);
                push_handle_tri(&mut out, tip, c1, c2, c, self.config.depth_mode);
                push_handle_tri(&mut out, tip, c2, c3, c, self.config.depth_mode);
                push_handle_tri(&mut out, tip, c3, c0, c, self.config.depth_mode);
            }
        }

        // Plane handle fills.
        if include_planes {
            let off = length_world * pv.translate_plane_offset_fraction.max(0.0);
            let size = length_world * pv.translate_plane_size_fraction.max(0.0);
            for &(u, v, base_color, handle) in &[
                (
                    axes[0],
                    axes[1],
                    mix_alpha(self.config.z_color, 0.55),
                    TranslateHandle::PlaneXY,
                ), // XY
                (
                    axes[0],
                    axes[2],
                    mix_alpha(self.config.y_color, 0.55),
                    TranslateHandle::PlaneXZ,
                ), // XZ
                (
                    axes[1],
                    axes[2],
                    mix_alpha(self.config.x_color, 0.55),
                    TranslateHandle::PlaneYZ,
                ), // YZ
            ] {
                let plane_axes = match handle {
                    TranslateHandle::PlaneXY => (0usize, 1usize),
                    TranslateHandle::PlaneXZ => (0usize, 2usize),
                    TranslateHandle::PlaneYZ => (1usize, 2usize),
                    _ => continue,
                };
                if !self.plane_allowed_by_mask(plane_axes) {
                    continue;
                }
                let handle_id = handle.id();
                let outline = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
                    mix_alpha(self.config.hover_color, 0.85)
                } else {
                    base_color
                };
                let fill = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
                    mix_alpha(outline, pv.translate_plane_fill_hover_alpha.clamp(0.0, 1.0))
                } else {
                    mix_alpha(outline, pv.translate_plane_fill_alpha.clamp(0.0, 1.0))
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
                    pv.occlusion.translate_plane_fill,
                );
            }
        }

        // Screen translate fill (center handle), always-on-top.
        if include_screen {
            if let Some(view_dir) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            {
                let (u, v) = plane_basis(view_dir);
                let half = length_world * pv.translate_center_half_fraction.max(0.0);
                let handle_id = TranslateHandle::Screen.id();
                let outline = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
                    mix_alpha(self.config.hover_color, 1.0)
                } else {
                    mix_alpha(self.config.hover_color, 0.65)
                };
                let fill = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
                    mix_alpha(outline, 0.65)
                } else {
                    mix_alpha(outline, 0.35)
                };

                let p0 = origin + (-u - v) * half;
                let p1 = origin + (u - v) * half;
                let p2 = origin + (u + v) * half;
                let p3 = origin + (-u + v) * half;
                self.push_tri(&mut out, p0, p1, p2, fill, DepthMode::Always);
                self.push_tri(&mut out, p0, p2, p3, fill, DepthMode::Always);
            }
        }

        out
    }

    pub(super) fn draw_translate_feedback(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        size_length_world: f32,
    ) -> GizmoDrawList3d {
        if self.state.drag_mode != GizmoMode::Translate {
            return GizmoDrawList3d::default();
        }
        if self.state.active.is_none() || !self.state.drag_has_started || !self.state.drag_snap {
            return GizmoDrawList3d::default();
        }
        let Some(step) = self
            .config
            .translate_snap_step
            .filter(|s| s.is_finite() && *s > 0.0)
        else {
            return GizmoDrawList3d::default();
        };

        let tick_len_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            10.0,
        )
        .unwrap_or(size_length_world.max(1e-6) * 0.06);
        let half = (tick_len_world * 0.5).max(1e-6);

        let minor = Color {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 0.20,
        };
        let major = Color {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 0.28,
        };
        let highlight = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.75,
        };

        let range = (size_length_world.max(1e-6) * 1.25).max(step * 2.0);
        let count = ((range / step).ceil() as i32).clamp(1, 24);

        let mut out = GizmoDrawList3d::default();

        if self.state.drag_translate_is_plane {
            let u = self.state.drag_translate_u.normalize_or_zero();
            let v = self.state.drag_translate_v.normalize_or_zero();
            if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
                return GizmoDrawList3d::default();
            }

            for k in -count..=count {
                let off = (k as f32) * step;
                let is_major = k % 5 == 0;
                let c = if is_major { major } else { minor };

                let a = origin + v * off - u * range;
                let b = origin + v * off + u * range;
                self.push_line(&mut out.lines, a, b, c, DepthMode::Always);

                let a = origin + u * off - v * range;
                let b = origin + u * off + v * range;
                self.push_line(&mut out.lines, a, b, c, DepthMode::Always);
            }

            let applied = self.state.drag_total_plane_applied;
            let p = origin + u * applied.x + v * applied.y;
            self.push_line(
                &mut out.lines,
                p - u * (half * 1.5),
                p + u * (half * 1.5),
                highlight,
                DepthMode::Always,
            );
            self.push_line(
                &mut out.lines,
                p - v * (half * 1.5),
                p + v * (half * 1.5),
                highlight,
                DepthMode::Always,
            );
        } else {
            let axis_dir = self.state.drag_axis_dir.normalize_or_zero();
            if axis_dir.length_squared() == 0.0 {
                return GizmoDrawList3d::default();
            }
            let tick_dir = Self::tick_perp_dir(
                view_projection,
                viewport,
                origin,
                self.config.depth_range,
                axis_dir,
            );
            if tick_dir.length_squared() == 0.0 {
                return GizmoDrawList3d::default();
            }

            for k in -count..=count {
                let off = (k as f32) * step;
                let pos = origin + axis_dir * off;
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

            let p = origin + axis_dir * self.state.drag_total_axis_applied;
            self.push_line(
                &mut out.lines,
                p - tick_dir * (half * 2.2),
                p + tick_dir * (half * 2.2),
                highlight,
                DepthMode::Always,
            );
        }

        out
    }
}

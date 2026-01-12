use super::*;

impl Gizmo {
    pub(super) fn draw_bounds(
        &self,
        out: &mut GizmoDrawList3d,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        basis: [Vec3; 3],
        size_length_world: f32,
        targets: &[GizmoTarget3d],
    ) {
        let pv = self.state.part_visuals;
        let allow_ghost = pv.occlusion.bounds;
        if targets.is_empty() {
            return;
        }
        if basis.iter().any(|v| v.length_squared() == 0.0) {
            return;
        }

        let (min_local, max_local) = self.bounds_min_max_local(
            view_projection,
            viewport,
            origin,
            basis,
            size_length_world,
            targets,
        );
        let center_local = (min_local + max_local) * 0.5;

        let to_world = |local: Vec3| -> Vec3 {
            origin + basis[0] * local.x + basis[1] * local.y + basis[2] * local.z
        };

        let corners_local = [
            Vec3::new(min_local.x, min_local.y, min_local.z),
            Vec3::new(max_local.x, min_local.y, min_local.z),
            Vec3::new(max_local.x, max_local.y, min_local.z),
            Vec3::new(min_local.x, max_local.y, min_local.z),
            Vec3::new(min_local.x, min_local.y, max_local.z),
            Vec3::new(max_local.x, min_local.y, max_local.z),
            Vec3::new(max_local.x, max_local.y, max_local.z),
            Vec3::new(min_local.x, max_local.y, max_local.z),
        ];
        let corners_world = corners_local.map(to_world);

        let box_color = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.45,
        };
        for (a, b) in [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ] {
            if allow_ghost {
                self.push_line(
                    &mut out.lines,
                    corners_world[a],
                    corners_world[b],
                    box_color,
                    self.config.depth_mode,
                );
            } else {
                self.push_line_no_ghost(
                    &mut out.lines,
                    corners_world[a],
                    corners_world[b],
                    box_color,
                    self.config.depth_mode,
                );
            }
        }

        let Some(view_dir) =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        else {
            return;
        };
        let (u, v) = plane_basis(view_dir);

        let handle_size_px = self.config.bounds_handle_size_px.max(1.0);
        let handle_half_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            handle_size_px,
        )
        .unwrap_or(1.0)
        .max(1e-6)
            * 0.5;

        let push_handle = |this: &Gizmo,
                           out: &mut GizmoDrawList3d,
                           pos: Vec3,
                           handle: HandleId,
                           base: Color,
                           bias: f32| {
            let outline = if this.is_handle_highlighted(GizmoMode::Scale, handle) {
                this.config.hover_color
            } else {
                base
            };
            let fill = mix_alpha(outline, (0.55 - bias).clamp(0.08, 0.55));
            let outline = mix_alpha(outline, (0.95 - bias).clamp(0.15, 0.95));

            let p0 = pos + (-u - v) * handle_half_world;
            let p1 = pos + (u - v) * handle_half_world;
            let p2 = pos + (u + v) * handle_half_world;
            let p3 = pos + (-u + v) * handle_half_world;
            let quad = [p0, p1, p2, p3];
            this.push_quad_fill(
                &mut out.triangles,
                quad,
                fill,
                this.config.depth_mode,
                allow_ghost,
            );
            this.push_quad_outline(
                &mut out.lines,
                quad,
                outline,
                this.config.depth_mode,
                allow_ghost,
            );
        };

        let corner_base = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.9,
        };
        for z_max in [false, true] {
            for y_max in [false, true] {
                for x_max in [false, true] {
                    let local = Vec3::new(
                        if x_max { max_local.x } else { min_local.x },
                        if y_max { max_local.y } else { min_local.y },
                        if z_max { max_local.z } else { min_local.z },
                    );
                    let world = to_world(local);
                    push_handle(
                        self,
                        out,
                        world,
                        Self::bounds_corner_id(x_max, y_max, z_max),
                        corner_base,
                        0.0,
                    );
                }
            }
        }

        let face_base = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.65,
        };
        for axis in 0..3 {
            for &max_side in &[false, true] {
                let mut local = center_local;
                local[axis] = if max_side {
                    max_local[axis]
                } else {
                    min_local[axis]
                };
                let world = to_world(local);
                push_handle(
                    self,
                    out,
                    world,
                    Self::bounds_face_id(axis, max_side),
                    face_base,
                    0.25,
                );
            }
        }
    }
}

use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PickHit {
    pub(super) handle: HandleId,
    pub(super) score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum MixedPickBand {
    TranslateCenter = 0,
    TranslatePlaneInside = 1,
    ScaleSolidInside = 2,
    TranslateAxisTipIntent = 3,
    Default = 4,
}

impl Gizmo {
    pub fn pick_hit(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        cursor_px: Vec2,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> Option<(HandleId, f32)> {
        if targets.is_empty() {
            return None;
        }

        let active_index = targets
            .iter()
            .position(|t| t.id == active_target)
            .unwrap_or(0);
        let active_transform = targets
            .get(active_index)
            .map(|t| t.transform)
            .unwrap_or_else(|| targets[0].transform);

        let origin = Self::pivot_origin(active_transform, targets, self.config.pivot_mode);
        let size_length_world =
            self.size_length_world_or_one(view_projection, viewport, origin, targets);

        let axes_raw = self.axis_dirs(&active_transform);
        let axes = self.flip_axes_for_view(
            view_projection,
            viewport,
            origin,
            axes_raw,
            size_length_world,
        );

        let hit: Option<PickHit> = if self.config.operation_mask.is_some() {
            self.pick_operation_mask_handle(
                view_projection,
                viewport,
                origin,
                cursor_px,
                axes,
                axes_raw,
                size_length_world,
                targets,
            )
            .map(|(h, _kind)| h)
        } else {
            match self.config.mode {
                GizmoMode::Translate => self.pick_translate_handle(
                    view_projection,
                    viewport,
                    origin,
                    cursor_px,
                    axes,
                    size_length_world,
                    true,
                    true,
                    true,
                    true,
                ),
                GizmoMode::Rotate => self.pick_rotate_axis(
                    view_projection,
                    viewport,
                    origin,
                    cursor_px,
                    axes,
                    size_length_world,
                ),
                GizmoMode::Scale => self.pick_scale_or_bounds_handle(
                    view_projection,
                    viewport,
                    origin,
                    cursor_px,
                    axes,
                    axes_raw,
                    size_length_world,
                    targets,
                ),
                GizmoMode::Universal => self
                    .pick_universal_handle(
                        view_projection,
                        viewport,
                        origin,
                        cursor_px,
                        axes,
                        size_length_world,
                    )
                    .map(|(h, _kind)| h),
            }
        };

        hit.map(|h| (h.handle, h.score))
    }

    pub(super) fn pick_translate_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        include_axes: bool,
        include_planes: bool,
        include_screen: bool,
        include_depth: bool,
    ) -> Option<PickHit> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;
        let axis_tip_len = length_world * self.translate_axis_tip_scale();

        // Picking priority ladder (editor UX):
        // 1) Center / screen-plane handle (when within radius, always win)
        // 2) Plane handles (when cursor is inside the plane quad)
        // 3) Axis handles (distance to segment)
        //
        // This avoids a common frustration where the axis segment "steals" clicks near the origin.
        if include_screen
            && let Some(p0) =
                project_point(view_projection, viewport, origin, self.config.depth_range)
        {
            let r = self.config.pick_radius_px.max(6.0);
            if let Some(d) = (PickCircle2d {
                center: p0.screen,
                radius: r,
            })
            .hit_distance(cursor)
            {
                return Some(PickHit {
                    handle: TranslateHandle::Screen.id(),
                    score: d,
                });
            }
        }

        // Dolly translation handle (a small ring in the view plane around the center).
        if include_depth
            && let Some(view_dir) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        {
            let axis_dir = view_dir.normalize_or_zero();
            if axis_dir.length_squared() > 0.0 {
                let (u, v) = plane_basis(axis_dir);
                let r_world = (length_world * pv.translate_depth_ring_radius_fraction.max(0.0))
                    .max(length_world * pv.translate_depth_ring_radius_min_fraction.max(0.0));
                let segments: usize = 36;
                let mut prev_world = origin + u * r_world;
                let mut best_d = f32::INFINITY;
                for i in 1..=segments {
                    let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                    let world = origin + (u * t.cos() + v * t.sin()) * r_world;
                    let Some(pa) = project_point(
                        view_projection,
                        viewport,
                        prev_world,
                        self.config.depth_range,
                    ) else {
                        prev_world = world;
                        continue;
                    };
                    let Some(pb) =
                        project_point(view_projection, viewport, world, self.config.depth_range)
                    else {
                        prev_world = world;
                        continue;
                    };
                    best_d = best_d.min(distance_point_to_segment_px(cursor, pa.screen, pb.screen));
                    prev_world = world;
                }
                let r = self.config.pick_radius_px.max(6.0);
                if best_d.is_finite() && best_d <= r {
                    return Some(PickHit {
                        handle: TranslateHandle::Depth.id(),
                        score: best_d,
                    });
                }
            }
        }

        // Plane handles (distance to projected quad; accept when inside).
        let off = length_world * pv.translate_plane_offset_fraction.max(0.0);
        let size = length_world * pv.translate_plane_size_fraction.max(0.0);
        let mut plane_inside: Option<(HandleId, f32)> = None;
        if include_planes {
            for &((u, v, handle), plane_axes) in &[
                (
                    (axes[0], axes[1], TranslateHandle::PlaneXY.id()),
                    (0usize, 1usize),
                ),
                (
                    (axes[0], axes[2], TranslateHandle::PlaneXZ.id()),
                    (0usize, 2usize),
                ),
                (
                    (axes[1], axes[2], TranslateHandle::PlaneYZ.id()),
                    (1usize, 2usize),
                ),
            ] {
                if !self.plane_allowed_by_mask(plane_axes) {
                    continue;
                }
                let world = translate_plane_quad_world(origin, u, v, off, size);
                let Some(p) =
                    project_quad(view_projection, viewport, world, self.config.depth_range)
                else {
                    continue;
                };
                let alpha = self.plane_visibility_alpha(view_projection, viewport, world);
                if alpha <= 0.01 {
                    continue;
                }

                let quad = PickConvexQuad2d { points: p };
                let inside = quad.contains(cursor);
                let edge_d = quad.edge_distance(cursor);
                if inside {
                    // When the cursor is actually inside the plane handle quad, always prefer plane
                    // drags over axis segments (common editor expectation).
                    //
                    // If multiple plane quads overlap in projection, prefer the one where the cursor
                    // is deeper inside (larger edge distance).
                    match plane_inside {
                        Some((_, best_edge_d)) if edge_d <= best_edge_d => {}
                        _ => plane_inside = Some((handle, edge_d)),
                    }
                } else {
                    // Edge-picking is handled below as part of the general "best score" selection.
                }
            }
        }

        if let Some((handle, _)) = plane_inside {
            return Some(PickHit { handle, score: 0.0 });
        }

        let mut best: Option<PickHit> = None;
        let mut consider = |handle: HandleId, score: f32| {
            if !score.is_finite() {
                return;
            }
            match best {
                Some(best) if score >= best.score => {}
                _ => best = Some(PickHit { handle, score }),
            }
        };

        // Axis handles (distance to projected segments).
        if include_axes {
            for &((axis_dir, handle), axis_index) in &[
                ((axes[0], TranslateHandle::AxisX.id()), 0usize),
                ((axes[1], TranslateHandle::AxisY.id()), 1usize),
                ((axes[2], TranslateHandle::AxisZ.id()), 2usize),
            ] {
                if self.axis_is_masked(axis_index) {
                    continue;
                }
                let a = origin;
                let b = origin + axis_dir * axis_tip_len;
                let Some(pa) = project_point(view_projection, viewport, a, self.config.depth_range)
                else {
                    continue;
                };
                let Some(pb) = project_point(view_projection, viewport, b, self.config.depth_range)
                else {
                    continue;
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
                let r = self.config.pick_radius_px * alpha.sqrt();
                if let Some(d) = (PickSegmentCapsule2d {
                    a: pa.screen,
                    b: pb.screen,
                    radius: r,
                })
                .hit_distance(cursor)
                {
                    consider(handle, d / alpha.max(0.05));
                }
            }
        }

        // Plane handle edge picking.
        if include_planes {
            for &((u, v, handle), plane_axes) in &[
                (
                    (axes[0], axes[1], TranslateHandle::PlaneXY.id()),
                    (0usize, 1usize),
                ),
                (
                    (axes[0], axes[2], TranslateHandle::PlaneXZ.id()),
                    (0usize, 2usize),
                ),
                (
                    (axes[1], axes[2], TranslateHandle::PlaneYZ.id()),
                    (1usize, 2usize),
                ),
            ] {
                if !self.plane_allowed_by_mask(plane_axes) {
                    continue;
                }
                let world = translate_plane_quad_world(origin, u, v, off, size);
                let Some(p) =
                    project_quad(view_projection, viewport, world, self.config.depth_range)
                else {
                    continue;
                };
                let alpha = self.plane_visibility_alpha(view_projection, viewport, world);
                if alpha <= 0.01 {
                    continue;
                }

                let edge_d = PickConvexQuad2d { points: p }.edge_distance(cursor);
                let r = self.config.pick_radius_px * alpha.sqrt();
                if edge_d <= r {
                    consider(handle, (edge_d + 0.9) / alpha.max(0.05));
                }
            }
        }

        best
    }

    pub(super) fn pick_scale_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        include_axes: bool,
        include_uniform: bool,
        include_planes: bool,
    ) -> Option<PickHit> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;

        // Picking priority ladder (editor UX):
        // 1) Uniform scale at origin (within radius, always win)
        // 2) Axis end boxes (match visuals; avoid picking the entire shaft)
        let mut best: Option<PickHit> = None;
        let mut consider = |handle: HandleId, score: f32| {
            if !score.is_finite() {
                return;
            }
            match best {
                Some(best) if score >= best.score => {}
                _ => best = Some(PickHit { handle, score }),
            }
        };

        // Uniform scale at the origin.
        if include_uniform
            && let Some(p0) =
                project_point(view_projection, viewport, origin, self.config.depth_range)
        {
            let r = self.config.pick_radius_px.max(6.0);
            if let Some(d) = (PickCircle2d {
                center: p0.screen,
                radius: r,
            })
            .hit_distance(cursor)
            {
                return Some(PickHit {
                    handle: ScaleHandle::Uniform.id(),
                    score: d,
                });
            }
        }

        // Axis scaling handles.
        if include_axes {
            let (u, v) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
                    .map(plane_basis)
                    .unwrap_or((Vec3::X, Vec3::Y));
            for &((axis_dir, handle), axis_index) in &[
                ((axes[0], ScaleHandle::AxisX.id()), 0usize),
                ((axes[1], ScaleHandle::AxisY.id()), 1usize),
                ((axes[2], ScaleHandle::AxisZ.id()), 2usize),
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

                let end = origin + axis_dir * length_world;
                let half = length_world * pv.scale_axis_end_box_half_fraction.max(0.0);
                let quad_world = [
                    end + (-u - v) * half,
                    end + (u - v) * half,
                    end + (u + v) * half,
                    end + (-u + v) * half,
                ];
                let Some(p) = project_quad(
                    view_projection,
                    viewport,
                    quad_world,
                    self.config.depth_range,
                ) else {
                    continue;
                };
                let quad = PickConvexQuad2d { points: p };
                let inside = quad.contains(cursor);
                let edge_d = quad.edge_distance(cursor);
                if inside {
                    consider(handle, 0.0);
                } else {
                    let r = self.config.pick_radius_px * alpha.sqrt();
                    if edge_d <= r {
                        consider(handle, edge_d / alpha.max(0.05));
                    }
                }
            }
        }

        if include_planes {
            let off = length_world * pv.scale_plane_offset_fraction.max(0.0);
            let size = length_world * pv.scale_plane_size_fraction.max(0.0);
            for &((u, v, handle), plane_axes) in &[
                (
                    (axes[0], axes[1], ScaleHandle::PlaneXY.id()),
                    (0usize, 1usize),
                ),
                (
                    (axes[0], axes[2], ScaleHandle::PlaneXZ.id()),
                    (0usize, 2usize),
                ),
                (
                    (axes[1], axes[2], ScaleHandle::PlaneYZ.id()),
                    (1usize, 2usize),
                ),
            ] {
                if !self.plane_allowed_by_mask(plane_axes) {
                    continue;
                }
                let world = translate_plane_quad_world(origin, u, v, off, size);
                let Some(p) =
                    project_quad(view_projection, viewport, world, self.config.depth_range)
                else {
                    continue;
                };
                let alpha = self.plane_visibility_alpha(view_projection, viewport, world);
                if alpha <= 0.01 {
                    continue;
                }

                let quad = PickConvexQuad2d { points: p };
                let inside = quad.contains(cursor);
                let edge_d = quad.edge_distance(cursor);
                if inside {
                    consider(handle, 0.20 / alpha.max(0.05));
                } else {
                    let r = self.config.pick_radius_px * alpha.sqrt();
                    if edge_d <= r {
                        consider(handle, (edge_d + 0.9) / alpha.max(0.05));
                    }
                }
            }
        }

        best
    }

    pub(super) fn pick_best_mixed_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        allow_translate_axis_tip_intent: bool,
        rotate: Option<(PickHit, GizmoMode)>,
        scale: Option<(PickHit, GizmoMode)>,
        translate: Option<(PickHit, GizmoMode)>,
    ) -> Option<(PickHit, GizmoMode)> {
        let rotate_present = rotate.is_some();

        let mut best: Option<(PickHit, GizmoMode, MixedPickBand, f32, u8)> = None;
        let mut consider = |candidate: Option<(PickHit, GizmoMode)>| {
            let Some((hit, kind)) = candidate else {
                return;
            };
            if !hit.score.is_finite() {
                return;
            }

            let band = self.mixed_pick_band(
                view_projection,
                viewport,
                origin,
                cursor,
                axes,
                size_length_world,
                rotate_present,
                allow_translate_axis_tip_intent,
                hit,
                kind,
            );
            let kind_priority: u8 = match kind {
                GizmoMode::Rotate => 0,
                GizmoMode::Scale => 1,
                GizmoMode::Translate => 2,
                GizmoMode::Universal => 3,
            };

            match best {
                Some((_best_hit, _best_kind, best_band, best_score, best_kind_pri)) => {
                    if band < best_band
                        || (band == best_band && (hit.score < best_score))
                        || (band == best_band
                            && hit.score == best_score
                            && kind_priority < best_kind_pri)
                    {
                        best = Some((hit, kind, band, hit.score, kind_priority));
                    }
                }
                None => best = Some((hit, kind, band, hit.score, kind_priority)),
            }
        };

        consider(rotate);
        consider(scale);
        consider(translate);
        best.map(|(hit, kind, _, _, _)| (hit, kind))
    }

    pub(super) fn pick_universal_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Option<(PickHit, GizmoMode)> {
        let translate = self
            .pick_translate_handle(
                view_projection,
                viewport,
                origin,
                cursor,
                axes,
                size_length_world,
                true,
                true,
                true,
                self.config.universal_includes_translate_depth,
            )
            .map(|h| (h, GizmoMode::Translate));
        let rotate = self
            .pick_rotate_axis(
                view_projection,
                viewport,
                origin,
                cursor,
                axes,
                size_length_world,
            )
            .map(|h| (h, GizmoMode::Rotate));
        let scale = self
            .config
            .universal_includes_scale
            .then(|| {
                self.pick_scale_handle(
                    view_projection,
                    viewport,
                    origin,
                    cursor,
                    axes,
                    size_length_world,
                    true,
                    false,
                    false,
                )
            })
            .flatten()
            .map(|h| (h, GizmoMode::Scale));

        self.pick_best_mixed_handle(
            view_projection,
            viewport,
            origin,
            cursor,
            axes,
            size_length_world,
            true,
            rotate,
            scale,
            translate,
        )
    }

    pub(super) fn pick_operation_mask_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes_flipped: [Vec3; 3],
        axes_raw: [Vec3; 3],
        size_length_world: f32,
        targets: &[GizmoTarget3d],
    ) -> Option<(PickHit, GizmoMode)> {
        let ops = self.effective_ops();

        let translate_enabled = ops.intersects(GizmoOps::translate_all());
        let translate = translate_enabled
            .then(|| {
                self.pick_translate_handle(
                    view_projection,
                    viewport,
                    origin,
                    cursor,
                    axes_flipped,
                    size_length_world,
                    ops.contains(GizmoOps::translate_axis()),
                    ops.contains(GizmoOps::translate_plane()),
                    ops.contains(GizmoOps::translate_view()),
                    ops.contains(GizmoOps::translate_depth()),
                )
            })
            .flatten()
            .map(|h| (h, GizmoMode::Translate));

        let rotate_enabled = ops.intersects(GizmoOps::rotate_all());
        let rotate = rotate_enabled
            .then(|| {
                self.pick_rotate_axis(
                    view_projection,
                    viewport,
                    origin,
                    cursor,
                    axes_flipped,
                    size_length_world,
                )
            })
            .flatten()
            .map(|h| (h, GizmoMode::Rotate));

        let scale_enabled = ops.intersects(GizmoOps::scale_all());
        let scale_pickers_enabled = ops.intersects(
            GizmoOps::scale_axis() | GizmoOps::scale_plane() | GizmoOps::scale_uniform(),
        );
        let bounds_enabled = ops.contains(GizmoOps::scale_bounds());

        let scale = scale_enabled
            .then(|| {
                let scale = scale_pickers_enabled
                    .then(|| {
                        self.pick_scale_handle(
                            view_projection,
                            viewport,
                            origin,
                            cursor,
                            axes_flipped,
                            size_length_world,
                            ops.contains(GizmoOps::scale_axis()),
                            ops.contains(GizmoOps::scale_uniform()),
                            ops.contains(GizmoOps::scale_plane()),
                        )
                    })
                    .flatten()
                    .map(|h| (h, 1usize));

                if let Some((hit, _)) = scale
                    && hit.handle == ScaleHandle::Uniform.id()
                {
                    return Some((hit, GizmoMode::Scale));
                }

                let bounds = bounds_enabled
                    .then(|| {
                        self.pick_bounds_handle(
                            view_projection,
                            viewport,
                            origin,
                            cursor,
                            axes_raw,
                            size_length_world,
                            targets,
                        )
                    })
                    .flatten()
                    .map(|h| (h, 0usize));

                // Bounds handles are explicit solid affordances. If the cursor is inside a bounds
                // handle, it should win over other scaling candidates that may overlap in
                // projection (axis end boxes, plane edges, etc).
                if let Some((hit, _)) = bounds
                    && hit.score <= self.config.pick_policy.bounds_inside_score_max
                {
                    return Some((hit, GizmoMode::Scale));
                }

                let mut best: Option<(PickHit, usize)> = None;
                let mut consider = |cand: Option<(PickHit, usize)>| {
                    let Some((hit, pri)) = cand else {
                        return;
                    };
                    if !hit.score.is_finite() {
                        return;
                    }
                    match best {
                        Some((best_hit, best_pri)) => {
                            if hit.score < best_hit.score
                                || (hit.score == best_hit.score && pri < best_pri)
                            {
                                best = Some((hit, pri));
                            }
                        }
                        None => best = Some((hit, pri)),
                    }
                };

                consider(bounds);
                consider(scale);
                best.map(|(h, _)| (h, GizmoMode::Scale))
            })
            .flatten();

        self.pick_best_mixed_handle(
            view_projection,
            viewport,
            origin,
            cursor,
            axes_flipped,
            size_length_world,
            ops.contains(GizmoOps::translate_axis()),
            rotate,
            scale,
            translate,
        )
    }

    pub(super) fn pick_scale_or_bounds_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes_flipped: [Vec3; 3],
        axes_raw: [Vec3; 3],
        size_length_world: f32,
        targets: &[GizmoTarget3d],
    ) -> Option<PickHit> {
        let scale = self
            .pick_scale_handle(
                view_projection,
                viewport,
                origin,
                cursor,
                axes_flipped,
                size_length_world,
                true,
                true,
                true,
            )
            .map(|h| (h, 1usize));

        if let Some((hit, _)) = scale
            && hit.handle == ScaleHandle::Uniform.id()
        {
            return Some(hit);
        }

        let bounds = self
            .config
            .show_bounds
            .then(|| {
                self.pick_bounds_handle(
                    view_projection,
                    viewport,
                    origin,
                    cursor,
                    axes_raw,
                    size_length_world,
                    targets,
                )
            })
            .flatten()
            .map(|h| (h, 0usize));

        // Bounds handles are explicit solid affordances. If the cursor is inside a bounds handle,
        // it should win over axis end-box scaling that may overlap in projection.
        if let Some((hit, _)) = bounds
            && hit.score <= self.config.pick_policy.bounds_inside_score_max
        {
            return Some(hit);
        }

        let mut best: Option<(PickHit, usize)> = None;
        let mut consider = |cand: Option<(PickHit, usize)>| {
            let Some((hit, pri)) = cand else {
                return;
            };
            if !hit.score.is_finite() {
                return;
            }
            match best {
                Some((best_hit, best_pri)) => {
                    if hit.score < best_hit.score || (hit.score == best_hit.score && pri < best_pri)
                    {
                        best = Some((hit, pri));
                    }
                }
                None => best = Some((hit, pri)),
            }
        };

        consider(bounds);
        consider(scale);
        best.map(|(h, _)| h)
    }

    pub(super) fn pick_bounds_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes_raw: [Vec3; 3],
        size_length_world: f32,
        targets: &[GizmoTarget3d],
    ) -> Option<PickHit> {
        if targets.is_empty() {
            return None;
        }
        let basis = [
            axes_raw[0].normalize_or_zero(),
            axes_raw[1].normalize_or_zero(),
            axes_raw[2].normalize_or_zero(),
        ];
        if basis.iter().any(|v| v.length_squared() == 0.0) {
            return None;
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

        let view_dir =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?;
        let (u, v) = plane_basis(view_dir);

        let handle_half_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.bounds_handle_size_px.max(1.0),
        )
        .unwrap_or(1.0)
        .max(1e-6)
            * 0.5;

        let mut best: Option<PickHit> = None;
        let mut consider = |handle: HandleId, score: f32| {
            if !score.is_finite() {
                return;
            }
            match best {
                Some(best) if score >= best.score => {}
                _ => best = Some(PickHit { handle, score }),
            }
        };

        // Corner handles.
        for z_max in [false, true] {
            for y_max in [false, true] {
                for x_max in [false, true] {
                    let local = Vec3::new(
                        if x_max { max_local.x } else { min_local.x },
                        if y_max { max_local.y } else { min_local.y },
                        if z_max { max_local.z } else { min_local.z },
                    );
                    let world =
                        origin + basis[0] * local.x + basis[1] * local.y + basis[2] * local.z;
                    let quad_world = [
                        world + (-u - v) * handle_half_world,
                        world + (u - v) * handle_half_world,
                        world + (u + v) * handle_half_world,
                        world + (-u + v) * handle_half_world,
                    ];
                    let Some(p) = project_quad(
                        view_projection,
                        viewport,
                        quad_world,
                        self.config.depth_range,
                    ) else {
                        continue;
                    };
                    let quad = PickConvexQuad2d { points: p };
                    let inside = quad.contains(cursor);
                    let edge_d = quad.edge_distance(cursor);
                    let handle = Self::bounds_corner_id(x_max, y_max, z_max);
                    if inside {
                        consider(handle, 0.0);
                    } else if edge_d <= self.config.pick_radius_px {
                        consider(handle, edge_d);
                    }
                }
            }
        }

        // Face handles.
        for axis in 0..3 {
            for &max_side in &[false, true] {
                let mut local = center_local;
                local[axis] = if max_side {
                    max_local[axis]
                } else {
                    min_local[axis]
                };
                let world = origin + basis[0] * local.x + basis[1] * local.y + basis[2] * local.z;
                let quad_world = [
                    world + (-u - v) * handle_half_world,
                    world + (u - v) * handle_half_world,
                    world + (u + v) * handle_half_world,
                    world + (-u + v) * handle_half_world,
                ];
                let Some(p) = project_quad(
                    view_projection,
                    viewport,
                    quad_world,
                    self.config.depth_range,
                ) else {
                    continue;
                };
                let quad = PickConvexQuad2d { points: p };
                let inside = quad.contains(cursor);
                let edge_d = quad.edge_distance(cursor);
                let handle = Self::bounds_face_id(axis, max_side);
                if inside {
                    consider(handle, 0.25);
                } else if edge_d <= self.config.pick_radius_px {
                    consider(handle, edge_d + 0.8);
                }
            }
        }

        best
    }

    pub(super) fn pick_rotate_axis(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Option<PickHit> {
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

        let pv = self.state.part_visuals;
        let radius_world = size_length_world.max(0.0);

        let segments: usize = 64;
        let mut best_axis: Option<PickHit> = None;

        if include_axis {
            for &((axis_dir, handle), axis_index) in &[
                ((axes[0], RotateHandle::AxisX.id()), 0usize),
                ((axes[1], RotateHandle::AxisY.id()), 1usize),
                ((axes[2], RotateHandle::AxisZ.id()), 2usize),
            ] {
                if self.axis_is_masked(axis_index) {
                    continue;
                }
                let axis_dir = axis_dir.normalize_or_zero();
                if axis_dir.length_squared() == 0.0 {
                    continue;
                }
                let alpha =
                    self.rotate_ring_visibility_alpha(view_projection, viewport, origin, axis_dir);
                if alpha <= 0.01 {
                    continue;
                }
                let (u, v) = plane_basis(axis_dir);
                // Robust sampling: if the ring wraps partially behind the camera, we still want
                // the visible arc to remain pickable. Avoid requiring the first sample point to
                // be valid.
                let mut prev: Option<crate::math::ProjectedPoint> = None;
                for i in 0..=segments {
                    let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                    let world = origin + (u * t.cos() + v * t.sin()) * radius_world;
                    let Some(p) =
                        project_point(view_projection, viewport, world, self.config.depth_range)
                    else {
                        prev = None;
                        continue;
                    };
                    if !p.inside_clip {
                        prev = None;
                        continue;
                    }

                    if let Some(prev) = prev {
                        let r = self.config.pick_radius_px * alpha.sqrt();
                        if let Some(d) = (PickSegmentCapsule2d {
                            a: prev.screen,
                            b: p.screen,
                            radius: r,
                        })
                        .hit_distance(cursor)
                        {
                            match best_axis {
                                Some(best) if d >= best.score => {}
                                _ => best_axis = Some(PickHit { handle, score: d }),
                            }
                        }
                    }

                    prev = Some(p);
                }
            }
        }

        let mut view_hit: Option<PickHit> = None;
        if include_view
            && let Some(view_dir) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        {
            let axis_dir = view_dir.normalize_or_zero();
            if axis_dir.length_squared() > 0.0 {
                let (u, v) = plane_basis(axis_dir);
                let handle = RotateHandle::View.id();
                let r = (radius_world * pv.rotate_view_ring_radius_scale).max(1e-6);
                let mut prev: Option<crate::math::ProjectedPoint> = None;
                for i in 0..=segments {
                    let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                    let world = origin + (u * t.cos() + v * t.sin()) * r;
                    let Some(p) =
                        project_point(view_projection, viewport, world, self.config.depth_range)
                    else {
                        prev = None;
                        continue;
                    };
                    if !p.inside_clip {
                        prev = None;
                        continue;
                    }

                    if let Some(prev) = prev
                        && let Some(d) = (PickSegmentCapsule2d {
                            a: prev.screen,
                            b: p.screen,
                            radius: self.config.pick_radius_px,
                        })
                        .hit_distance(cursor)
                    {
                        match view_hit {
                            Some(best) if d >= best.score => {}
                            _ => view_hit = Some(PickHit { handle, score: d }),
                        };
                    }

                    prev = Some(p);
                }
            }
        }

        let ring_hit = match (best_axis, view_hit) {
            (Some(axis), Some(view)) => {
                // View ring is an outer, always-on-top affordance. Make it slightly easier to hit
                // while preventing it from stealing clearly-intended axis ring drags.
                //
                // Rule of thumb:
                // - If the cursor is close to an axis ring (strong intent), axis wins.
                // - Otherwise the view ring can win only if it is meaningfully closer.
                let axis_strong = axis.score <= self.config.pick_radius_px.max(1.0) * 0.35;
                let view_score = (view.score - 0.15).max(0.0);
                if !axis_strong && view_score + 0.75 < axis.score {
                    Some(PickHit {
                        handle: view.handle,
                        score: view_score,
                    })
                } else {
                    Some(axis)
                }
            }
            (Some(axis), None) => Some(axis),
            (None, Some(view)) => Some(view),
            (None, None) => None,
        };
        if ring_hit.is_some() {
            return ring_hit;
        }

        if include_arcball {
            let center = project_point(view_projection, viewport, origin, self.config.depth_range)?;
            let r = match self.config.size_policy {
                GizmoSizePolicy::ConstantPixels => {
                    self.config.size_px * self.config.arcball_radius_scale
                }
                GizmoSizePolicy::PixelsClampedBySelectionBounds { .. }
                | GizmoSizePolicy::SelectionBounds { .. } => {
                    let r_world = (radius_world * self.config.arcball_radius_scale).max(1e-6);
                    // The arcball circle is camera-facing, so any in-plane direction yields a
                    // stable projected radius.
                    let (u, _) = view_dir_at_origin(
                        view_projection,
                        viewport,
                        origin,
                        self.config.depth_range,
                    )
                    .map(plane_basis)
                    .unwrap_or((Vec3::X, Vec3::Y));
                    axis_segment_len_px(
                        view_projection,
                        viewport,
                        origin,
                        self.config.depth_range,
                        u,
                        r_world,
                    )
                    .unwrap_or(0.0)
                }
            }
            .max(self.config.pick_radius_px.max(6.0));
            if let Some(d) = (PickCircle2d {
                center: center.screen,
                radius: r,
            })
            .hit_distance(cursor)
            {
                return Some(PickHit {
                    handle: RotateHandle::Arcball.id(),
                    score: 10.0 + (d / r.max(1.0)),
                });
            }
        }

        None
    }
}

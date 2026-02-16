use super::*;

impl Gizmo {
    pub fn begin_drag_with_handle(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
        active_handle: HandleId,
    ) -> Option<GizmoUpdate> {
        if targets.is_empty() {
            self.state.hovered = None;
            self.state.active = None;
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
        let cursor_ray = ray_from_screen(
            view_projection,
            viewport,
            input.cursor_px,
            self.config.depth_range,
        )?;

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

        let begin = match handle_group(active_handle) {
            HANDLE_GROUP_TRANSLATE => {
                self.state.hovered_kind = Some(GizmoMode::Translate);
                self.begin_translate_drag(
                    view_projection,
                    viewport,
                    input,
                    targets,
                    cursor_ray,
                    origin,
                    active_handle,
                    axes,
                )
            }
            HANDLE_GROUP_ROTATE => {
                self.state.hovered_kind = Some(GizmoMode::Rotate);
                self.begin_rotate_drag(
                    view_projection,
                    viewport,
                    input,
                    targets,
                    cursor_ray,
                    origin,
                    active_handle,
                    axes,
                    size_length_world,
                )
            }
            HANDLE_GROUP_SCALE => {
                self.state.hovered_kind = Some(GizmoMode::Scale);
                if self.config.show_bounds {
                    if let Some(bounds_handle) = Self::bounds_handle_from_id(active_handle) {
                        let origin_z01 =
                            origin_z01(view_projection, viewport, origin, self.config.depth_range)?;
                        self.begin_bounds_drag(
                            view_projection,
                            viewport,
                            input,
                            targets,
                            cursor_ray,
                            origin,
                            origin_z01,
                            size_length_world,
                            bounds_handle,
                            active_handle,
                            axes_raw,
                        )
                    } else {
                        self.begin_scale_drag(
                            view_projection,
                            viewport,
                            input,
                            targets,
                            cursor_ray,
                            origin,
                            active_handle,
                            axes,
                            size_length_world,
                        )
                    }
                } else {
                    self.begin_scale_drag(
                        view_projection,
                        viewport,
                        input,
                        targets,
                        cursor_ray,
                        origin,
                        active_handle,
                        axes,
                        size_length_world,
                    )
                }
            }
            _ => None,
        };

        if self.config.drag_start_threshold_px > 0.0 {
            let _ = begin;
            None
        } else {
            begin
        }
    }

    pub fn update(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> Option<GizmoUpdate> {
        if targets.is_empty() {
            self.state.hovered = None;
            self.state.active = None;
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
        let cursor_ray = ray_from_screen(
            view_projection,
            viewport,
            input.cursor_px,
            self.config.depth_range,
        )?;

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
        let mut hovered: Option<HandleId> = None;
        let mut hovered_kind: Option<GizmoMode> = None;
        if self.state.active.is_none() && input.hovered {
            let pick = if self.config.operation_mask.is_some() {
                self.pick_operation_mask_handle(
                    view_projection,
                    viewport,
                    origin,
                    input.cursor_px,
                    axes,
                    axes_raw,
                    size_length_world,
                    targets,
                )
            } else {
                match self.config.mode {
                    GizmoMode::Translate => self
                        .pick_translate_handle(
                            view_projection,
                            viewport,
                            origin,
                            input.cursor_px,
                            axes,
                            size_length_world,
                            true,
                            true,
                            true,
                            true,
                        )
                        .map(|h| (h, GizmoMode::Translate)),
                    GizmoMode::Rotate => self
                        .pick_rotate_axis(
                            view_projection,
                            viewport,
                            origin,
                            input.cursor_px,
                            axes,
                            size_length_world,
                        )
                        .map(|h| (h, GizmoMode::Rotate)),
                    GizmoMode::Scale => self
                        .pick_scale_or_bounds_handle(
                            view_projection,
                            viewport,
                            origin,
                            input.cursor_px,
                            axes,
                            axes_raw,
                            size_length_world,
                            targets,
                        )
                        .map(|h| (h, GizmoMode::Scale)),
                    GizmoMode::Universal => self.pick_universal_handle(
                        view_projection,
                        viewport,
                        origin,
                        input.cursor_px,
                        axes,
                        size_length_world,
                    ),
                }
            };

            if let Some((h, kind)) = pick {
                hovered = Some(h.handle);
                hovered_kind = Some(kind);
            }
        }
        self.state.hovered = hovered;
        self.state.hovered_kind = hovered_kind;

        if self.state.active.is_none() {
            if input.drag_started
                && let Some(h) = hovered
            {
                let begin = if self.config.operation_mask.is_some() {
                    match self.state.hovered_kind {
                        Some(GizmoMode::Translate) => self.begin_translate_drag(
                            view_projection,
                            viewport,
                            input,
                            targets,
                            cursor_ray,
                            origin,
                            h,
                            axes,
                        ),
                        Some(GizmoMode::Rotate) => self.begin_rotate_drag(
                            view_projection,
                            viewport,
                            input,
                            targets,
                            cursor_ray,
                            origin,
                            h,
                            axes,
                            size_length_world,
                        ),
                        Some(GizmoMode::Scale) => {
                            if let Some(bounds_handle) = Self::bounds_handle_from_id(h) {
                                let origin_z01 = origin_z01(
                                    view_projection,
                                    viewport,
                                    origin,
                                    self.config.depth_range,
                                )?;
                                self.begin_bounds_drag(
                                    view_projection,
                                    viewport,
                                    input,
                                    targets,
                                    cursor_ray,
                                    origin,
                                    origin_z01,
                                    size_length_world,
                                    bounds_handle,
                                    h,
                                    axes_raw,
                                )
                            } else {
                                self.begin_scale_drag(
                                    view_projection,
                                    viewport,
                                    input,
                                    targets,
                                    cursor_ray,
                                    origin,
                                    h,
                                    axes,
                                    size_length_world,
                                )
                            }
                        }
                        _ => None,
                    }
                } else {
                    match self.config.mode {
                        GizmoMode::Translate => self.begin_translate_drag(
                            view_projection,
                            viewport,
                            input,
                            targets,
                            cursor_ray,
                            origin,
                            h,
                            axes,
                        ),
                        GizmoMode::Rotate => self.begin_rotate_drag(
                            view_projection,
                            viewport,
                            input,
                            targets,
                            cursor_ray,
                            origin,
                            h,
                            axes,
                            size_length_world,
                        ),
                        GizmoMode::Scale => {
                            if self.config.show_bounds {
                                if let Some(bounds_handle) = Self::bounds_handle_from_id(h) {
                                    let origin_z01 = origin_z01(
                                        view_projection,
                                        viewport,
                                        origin,
                                        self.config.depth_range,
                                    )?;
                                    self.begin_bounds_drag(
                                        view_projection,
                                        viewport,
                                        input,
                                        targets,
                                        cursor_ray,
                                        origin,
                                        origin_z01,
                                        size_length_world,
                                        bounds_handle,
                                        h,
                                        axes_raw,
                                    )
                                } else {
                                    self.begin_scale_drag(
                                        view_projection,
                                        viewport,
                                        input,
                                        targets,
                                        cursor_ray,
                                        origin,
                                        h,
                                        axes,
                                        size_length_world,
                                    )
                                }
                            } else {
                                self.begin_scale_drag(
                                    view_projection,
                                    viewport,
                                    input,
                                    targets,
                                    cursor_ray,
                                    origin,
                                    h,
                                    axes,
                                    size_length_world,
                                )
                            }
                        }
                        GizmoMode::Universal => match self.state.hovered_kind {
                            Some(GizmoMode::Translate) => self.begin_translate_drag(
                                view_projection,
                                viewport,
                                input,
                                targets,
                                cursor_ray,
                                origin,
                                h,
                                axes,
                            ),
                            Some(GizmoMode::Rotate) => self.begin_rotate_drag(
                                view_projection,
                                viewport,
                                input,
                                targets,
                                cursor_ray,
                                origin,
                                h,
                                axes,
                                size_length_world,
                            ),
                            Some(GizmoMode::Scale) => self.begin_scale_drag(
                                view_projection,
                                viewport,
                                input,
                                targets,
                                cursor_ray,
                                origin,
                                h,
                                axes,
                                size_length_world,
                            ),
                            _ => None,
                        },
                    }
                };

                // If a drag threshold is configured, we arm the interaction on pointer down
                // but only emit the `Begin` phase once the pointer has actually moved.
                if self.config.drag_start_threshold_px > 0.0 {
                    return None;
                }
                return begin;
            }
            return None;
        }

        let active = self.state.active.unwrap();
        let precision = input_precision(input.precision);

        match self.state.drag_mode {
            GizmoMode::Translate => {
                let axis_dir = self.state.drag_axis_dir;

                if input.cancel {
                    let total = if self.state.drag_translate_is_plane {
                        self.state.drag_translate_u * self.state.drag_total_plane_applied.x
                            + self.state.drag_translate_v * self.state.drag_total_plane_applied.y
                    } else {
                        self.state.drag_total_axis_applied * axis_dir
                    };
                    self.state.active = None;
                    self.state.drag_start_targets.clear();
                    return Some(GizmoUpdate {
                        phase: GizmoPhase::Cancel,
                        active,
                        result: GizmoResult::Translation {
                            delta: Vec3::ZERO,
                            total,
                        },
                        updated_targets: targets.to_vec(),
                        custom_edits: Vec::new(),
                    });
                }

                if input.dragging {
                    self.state.drag_snap = input.snap;
                    let started_this_call = if !self.state.drag_has_started {
                        let threshold = self.config.drag_start_threshold_px.max(0.0);
                        if threshold > 0.0
                            && (input.cursor_px - self.state.drag_start_cursor_px).length()
                                < threshold
                        {
                            return None;
                        }
                        self.state.drag_has_started = true;
                        true
                    } else {
                        false
                    };
                    let (delta, total) = if self.state.drag_translate_is_dolly {
                        // Dolly (depth) translation is screen-delta driven rather than ray-plane driven:
                        // translating along the view direction has no stable plane intersection anchor.
                        //
                        // Invariant: returning the cursor to `drag_start_cursor_px` returns total close to 0.
                        let dy = input.cursor_px.y - self.state.drag_start_cursor_px.y;
                        if !dy.is_finite() {
                            return None;
                        }
                        self.state.drag_total_axis_raw =
                            dy * self.state.drag_translate_dolly_world_per_px * precision;
                        let desired_total = if input.snap {
                            self.config
                                .translate_snap_step
                                .filter(|s| s.is_finite() && *s > 0.0)
                                .map(|step| (self.state.drag_total_axis_raw / step).round() * step)
                                .unwrap_or(self.state.drag_total_axis_raw)
                        } else {
                            self.state.drag_total_axis_raw
                        };
                        let delta_axis = desired_total - self.state.drag_total_axis_applied;
                        self.state.drag_total_axis_applied = desired_total;
                        (delta_axis * axis_dir, desired_total * axis_dir)
                    } else {
                        let hit_world = ray_plane_intersect(
                            cursor_ray,
                            self.state.drag_origin,
                            self.state.drag_plane_normal,
                        )
                        .filter(|p| p.is_finite())
                        .unwrap_or_else(|| {
                            unproject_point(
                                view_projection,
                                viewport,
                                input.cursor_px,
                                self.config.depth_range,
                                self.state.drag_origin_z01,
                            )
                            .unwrap_or(self.state.drag_origin)
                        });

                        let diff_world = hit_world - self.state.drag_start_hit_world;
                        self.state.drag_prev_hit_world = hit_world;

                        if self.state.drag_translate_is_plane {
                            let u = self.state.drag_translate_u;
                            let v = self.state.drag_translate_v;
                            let raw = Vec2::new(diff_world.dot(u), diff_world.dot(v));
                            let delta_raw = raw - self.state.drag_translate_prev_plane_raw;
                            self.state.drag_translate_prev_plane_raw = raw;
                            self.state.drag_total_plane_raw += delta_raw * precision;
                            let desired_total = if input.snap {
                                self.config
                                    .translate_snap_step
                                    .filter(|s| s.is_finite() && *s > 0.0)
                                    .map(|step| {
                                        Vec2::new(
                                            (self.state.drag_total_plane_raw.x / step).round()
                                                * step,
                                            (self.state.drag_total_plane_raw.y / step).round()
                                                * step,
                                        )
                                    })
                                    .unwrap_or(self.state.drag_total_plane_raw)
                            } else {
                                self.state.drag_total_plane_raw
                            };
                            let delta_plane = desired_total - self.state.drag_total_plane_applied;
                            self.state.drag_total_plane_applied = desired_total;
                            let delta = u * delta_plane.x + v * delta_plane.y;
                            let total = u * desired_total.x + v * desired_total.y;
                            (delta, total)
                        } else {
                            let raw = diff_world.dot(axis_dir);
                            let delta_raw = raw - self.state.drag_translate_prev_axis_raw;
                            self.state.drag_translate_prev_axis_raw = raw;
                            self.state.drag_total_axis_raw += delta_raw * precision;
                            let desired_total = if input.snap {
                                self.config
                                    .translate_snap_step
                                    .filter(|s| s.is_finite() && *s > 0.0)
                                    .map(|step| {
                                        (self.state.drag_total_axis_raw / step).round() * step
                                    })
                                    .unwrap_or(self.state.drag_total_axis_raw)
                            } else {
                                self.state.drag_total_axis_raw
                            };
                            let delta_axis = desired_total - self.state.drag_total_axis_applied;
                            self.state.drag_total_axis_applied = desired_total;
                            (delta_axis * axis_dir, desired_total * axis_dir)
                        }
                    };
                    let updated_targets = self
                        .state
                        .drag_start_targets
                        .iter()
                        .map(|t| GizmoTarget3d {
                            id: t.id,
                            transform: Transform3d {
                                translation: t.transform.translation + total,
                                ..t.transform
                            },
                            local_bounds: t.local_bounds,
                        })
                        .collect::<Vec<_>>();
                    return Some(GizmoUpdate {
                        phase: if started_this_call {
                            GizmoPhase::Begin
                        } else {
                            GizmoPhase::Update
                        },
                        active,
                        result: GizmoResult::Translation { delta, total },
                        updated_targets,
                        custom_edits: Vec::new(),
                    });
                }

                if !self.state.drag_has_started {
                    self.state.active = None;
                    self.state.drag_start_targets.clear();
                    return None;
                }

                // Pointer released: end the interaction. The host is responsible for undo/redo boundaries.
                let total = if self.state.drag_translate_is_plane {
                    self.state.drag_translate_u * self.state.drag_total_plane_applied.x
                        + self.state.drag_translate_v * self.state.drag_total_plane_applied.y
                } else {
                    self.state.drag_total_axis_applied * axis_dir
                };
                self.state.active = None;
                self.state.drag_start_targets.clear();
                Some(GizmoUpdate {
                    phase: GizmoPhase::Commit,
                    active,
                    result: GizmoResult::Translation {
                        delta: Vec3::ZERO,
                        total,
                    },
                    updated_targets: targets.to_vec(),
                    custom_edits: Vec::new(),
                })
            }
            GizmoMode::Rotate => {
                if self.state.drag_rotate_is_arcball {
                    let total = self.state.drag_total_arcball_applied;
                    if input.cancel {
                        self.state.active = None;
                        self.state.drag_rotate_is_arcball = false;
                        self.state.drag_start_targets.clear();
                        return Some(GizmoUpdate {
                            phase: GizmoPhase::Cancel,
                            active,
                            result: GizmoResult::Arcball {
                                delta: Quat::IDENTITY,
                                total,
                            },
                            updated_targets: targets.to_vec(),
                            custom_edits: Vec::new(),
                        });
                    }

                    if input.dragging {
                        self.state.drag_snap = input.snap;
                        let started_this_call = if !self.state.drag_has_started {
                            let threshold = self.config.drag_start_threshold_px.max(0.0);
                            if threshold > 0.0
                                && (input.cursor_px - self.state.drag_start_cursor_px).length()
                                    < threshold
                            {
                                return None;
                            }
                            self.state.drag_has_started = true;
                            true
                        } else {
                            false
                        };

                        let current = self.arcball_vector_world(input.cursor_px)?;
                        let prev = self.state.drag_arcball_prev_vec.normalize_or_zero();
                        self.state.drag_arcball_prev_vec = current;
                        if prev.length_squared() == 0.0 {
                            return None;
                        }

                        let mut delta_q = Quat::from_rotation_arc(prev, current);
                        let mut angle_scale = precision;
                        if let Some(speed) = self
                            .config
                            .arcball_rotation_speed
                            .is_finite()
                            .then_some(self.config.arcball_rotation_speed)
                            .filter(|s| *s > 0.0 && (*s - 1.0).abs() > 1e-3)
                        {
                            angle_scale *= speed;
                        }
                        if (angle_scale - 1.0).abs() > 1e-3 {
                            let (axis, angle) = quat_axis_angle(delta_q);
                            delta_q = Quat::from_axis_angle(axis, angle * angle_scale);
                        }

                        self.state.drag_total_arcball_raw =
                            (delta_q * self.state.drag_total_arcball_raw).normalize();

                        let desired_total = if input.snap {
                            self.config
                                .rotate_snap_step_radians
                                .filter(|s| s.is_finite() && *s > 0.0)
                                .map(|step| {
                                    snap_quat_to_angle_step(self.state.drag_total_arcball_raw, step)
                                })
                                .unwrap_or(self.state.drag_total_arcball_raw)
                        } else {
                            self.state.drag_total_arcball_raw
                        };

                        let delta_apply = (desired_total
                            * self.state.drag_total_arcball_applied.inverse())
                        .normalize();
                        self.state.drag_total_arcball_applied = desired_total;

                        let updated_targets = self
                            .state
                            .drag_start_targets
                            .iter()
                            .map(|t| GizmoTarget3d {
                                id: t.id,
                                transform: Transform3d {
                                    translation: self.state.drag_origin
                                        + desired_total
                                            * (t.transform.translation - self.state.drag_origin),
                                    rotation: (desired_total * t.transform.rotation).normalize(),
                                    ..t.transform
                                },
                                local_bounds: t.local_bounds,
                            })
                            .collect::<Vec<_>>();
                        return Some(GizmoUpdate {
                            phase: if started_this_call {
                                GizmoPhase::Begin
                            } else {
                                GizmoPhase::Update
                            },
                            active,
                            result: GizmoResult::Arcball {
                                delta: delta_apply,
                                total: desired_total,
                            },
                            updated_targets,
                            custom_edits: Vec::new(),
                        });
                    }

                    if !self.state.drag_has_started {
                        self.state.active = None;
                        self.state.drag_rotate_is_arcball = false;
                        self.state.drag_start_targets.clear();
                        return None;
                    }

                    self.state.active = None;
                    self.state.drag_rotate_is_arcball = false;
                    self.state.drag_start_targets.clear();
                    Some(GizmoUpdate {
                        phase: GizmoPhase::Commit,
                        active,
                        result: GizmoResult::Arcball {
                            delta: Quat::IDENTITY,
                            total,
                        },
                        updated_targets: targets.to_vec(),
                        custom_edits: Vec::new(),
                    })
                } else {
                    let axis_dir = self.state.drag_axis_dir.normalize_or_zero();
                    if axis_dir.length_squared() == 0.0 {
                        self.state.active = None;
                        self.state.drag_start_targets.clear();
                        return None;
                    }

                    if input.cancel {
                        let total = self.state.drag_total_angle_applied;
                        self.state.active = None;
                        self.state.drag_start_targets.clear();
                        return Some(GizmoUpdate {
                            phase: GizmoPhase::Cancel,
                            active,
                            result: GizmoResult::Rotation {
                                axis: axis_dir,
                                delta_radians: 0.0,
                                total_radians: total,
                            },
                            updated_targets: targets.to_vec(),
                            custom_edits: Vec::new(),
                        });
                    }

                    if input.dragging {
                        self.state.drag_snap = input.snap;
                        let started_this_call = if !self.state.drag_has_started {
                            let threshold = self.config.drag_start_threshold_px.max(0.0);
                            if threshold > 0.0
                                && (input.cursor_px - self.state.drag_start_cursor_px).length()
                                    < threshold
                            {
                                return None;
                            }
                            self.state.drag_has_started = true;
                            true
                        } else {
                            false
                        };
                        let hit_world = ray_plane_intersect(
                            cursor_ray,
                            self.state.drag_origin,
                            self.state.drag_plane_normal,
                        )
                        .filter(|p| p.is_finite())
                        .unwrap_or_else(|| {
                            unproject_point(
                                view_projection,
                                viewport,
                                input.cursor_px,
                                self.config.depth_range,
                                self.state.drag_origin_z01,
                            )
                            .unwrap_or(self.state.drag_origin)
                        });

                        let mut angle = angle_on_plane(
                            self.state.drag_origin,
                            hit_world,
                            axis_dir,
                            self.state.drag_basis_u,
                            self.state.drag_basis_v,
                        )?;
                        angle *= self.handedness_rotation_sign();

                        let delta_angle =
                            wrap_angle(angle - self.state.drag_prev_angle) * precision;
                        self.state.drag_prev_angle = angle;
                        self.state.drag_total_angle_raw += delta_angle;

                        let desired_total = if input.snap {
                            self.config
                                .rotate_snap_step_radians
                                .filter(|s| s.is_finite() && *s > 0.0)
                                .map(|step| (self.state.drag_total_angle_raw / step).round() * step)
                                .unwrap_or(self.state.drag_total_angle_raw)
                        } else {
                            self.state.drag_total_angle_raw
                        };
                        let delta_apply = desired_total - self.state.drag_total_angle_applied;
                        self.state.drag_total_angle_applied = desired_total;

                        let total_q = Quat::from_axis_angle(axis_dir, desired_total);
                        let updated_targets = self
                            .state
                            .drag_start_targets
                            .iter()
                            .map(|t| GizmoTarget3d {
                                id: t.id,
                                transform: Transform3d {
                                    translation: self.state.drag_origin
                                        + total_q
                                            * (t.transform.translation - self.state.drag_origin),
                                    rotation: (total_q * t.transform.rotation).normalize(),
                                    ..t.transform
                                },
                                local_bounds: t.local_bounds,
                            })
                            .collect::<Vec<_>>();
                        return Some(GizmoUpdate {
                            phase: if started_this_call {
                                GizmoPhase::Begin
                            } else {
                                GizmoPhase::Update
                            },
                            active,
                            result: GizmoResult::Rotation {
                                axis: axis_dir,
                                delta_radians: delta_apply,
                                total_radians: desired_total,
                            },
                            updated_targets,
                            custom_edits: Vec::new(),
                        });
                    }

                    if !self.state.drag_has_started {
                        self.state.active = None;
                        self.state.drag_start_targets.clear();
                        return None;
                    }

                    let total = self.state.drag_total_angle_applied;
                    self.state.active = None;
                    self.state.drag_start_targets.clear();
                    Some(GizmoUpdate {
                        phase: GizmoPhase::Commit,
                        active,
                        result: GizmoResult::Rotation {
                            axis: axis_dir,
                            delta_radians: 0.0,
                            total_radians: total,
                        },
                        updated_targets: targets.to_vec(),
                        custom_edits: Vec::new(),
                    })
                }
            }
            GizmoMode::Scale => {
                let _ = (view_projection, viewport);
                let length_world = self.state.drag_size_length_world.max(1e-6);

                let total_vec = |total_factor: f32| -> Vec3 {
                    if self.state.drag_scale_is_uniform {
                        Vec3::splat(total_factor)
                    } else if let Some(axis) = self.state.drag_scale_axis {
                        let mut v = Vec3::ONE;
                        v[axis] = total_factor;
                        v
                    } else {
                        Vec3::ONE
                    }
                };
                let total_plane_vec = |total_factors: Vec2| -> Vec3 {
                    let Some((a, b)) = self.state.drag_scale_plane_axes else {
                        return Vec3::ONE;
                    };
                    let mut v = Vec3::ONE;
                    v[a] = total_factors.x;
                    v[b] = total_factors.y;
                    v
                };

                if input.cancel {
                    let total = if self.state.drag_scale_is_bounds {
                        self.state.drag_bounds_total_applied
                    } else if self.state.drag_scale_plane_axes.is_some() {
                        total_plane_vec(self.state.drag_total_scale_plane_applied)
                    } else {
                        total_vec(self.state.drag_total_scale_applied)
                    };
                    self.state.active = None;
                    self.state.drag_scale_is_bounds = false;
                    self.state.drag_start_targets.clear();
                    return Some(GizmoUpdate {
                        phase: GizmoPhase::Cancel,
                        active,
                        result: GizmoResult::Scale {
                            delta: Vec3::ONE,
                            total,
                        },
                        updated_targets: targets.to_vec(),
                        custom_edits: Vec::new(),
                    });
                }

                if input.dragging {
                    self.state.drag_snap = input.snap;
                    let started_this_call = if !self.state.drag_has_started {
                        let threshold = self.config.drag_start_threshold_px.max(0.0);
                        if threshold > 0.0
                            && (input.cursor_px - self.state.drag_start_cursor_px).length()
                                < threshold
                        {
                            return None;
                        }
                        self.state.drag_has_started = true;
                        true
                    } else {
                        false
                    };

                    if self.state.drag_scale_is_bounds {
                        let hit_world = ray_plane_intersect(
                            cursor_ray,
                            self.state.drag_origin,
                            self.state.drag_plane_normal,
                        )
                        .filter(|p| p.is_finite())
                        .unwrap_or_else(|| {
                            unproject_point(
                                view_projection,
                                viewport,
                                input.cursor_px,
                                self.config.depth_range,
                                self.state.drag_origin_z01,
                            )
                            .unwrap_or(self.state.drag_origin)
                        });

                        let basis = self.state.drag_bounds_basis;
                        let diff_world = hit_world - self.state.drag_start_hit_world;
                        self.state.drag_prev_hit_world = hit_world;

                        let raw_local = Vec3::new(
                            diff_world.dot(basis[0]),
                            diff_world.dot(basis[1]),
                            diff_world.dot(basis[2]),
                        );
                        let delta_local =
                            (raw_local - self.state.drag_bounds_prev_local_raw) * precision;
                        self.state.drag_bounds_prev_local_raw = raw_local;

                        for i in 0..3 {
                            if self.state.drag_bounds_axes_mask[i] {
                                let sign = self.state.drag_bounds_axis_sign[i];
                                let extent = self.state.drag_bounds_start_extent[i].max(1e-6);
                                self.state.drag_bounds_total_raw[i] +=
                                    (delta_local[i] * sign) / extent;
                            }
                        }

                        let mut desired = Vec3::ONE;
                        for i in 0..3 {
                            if self.state.drag_bounds_axes_mask[i] {
                                let mut factor =
                                    (1.0 + self.state.drag_bounds_total_raw[i]).max(0.01);
                                if input.snap {
                                    if let Some(steps) =
                                        self.config.bounds_snap_step.filter(|v| v.is_finite())
                                    {
                                        let step = steps[i];
                                        if step.is_finite() && step > 0.0 {
                                            factor = snap_bounds_extent_factor(
                                                self.state.drag_bounds_start_extent[i].max(1e-6),
                                                factor,
                                                step,
                                            );
                                        }
                                    } else if let Some(step) = self
                                        .config
                                        .scale_snap_step
                                        .filter(|s| s.is_finite() && *s > 0.0)
                                    {
                                        factor = 1.0 + ((factor - 1.0) / step).round() * step;
                                        factor = factor.max(0.01);
                                    }
                                }
                                desired[i] = factor;
                            }
                        }

                        let delta = Vec3::new(
                            desired.x / self.state.drag_bounds_total_applied.x,
                            desired.y / self.state.drag_bounds_total_applied.y,
                            desired.z / self.state.drag_bounds_total_applied.z,
                        );
                        self.state.drag_bounds_total_applied = desired;

                        let updated_targets = self
                            .state
                            .drag_start_targets
                            .iter()
                            .map(|t| {
                                let origin = self.state.drag_origin;
                                let basis = self.state.drag_bounds_basis;
                                let anchor = self.state.drag_bounds_anchor_local;

                                let p = t.transform.translation - origin;
                                let coords =
                                    Vec3::new(p.dot(basis[0]), p.dot(basis[1]), p.dot(basis[2]));
                                let mut next = coords;
                                for i in 0..3 {
                                    if self.state.drag_bounds_axes_mask[i] {
                                        next[i] = anchor[i] + (coords[i] - anchor[i]) * desired[i];
                                    }
                                }
                                let translation = origin
                                    + basis[0] * next.x
                                    + basis[1] * next.y
                                    + basis[2] * next.z;

                                let mut scale = t.transform.scale;
                                for i in 0..3 {
                                    if self.state.drag_bounds_axes_mask[i] {
                                        scale[i] = (scale[i] * desired[i]).max(1e-4);
                                    }
                                }

                                GizmoTarget3d {
                                    id: t.id,
                                    transform: Transform3d {
                                        translation,
                                        scale,
                                        ..t.transform
                                    },
                                    local_bounds: t.local_bounds,
                                }
                            })
                            .collect::<Vec<_>>();

                        return Some(GizmoUpdate {
                            phase: if started_this_call {
                                GizmoPhase::Begin
                            } else {
                                GizmoPhase::Update
                            },
                            active,
                            result: GizmoResult::Scale {
                                delta,
                                total: desired,
                            },
                            updated_targets,
                            custom_edits: Vec::new(),
                        });
                    }

                    let hit_world = ray_plane_intersect(
                        cursor_ray,
                        self.state.drag_origin,
                        self.state.drag_plane_normal,
                    )
                    .filter(|p| p.is_finite())
                    .unwrap_or_else(|| {
                        unproject_point(
                            view_projection,
                            viewport,
                            input.cursor_px,
                            self.config.depth_range,
                            self.state.drag_origin_z01,
                        )
                        .unwrap_or(self.state.drag_origin)
                    });

                    let diff_world = hit_world - self.state.drag_start_hit_world;
                    self.state.drag_prev_hit_world = hit_world;

                    if let Some((a, b)) = self.state.drag_scale_plane_axes {
                        let u_dir = self.state.drag_scale_plane_u.normalize_or_zero();
                        let v_dir = self.state.drag_scale_plane_v.normalize_or_zero();
                        if u_dir.length_squared() == 0.0 || v_dir.length_squared() == 0.0 {
                            return None;
                        }

                        let raw = Vec2::new(diff_world.dot(u_dir), diff_world.dot(v_dir));
                        let delta_raw = raw - self.state.drag_scale_prev_plane_raw;
                        self.state.drag_scale_prev_plane_raw = raw;
                        self.state.drag_total_scale_plane_raw += delta_raw * precision;

                        let delta_norm = self.state.drag_total_scale_plane_raw / length_world;
                        let mut desired = if input.snap {
                            self.config
                                .scale_snap_step
                                .filter(|s| s.is_finite() && *s > 0.0)
                                .map(|step| {
                                    Vec2::new(
                                        1.0 + (delta_norm.x / step).round() * step,
                                        1.0 + (delta_norm.y / step).round() * step,
                                    )
                                })
                                .unwrap_or(Vec2::ONE + delta_norm)
                        } else {
                            Vec2::ONE + delta_norm
                        };
                        desired.x = desired.x.max(0.01);
                        desired.y = desired.y.max(0.01);

                        let delta_factors = Vec2::new(
                            desired.x / self.state.drag_total_scale_plane_applied.x,
                            desired.y / self.state.drag_total_scale_plane_applied.y,
                        );
                        self.state.drag_total_scale_plane_applied = desired;

                        let mut delta = Vec3::ONE;
                        delta[a] = delta_factors.x;
                        delta[b] = delta_factors.y;

                        let total = total_plane_vec(desired);

                        let updated_targets = self
                            .state
                            .drag_start_targets
                            .iter()
                            .map(|t| {
                                let origin = self.state.drag_origin;
                                let offset = t.transform.translation - origin;
                                let comp_u = u_dir * offset.dot(u_dir);
                                let comp_v = v_dir * offset.dot(v_dir);
                                let translation = origin
                                    + (offset
                                        + comp_u * (desired.x - 1.0)
                                        + comp_v * (desired.y - 1.0));

                                let mut scale = t.transform.scale;
                                scale[a] = (scale[a] * desired.x).max(1e-4);
                                scale[b] = (scale[b] * desired.y).max(1e-4);

                                GizmoTarget3d {
                                    id: t.id,
                                    transform: Transform3d {
                                        translation,
                                        scale,
                                        ..t.transform
                                    },
                                    local_bounds: t.local_bounds,
                                }
                            })
                            .collect::<Vec<_>>();

                        return Some(GizmoUpdate {
                            phase: if started_this_call {
                                GizmoPhase::Begin
                            } else {
                                GizmoPhase::Update
                            },
                            active,
                            result: GizmoResult::Scale { delta, total },
                            updated_targets,
                            custom_edits: Vec::new(),
                        });
                    }

                    let scale_dir = self.state.drag_axis_dir.normalize_or_zero();
                    if scale_dir.length_squared() == 0.0 {
                        return None;
                    }
                    let raw = diff_world.dot(scale_dir);
                    let delta_raw = raw - self.state.drag_scale_prev_axis_raw;
                    self.state.drag_scale_prev_axis_raw = raw;
                    self.state.drag_total_scale_raw += delta_raw * precision;

                    let delta_norm = self.state.drag_total_scale_raw / length_world;
                    let mut desired_factor = if input.snap {
                        self.config
                            .scale_snap_step
                            .filter(|s| s.is_finite() && *s > 0.0)
                            .map(|step| 1.0 + (delta_norm / step).round() * step)
                            .unwrap_or(1.0 + delta_norm)
                    } else {
                        1.0 + delta_norm
                    };
                    desired_factor = desired_factor.max(0.01);

                    let delta_factor = desired_factor / self.state.drag_total_scale_applied;
                    self.state.drag_total_scale_applied = desired_factor;

                    let delta = if self.state.drag_scale_is_uniform {
                        Vec3::splat(delta_factor)
                    } else if let Some(axis) = self.state.drag_scale_axis {
                        let mut v = Vec3::ONE;
                        v[axis] = delta_factor;
                        v
                    } else {
                        Vec3::ONE
                    };
                    let total = total_vec(desired_factor);

                    let updated_targets = self
                        .state
                        .drag_start_targets
                        .iter()
                        .map(|t| {
                            let origin = self.state.drag_origin;
                            let offset = t.transform.translation - origin;
                            let axis_dir = self.state.drag_axis_dir.normalize_or_zero();
                            let translation = if self.state.drag_scale_is_uniform {
                                origin + offset * desired_factor
                            } else if axis_dir.length_squared() > 0.0 {
                                let component = axis_dir * offset.dot(axis_dir);
                                origin + (offset + component * (desired_factor - 1.0))
                            } else {
                                t.transform.translation
                            };

                            let mut scale = t.transform.scale;
                            if self.state.drag_scale_is_uniform {
                                scale *= desired_factor;
                            } else if let Some(axis) = self.state.drag_scale_axis {
                                scale[axis] = (scale[axis] * desired_factor).max(1e-4);
                            }
                            GizmoTarget3d {
                                id: t.id,
                                transform: Transform3d {
                                    translation,
                                    scale,
                                    ..t.transform
                                },
                                local_bounds: t.local_bounds,
                            }
                        })
                        .collect::<Vec<_>>();

                    return Some(GizmoUpdate {
                        phase: if started_this_call {
                            GizmoPhase::Begin
                        } else {
                            GizmoPhase::Update
                        },
                        active,
                        result: GizmoResult::Scale { delta, total },
                        updated_targets,
                        custom_edits: Vec::new(),
                    });
                }

                if !self.state.drag_has_started {
                    self.state.active = None;
                    self.state.drag_scale_is_bounds = false;
                    self.state.drag_start_targets.clear();
                    return None;
                }

                let total = if self.state.drag_scale_is_bounds {
                    self.state.drag_bounds_total_applied
                } else if self.state.drag_scale_plane_axes.is_some() {
                    total_plane_vec(self.state.drag_total_scale_plane_applied)
                } else {
                    total_vec(self.state.drag_total_scale_applied)
                };
                self.state.active = None;
                self.state.drag_scale_is_bounds = false;
                self.state.drag_start_targets.clear();
                Some(GizmoUpdate {
                    phase: GizmoPhase::Commit,
                    active,
                    result: GizmoResult::Scale {
                        delta: Vec3::ONE,
                        total,
                    },
                    updated_targets: targets.to_vec(),
                    custom_edits: Vec::new(),
                })
            }
            GizmoMode::Universal => {
                self.state.active = None;
                None
            }
        }
    }

    fn begin_translate_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        cursor_ray: Ray3d,
        origin: Vec3,
        active: HandleId,
        axes: [Vec3; 3],
    ) -> Option<GizmoUpdate> {
        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;
        let constraint = translate_constraint_for_handle(
            view_projection,
            viewport,
            self.config.depth_range,
            origin,
            active,
            axes,
        )?;

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Translate;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_total_axis_raw = 0.0;
        self.state.drag_total_axis_applied = 0.0;
        self.state.drag_total_plane_raw = Vec2::ZERO;
        self.state.drag_total_plane_applied = Vec2::ZERO;

        self.state.drag_start_targets = targets.to_vec();

        match constraint {
            TranslateConstraint::Axis { axis_dir } => {
                self.state.drag_translate_is_plane = false;
                self.state.drag_translate_is_dolly = false;
                self.state.drag_translate_dolly_world_per_px = 0.0;
                self.state.drag_axis_dir = axis_dir;
                let plane_normal = axis_drag_plane_normal(
                    view_projection,
                    viewport,
                    self.config.depth_range,
                    origin,
                    axis_dir,
                )?;
                self.state.drag_plane_normal = plane_normal;
            }
            TranslateConstraint::Plane { u, v, normal } => {
                self.state.drag_translate_is_plane = true;
                self.state.drag_translate_is_dolly = false;
                self.state.drag_translate_dolly_world_per_px = 0.0;
                self.state.drag_translate_u = u;
                self.state.drag_translate_v = v;
                self.state.drag_plane_normal = normal;
            }
            TranslateConstraint::Dolly { view_dir } => {
                let axis_dir = view_dir.normalize_or_zero();
                if axis_dir.length_squared() == 0.0 {
                    return None;
                }
                self.state.drag_translate_is_plane = false;
                self.state.drag_translate_is_dolly = true;
                self.state.drag_axis_dir = axis_dir;

                // A stable plane isn't required for dolly updates, but we still set a sane value so
                // fallback unprojection stays close to the origin depth when needed.
                self.state.drag_plane_normal = axis_dir;

                let world_per_px = axis_length_world(
                    view_projection,
                    viewport,
                    origin,
                    self.config.depth_range,
                    1.0,
                )
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or_else(|| {
                    let size_length_world =
                        self.size_length_world_or_one(view_projection, viewport, origin, targets);
                    let px = self.config.size_px.max(1.0);
                    let v = size_length_world / px;
                    if v.is_finite() && v > 0.0 { v } else { 1e-3 }
                });
                self.state.drag_translate_dolly_world_per_px = world_per_px;
            }
        }

        let start_hit_world = ray_plane_intersect(cursor_ray, origin, self.state.drag_plane_normal)
            .filter(|p| p.is_finite())
            .unwrap_or_else(|| {
                unproject_point(
                    view_projection,
                    viewport,
                    input.cursor_px,
                    self.config.depth_range,
                    origin_z01,
                )
                .unwrap_or(origin)
            });
        self.state.drag_start_hit_world = start_hit_world;
        self.state.drag_prev_hit_world = start_hit_world;
        self.state.drag_translate_prev_axis_raw = 0.0;
        self.state.drag_translate_prev_plane_raw = Vec2::ZERO;

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Translation {
                delta: Vec3::ZERO,
                total: Vec3::ZERO,
            },
            updated_targets: targets.to_vec(),
            custom_edits: Vec::new(),
        })
    }

    fn begin_rotate_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        cursor_ray: Ray3d,
        origin: Vec3,
        active: HandleId,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Option<GizmoUpdate> {
        if active == Self::ROTATE_ARCBALL_HANDLE {
            return self.begin_arcball_drag(
                view_projection,
                viewport,
                input,
                targets,
                origin,
                size_length_world,
                active,
            );
        }

        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;
        let axis_dir = if active == Self::ROTATE_VIEW_HANDLE {
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?
        } else {
            let (_, axis_index) = axis_for_handle(active);
            axes[axis_index]
        }
        .normalize_or_zero();
        if axis_dir.length_squared() == 0.0 {
            return None;
        }

        let (u, v) = plane_basis(axis_dir);

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Rotate;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_axis_dir = axis_dir;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_plane_normal = axis_dir;
        self.state.drag_basis_u = u;
        self.state.drag_basis_v = v;
        self.state.drag_total_angle_raw = 0.0;
        self.state.drag_total_angle_applied = 0.0;
        self.state.drag_rotate_is_arcball = false;
        self.state.drag_total_arcball_raw = Quat::IDENTITY;
        self.state.drag_total_arcball_applied = Quat::IDENTITY;
        self.state.drag_start_targets = targets.to_vec();

        let start_hit_world = ray_plane_intersect(cursor_ray, origin, axis_dir)
            .filter(|p| p.is_finite())
            .unwrap_or_else(|| {
                unproject_point(
                    view_projection,
                    viewport,
                    input.cursor_px,
                    self.config.depth_range,
                    origin_z01,
                )
                .unwrap_or(origin + u)
            });

        let mut angle = angle_on_plane(origin, start_hit_world, axis_dir, u, v)?;
        angle *= self.handedness_rotation_sign();
        self.state.drag_start_angle = angle;
        self.state.drag_prev_angle = angle;

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Rotation {
                axis: axis_dir,
                delta_radians: 0.0,
                total_radians: 0.0,
            },
            updated_targets: targets.to_vec(),
            custom_edits: Vec::new(),
        })
    }

    fn begin_arcball_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        origin: Vec3,
        size_length_world: f32,
        active: HandleId,
    ) -> Option<GizmoUpdate> {
        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;
        let view_dir =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?;
        let n = (-view_dir).normalize_or_zero();
        if n.length_squared() == 0.0 {
            return None;
        }
        let (u, v) = plane_basis(n);

        let center_px =
            project_point(view_projection, viewport, origin, self.config.depth_range)?.screen;
        let radius_px = match self.config.size_policy {
            GizmoSizePolicy::ConstantPixels => {
                self.config.size_px * self.config.arcball_radius_scale
            }
            GizmoSizePolicy::PixelsClampedBySelectionBounds { .. }
            | GizmoSizePolicy::SelectionBounds { .. } => {
                let r_world = (size_length_world * self.config.arcball_radius_scale).max(1e-6);
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

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Rotate;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_axis_dir = n;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_plane_normal = n;
        self.state.drag_basis_u = u;
        self.state.drag_basis_v = v;
        self.state.drag_rotate_is_arcball = true;
        self.state.drag_arcball_center_px = center_px;
        self.state.drag_arcball_radius_px = radius_px;
        self.state.drag_arcball_prev_vec = self.arcball_vector_world(input.cursor_px)?;
        self.state.drag_total_arcball_raw = Quat::IDENTITY;
        self.state.drag_total_arcball_applied = Quat::IDENTITY;
        self.state.drag_total_angle_raw = 0.0;
        self.state.drag_total_angle_applied = 0.0;
        self.state.drag_start_targets = targets.to_vec();

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Arcball {
                delta: Quat::IDENTITY,
                total: Quat::IDENTITY,
            },
            updated_targets: targets.to_vec(),
            custom_edits: Vec::new(),
        })
    }

    fn begin_scale_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        cursor_ray: Ray3d,
        origin: Vec3,
        active: HandleId,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Option<GizmoUpdate> {
        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;

        if handle_group(active) != HANDLE_GROUP_SCALE {
            return None;
        }

        let id = handle_sub_id(active) as u64;
        let (scale_dir, plane_normal, axis, plane_axes, plane_u, plane_v) = match id {
            1..=3 => {
                let (_, axis_index) = axis_for_handle(active);
                let axis_dir = axes[axis_index].normalize_or_zero();
                if axis_dir.length_squared() == 0.0 {
                    return None;
                }
                let plane_normal = axis_drag_plane_normal(
                    view_projection,
                    viewport,
                    self.config.depth_range,
                    origin,
                    axis_dir,
                )?;
                (
                    axis_dir,
                    plane_normal,
                    Some(axis_index),
                    None,
                    Vec3::X,
                    Vec3::Y,
                )
            }
            7 => {
                let view_dir =
                    view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?;
                let (u, v) = plane_basis(view_dir);
                let dir = (u + v).normalize_or_zero();
                let n = view_dir.normalize_or_zero();
                if dir.length_squared() == 0.0 || n.length_squared() == 0.0 {
                    return None;
                }
                (dir, n, None, None, Vec3::X, Vec3::Y)
            }
            14..=16 => {
                let (a, b) = match id {
                    14 => (0, 1),
                    15 => (0, 2),
                    16 => (1, 2),
                    _ => unreachable!(),
                };
                let u = axes[a].normalize_or_zero();
                let v = axes[b].normalize_or_zero();
                if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
                    return None;
                }
                let n = u.cross(v).normalize_or_zero();
                if n.length_squared() == 0.0 {
                    return None;
                }
                (Vec3::ZERO, n, None, Some((a, b)), u, v)
            }
            _ => return None,
        };

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Scale;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_size_length_world = size_length_world;
        self.state.drag_plane_normal = plane_normal;
        self.state.drag_axis_dir = scale_dir;
        self.state.drag_scale_axis = axis;
        self.state.drag_scale_plane_axes = plane_axes;
        self.state.drag_scale_plane_u = plane_u;
        self.state.drag_scale_plane_v = plane_v;
        let start_hit_world = ray_plane_intersect(cursor_ray, origin, plane_normal)
            .filter(|p| p.is_finite())
            .unwrap_or_else(|| {
                unproject_point(
                    view_projection,
                    viewport,
                    input.cursor_px,
                    self.config.depth_range,
                    origin_z01,
                )
                .unwrap_or(origin)
            });
        self.state.drag_start_hit_world = start_hit_world;
        self.state.drag_prev_hit_world = start_hit_world;
        self.state.drag_scale_prev_axis_raw = 0.0;
        self.state.drag_scale_prev_plane_raw = Vec2::ZERO;
        self.state.drag_total_scale_raw = 0.0;
        self.state.drag_total_scale_applied = 1.0;
        self.state.drag_total_scale_plane_raw = Vec2::ZERO;
        self.state.drag_total_scale_plane_applied = Vec2::ONE;
        self.state.drag_scale_is_uniform = axis.is_none() && plane_axes.is_none();
        self.state.drag_scale_is_bounds = false;
        self.state.drag_start_targets = targets.to_vec();

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Scale {
                delta: Vec3::ONE,
                total: Vec3::ONE,
            },
            updated_targets: targets.to_vec(),
            custom_edits: Vec::new(),
        })
    }

    fn begin_bounds_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        cursor_ray: Ray3d,
        origin: Vec3,
        origin_z01: f32,
        size_length_world: f32,
        handle: BoundsHandle,
        active: HandleId,
        axes_raw: [Vec3; 3],
    ) -> Option<GizmoUpdate> {
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
        let extent = (max_local - min_local).max(Vec3::splat(1e-6));

        let mut axes_mask = [false; 3];
        let mut axis_sign = [1.0f32; 3];
        let mut anchor_local = min_local;

        match handle {
            BoundsHandle::Corner {
                x_max,
                y_max,
                z_max,
            } => {
                let sides = [x_max, y_max, z_max];
                for i in 0..3 {
                    axes_mask[i] = true;
                    axis_sign[i] = if sides[i] { 1.0 } else { -1.0 };
                    anchor_local[i] = if sides[i] { min_local[i] } else { max_local[i] };
                }
            }
            BoundsHandle::Face { axis, max_side } => {
                axes_mask[axis.min(2)] = true;
                axis_sign[axis.min(2)] = if max_side { 1.0 } else { -1.0 };
                for i in 0..3 {
                    anchor_local[i] = match (i, axis.min(2)) {
                        (a, b) if a == b => {
                            if max_side {
                                min_local[i]
                            } else {
                                max_local[i]
                            }
                        }
                        _ => center_local[i],
                    };
                }
            }
        }

        let axes_count = axes_mask.iter().filter(|v| **v).count();
        let plane_normal = if axes_count == 1 {
            let axis = axes_mask.iter().position(|v| *v).unwrap_or(0);
            let axis_dir = basis[axis] * axis_sign[axis];
            axis_drag_plane_normal_facing_camera(
                view_projection,
                viewport,
                self.config.depth_range,
                origin,
                axis_dir,
            )?
        } else {
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?
        };

        let start_hit_world = ray_plane_intersect(cursor_ray, origin, plane_normal)
            .filter(|p| p.is_finite())
            .unwrap_or(origin);

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Scale;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_size_length_world = size_length_world;
        self.state.drag_plane_normal = plane_normal.normalize_or_zero();
        self.state.drag_start_hit_world = start_hit_world;
        self.state.drag_prev_hit_world = start_hit_world;
        self.state.drag_bounds_prev_local_raw = Vec3::ZERO;

        self.state.drag_scale_is_bounds = true;
        self.state.drag_bounds_basis = basis;
        self.state.drag_bounds_min_local = min_local;
        self.state.drag_bounds_max_local = max_local;
        self.state.drag_bounds_anchor_local = anchor_local;
        self.state.drag_bounds_axes_mask = axes_mask;
        self.state.drag_bounds_axis_sign = axis_sign;
        self.state.drag_bounds_start_extent = extent;
        self.state.drag_bounds_total_raw = Vec3::ZERO;
        self.state.drag_bounds_total_applied = Vec3::ONE;

        self.state.drag_total_scale_raw = 0.0;
        self.state.drag_total_scale_applied = 1.0;
        self.state.drag_total_scale_plane_raw = Vec2::ZERO;
        self.state.drag_total_scale_plane_applied = Vec2::ONE;
        self.state.drag_scale_axis = None;
        self.state.drag_scale_plane_axes = None;
        self.state.drag_scale_is_uniform = false;
        self.state.drag_start_targets = targets.to_vec();

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Scale {
                delta: Vec3::ONE,
                total: Vec3::ONE,
            },
            updated_targets: targets.to_vec(),
            custom_edits: Vec::new(),
        })
    }
}

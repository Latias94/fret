use super::*;

fn test_view_projection(viewport_px: (f32, f32)) -> Mat4 {
    let aspect = viewport_px.0.max(1.0) / viewport_px.1.max(1.0);
    let eye = Vec3::new(3.0, 2.0, 4.0);
    let target = Vec3::ZERO;
    let view = Mat4::look_at_rh(eye, target, Vec3::Y);
    let proj = Mat4::perspective_rh(60.0_f32.to_radians(), aspect, 0.05, 100.0);
    proj * view
}

fn test_view_projection_lh(viewport_px: (f32, f32)) -> Mat4 {
    let aspect = viewport_px.0.max(1.0) / viewport_px.1.max(1.0);
    let eye = Vec3::new(3.0, 2.0, 4.0);
    let target = Vec3::ZERO;
    let view = Mat4::look_at_lh(eye, target, Vec3::Y);
    let proj = Mat4::perspective_lh(60.0_f32.to_radians(), aspect, 0.05, 100.0);
    proj * view
}

fn test_view_projection_fov(viewport_px: (f32, f32), fov_degrees: f32, eye: Vec3) -> Mat4 {
    let aspect = viewport_px.0.max(1.0) / viewport_px.1.max(1.0);
    let target = Vec3::ZERO;
    let view = Mat4::look_at_rh(eye, target, Vec3::Y);
    let proj = Mat4::perspective_rh(
        fov_degrees.clamp(1.0, 179.0).to_radians(),
        aspect,
        0.05,
        100.0,
    );
    proj * view
}

fn test_size_length_world_no_targets(
    gizmo: &Gizmo,
    view_projection: Mat4,
    viewport: ViewportRect,
    origin: Vec3,
) -> f32 {
    axis_length_world(
        view_projection,
        viewport,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap_or(1.0)
}

fn test_view_projection_ortho(viewport_px: (f32, f32), eye: Vec3) -> Mat4 {
    let aspect = viewport_px.0.max(1.0) / viewport_px.1.max(1.0);
    let target = Vec3::ZERO;
    let view = Mat4::look_at_rh(eye, target, Vec3::Y);

    let half_h = 2.0;
    let half_w = half_h * aspect;
    let proj = Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, 0.05, 100.0);
    proj * view
}

fn base_gizmo(mode: GizmoMode) -> Gizmo {
    let mut config = GizmoConfig {
        mode,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    // Keep tests deterministic: axis flip + visibility thresholds are UX heuristics that can
    // vary with camera orientation and viewport shape.
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.axis_mask = [false; 3];
    Gizmo::new(config)
}

#[test]
fn translate_plane_fill_can_disable_occluded_ghost_pass() {
    let mut gizmo = base_gizmo(GizmoMode::Translate);
    gizmo.config.depth_mode = DepthMode::Test;
    gizmo.config.show_occluded = true;
    gizmo.config.operation_mask = Some(GizmoOps::translate_plane());

    let mut pv = GizmoPartVisuals::classic();
    pv.occlusion.translate_plane_fill = false;
    gizmo.set_part_visuals(pv);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let target = GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    };
    let draw = gizmo.draw(view_proj, vp, target.id, &[target]);
    assert!(
        draw.triangles.iter().any(|t| t.depth == DepthMode::Test),
        "expected translate plane fill to emit depth-tested triangles"
    );
    assert!(
        draw.triangles.iter().all(|t| t.depth != DepthMode::Ghost),
        "expected translate plane fill to be able to suppress occluded ghost pass"
    );
}

#[test]
fn rotate_axis_ring_can_disable_occluded_ghost_pass() {
    let mut gizmo = base_gizmo(GizmoMode::Rotate);
    gizmo.config.depth_mode = DepthMode::Test;
    gizmo.config.show_occluded = true;
    gizmo.config.show_view_axis_ring = false;
    gizmo.config.show_arcball = false;
    gizmo.config.operation_mask = Some(GizmoOps::rotate_axis());

    let mut pv = GizmoPartVisuals::classic();
    pv.occlusion.rotate_axis_rings = false;
    gizmo.set_part_visuals(pv);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let target = GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    };
    let draw = gizmo.draw(view_proj, vp, target.id, &[target]);
    assert!(
        draw.triangles.iter().any(|t| t.depth == DepthMode::Test),
        "expected rotate axis rings to emit depth-tested triangles"
    );
    assert!(
        draw.triangles.iter().all(|t| t.depth != DepthMode::Ghost),
        "expected rotate axis rings to be able to suppress occluded ghost pass"
    );
    assert!(
        draw.lines.iter().all(|l| l.depth != DepthMode::Ghost),
        "expected rotate axis ring edge strokes to be able to suppress occluded ghost pass"
    );
}

#[test]
fn bounds_can_disable_occluded_ghost_pass() {
    let mut gizmo = base_gizmo(GizmoMode::Scale);
    gizmo.config.depth_mode = DepthMode::Test;
    gizmo.config.show_occluded = true;

    let mut pv = GizmoPartVisuals::classic();
    pv.occlusion.bounds = false;
    gizmo.set_part_visuals(pv);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = [Vec3::X, Vec3::Y, Vec3::Z];
    let size_length_world = test_size_length_world_no_targets(&gizmo, view_proj, vp, origin);

    let target = GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: Some(Aabb3 {
            min: Vec3::splat(-0.5),
            max: Vec3::splat(0.5),
        }),
    };

    let mut out = GizmoDrawList3d::default();
    gizmo.draw_bounds(
        &mut out,
        view_proj,
        vp,
        origin,
        axes,
        size_length_world,
        &[target],
    );
    assert!(
        out.lines.iter().any(|l| l.depth == DepthMode::Test),
        "expected bounds to emit depth-tested lines"
    );
    assert!(
        out.triangles.iter().any(|t| t.depth == DepthMode::Test),
        "expected bounds handles to emit depth-tested triangles"
    );
    assert!(
        out.lines.iter().all(|l| l.depth != DepthMode::Ghost),
        "expected bounds to be able to suppress occluded ghost pass for lines"
    );
    assert!(
        out.triangles.iter().all(|t| t.depth != DepthMode::Ghost),
        "expected bounds to be able to suppress occluded ghost pass for triangles"
    );
}

#[test]
fn rotate_feedback_never_emits_ghost_triangles_even_when_enabled() {
    let mut gizmo = base_gizmo(GizmoMode::Rotate);
    gizmo.config.depth_mode = DepthMode::Test;
    gizmo.config.show_occluded = true;

    let mut pv = GizmoPartVisuals::classic();
    pv.occlusion.feedback = true;
    gizmo.set_part_visuals(pv);

    // Put the gizmo into a "mid-drag rotate" state so feedback draws.
    gizmo.state.drag_mode = GizmoMode::Rotate;
    gizmo.state.active = Some(RotateHandle::AxisX.id());
    gizmo.state.drag_has_started = true;
    gizmo.state.drag_axis_dir = Vec3::X;
    gizmo.state.drag_basis_u = Vec3::Y;
    gizmo.state.drag_basis_v = Vec3::Z;
    gizmo.state.drag_start_angle = 0.0;
    gizmo.state.drag_total_angle_applied = std::f32::consts::FRAC_PI_2;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let out = gizmo.draw_rotate_feedback(view_proj, vp, origin, 1.0);
    assert!(
        out.triangles.iter().all(|t| t.depth != DepthMode::Ghost),
        "feedback triangles should remain non-ghost to preserve legibility"
    );
}

#[test]
fn translate_center_handle_wins_near_origin() {
    let gizmo = base_gizmo(GizmoMode::Translate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let size_length_world = test_size_length_world_no_targets(&gizmo, view_proj, vp, origin);

    let p0 = project_point(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
    let hit = gizmo
        .pick_translate_handle(
            view_proj,
            vp,
            origin,
            p0.screen,
            axes,
            size_length_world,
            true,
            true,
            true,
            true,
        )
        .unwrap();
    assert_eq!(hit.handle, TranslateHandle::Screen.id());
}

#[test]
fn operation_mask_translate_view_picks_screen_handle_at_origin() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let mut config = GizmoConfig {
        mode: GizmoMode::Universal,
        ..Default::default()
    };
    config.operation_mask = Some(GizmoOps::translate_view());
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);

    let mut gizmo = Gizmo::new(config);
    let origin = Vec3::ZERO;
    let cursor = project_point(view_proj, vp, origin, gizmo.config.depth_range)
        .unwrap()
        .screen;
    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let _ = gizmo.update(
        view_proj,
        vp,
        GizmoInput {
            cursor_px: cursor,
            hovered: true,
            drag_started: false,
            dragging: false,
            snap: false,
            cancel: false,
            precision: 1.0,
        },
        targets[0].id,
        &targets,
    );

    assert_eq!(gizmo.state.hovered, Some(TranslateHandle::Screen.id()));
    assert_eq!(gizmo.state.hovered_kind, Some(GizmoMode::Translate));
}

#[test]
fn operation_mask_scale_uniform_picks_uniform_handle_at_origin() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let mut config = GizmoConfig {
        mode: GizmoMode::Universal,
        ..Default::default()
    };
    config.operation_mask = Some(GizmoOps::scale_uniform());
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);

    let mut gizmo = Gizmo::new(config);
    let origin = Vec3::ZERO;
    let cursor = project_point(view_proj, vp, origin, gizmo.config.depth_range)
        .unwrap()
        .screen;
    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let _ = gizmo.update(
        view_proj,
        vp,
        GizmoInput {
            cursor_px: cursor,
            hovered: true,
            drag_started: false,
            dragging: false,
            snap: false,
            cancel: false,
            precision: 1.0,
        },
        targets[0].id,
        &targets,
    );

    assert_eq!(gizmo.state.hovered, Some(ScaleHandle::Uniform.id()));
    assert_eq!(gizmo.state.hovered_kind, Some(GizmoMode::Scale));
}

#[test]
fn operation_mask_translate_axis_tip_wins_over_rotate_rings() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let mut config = GizmoConfig {
        mode: GizmoMode::Universal,
        ..Default::default()
    };
    config.operation_mask = Some(GizmoOps::translate_axis() | GizmoOps::rotate_all());
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);

    let gizmo = Gizmo::new(config);
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let tip_world = origin + axes[0] * (length_world * Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE);
    let tip = project_point(view_proj, vp, tip_world, gizmo.config.depth_range).unwrap();

    assert!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, tip.screen, axes, length_world)
            .is_some(),
        "expected rotate rings to be pickable near the translate tip in this projection"
    );

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let (hit, kind) = gizmo
        .pick_operation_mask_handle(
            view_proj,
            vp,
            origin,
            tip.screen,
            axes,
            axes,
            length_world,
            &targets,
        )
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, TranslateHandle::AxisX.id());
}

#[test]
fn translate_axis_drag_returns_to_zero_when_cursor_returns() {
    let mut gizmo = base_gizmo(GizmoMode::Translate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let a = origin;
    let b = origin + axes[0].normalize_or_zero() * length_world;
    let pa = project_point(view_proj, vp, a, gizmo.config.depth_range).unwrap();
    let pb = project_point(view_proj, vp, b, gizmo.config.depth_range).unwrap();
    let axis_dir_screen = (pb.screen - pa.screen).normalize_or_zero();
    assert!(axis_dir_screen.length_squared() > 0.0);

    let cursor_start = pa.screen.lerp(pb.screen, 0.5);
    let cursor_moved = cursor_start + axis_dir_screen * 40.0;

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();

    let moved_total = match moved.result {
        GizmoResult::Translation { total, .. } => total,
        _ => panic!("expected translation"),
    };
    assert!(moved_total.x.is_finite());
    assert!(moved_total.x > 0.0);
    assert!(moved_total.y.abs() < 1e-3);
    assert!(moved_total.z.abs() < 1e-3);
    assert!(
        moved.updated_targets[0]
            .transform
            .translation
            .distance(moved_total)
            < 1e-3,
        "updated={:?} total={moved_total:?}",
        moved.updated_targets[0].transform.translation
    );

    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();

    let back_total = match back.result {
        GizmoResult::Translation { total, .. } => total,
        _ => panic!("expected translation"),
    };
    assert!(back_total.length() < 1e-3, "total={back_total:?}");
    assert!(
        back.updated_targets[0].transform.translation.length() < 1e-3,
        "updated={:?}",
        back.updated_targets[0].transform.translation
    );
}

#[test]
fn translate_dolly_drag_returns_to_zero_when_cursor_returns() {
    let mut gizmo = base_gizmo(GizmoMode::Translate);
    // Ensure the depth ring sits comfortably outside the center pick radius.
    gizmo.config.size_px = 120.0;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let size_length_world = test_size_length_world_no_targets(&gizmo, view_proj, vp, origin);

    let view_dir = view_dir_at_origin(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
    let (u, _v) = plane_basis(view_dir);
    let pv = gizmo.part_visuals();
    let r_world = (size_length_world * pv.translate_depth_ring_radius_fraction.max(0.0))
        .max(size_length_world * pv.translate_depth_ring_radius_min_fraction.max(0.0));

    let origin_px = project_point(view_proj, vp, origin, gizmo.config.depth_range)
        .unwrap()
        .screen;
    let ring_px = project_point(
        view_proj,
        vp,
        origin + u.normalize_or_zero() * r_world,
        gizmo.config.depth_range,
    )
    .unwrap()
    .screen;
    assert!(
        (ring_px - origin_px).length() > gizmo.config.pick_radius_px.max(6.0) + 2.0,
        "depth ring should be outside center handle radius"
    );

    let hit = gizmo
        .pick_translate_handle(
            view_proj,
            vp,
            origin,
            ring_px,
            axes,
            size_length_world,
            true,
            true,
            true,
            true,
        )
        .unwrap();
    assert_eq!(hit.handle, TranslateHandle::Depth.id());

    let cursor_start = ring_px;
    let cursor_moved = cursor_start + Vec2::new(0.0, 40.0);

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();

    let moved_total = match moved.result {
        GizmoResult::Translation { total, .. } => total,
        _ => panic!("expected translation"),
    };
    assert!(moved_total.is_finite());
    assert!(
        moved_total.dot(view_dir.normalize_or_zero()) > 0.0,
        "expected moving cursor down to translate along view_dir (away from camera)"
    );

    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();

    let back_total = match back.result {
        GizmoResult::Translation { total, .. } => total,
        _ => panic!("expected translation"),
    };
    assert!(back_total.length() < 1e-3, "total={back_total:?}");
}

#[test]
fn translate_axis_drag_returns_to_zero_in_orthographic() {
    let mut gizmo = base_gizmo(GizmoMode::Translate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_ortho((800.0, 600.0), Vec3::new(3.0, 2.0, 4.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let a = origin;
    let b = origin + axes[0].normalize_or_zero() * length_world;
    let pa = project_point(view_proj, vp, a, gizmo.config.depth_range).unwrap();
    let pb = project_point(view_proj, vp, b, gizmo.config.depth_range).unwrap();
    let axis_dir_screen = (pb.screen - pa.screen).normalize_or_zero();
    assert!(axis_dir_screen.length_squared() > 0.0);

    let cursor_start = pa.screen.lerp(pb.screen, 0.6);
    let cursor_moved = cursor_start + axis_dir_screen * 50.0;

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();

    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();

    let back_total = match back.result {
        GizmoResult::Translation { total, .. } => total,
        _ => panic!("expected translation"),
    };
    assert!(back_total.length() < 1e-3, "total={back_total:?}");
}

#[test]
fn rotate_axis_drag_returns_to_zero_in_orthographic() {
    let mut gizmo = base_gizmo(GizmoMode::Rotate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_ortho((800.0, 600.0), Vec3::new(3.0, 2.0, 4.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let radius_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let axis_dir = axes[0].normalize_or_zero();
    let (u, v) = plane_basis(axis_dir);
    let p_start_world = origin + u * radius_world;
    let p_move_world = origin + (u * 0.98 + v * 0.2).normalize_or_zero() * radius_world;

    let p_start = project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Rotation { total_radians, .. } => total_radians,
        _ => panic!("expected rotation"),
    };
    assert!(moved_total.is_finite());
    assert!(moved_total.abs() > 1e-6);

    let input_back = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Rotation { total_radians, .. } => total_radians,
        _ => panic!("expected rotation"),
    };
    assert!(back_total.abs() < 1e-3, "total={back_total}");
}

#[test]
fn scale_axis_drag_returns_to_one_in_orthographic() {
    let mut gizmo = base_gizmo(GizmoMode::Scale);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_ortho((800.0, 600.0), Vec3::new(3.0, 2.0, 4.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let axis_dir = axes[0].normalize_or_zero();
    assert!(axis_dir.length_squared() > 0.0);
    let p_start_world = origin + axis_dir * length_world;
    let p_move_world = origin + axis_dir * (length_world * 1.35);

    let p_start = project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(moved_total.x.is_finite());
    assert!(moved_total.x > 1.0 + 1e-6);

    let input_back = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(
        (back_total - Vec3::ONE).length() < 1e-3,
        "total={back_total:?}"
    );
}

#[test]
fn translate_axis_drag_returns_to_zero_with_wide_fov() {
    let mut gizmo = base_gizmo(GizmoMode::Translate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 120.0, Vec3::new(3.0, 2.0, 4.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let a = origin;
    let b = origin + axes[0].normalize_or_zero() * length_world;
    let pa = project_point(view_proj, vp, a, gizmo.config.depth_range).unwrap();
    let pb = project_point(view_proj, vp, b, gizmo.config.depth_range).unwrap();
    let axis_dir_screen = (pb.screen - pa.screen).normalize_or_zero();
    assert!(axis_dir_screen.length_squared() > 0.0);

    let cursor_start = pa.screen.lerp(pb.screen, 0.5);
    let cursor_moved = cursor_start + axis_dir_screen * 45.0;

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();

    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();

    let back_total = match back.result {
        GizmoResult::Translation { total, .. } => total,
        _ => panic!("expected translation"),
    };
    assert!(back_total.length() < 1e-3, "total={back_total:?}");
}

#[test]
fn translate_axis_drag_returns_to_zero_near_near_plane() {
    let mut gizmo = base_gizmo(GizmoMode::Translate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 0.06));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let a = origin;
    let b = origin + axes[0].normalize_or_zero() * length_world;
    let pa = project_point(view_proj, vp, a, gizmo.config.depth_range).unwrap();
    let pb = project_point(view_proj, vp, b, gizmo.config.depth_range).unwrap();
    let axis_dir_screen = (pb.screen - pa.screen).normalize_or_zero();
    assert!(axis_dir_screen.length_squared() > 0.0);

    let cursor_start = pa.screen.lerp(pb.screen, 0.5);
    let cursor_moved = cursor_start + axis_dir_screen * 60.0;

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();

    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();

    let back_total = match back.result {
        GizmoResult::Translation { total, .. } => total,
        _ => panic!("expected translation"),
    };
    assert!(back_total.length() < 1e-3, "total={back_total:?}");
}

#[test]
fn rotate_axis_drag_returns_to_zero_near_near_plane() {
    let mut gizmo = base_gizmo(GizmoMode::Rotate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 0.06));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let radius_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    // Use the Z ring: with this camera setup (looking down -Z), the X ring includes segments
    // that can cross the near plane (because that ring swings toward the camera in +Z), which
    // makes picking brittle. The Z ring lies in the XY plane at a stable depth.
    let axis_dir = axes[2].normalize_or_zero();
    let (u, v) = plane_basis(axis_dir);
    let p_start_world = origin + u * radius_world;
    let p_move_world = origin + (u * 0.98 + v * 0.2).normalize_or_zero() * radius_world;

    let p_start = project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Rotation { total_radians, .. } => total_radians,
        _ => panic!("expected rotation"),
    };
    assert!(moved_total.is_finite());
    assert!(moved_total.abs() > 1e-6);

    let input_back = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Rotation { total_radians, .. } => total_radians,
        _ => panic!("expected rotation"),
    };
    assert!(back_total.abs() < 1e-3, "total={back_total}");
}

#[test]
fn scale_axis_drag_returns_to_one_near_near_plane() {
    let mut gizmo = base_gizmo(GizmoMode::Scale);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 0.06));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let axis_dir = axes[0].normalize_or_zero();
    assert!(axis_dir.length_squared() > 0.0);
    let p_start_world = origin + axis_dir * length_world;
    let p_move_world = origin + axis_dir * (length_world * 1.35);

    let p_start = project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(moved_total.x.is_finite());
    assert!(moved_total.x > 1.0 + 1e-6);

    let input_back = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(
        (back_total - Vec3::ONE).length() < 1e-3,
        "total={back_total:?}"
    );
}

#[test]
fn behind_camera_is_not_pickable() {
    let gizmo = base_gizmo(GizmoMode::Translate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));

    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 1.0));
    let origin = Vec3::new(0.0, 0.0, 2.0);
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let size_length_world = test_size_length_world_no_targets(&gizmo, view_proj, vp, origin);

    assert!(
        project_point(view_proj, vp, origin, gizmo.config.depth_range).is_none(),
        "behind-camera project_point should return None"
    );

    let cursor = Vec2::new(400.0, 300.0);
    assert!(
        gizmo
            .pick_translate_handle(
                view_proj,
                vp,
                origin,
                cursor,
                axes,
                size_length_world,
                true,
                true,
                true,
                true,
            )
            .is_none(),
        "behind-camera gizmo should not be pickable"
    );
}

#[test]
fn rotate_and_scale_are_not_pickable_when_origin_is_behind_camera() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));

    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 1.0));
    let origin = Vec3::new(0.0, 0.0, 2.0);

    let rotate = base_gizmo(GizmoMode::Rotate);
    let scale = base_gizmo(GizmoMode::Scale);
    let axes = rotate.axis_dirs(&Transform3d::default());
    let cursor = Vec2::new(400.0, 300.0);

    assert!(
        rotate
            .pick_rotate_axis(view_proj, vp, origin, cursor, axes, 1.0)
            .is_none()
    );
    assert!(
        scale
            .pick_scale_handle(view_proj, vp, origin, cursor, axes, 1.0, true, true, true)
            .is_none()
    );
}

#[test]
fn axis_mask_hides_translate_axis_pick() {
    let mut config = GizmoConfig {
        mode: GizmoMode::Translate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.axis_mask = [true, false, false]; // hide X
    let gizmo = Gizmo::new(config);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let pa = project_point(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
    let pb = project_point(
        view_proj,
        vp,
        origin + axes[0].normalize_or_zero() * length_world,
        gizmo.config.depth_range,
    )
    .unwrap();
    let cursor = pa.screen.lerp(pb.screen, 0.65);

    let hit = gizmo.pick_translate_handle(
        view_proj,
        vp,
        origin,
        cursor,
        axes,
        length_world,
        true,
        true,
        true,
        true,
    );
    assert!(
        hit.is_none() || hit.unwrap().handle != TranslateHandle::AxisX.id(),
        "masked X axis should not be pickable"
    );
}

#[test]
fn axis_mask_single_axis_shows_only_perp_plane() {
    let mut config = GizmoConfig {
        mode: GizmoMode::Translate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.axis_mask = [true, false, false]; // hide X -> only YZ plane should remain
    let gizmo = Gizmo::new(config);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let pv = gizmo.part_visuals();
    let off = length_world * pv.translate_plane_offset_fraction.max(0.0);
    let size = length_world * pv.translate_plane_size_fraction.max(0.0);

    let quad_xy = translate_plane_quad_world(origin, axes[0], axes[1], off, size);
    let quad_xz = translate_plane_quad_world(origin, axes[0], axes[2], off, size);
    let quad_yz = translate_plane_quad_world(origin, axes[1], axes[2], off, size);
    let p_xy = project_quad(view_proj, vp, quad_xy, gizmo.config.depth_range).unwrap();
    let p_xz = project_quad(view_proj, vp, quad_xz, gizmo.config.depth_range).unwrap();
    let p_yz = project_quad(view_proj, vp, quad_yz, gizmo.config.depth_range).unwrap();

    let c_xy = (p_xy[0] + p_xy[1] + p_xy[2] + p_xy[3]) * 0.25;
    let c_xz = (p_xz[0] + p_xz[1] + p_xz[2] + p_xz[3]) * 0.25;
    let c_yz = (p_yz[0] + p_yz[1] + p_yz[2] + p_yz[3]) * 0.25;

    let h_xy = gizmo.pick_translate_handle(
        view_proj,
        vp,
        origin,
        c_xy,
        axes,
        length_world,
        true,
        true,
        true,
        true,
    );
    assert!(
        h_xy.is_none() || h_xy.unwrap().handle != TranslateHandle::PlaneXY.id(),
        "XY plane should be hidden when X is masked"
    );

    let h_xz = gizmo.pick_translate_handle(
        view_proj,
        vp,
        origin,
        c_xz,
        axes,
        length_world,
        true,
        true,
        true,
        true,
    );
    assert!(
        h_xz.is_none() || h_xz.unwrap().handle != TranslateHandle::PlaneXZ.id(),
        "XZ plane should be hidden when X is masked"
    );

    let h_yz = gizmo
        .pick_translate_handle(
            view_proj,
            vp,
            origin,
            c_yz,
            axes,
            length_world,
            true,
            true,
            true,
            true,
        )
        .unwrap();
    assert_eq!(h_yz.handle, TranslateHandle::PlaneYZ.id());
}

#[test]
fn translate_plane_inside_wins_over_axis_when_both_hit() {
    let mut config = GizmoConfig {
        mode: GizmoMode::Translate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    // Make it easy for an axis segment to be "also hit" while still allowing the cursor to be
    // far enough from the origin that the center handle does not steal the interaction.
    config.pick_radius_px = 20.0;
    let gizmo = Gizmo::new(config);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();
    let axis_tip_len = length_world * gizmo.translate_axis_tip_scale();

    let pv = gizmo.part_visuals();
    let off = length_world * pv.translate_plane_offset_fraction.max(0.0);
    let size = length_world * pv.translate_plane_size_fraction.max(0.0);
    let quad_world = translate_plane_quad_world(origin, axes[0], axes[1], off, size);
    let quad_screen = project_quad(view_proj, vp, quad_world, gizmo.config.depth_range).unwrap();

    let pa = project_point(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
    let pb = project_point(
        view_proj,
        vp,
        origin + axes[0].normalize_or_zero() * axis_tip_len,
        gizmo.config.depth_range,
    )
    .unwrap();

    let mut cursor: Option<Vec2> = None;
    // Search for a point that is inside the plane quad but also within the X axis pick radius.
    for i in 0..=24 {
        for j in 0..=24 {
            // Prefer small `t` (close to the X axis direction in screen space) and larger `s`
            // (further from origin so the center handle doesn't steal).
            let s = 0.25 + 0.70 * (i as f32) / 24.0;
            let t = 0.01 + 0.25 * (j as f32) / 24.0;
            let candidate = quad_screen[0] * (1.0 - s) * (1.0 - t)
                + quad_screen[1] * s * (1.0 - t)
                + quad_screen[3] * (1.0 - s) * t
                + quad_screen[2] * s * t;
            if !(PickConvexQuad2d {
                points: quad_screen,
            }
            .contains(candidate))
            {
                continue;
            }
            let d_center = (candidate - pa.screen).length();
            if d_center <= gizmo.config.pick_radius_px {
                continue;
            }
            let d_axis = distance_point_to_segment_px(candidate, pa.screen, pb.screen);
            if d_axis <= gizmo.config.pick_radius_px {
                cursor = Some(candidate);
                break;
            }
        }
        if cursor.is_some() {
            break;
        }
    }

    let cursor = cursor.expect("expected a point where both plane-inside and axis hit");
    let size_length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap_or(1.0);
    let hit = gizmo
        .pick_translate_handle(
            view_proj,
            vp,
            origin,
            cursor,
            axes,
            size_length_world,
            true,
            true,
            true,
            false,
        )
        .unwrap();
    assert_eq!(hit.handle, TranslateHandle::PlaneXY.id());
}

#[test]
fn allow_axis_flip_prefers_more_visible_direction() {
    let mut config = GizmoConfig {
        mode: GizmoMode::Translate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = true;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    let gizmo = Gizmo::new(config);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();
    let flipped = gizmo.flip_axes_for_view(view_proj, vp, origin, axes, length_world);

    let plus = axis_segment_len_px(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        axes[0],
        length_world,
    )
    .unwrap();
    let minus = axis_segment_len_px(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        -axes[0],
        length_world,
    )
    .unwrap();
    let chosen = axis_segment_len_px(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        flipped[0],
        length_world,
    )
    .unwrap();

    assert!(
        (chosen - plus).abs() < 1e-3 || (chosen - minus).abs() < 1e-3,
        "chosen axis should match +/- axis"
    );
    assert!(chosen >= plus.min(minus) - 1e-3);
    assert!(chosen >= plus.max(minus) - 1e-2);
}

#[test]
fn fade_reduces_scale_axis_edge_pick_radius() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 5.0));
    let origin = Vec3::ZERO;

    let mut config = GizmoConfig {
        mode: GizmoMode::Scale,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_mask = [true, true, false]; // only Z

    let mut no_fade = config;
    no_fade.axis_fade_px = (f32::NAN, f32::NAN);
    no_fade.plane_fade_px2 = (f32::NAN, f32::NAN);
    let no_fade = Gizmo::new(no_fade);

    let mut fade = config;
    fade.axis_fade_px = (0.0, 1000.0);
    fade.plane_fade_px2 = (f32::NAN, f32::NAN);
    let fade = Gizmo::new(fade);

    let axes = no_fade.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        no_fade.config.depth_range,
        no_fade.config.size_px,
    )
    .unwrap();

    let (u, v) = view_dir_at_origin(view_proj, vp, origin, no_fade.config.depth_range)
        .map(plane_basis)
        .unwrap();
    let end = origin + axes[2] * length_world;
    let pv = no_fade.part_visuals();
    let half = length_world * pv.scale_axis_end_box_half_fraction.max(0.0);
    let quad_world = [
        end + (-u - v) * half,
        end + (u - v) * half,
        end + (u + v) * half,
        end + (-u + v) * half,
    ];
    let p = project_quad(view_proj, vp, quad_world, no_fade.config.depth_range).unwrap();
    let c = (p[0] + p[1] + p[2] + p[3]) * 0.25;
    let diag_half = (p[0] - c).length();
    let dir = (p[0] - c).normalize_or_zero();
    assert!(dir.length_squared() > 0.0);

    // Cursor is outside the end-box quad but within the default pick radius.
    let cursor = c + dir * (diag_half + 6.0);
    let edge_d = PickConvexQuad2d { points: p }.edge_distance(cursor);
    assert!(edge_d.is_finite() && edge_d > 0.1);
    assert!(edge_d < no_fade.config.pick_radius_px);

    let hit_no_fade = no_fade.pick_scale_handle(
        view_proj,
        vp,
        origin,
        cursor,
        axes,
        length_world,
        true,
        false,
        false,
    );
    assert_eq!(hit_no_fade.unwrap().handle, ScaleHandle::AxisZ.id());

    let hit_fade = fade.pick_scale_handle(
        view_proj,
        vp,
        origin,
        cursor,
        axes,
        length_world,
        true,
        false,
        false,
    );
    assert!(
        hit_fade.is_none(),
        "faded axis should be harder to edge-pick"
    );
}

#[test]
fn translate_plane_drag_returns_to_zero_when_cursor_returns() {
    let mut gizmo = base_gizmo(GizmoMode::Translate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let pv = gizmo.part_visuals();
    let off = length_world * pv.translate_plane_offset_fraction.max(0.0);
    let size = length_world * pv.translate_plane_size_fraction.max(0.0);
    let quad_world = translate_plane_quad_world(origin, axes[0], axes[1], off, size);
    let quad_screen = project_quad(view_proj, vp, quad_world, gizmo.config.depth_range).unwrap();
    let cursor_start = (quad_screen[0] + quad_screen[1] + quad_screen[2] + quad_screen[3]) * 0.25;
    let cursor_moved = cursor_start + Vec2::new(25.0, -15.0);

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();

    let moved_total = match moved.result {
        GizmoResult::Translation { total, .. } => total,
        _ => panic!("expected translation"),
    };
    assert!(moved_total.length() > 1e-6);
    assert!(
        moved.updated_targets[0]
            .transform
            .translation
            .distance(moved_total)
            < 1e-3,
        "updated={:?} total={moved_total:?}",
        moved.updated_targets[0].transform.translation
    );

    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();

    let back_total = match back.result {
        GizmoResult::Translation { total, .. } => total,
        _ => panic!("expected translation"),
    };
    assert!(back_total.length() < 1e-3, "total={back_total:?}");
    assert!(
        back.updated_targets[0].transform.translation.length() < 1e-3,
        "updated={:?}",
        back.updated_targets[0].transform.translation
    );
}

#[test]
fn rotate_axis_drag_returns_to_zero_when_cursor_returns() {
    let mut gizmo = base_gizmo(GizmoMode::Rotate);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let radius_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let axis_dir = axes[0].normalize_or_zero();
    let (u, v) = plane_basis(axis_dir);
    let p_start_world = origin + u * radius_world;
    let p_move_world = origin + (u * 0.98 + v * 0.2).normalize_or_zero() * radius_world;

    let p_start = project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Rotation { total_radians, .. } => total_radians,
        _ => panic!("expected rotation"),
    };
    assert!(moved_total.is_finite());
    assert!(moved_total.abs() > 1e-6);

    let input_back = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Rotation { total_radians, .. } => total_radians,
        _ => panic!("expected rotation"),
    };
    assert!(back_total.abs() < 1e-3, "total={back_total}");
}

#[test]
fn rotate_axis_drag_sign_flips_with_handedness() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let mut base_cfg = GizmoConfig {
        mode: GizmoMode::Rotate,
        ..Default::default()
    };
    base_cfg.depth_range = DepthRange::ZeroToOne;
    base_cfg.drag_start_threshold_px = 0.0;
    base_cfg.allow_axis_flip = false;
    base_cfg.axis_fade_px = (f32::NAN, f32::NAN);
    base_cfg.plane_fade_px2 = (f32::NAN, f32::NAN);
    base_cfg.show_view_axis_ring = false;
    base_cfg.show_arcball = false;

    let mut gizmo_rh = Gizmo::new(base_cfg);
    gizmo_rh.config.handedness = GizmoHandedness::RightHanded;

    let mut gizmo_lh = Gizmo::new(base_cfg);
    gizmo_lh.config.handedness = GizmoHandedness::LeftHanded;

    let axes = gizmo_rh.axis_dirs(&Transform3d::default());
    let radius_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo_rh.config.depth_range,
        gizmo_rh.config.size_px,
    )
    .unwrap();

    let axis_dir = axes[0].normalize_or_zero();
    let (u, v) = plane_basis(axis_dir);
    let p_start_world = origin + u * radius_world;
    let p_move_world = origin + (u * 0.98 + v * 0.2).normalize_or_zero() * radius_world;

    let p_start = project_point(view_proj, vp, p_start_world, gizmo_rh.config.depth_range).unwrap();
    let p_move = project_point(view_proj, vp, p_move_world, gizmo_rh.config.depth_range).unwrap();

    let drag = |gizmo: &mut Gizmo| -> (f32, f32) {
        let input_down = GizmoInput {
            cursor_px: p_start.screen,
            hovered: true,
            drag_started: true,
            dragging: true,
            snap: false,
            cancel: false,
            precision: 1.0,
        };
        let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

        let input_move = GizmoInput {
            cursor_px: p_move.screen,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
            precision: 1.0,
        };
        let moved = gizmo
            .update(view_proj, vp, input_move, targets[0].id, &targets)
            .unwrap();
        let moved_total = match moved.result {
            GizmoResult::Rotation { total_radians, .. } => total_radians,
            _ => panic!("expected rotation"),
        };

        let input_back = GizmoInput {
            cursor_px: p_start.screen,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
            precision: 1.0,
        };
        let back = gizmo
            .update(view_proj, vp, input_back, targets[0].id, &targets)
            .unwrap();
        let back_total = match back.result {
            GizmoResult::Rotation { total_radians, .. } => total_radians,
            _ => panic!("expected rotation"),
        };

        (moved_total, back_total)
    };

    let (moved_rh, back_rh) = drag(&mut gizmo_rh);
    let (moved_lh, back_lh) = drag(&mut gizmo_lh);

    assert!(moved_rh.is_finite() && moved_lh.is_finite());
    assert!(moved_rh.abs() > 1e-6 && moved_lh.abs() > 1e-6);
    assert!(
        moved_rh * moved_lh < 0.0,
        "expected opposite signs, rh={moved_rh} lh={moved_lh}"
    );
    assert!(
        (moved_rh.abs() - moved_lh.abs()).abs() < 1e-5,
        "expected same magnitude, rh={moved_rh} lh={moved_lh}"
    );
    assert!(back_rh.abs() < 1e-3, "rh total={back_rh}");
    assert!(back_lh.abs() < 1e-3, "lh total={back_lh}");
}

#[test]
fn rotate_axis_drag_returns_to_zero_when_cursor_returns_with_left_handed_view_projection() {
    let mut gizmo = base_gizmo(GizmoMode::Rotate);
    gizmo.config.show_view_axis_ring = false;
    gizmo.config.show_arcball = false;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_lh((800.0, 600.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let radius_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let axis_dir = axes[0].normalize_or_zero();
    let (u, v) = plane_basis(axis_dir);
    let p_start_world = origin + u * radius_world;
    let p_move_world = origin + (u * 0.98 + v * 0.2).normalize_or_zero() * radius_world;

    let p_start = project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Rotation { total_radians, .. } => total_radians,
        _ => panic!("expected rotation"),
    };
    assert!(moved_total.is_finite());
    assert!(moved_total.abs() > 1e-6);

    let input_back = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Rotation { total_radians, .. } => total_radians,
        _ => panic!("expected rotation"),
    };
    assert!(back_total.abs() < 1e-3, "total={back_total}");
}

#[test]
fn rotate_ring_fade_hides_edge_on_axis_ring() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 5.0));
    let origin = Vec3::ZERO;

    let mut config = GizmoConfig {
        mode: GizmoMode::Rotate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.show_view_axis_ring = false;
    config.show_arcball = false;
    // Only show rings when looking almost along the axis direction.
    config.rotate_ring_fade_dot = (0.90, 0.95);
    let gizmo = Gizmo::new(config);

    let axes = gizmo.axis_dirs(&Transform3d::default());
    let radius_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    // Looking down -Z: X ring is edge-on and should be hidden.
    let axis_x = axes[0].normalize_or_zero();
    let (ux, _vx) = plane_basis(axis_x);
    let px = project_point(
        view_proj,
        vp,
        origin + ux * radius_world,
        gizmo.config.depth_range,
    )
    .unwrap()
    .screen;
    assert!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, px, axes, radius_world)
            .is_none(),
        "edge-on ring should not be pickable when faded"
    );

    // Z ring faces the camera and should remain pickable.
    let axis_z = axes[2].normalize_or_zero();
    let (uz, _vz) = plane_basis(axis_z);
    let pz = project_point(
        view_proj,
        vp,
        origin + uz * radius_world,
        gizmo.config.depth_range,
    )
    .unwrap()
    .screen;
    let hit = gizmo
        .pick_rotate_axis(view_proj, vp, origin, pz, axes, radius_world)
        .unwrap();
    assert_eq!(hit.handle, RotateHandle::AxisZ.id());
}

#[test]
fn rotate_view_ring_does_not_steal_axis_ring_when_both_hit() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    // Use an axis-aligned camera so the view ring and the Z axis ring are coplanar.
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 5.0));
    let origin = Vec3::ZERO;

    let mut config = GizmoConfig {
        mode: GizmoMode::Rotate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.pick_radius_px = 18.0;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.show_arcball = false;
    // Force the view ring to coincide with the most camera-facing axis ring so we can
    // deterministically hit both at the same cursor point.
    let mut gizmo = Gizmo::new(config);
    let mut pv = gizmo.part_visuals();
    pv.rotate_view_ring_radius_scale = 1.0;
    gizmo.set_part_visuals(pv);

    let mut axis_only_cfg = config;
    axis_only_cfg.show_view_axis_ring = false;
    let axis_only = Gizmo::new(axis_only_cfg);

    let mut view_only_cfg = config;
    view_only_cfg.axis_mask = [true; 3];
    let mut view_only = Gizmo::new(view_only_cfg);
    let mut pv = view_only.part_visuals();
    pv.rotate_view_ring_radius_scale = 1.0;
    view_only.set_part_visuals(pv);

    let axes = gizmo.axis_dirs(&Transform3d::default());
    let radius_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let view_dir = view_dir_at_origin(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
    let view_dir_n = view_dir.normalize_or_zero();
    assert!(view_dir_n.length_squared() > 0.0);

    let best_axis_index = axes
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            view_dir_n
                .dot(a.normalize_or_zero())
                .abs()
                .partial_cmp(&view_dir_n.dot(b.normalize_or_zero()).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| idx)
        .unwrap();
    let axis_handle = match best_axis_index {
        0 => RotateHandle::AxisX.id(),
        1 => RotateHandle::AxisY.id(),
        2 => RotateHandle::AxisZ.id(),
        _ => RotateHandle::AxisX.id(),
    };

    let (u, _v) = plane_basis(view_dir_n);
    let cursor = project_point(
        view_proj,
        vp,
        origin + u * radius_world,
        gizmo.config.depth_range,
    )
    .unwrap()
    .screen;

    let axis_hit = axis_only
        .pick_rotate_axis(view_proj, vp, origin, cursor, axes, radius_world)
        .unwrap();
    assert_eq!(axis_hit.handle, axis_handle);
    assert!(
        axis_hit.score <= gizmo.config.pick_radius_px * 0.35,
        "expected a strong axis-ring hit score={}, pick_radius={}",
        axis_hit.score,
        gizmo.config.pick_radius_px
    );

    let view_hit = view_only
        .pick_rotate_axis(view_proj, vp, origin, cursor, axes, radius_world)
        .unwrap();
    assert_eq!(view_hit.handle, Gizmo::ROTATE_VIEW_HANDLE);

    let hit = gizmo
        .pick_rotate_axis(view_proj, vp, origin, cursor, axes, radius_world)
        .unwrap();
    assert_eq!(hit.handle, axis_handle);
}

#[test]
fn arcball_drag_returns_to_identity_when_cursor_returns() {
    let mut gizmo = base_gizmo(GizmoMode::Rotate);
    gizmo.config.show_arcball = true;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;

    let center = project_point(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
    let r = gizmo.config.size_px * gizmo.config.arcball_radius_scale;
    assert!(r > 10.0);

    let cursor_start = center.screen + Vec2::new(r * 0.25, 0.0);
    let cursor_moved = center.screen + Vec2::new(0.0, r * 0.25);

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Arcball { total, .. } => total,
        _ => panic!("expected arcball"),
    };
    assert!(
        moved.updated_targets[0]
            .transform
            .rotation
            .dot(moved_total)
            .abs()
            > 1.0 - 1e-3,
        "updated={:?} total={moved_total:?}",
        moved.updated_targets[0].transform.rotation
    );
    let moved_angle = 2.0
        * moved_total
            .dot(Quat::IDENTITY)
            .abs()
            .clamp(-1.0, 1.0)
            .acos();
    assert!(moved_angle.is_finite());
    assert!(moved_angle > 1e-5);

    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Arcball { total, .. } => total,
        _ => panic!("expected arcball"),
    };
    let back_angle = 2.0 * back_total.dot(Quat::IDENTITY).abs().clamp(-1.0, 1.0).acos();
    assert!(
        back_angle.abs() < 5e-3,
        "angle={back_angle} total={back_total:?}"
    );
}

#[test]
fn rotate_axis_ring_is_pickable_when_partially_behind_camera() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    // Put the camera extremely close to the origin so a large axis ring can wrap behind it.
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.20, 0.0, 0.20));
    let origin = Vec3::ZERO;

    let mut config = GizmoConfig {
        mode: GizmoMode::Rotate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.pick_radius_px = 18.0;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.show_view_axis_ring = false;
    config.show_arcball = false;
    // Only enable the X axis ring so the expected handle is deterministic.
    config.axis_mask = [false, true, true];
    let gizmo = Gizmo::new(config);

    let axes = gizmo.axis_dirs(&Transform3d::default());

    // Make the ring radius larger than the camera distance so some ring points are behind the
    // camera (project_point returns None), while the opposite arc remains in front.
    let radius_world = 1.0;
    let front = project_point(
        view_proj,
        vp,
        origin + Vec3::new(0.0, 0.0, -radius_world),
        gizmo.config.depth_range,
    )
    .unwrap();

    let hit = gizmo
        .pick_rotate_axis(view_proj, vp, origin, front.screen, axes, radius_world)
        .expect("expected visible arc of the X axis ring to remain pickable");
    assert_eq!(hit.handle, RotateHandle::AxisX.id());
}

#[test]
fn scale_axis_drag_returns_to_one_when_cursor_returns() {
    let mut gizmo = base_gizmo(GizmoMode::Scale);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let axis_dir = axes[0].normalize_or_zero();
    assert!(axis_dir.length_squared() > 0.0);
    let p_start_world = origin + axis_dir * length_world;
    let p_move_world = origin + axis_dir * (length_world * 1.35);

    let p_start = project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(moved_total.x.is_finite());
    assert!(moved_total.x > 1.0 + 1e-6);
    assert!(
        moved.updated_targets[0]
            .transform
            .scale
            .distance(moved_total)
            < 1e-3,
        "updated={:?} total={moved_total:?}",
        moved.updated_targets[0].transform.scale
    );

    let input_back = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(
        (back_total - Vec3::ONE).length() < 1e-3,
        "total={back_total:?}"
    );
    assert!(
        (back.updated_targets[0].transform.scale - Vec3::ONE).length() < 1e-3,
        "updated={:?}",
        back.updated_targets[0].transform.scale
    );
}

#[test]
fn scale_axis_scales_multiple_targets_about_pivot() {
    let mut gizmo = base_gizmo(GizmoMode::Scale);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let axis_dir = axes[0].normalize_or_zero();
    assert!(axis_dir.length_squared() > 0.0);
    let p_start_world = origin + axis_dir * length_world;
    let p_move_world = origin + axis_dir * (length_world * 1.35);

    let p_start = project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [
        GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d::default(),
            local_bounds: None,
        },
        GizmoTarget3d {
            id: GizmoTargetId(2),
            transform: Transform3d {
                translation: Vec3::new(2.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.0, 2.0, 3.0),
            },
            local_bounds: None,
        },
    ];

    let input_down = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(moved_total.x.is_finite());
    assert!((moved_total.y - 1.0).abs() < 1e-5);
    assert!((moved_total.z - 1.0).abs() < 1e-5);
    assert!(moved_total.x > 1.0 + 1e-6);

    let t2 = moved.updated_targets.iter().find(|t| t.id.0 == 2).unwrap();
    let expected_translation = Vec3::new(2.0 * moved_total.x, 0.0, 0.0);
    assert!(
        t2.transform.translation.distance(expected_translation) < 1e-3,
        "translation={:?} expected={:?}",
        t2.transform.translation,
        expected_translation
    );
    let expected_scale = Vec3::new(1.0 * moved_total.x, 2.0, 3.0);
    assert!(
        t2.transform.scale.distance(expected_scale) < 1e-3,
        "scale={:?} expected={:?}",
        t2.transform.scale,
        expected_scale
    );

    // Host may feed back intermediate transforms; returning the cursor should still restore
    // the original transforms.
    let moved_targets = moved.updated_targets.clone();
    let input_back = GizmoInput {
        cursor_px: p_start.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &moved_targets)
        .unwrap();

    let back_total = match back.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(
        (back_total - Vec3::ONE).length() < 5e-3,
        "total={back_total:?}"
    );

    for t in &back.updated_targets {
        let start = targets.iter().find(|s| s.id == t.id).unwrap();
        assert!(
            t.transform
                .translation
                .distance(start.transform.translation)
                < 5e-3,
            "id={:?} translation={:?} start={:?}",
            t.id,
            t.transform.translation,
            start.transform.translation
        );
        assert!(
            t.transform.scale.distance(start.transform.scale) < 5e-3,
            "id={:?} scale={:?} start={:?}",
            t.id,
            t.transform.scale,
            start.transform.scale
        );
    }
}

#[test]
fn scale_plane_drag_returns_to_one_when_cursor_returns() {
    let mut gizmo = base_gizmo(GizmoMode::Scale);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let pv = gizmo.part_visuals();
    let off = length_world * pv.scale_plane_offset_fraction.max(0.0);
    let size = length_world * pv.scale_plane_size_fraction.max(0.0);
    let quad_world = translate_plane_quad_world(origin, axes[0], axes[1], off, size);
    let quad_screen = project_quad(view_proj, vp, quad_world, gizmo.config.depth_range).unwrap();
    let cursor_start = (quad_screen[0] + quad_screen[1] + quad_screen[2] + quad_screen[3]) * 0.25;
    let cursor_moved = cursor_start + Vec2::new(30.0, -20.0);

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(moved_total.x.is_finite());
    assert!(moved_total.y.is_finite());
    assert!((moved_total.x - 1.0).abs() > 1e-6 || (moved_total.y - 1.0).abs() > 1e-6);
    assert!(
        moved.updated_targets[0]
            .transform
            .scale
            .distance(moved_total)
            < 1e-3,
        "updated={:?} total={moved_total:?}",
        moved.updated_targets[0].transform.scale
    );

    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(
        (back_total - Vec3::ONE).length() < 1e-3,
        "total={back_total:?}"
    );
    assert!(
        (back.updated_targets[0].transform.scale - Vec3::ONE).length() < 1e-3,
        "updated={:?}",
        back.updated_targets[0].transform.scale
    );
}

#[test]
fn scale_plane_scales_multiple_targets_about_pivot() {
    let mut gizmo = base_gizmo(GizmoMode::Scale);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());
    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let pv = gizmo.part_visuals();
    let off = length_world * pv.scale_plane_offset_fraction.max(0.0);
    let size = length_world * pv.scale_plane_size_fraction.max(0.0);
    let quad_world = translate_plane_quad_world(origin, axes[0], axes[1], off, size);
    let quad_screen = project_quad(view_proj, vp, quad_world, gizmo.config.depth_range).unwrap();
    let cursor_start = (quad_screen[0] + quad_screen[1] + quad_screen[2] + quad_screen[3]) * 0.25;
    let cursor_moved = cursor_start + Vec2::new(30.0, -20.0);

    let targets = [
        GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d::default(),
            local_bounds: None,
        },
        GizmoTarget3d {
            id: GizmoTargetId(2),
            transform: Transform3d {
                translation: Vec3::new(1.0, 2.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.5, 2.5, 3.5),
            },
            local_bounds: None,
        },
    ];

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(moved_total.x.is_finite());
    assert!(moved_total.y.is_finite());
    assert!((moved_total.z - 1.0).abs() < 1e-5);

    let t2 = moved.updated_targets.iter().find(|t| t.id.0 == 2).unwrap();
    let expected_translation = Vec3::new(1.0 * moved_total.x, 2.0 * moved_total.y, 0.0);
    assert!(
        t2.transform.translation.distance(expected_translation) < 1e-3,
        "translation={:?} expected={:?}",
        t2.transform.translation,
        expected_translation
    );
    let expected_scale = Vec3::new(1.5 * moved_total.x, 2.5 * moved_total.y, 3.5);
    assert!(
        t2.transform.scale.distance(expected_scale) < 1e-3,
        "scale={:?} expected={:?}",
        t2.transform.scale,
        expected_scale
    );

    let moved_targets = moved.updated_targets.clone();
    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &moved_targets)
        .unwrap();

    let back_total = match back.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(
        (back_total - Vec3::ONE).length() < 5e-3,
        "total={back_total:?}"
    );

    for t in &back.updated_targets {
        let start = targets.iter().find(|s| s.id == t.id).unwrap();
        assert!(
            t.transform
                .translation
                .distance(start.transform.translation)
                < 5e-3,
            "id={:?} translation={:?} start={:?}",
            t.id,
            t.transform.translation,
            start.transform.translation
        );
        assert!(
            t.transform.scale.distance(start.transform.scale) < 5e-3,
            "id={:?} scale={:?} start={:?}",
            t.id,
            t.transform.scale,
            start.transform.scale
        );
    }
}

#[test]
fn bounds_face_scale_returns_to_one_when_cursor_returns() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let mut config = GizmoConfig {
        mode: GizmoMode::Scale,
        ..Default::default()
    };
    config.pivot_mode = GizmoPivotMode::Center;
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.show_bounds = true;
    // Avoid conflicts with axis/plane scale handles in this test.
    config.axis_mask = [true; 3];

    let mut gizmo = Gizmo::new(config);

    let targets = [
        GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d {
                translation: Vec3::new(-1.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            local_bounds: None,
        },
        GizmoTarget3d {
            id: GizmoTargetId(2),
            transform: Transform3d {
                translation: Vec3::new(1.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            local_bounds: None,
        },
    ];

    let origin = Vec3::ZERO;
    let axes_raw = gizmo.axis_dirs(&Transform3d::default());
    let basis = [
        axes_raw[0].normalize_or_zero(),
        axes_raw[1].normalize_or_zero(),
        axes_raw[2].normalize_or_zero(),
    ];
    let size_length_world = gizmo.size_length_world_or_one(view_proj, vp, origin, &targets);
    let (min_local, max_local) =
        gizmo.bounds_min_max_local(view_proj, vp, origin, basis, size_length_world, &targets);
    let center_local = (min_local + max_local) * 0.5;

    // Drag the +X face handle.
    let handle_local = Vec3::new(max_local.x, center_local.y, center_local.z);
    let handle_world =
        origin + basis[0] * handle_local.x + basis[1] * handle_local.y + basis[2] * handle_local.z;
    let handle_screen =
        project_point(view_proj, vp, handle_world, gizmo.config.depth_range).unwrap();
    let cursor_start = handle_screen.screen;
    let cursor_moved = cursor_start + Vec2::new(40.0, 0.0);

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(moved_total.is_finite());
    assert!(moved_total.distance(Vec3::ONE) > 1e-5);
    let moved_targets = moved.updated_targets.clone();

    let input_back = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &moved_targets)
        .unwrap();

    let back_total = match back.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(
        back_total.distance(Vec3::ONE) < 5e-3,
        "total={back_total:?}"
    );

    for t in &back.updated_targets {
        let start = targets.iter().find(|s| s.id == t.id).unwrap();
        assert!(
            t.transform
                .translation
                .distance(start.transform.translation)
                < 5e-3,
            "id={:?} translation={:?} start={:?}",
            t.id,
            t.transform.translation,
            start.transform.translation
        );
        assert!(
            t.transform.scale.distance(start.transform.scale) < 5e-3,
            "id={:?} scale={:?} start={:?}",
            t.id,
            t.transform.scale,
            start.transform.scale
        );
    }
}

#[test]
fn bounds_face_scale_snaps_to_extent_step_when_enabled() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let mut config = GizmoConfig {
        mode: GizmoMode::Scale,
        ..Default::default()
    };
    config.pivot_mode = GizmoPivotMode::Center;
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.show_bounds = true;
    config.bounds_snap_step = Some(Vec3::splat(0.5));
    // Avoid conflicts with axis/plane scale handles in this test.
    config.axis_mask = [true; 3];

    let mut gizmo = Gizmo::new(config);

    let targets = [
        GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d {
                translation: Vec3::new(-1.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            local_bounds: None,
        },
        GizmoTarget3d {
            id: GizmoTargetId(2),
            transform: Transform3d {
                translation: Vec3::new(1.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            local_bounds: None,
        },
    ];

    let origin = Vec3::ZERO;
    let basis = [Vec3::X, Vec3::Y, Vec3::Z];
    let size_length_world = gizmo.size_length_world_or_one(view_proj, vp, origin, &targets);
    let (min_local, max_local) =
        gizmo.bounds_min_max_local(view_proj, vp, origin, basis, size_length_world, &targets);
    let center_local = (min_local + max_local) * 0.5;
    let start_extent_x = (max_local.x - min_local.x).abs().max(1e-6);

    // Drag the +X face handle with snapping enabled.
    let handle_local = Vec3::new(max_local.x, center_local.y, center_local.z);
    let handle_world = origin + handle_local;
    let handle_screen =
        project_point(view_proj, vp, handle_world, gizmo.config.depth_range).unwrap();
    let cursor_start = handle_screen.screen;
    let cursor_moved = cursor_start + Vec2::new(240.0, 0.0);

    let input_down = GizmoInput {
        cursor_px: cursor_start,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: true,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: cursor_moved,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: true,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };

    let extent_x = start_extent_x * moved_total.x;
    let snapped = (extent_x / 0.5).round() * 0.5;
    assert!(
        (extent_x - snapped).abs() < 2e-3,
        "extent_x={extent_x} snapped={snapped}"
    );
    assert!((moved_total.x - 1.0).abs() > 1e-6, "total={moved_total:?}");
}

#[test]
fn scale_prefers_bounds_face_handle_over_axis_end_box_when_overlapping() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let mut config = GizmoConfig {
        mode: GizmoMode::Scale,
        ..Default::default()
    };
    config.pivot_mode = GizmoPivotMode::Center;
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.show_bounds = true;
    let gizmo = Gizmo::new(config);

    let origin = Vec3::ZERO;
    let axes_raw = gizmo.axis_dirs(&Transform3d::default());
    let axes_flipped = axes_raw;

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    // Construct a selection whose bounds +X face center coincides with the X axis end-box.
    let targets = [
        GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d {
                translation: Vec3::new(-length_world, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            local_bounds: None,
        },
        GizmoTarget3d {
            id: GizmoTargetId(2),
            transform: Transform3d {
                translation: Vec3::new(length_world, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            local_bounds: None,
        },
    ];

    let cursor_world = origin + axes_raw[0].normalize_or_zero() * length_world;
    let cursor = project_point(view_proj, vp, cursor_world, gizmo.config.depth_range)
        .unwrap()
        .screen;

    let hit = gizmo
        .pick_scale_or_bounds_handle(
            view_proj,
            vp,
            origin,
            cursor,
            axes_flipped,
            axes_raw,
            length_world,
            &targets,
        )
        .unwrap();
    assert_eq!(hit.handle, Gizmo::bounds_face_id(0, true));
}

#[test]
fn bounds_uses_local_bounds_when_provided() {
    let gizmo = base_gizmo(GizmoMode::Scale);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let basis = [Vec3::X, Vec3::Y, Vec3::Z];

    let aabb = Aabb3 {
        min: Vec3::new(-2.0, -1.0, -3.0),
        max: Vec3::new(2.0, 1.0, 3.0),
    };
    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: Some(aabb),
    }];

    let size_length_world = gizmo.size_length_world_or_one(view_proj, vp, origin, &targets);
    let (min_local, max_local) =
        gizmo.bounds_min_max_local(view_proj, vp, origin, basis, size_length_world, &targets);

    assert!((min_local.x + 2.0).abs() < 1e-3, "min_local={min_local:?}");
    assert!((min_local.y + 1.0).abs() < 1e-3, "min_local={min_local:?}");
    assert!((min_local.z + 3.0).abs() < 1e-3, "min_local={min_local:?}");
    assert!((max_local.x - 2.0).abs() < 1e-3, "max_local={max_local:?}");
    assert!((max_local.y - 1.0).abs() < 1e-3, "max_local={max_local:?}");
    assert!((max_local.z - 3.0).abs() < 1e-3, "max_local={max_local:?}");
}

#[test]
fn bounds_snap_snaps_extent_not_factor() {
    let snapped = snap_bounds_extent_factor(3.0, 1.1, 0.5);
    assert!((snapped - (3.5 / 3.0)).abs() < 1e-6, "snapped={snapped}");
}

#[test]
fn scale_uniform_drag_returns_to_one_when_cursor_returns() {
    let mut gizmo = base_gizmo(GizmoMode::Scale);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let p0 = project_point(view_proj, vp, origin, gizmo.config.depth_range).unwrap();

    let radius_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();
    let view_dir = view_dir_at_origin(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
    let (u, v) = plane_basis(view_dir);
    let dir = (u + v).normalize_or_zero();
    assert!(dir.length_squared() > 0.0);
    let p_move_world = origin + dir * (radius_world * 0.6);
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: None,
    }];

    let input_down = GizmoInput {
        cursor_px: p0.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(moved_total.x.is_finite());
    assert!(moved_total.x > 1.0 + 1e-6);
    assert!((moved_total.x - moved_total.y).abs() < 1e-5);
    assert!((moved_total.x - moved_total.z).abs() < 1e-5);

    let input_back = GizmoInput {
        cursor_px: p0.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let back = gizmo
        .update(view_proj, vp, input_back, targets[0].id, &targets)
        .unwrap();
    let back_total = match back.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(
        (back_total - Vec3::ONE).length() < 1e-3,
        "total={back_total:?}"
    );
}

#[test]
fn scale_uniform_scales_multiple_targets_about_pivot() {
    let mut gizmo = base_gizmo(GizmoMode::Scale);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));

    let origin = Vec3::ZERO;
    let p0 = project_point(view_proj, vp, origin, gizmo.config.depth_range).unwrap();

    let radius_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();
    let view_dir = view_dir_at_origin(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
    let (u, v) = plane_basis(view_dir);
    let dir = (u + v).normalize_or_zero();
    assert!(dir.length_squared() > 0.0);
    let p_move_world = origin + dir * (radius_world * 0.6);
    let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

    let targets = [
        GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d::default(),
            local_bounds: None,
        },
        GizmoTarget3d {
            id: GizmoTargetId(2),
            transform: Transform3d {
                translation: Vec3::new(1.0, 2.0, 3.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(2.0, 3.0, 4.0),
            },
            local_bounds: None,
        },
    ];

    let input_down = GizmoInput {
        cursor_px: p0.screen,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    let input_move = GizmoInput {
        cursor_px: p_move.screen,
        hovered: true,
        drag_started: false,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let moved = gizmo
        .update(view_proj, vp, input_move, targets[0].id, &targets)
        .unwrap();
    let moved_total = match moved.result {
        GizmoResult::Scale { total, .. } => total,
        _ => panic!("expected scale"),
    };
    assert!(moved_total.x.is_finite());
    assert!((moved_total.x - moved_total.y).abs() < 1e-5);
    assert!((moved_total.x - moved_total.z).abs() < 1e-5);
    assert!(moved_total.x > 1.0 + 1e-6);
    let factor = moved_total.x;

    let t2 = moved.updated_targets.iter().find(|t| t.id.0 == 2).unwrap();
    let expected_translation = Vec3::new(1.0 * factor, 2.0 * factor, 3.0 * factor);
    assert!(
        t2.transform.translation.distance(expected_translation) < 1e-3,
        "translation={:?} expected={:?}",
        t2.transform.translation,
        expected_translation
    );
    let expected_scale = Vec3::new(2.0 * factor, 3.0 * factor, 4.0 * factor);
    assert!(
        t2.transform.scale.distance(expected_scale) < 1e-3,
        "scale={:?} expected={:?}",
        t2.transform.scale,
        expected_scale
    );
}

#[test]
fn universal_picks_scale_on_end_box() {
    let gizmo = base_gizmo(GizmoMode::Universal);
    assert!(gizmo.config.universal_includes_scale);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let end_world = origin + axes[0] * length_world;
    let end = project_point(view_proj, vp, end_world, gizmo.config.depth_range).unwrap();

    assert!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, end.screen, axes, length_world)
            .is_some(),
        "expected rotate rings to be pickable near the scale end box in this projection"
    );

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, end.screen, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Scale);
    assert_eq!(hit.handle, ScaleHandle::AxisX.id());
}

#[test]
fn universal_picks_translate_on_arrow_tip() {
    let gizmo = base_gizmo(GizmoMode::Universal);
    assert!(gizmo.config.universal_includes_scale);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let tip_world = origin + axes[0] * (length_world * Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE);
    let tip = project_point(view_proj, vp, tip_world, gizmo.config.depth_range).unwrap();

    assert!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, tip.screen, axes, length_world)
            .is_some(),
        "expected rotate rings to be pickable near the translate tip in this projection"
    );

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, tip.screen, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, TranslateHandle::AxisX.id());
}

#[test]
fn universal_can_pick_translate_depth_when_enabled() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    gizmo.config.universal_includes_translate_depth = true;
    gizmo.config.size_px = 120.0;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let view_dir = view_dir_at_origin(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
    let (u, _v) = plane_basis(view_dir);
    let pv = gizmo.part_visuals();
    let r_world = (length_world * pv.translate_depth_ring_radius_fraction.max(0.0))
        .max(length_world * pv.translate_depth_ring_radius_min_fraction.max(0.0));
    let ring_px = project_point(
        view_proj,
        vp,
        origin + u.normalize_or_zero() * r_world,
        gizmo.config.depth_range,
    )
    .unwrap()
    .screen;

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, ring_px, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, TranslateHandle::Depth.id());
}

#[test]
fn universal_translate_tip_intent_works_in_orthographic() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    assert!(gizmo.config.universal_includes_scale);
    // In orthographic projection the translate arrow tip is further from the rotate ring in
    // screen space than in perspective. Reduce the overall gizmo size so the default
    // `pick_radius_px` can still cover the tip-intent window.
    gizmo.config.size_px = 32.0;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_ortho((800.0, 600.0), Vec3::new(3.0, 2.0, 4.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    // Use the translate arrow tip position; in this orthographic projection, ensure the rotate
    // ring is still within pick radius so the overlap resolution logic is exercised.
    let tip_world = origin + axes[0] * (length_world * Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE);
    let tip = project_point(view_proj, vp, tip_world, gizmo.config.depth_range).unwrap();
    assert!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, tip.screen, axes, length_world)
            .is_some(),
        "expected rotate rings to be pickable near the translate tip in this orthographic projection"
    );

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, tip.screen, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, TranslateHandle::AxisX.id());
}

#[test]
fn universal_translate_tip_intent_works_with_wide_fov() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    assert!(gizmo.config.universal_includes_scale);

    // Keep the gizmo reasonably small so the overlap window between rotate rings and translate
    // arrow tips stays within the default pick radius at very wide FOV.
    gizmo.config.size_px = 48.0;
    // Wide FOV projections can spread sub-handles in screen space. Inflate the pick radius so the
    // rotate ring is still considered near the translate tip, ensuring the universal overlap
    // policy is exercised in this projection.
    gizmo.config.pick_radius_px = 64.0;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 160.0, Vec3::new(0.8, 0.6, 1.2));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let tip_world = origin + axes[0] * (length_world * Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE);
    let tip = project_point(view_proj, vp, tip_world, gizmo.config.depth_range).unwrap();

    assert!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, tip.screen, axes, length_world)
            .is_some(),
        "expected rotate rings to be considered near the translate tip at wide FOV"
    );

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, tip.screen, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, TranslateHandle::AxisX.id());
}

#[test]
fn universal_translate_tip_intent_works_with_close_camera_near_plane() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    assert!(gizmo.config.universal_includes_scale);

    // When the camera is very close to the gizmo origin, tiny projection changes can make
    // overlap windows narrower. Increase pick radius so this test exercises the overlap
    // resolution rather than "did we happen to land on the ring polyline".
    gizmo.config.pick_radius_px = 48.0;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    // Eye is close to origin, but still in front of the near plane used by `test_view_projection_fov`.
    let view_proj = test_view_projection_fov((800.0, 600.0), 75.0, Vec3::new(0.0, 0.0, 0.20));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let tip_world = origin + axes[0] * (length_world * Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE);
    let tip = project_point(view_proj, vp, tip_world, gizmo.config.depth_range).unwrap();

    assert!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, tip.screen, axes, length_world)
            .is_some(),
        "expected rotate rings to remain pickable near the translate tip with a close camera"
    );

    // At very close camera distances, projected planes can overlap axis tips. Universal is
    // expected to keep translate planes "protected" over rotate rings in these cases.
    let translate = gizmo
        .pick_translate_handle(
            view_proj,
            vp,
            origin,
            tip.screen,
            axes,
            length_world,
            true,
            true,
            true,
            false,
        )
        .expect("expected some translate handle at this cursor position");

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, tip.screen, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, translate.handle);
}

#[test]
fn universal_translate_tip_intent_works_in_orthographic_with_close_camera() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    assert!(gizmo.config.universal_includes_scale);
    gizmo.config.pick_radius_px = 48.0;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_ortho((800.0, 600.0), Vec3::new(0.0, 0.0, 0.20));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    // Ensure the rotate ring sampling encounters near-plane / behind-camera clipping in this
    // orthographic + close camera setup (i.e. not all ring points are inside the clip volume).
    let sample = project_point(
        view_proj,
        vp,
        origin + Vec3::Z * length_world,
        gizmo.config.depth_range,
    );
    assert!(
        sample.is_none() || !sample.unwrap().inside_clip,
        "expected some rotate ring points to be clipped near the camera in this setup"
    );

    let tip_world = origin + axes[0] * (length_world * Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE);
    let tip = project_point(view_proj, vp, tip_world, gizmo.config.depth_range).unwrap();

    assert!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, tip.screen, axes, length_world)
            .is_some(),
        "expected rotate rings to remain pickable near the translate tip with a close orthographic camera"
    );

    let translate = gizmo
        .pick_translate_handle(
            view_proj,
            vp,
            origin,
            tip.screen,
            axes,
            length_world,
            true,
            true,
            true,
            false,
        )
        .expect("expected some translate handle at this cursor position");

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, tip.screen, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, translate.handle);
}

#[test]
fn universal_translate_tip_intent_protects_against_view_ring_overlap() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    assert!(gizmo.config.universal_includes_scale);

    // Axis-aligned view so the view ring basis aligns with world X/Y.
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 5.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    // Force the view ring to pass through the translate arrow tip position so picking overlaps.
    let mut pv = gizmo.part_visuals();
    pv.rotate_view_ring_radius_scale = Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE;
    gizmo.set_part_visuals(pv);

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let tip_world = origin + axes[0] * (length_world * Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE);
    let tip = project_point(view_proj, vp, tip_world, gizmo.config.depth_range).unwrap();

    assert_eq!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, tip.screen, axes, length_world)
            .unwrap()
            .handle,
        RotateHandle::View.id(),
        "expected view ring to overlap the translate arrow tip in this configuration"
    );

    let translate = gizmo
        .pick_translate_handle(
            view_proj,
            vp,
            origin,
            tip.screen,
            axes,
            length_world,
            true,
            true,
            true,
            false,
        )
        .expect("expected some translate handle at this cursor position");

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, tip.screen, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, translate.handle);
}

#[test]
fn universal_translate_tip_intent_protects_against_view_ring_overlap_in_orthographic() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    assert!(gizmo.config.universal_includes_scale);

    // Disable axis rings so rotate picking deterministically returns the view ring hit.
    gizmo.config.rotate_ring_fade_dot = (1.10, 1.20);
    gizmo.config.universal_includes_rotate_view_ring = true;
    gizmo.config.universal_includes_arcball = false;

    // Axis-aligned orthographic view so the view ring basis aligns with world X/Y.
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_ortho((800.0, 600.0), Vec3::new(0.0, 0.0, 0.20));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    // Force the view ring to pass through the translate arrow tip position so picking overlaps.
    let mut pv = gizmo.part_visuals();
    pv.rotate_view_ring_radius_scale = Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE;
    gizmo.set_part_visuals(pv);

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let tip_world = origin + axes[0] * (length_world * Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE);
    let tip = project_point(view_proj, vp, tip_world, gizmo.config.depth_range).unwrap();

    assert_eq!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, tip.screen, axes, length_world)
            .unwrap()
            .handle,
        RotateHandle::View.id(),
        "expected view ring to overlap the translate arrow tip in this orthographic projection"
    );

    let translate = gizmo
        .pick_translate_handle(
            view_proj,
            vp,
            origin,
            tip.screen,
            axes,
            length_world,
            true,
            true,
            true,
            false,
        )
        .expect("expected some translate handle at this cursor position");

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, tip.screen, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, translate.handle);
}

#[test]
fn universal_translate_screen_handle_wins_over_arcball_at_origin() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    gizmo.config.universal_includes_arcball = true;
    gizmo.config.universal_includes_rotate_view_ring = false;

    // Hide axis rotation rings so rotate picking can fall through to arcball.
    gizmo.config.rotate_ring_fade_dot = (1.10, 1.20);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 5.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let center = project_point(view_proj, vp, origin, gizmo.config.depth_range)
        .unwrap()
        .screen;

    assert_eq!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, center, axes, length_world)
            .unwrap()
            .handle,
        RotateHandle::Arcball.id(),
        "expected arcball to be pickable at the origin when axis rings and view ring are disabled"
    );

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, center, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, TranslateHandle::Screen.id());
}

#[test]
fn universal_view_ring_overlap_is_stable_with_pixels_clamped_size_policy() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    assert!(gizmo.config.universal_includes_scale);

    gizmo.config.size_policy = GizmoSizePolicy::PixelsClampedBySelectionBounds {
        min_fraction_of_max_extent: 0.01,
        max_fraction_of_max_extent: 0.02,
    };

    // Disable axis rings so rotate picking deterministically returns the view ring hit.
    gizmo.config.rotate_ring_fade_dot = (1.10, 1.20);
    gizmo.config.universal_includes_rotate_view_ring = true;
    gizmo.config.universal_includes_arcball = false;

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 5.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    // Tiny selection bounds force the size policy to clamp down hard.
    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: Some(Aabb3 {
            min: Vec3::splat(-0.5),
            max: Vec3::splat(0.5),
        }),
    }];

    // Force the view ring to pass through the translate arrow tip position so picking overlaps.
    let mut pv = gizmo.part_visuals();
    pv.rotate_view_ring_radius_scale = Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE;
    gizmo.set_part_visuals(pv);

    let length_world = gizmo.size_length_world_or_one(view_proj, vp, origin, &targets);
    assert!(length_world.is_finite() && length_world > 0.0);

    let tip_world = origin + axes[0] * (length_world * Gizmo::UNIVERSAL_TRANSLATE_TIP_SCALE);
    let tip = project_point(view_proj, vp, tip_world, gizmo.config.depth_range).unwrap();

    assert_eq!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, tip.screen, axes, length_world)
            .unwrap()
            .handle,
        RotateHandle::View.id(),
        "expected view ring to overlap the translate arrow tip under clamped size policy"
    );

    let translate = gizmo
        .pick_translate_handle(
            view_proj,
            vp,
            origin,
            tip.screen,
            axes,
            length_world,
            true,
            true,
            true,
            false,
        )
        .expect("expected some translate handle at this cursor position");

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, tip.screen, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, translate.handle);
}

#[test]
fn universal_arcball_overlap_is_stable_with_selection_bounds_size_policy() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    gizmo.config.size_policy = GizmoSizePolicy::SelectionBounds {
        fraction_of_max_extent: 0.25,
    };
    gizmo.config.universal_includes_arcball = true;
    gizmo.config.universal_includes_rotate_view_ring = false;

    // Hide axis rings so rotate picking can fall through to arcball, even when the size is
    // derived from bounds instead of pixels.
    gizmo.config.rotate_ring_fade_dot = (1.10, 1.20);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 5.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: Some(Aabb3 {
            min: Vec3::splat(-2.0),
            max: Vec3::splat(2.0),
        }),
    }];

    let length_world = gizmo.size_length_world_or_one(view_proj, vp, origin, &targets);
    assert!(length_world.is_finite() && length_world > 0.0);

    let center = project_point(view_proj, vp, origin, gizmo.config.depth_range)
        .unwrap()
        .screen;

    assert_eq!(
        gizmo
            .pick_rotate_axis(view_proj, vp, origin, center, axes, length_world)
            .unwrap()
            .handle,
        RotateHandle::Arcball.id(),
        "expected arcball to be pickable at the origin when axis rings and view ring are disabled"
    );

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, center, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, TranslateHandle::Screen.id());
}

#[test]
fn universal_can_pick_translate_plane_xy_inside() {
    let gizmo = base_gizmo(GizmoMode::Universal);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;
    let axes = gizmo.axis_dirs(&Transform3d::default());

    let length_world = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();

    let pv = gizmo.part_visuals();
    let off = length_world * pv.translate_plane_offset_fraction.max(0.0);
    let size = length_world * pv.translate_plane_size_fraction.max(0.0);
    let quad_world = translate_plane_quad_world(origin, axes[0], axes[1], off, size);
    let quad = project_quad(view_proj, vp, quad_world, gizmo.config.depth_range).unwrap();

    // Pick at the center of the plane quad.
    let center = (quad[0] + quad[2]) * 0.5;

    let (hit, kind) = gizmo
        .pick_universal_handle(view_proj, vp, origin, center, axes, length_world)
        .unwrap();
    assert_eq!(kind, GizmoMode::Translate);
    assert_eq!(hit.handle, TranslateHandle::PlaneXY.id());
}

#[test]
fn universal_is_not_pickable_when_origin_is_behind_camera() {
    let gizmo = base_gizmo(GizmoMode::Universal);
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));

    let view_proj = test_view_projection_fov((800.0, 600.0), 60.0, Vec3::new(0.0, 0.0, 1.0));
    let origin = Vec3::new(0.0, 0.0, 2.0);
    let cursor = Vec2::new(400.0, 300.0);
    let axes = gizmo.axis_dirs(&Transform3d::default());

    assert!(
        gizmo
            .pick_universal_handle(view_proj, vp, origin, cursor, axes, 1.0)
            .is_none()
    );
}

#[test]
fn universal_effective_ops_respects_universal_rotate_toggles() {
    let mut gizmo = base_gizmo(GizmoMode::Universal);
    gizmo.config.operation_mask = None;

    // Default: include rotate view ring, exclude arcball.
    let ops = gizmo.effective_ops();
    assert!(ops.contains(GizmoOps::rotate_view()));
    assert!(!ops.contains(GizmoOps::rotate_arcball()));

    gizmo.config.universal_includes_rotate_view_ring = false;
    gizmo.config.universal_includes_arcball = true;
    let ops = gizmo.effective_ops();
    assert!(!ops.contains(GizmoOps::rotate_view()));
    assert!(ops.contains(GizmoOps::rotate_arcball()));
}

#[test]
fn rotate_effective_ops_ignores_universal_rotate_toggles() {
    let mut gizmo = base_gizmo(GizmoMode::Rotate);
    gizmo.config.operation_mask = None;

    // Rotate mode uses the rotate-specific toggles.
    gizmo.config.show_view_axis_ring = false;
    gizmo.config.show_arcball = false;

    // Flip the universal toggles; rotate mode should ignore them.
    gizmo.config.universal_includes_rotate_view_ring = true;
    gizmo.config.universal_includes_arcball = true;

    let ops = gizmo.effective_ops();
    assert!(!ops.contains(GizmoOps::rotate_view()));
    assert!(!ops.contains(GizmoOps::rotate_arcball()));
}

#[test]
fn size_policy_pixels_clamped_by_selection_bounds_clamps_world_length() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;

    let mut config = GizmoConfig {
        mode: GizmoMode::Translate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.size_policy = GizmoSizePolicy::PixelsClampedBySelectionBounds {
        min_fraction_of_max_extent: 0.2,
        max_fraction_of_max_extent: 0.4,
    };
    let gizmo = Gizmo::new(config);

    let huge = Aabb3 {
        min: Vec3::splat(-50.0),
        max: Vec3::splat(50.0),
    };
    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: Some(huge),
    }];

    let base = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();
    let max_extent = (huge.max - huge.min).abs().max_element();
    let min_world = max_extent * 0.2;
    let max_world = max_extent * 0.4;
    let expected = base.clamp(min_world, max_world);

    let length_world = gizmo
        .size_length_world(view_proj, vp, origin, &targets)
        .unwrap();
    assert!(
        (length_world - expected).abs() < 1e-3,
        "length_world={length_world} expected={expected}"
    );
}

#[test]
fn size_policy_pixels_clamped_by_selection_bounds_can_clamp_down() {
    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let origin = Vec3::ZERO;

    let mut config = GizmoConfig {
        mode: GizmoMode::Translate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.size_policy = GizmoSizePolicy::PixelsClampedBySelectionBounds {
        min_fraction_of_max_extent: 0.2,
        max_fraction_of_max_extent: 0.4,
    };
    let gizmo = Gizmo::new(config);

    let tiny = Aabb3 {
        min: Vec3::splat(-0.05),
        max: Vec3::splat(0.05),
    };
    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d::default(),
        local_bounds: Some(tiny),
    }];

    let base = axis_length_world(
        view_proj,
        vp,
        origin,
        gizmo.config.depth_range,
        gizmo.config.size_px,
    )
    .unwrap();
    let max_extent = (tiny.max - tiny.min).abs().max_element();
    let min_world = max_extent * 0.2;
    let max_world = max_extent * 0.4;
    let expected = base.clamp(min_world, max_world);

    let length_world = gizmo
        .size_length_world(view_proj, vp, origin, &targets)
        .unwrap();
    assert!(
        (length_world - expected).abs() < 1e-3,
        "length_world={length_world} expected={expected}"
    );
}

#[test]
fn pivot_center_uses_selection_bounds_center_when_bounds_present() {
    let mut config = GizmoConfig {
        mode: GizmoMode::Translate,
        ..Default::default()
    };
    config.depth_range = DepthRange::ZeroToOne;
    config.drag_start_threshold_px = 0.0;
    config.allow_axis_flip = false;
    config.axis_fade_px = (f32::NAN, f32::NAN);
    config.plane_fade_px2 = (f32::NAN, f32::NAN);
    config.pivot_mode = GizmoPivotMode::Center;
    let mut gizmo = Gizmo::new(config);

    let aabb = Aabb3 {
        min: Vec3::new(-1.0, -1.0, -1.0),
        max: Vec3::new(3.0, 1.0, 1.0),
    };
    let targets = [GizmoTarget3d {
        id: GizmoTargetId(1),
        transform: Transform3d {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        local_bounds: Some(aabb),
    }];

    // Center should come from bounds, not translation average.
    let expected_origin = targets[0]
        .transform
        .to_mat4()
        .transform_point3((aabb.min + aabb.max) * 0.5);

    let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let view_proj = test_view_projection((800.0, 600.0));
    let center_px = project_point(view_proj, vp, expected_origin, gizmo.config.depth_range)
        .unwrap()
        .screen;

    let input_down = GizmoInput {
        cursor_px: center_px,
        hovered: true,
        drag_started: true,
        dragging: true,
        snap: false,
        cancel: false,
        precision: 1.0,
    };
    let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

    // If we used translation average instead, the center handle would not be hit here and the
    // drag would not activate.
    assert!(
        gizmo.state.active.is_some(),
        "expected gizmo to start using the center handle at bounds center"
    );
}

// Note: we do not currently assert a deterministic rotate-vs-translate ambiguity case in
// Universal, because in this projection the overlap region is too small and becomes brittle
// across minor changes to pick heuristics.

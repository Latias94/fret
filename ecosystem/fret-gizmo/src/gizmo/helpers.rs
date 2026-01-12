use super::*;

pub(super) fn axis_for_handle(handle: HandleId) -> (Vec3, usize) {
    match handle.0 {
        1 => (Vec3::X, 0),
        2 => (Vec3::Y, 1),
        3 => (Vec3::Z, 2),
        _ => (Vec3::X, 0),
    }
}

pub(super) fn plane_basis(normal: Vec3) -> (Vec3, Vec3) {
    let n = normal.normalize_or_zero();
    let a = if n.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let u = n.cross(a).normalize_or_zero();
    let v = n.cross(u).normalize_or_zero();
    (u, v)
}

pub(super) fn angle_on_plane(
    origin: Vec3,
    point: Vec3,
    axis_dir: Vec3,
    u: Vec3,
    v: Vec3,
) -> Option<f32> {
    let axis = axis_dir.normalize_or_zero();
    if axis.length_squared() == 0.0 {
        return None;
    }

    let w = point - origin;
    let w_plane = w - axis * w.dot(axis);
    if w_plane.length_squared() < 1e-6 {
        return None;
    }
    let w_plane = w_plane.normalize();
    let x = w_plane.dot(u);
    let y = w_plane.dot(v);
    Some(y.atan2(x))
}

pub(super) fn wrap_angle(mut a: f32) -> f32 {
    // Map to [-pi, pi] for stable incremental deltas.
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

pub(super) fn quat_axis_angle(q: Quat) -> (Vec3, f32) {
    let q = q.normalize();
    let (axis, angle) = q.to_axis_angle();
    let axis = axis.normalize_or_zero();
    if axis.length_squared() == 0.0 || !angle.is_finite() {
        (Vec3::Z, 0.0)
    } else {
        (axis, angle)
    }
}

pub(super) fn snap_quat_to_angle_step(total: Quat, step: f32) -> Quat {
    let step = step.max(1e-6);
    let (axis, angle) = quat_axis_angle(total);
    if angle.abs() < 1e-6 {
        return Quat::IDENTITY;
    }
    let snapped = (angle / step).round() * step;
    if snapped.abs() < 1e-6 {
        Quat::IDENTITY
    } else {
        Quat::from_axis_angle(axis, snapped).normalize()
    }
}

pub(super) fn snap_bounds_extent_factor(start_extent: f32, factor: f32, step: f32) -> f32 {
    if !start_extent.is_finite() || start_extent <= 0.0 {
        return factor;
    }
    if !factor.is_finite() {
        return factor;
    }
    if !step.is_finite() || step <= 0.0 {
        return factor;
    }

    let extent = start_extent.max(1e-6);
    let desired_extent = (extent * factor).max(0.0);
    let snapped_extent = (desired_extent / step).round() * step;
    (snapped_extent / extent).max(0.01)
}

pub(super) fn ray_plane_intersect(
    ray: Ray3d,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Option<Vec3> {
    let n = plane_normal.normalize_or_zero();
    if n.length_squared() == 0.0 {
        return None;
    }
    let denom = n.dot(ray.dir);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = n.dot(plane_point - ray.origin) / denom;
    if !t.is_finite() || t < 0.0 {
        return None;
    }
    Some(ray.origin + ray.dir * t)
}

pub(super) fn axis_drag_plane_normal(
    view_projection: Mat4,
    viewport: ViewportRect,
    depth: DepthRange,
    origin: Vec3,
    axis_dir: Vec3,
) -> Option<Vec3> {
    // Best-practice axis dragging: intersect the cursor ray against a plane that:
    // - passes through the gizmo origin
    // - contains the axis direction
    // - is as "camera-facing" as possible for stability
    //
    // A robust choice is: plane normal = axis x view_dir_at_origin.
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let view_ray = ray_from_screen(view_projection, viewport, p0.screen, depth)?;

    let axis = axis_dir.normalize_or_zero();
    if axis.length_squared() == 0.0 {
        return None;
    }
    let view_dir = view_ray.dir.normalize_or_zero();

    let mut n = axis.cross(view_dir);
    if n.length_squared() < 1e-6 {
        // Degenerate when viewing straight down the axis; pick a stable fallback basis.
        n = axis.cross(Vec3::Y);
        if n.length_squared() < 1e-6 {
            n = axis.cross(Vec3::X);
        }
    }
    let plane_normal = n.normalize_or_zero();
    (plane_normal.length_squared() > 0.0).then_some(plane_normal)
}

pub(super) fn axis_drag_plane_normal_facing_camera(
    view_projection: Mat4,
    viewport: ViewportRect,
    depth: DepthRange,
    origin: Vec3,
    axis_dir: Vec3,
) -> Option<Vec3> {
    // Camera-facing variant for axis dragging:
    // - plane passes through origin
    // - contains the axis direction (plane normal is orthogonal to axis)
    // - normal is as close to the view direction as possible (stable ray-plane intersection)
    //
    // A good choice is: n = axis x (axis x view_dir) = axis*(axis·view_dir) - view_dir.
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let view_ray = ray_from_screen(view_projection, viewport, p0.screen, depth)?;

    let axis = axis_dir.normalize_or_zero();
    if axis.length_squared() == 0.0 {
        return None;
    }
    let view_dir = view_ray.dir.normalize_or_zero();

    let mut n = axis.cross(axis.cross(view_dir));
    if n.length_squared() < 1e-6 {
        // Degenerate when viewing straight down the axis; pick a stable fallback basis.
        n = axis.cross(Vec3::Y);
        if n.length_squared() < 1e-6 {
            n = axis.cross(Vec3::X);
        }
    }
    let plane_normal = n.normalize_or_zero();
    (plane_normal.length_squared() > 0.0).then_some(plane_normal)
}

pub(super) fn axis_segment_len_px(
    view_projection: Mat4,
    viewport: ViewportRect,
    origin: Vec3,
    depth: DepthRange,
    axis_dir: Vec3,
    axis_len_world: f32,
) -> Option<f32> {
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let p1 = project_point(
        view_projection,
        viewport,
        origin + axis_dir * axis_len_world,
        depth,
    )?;
    let d = (p1.screen - p0.screen).length();
    d.is_finite().then_some(d)
}

pub(super) fn quad_area_px2(q: [Vec2; 4]) -> f32 {
    // Shoelace formula (absolute area of the convex quad).
    let mut a = 0.0f32;
    for i in 0..4 {
        let p0 = q[i];
        let p1 = q[(i + 1) % 4];
        a += p0.x * p1.y - p1.x * p0.y;
    }
    (a * 0.5).abs()
}

pub(super) fn mix_alpha(mut c: Color, a: f32) -> Color {
    c.a = (c.a * a).clamp(0.0, 1.0);
    c
}

pub(super) fn translate_plane_quad_world(
    origin: Vec3,
    u: Vec3,
    v: Vec3,
    off: f32,
    size: f32,
) -> [Vec3; 4] {
    let u = u.normalize_or_zero();
    let v = v.normalize_or_zero();
    let p0 = origin + u * off + v * off;
    let p1 = origin + u * (off + size) + v * off;
    let p2 = origin + u * (off + size) + v * (off + size);
    let p3 = origin + u * off + v * (off + size);
    [p0, p1, p2, p3]
}

pub(super) fn project_quad(
    view_projection: Mat4,
    viewport: ViewportRect,
    world: [Vec3; 4],
    depth: DepthRange,
) -> Option<[Vec2; 4]> {
    let mut out = [Vec2::ZERO; 4];
    for (i, w) in world.iter().enumerate() {
        let p = project_point(view_projection, viewport, *w, depth)?;
        if !p.screen.is_finite() {
            return None;
        }
        out[i] = p.screen;
    }
    Some(out)
}

pub(super) fn view_dir_at_origin(
    view_projection: Mat4,
    viewport: ViewportRect,
    origin: Vec3,
    depth: DepthRange,
) -> Option<Vec3> {
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let ray = ray_from_screen(view_projection, viewport, p0.screen, depth)?;
    Some(ray.dir.normalize_or_zero())
}

pub(super) fn origin_z01(
    view_projection: Mat4,
    viewport: ViewportRect,
    origin: Vec3,
    depth: DepthRange,
) -> Option<f32> {
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let z01 = match depth {
        DepthRange::ZeroToOne => p0.ndc_z,
        DepthRange::NegOneToOne => (p0.ndc_z + 1.0) * 0.5,
    };
    Some(z01.clamp(0.0, 1.0))
}

pub(super) fn input_precision(precision: f32) -> f32 {
    if !precision.is_finite() {
        return 1.0;
    }
    precision.clamp(0.01, 100.0)
}

pub(super) fn axis_length_world(
    view_projection: Mat4,
    viewport: ViewportRect,
    origin: Vec3,
    depth: DepthRange,
    desired_px: f32,
) -> Option<f32> {
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let z01 = match depth {
        DepthRange::ZeroToOne => p0.ndc_z,
        DepthRange::NegOneToOne => (p0.ndc_z + 1.0) * 0.5,
    }
    .clamp(0.0, 1.0);

    let p_world = unproject_point(view_projection, viewport, p0.screen, depth, z01)?;
    let p_world_dx = unproject_point(
        view_projection,
        viewport,
        p0.screen + Vec2::new(1.0, 0.0),
        depth,
        z01,
    )?;
    let d = (p_world_dx - p_world).length();
    if !d.is_finite() || d <= 1e-7 {
        return None;
    }
    Some(d * desired_px.max(0.0))
}

pub(super) fn translate_constraint_for_handle(
    view_projection: Mat4,
    viewport: ViewportRect,
    depth: DepthRange,
    origin: Vec3,
    handle: HandleId,
    axes: [Vec3; 3],
) -> Option<TranslateConstraint> {
    let axes = [
        axes[0].normalize_or_zero(),
        axes[1].normalize_or_zero(),
        axes[2].normalize_or_zero(),
    ];
    match handle.0 {
        1 => Some(TranslateConstraint::Axis { axis_dir: axes[0] }),
        2 => Some(TranslateConstraint::Axis { axis_dir: axes[1] }),
        3 => Some(TranslateConstraint::Axis { axis_dir: axes[2] }),
        4 => plane_constraint(axes[0], axes[1]),
        5 => plane_constraint(axes[0], axes[2]),
        6 => plane_constraint(axes[1], axes[2]),
        10 => {
            let view_dir = view_dir_at_origin(view_projection, viewport, origin, depth)?;
            let (u, v) = plane_basis(view_dir);
            let n = view_dir.normalize_or_zero();
            (n.length_squared() > 0.0).then_some(TranslateConstraint::Plane { u, v, normal: n })
        }
        11 => {
            let view_dir = view_dir_at_origin(view_projection, viewport, origin, depth)?;
            let n = view_dir.normalize_or_zero();
            (n.length_squared() > 0.0).then_some(TranslateConstraint::Dolly { view_dir: n })
        }
        _ => None,
    }
}

pub(super) fn plane_constraint(u: Vec3, v: Vec3) -> Option<TranslateConstraint> {
    let u = u.normalize_or_zero();
    let v = v.normalize_or_zero();
    if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
        return None;
    }
    let n = u.cross(v).normalize_or_zero();
    (n.length_squared() > 0.0).then_some(TranslateConstraint::Plane { u, v, normal: n })
}

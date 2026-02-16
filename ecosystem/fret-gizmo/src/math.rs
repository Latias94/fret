use glam::{Mat4, Vec2, Vec3};

pub use fret_viewport_tooling::{ScreenPoint, ViewportRect};

#[cfg(not(feature = "f64-math"))]
use glam::Vec4;

#[cfg(feature = "f64-math")]
use glam::{DMat4, DVec2, DVec3, DVec4};

/// Depth range convention used by the camera projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DepthRange {
    /// Normalized device coordinates Z is in `[-1, 1]` (OpenGL-style).
    NegOneToOne,
    /// Normalized device coordinates Z is in `[0, 1]` (wgpu/D3D/Vulkan-style).
    #[default]
    ZeroToOne,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectedPoint {
    pub screen: ScreenPoint,
    pub ndc_z: f32,
    pub w: f32,
    pub inside_clip: bool,
}

pub fn project_point(
    view_projection: Mat4,
    viewport: ViewportRect,
    world: Vec3,
    depth: DepthRange,
) -> Option<ProjectedPoint> {
    #[cfg(feature = "f64-math")]
    {
        return project_point_f64(view_projection, viewport, world, depth);
    }
    #[cfg(not(feature = "f64-math"))]
    {
        project_point_f32(view_projection, viewport, world, depth)
    }
}

#[cfg(not(feature = "f64-math"))]
fn project_point_f32(
    view_projection: Mat4,
    viewport: ViewportRect,
    world: Vec3,
    depth: DepthRange,
) -> Option<ProjectedPoint> {
    let clip: Vec4 = view_projection * world.extend(1.0);
    // For perspective projections, `clip.w` is proportional to view-space depth. Reject points
    // behind the camera to avoid unstable projections/picking.
    if !clip.w.is_finite() || clip.w <= 0.0 {
        return None;
    }
    if clip.w.abs() < 1e-6 {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    if !ndc.x.is_finite() || !ndc.y.is_finite() || !ndc.z.is_finite() {
        return None;
    }

    let ndc_z = ndc.z;
    let inside_clip = match depth {
        DepthRange::NegOneToOne => (-1.0..=1.0).contains(&ndc_z),
        DepthRange::ZeroToOne => (0.0..=1.0).contains(&ndc_z),
    } && (-1.0..=1.0).contains(&ndc.x)
        && (-1.0..=1.0).contains(&ndc.y);

    let w = viewport.size.x.max(1.0);
    let h = viewport.size.y.max(1.0);
    let x = viewport.min.x + (ndc.x + 1.0) * 0.5 * w;
    let y = viewport.min.y + (1.0 - (ndc.y + 1.0) * 0.5) * h;

    Some(ProjectedPoint {
        screen: Vec2::new(x, y),
        ndc_z,
        w: clip.w,
        inside_clip,
    })
}

#[cfg(not(feature = "f64-math"))]
fn screen_to_ndc(viewport: ViewportRect, screen: ScreenPoint) -> Vec2 {
    let w = viewport.size.x.max(1.0);
    let h = viewport.size.y.max(1.0);
    let x = ((screen.x - viewport.min.x) / w) * 2.0 - 1.0;
    let y = 1.0 - ((screen.y - viewport.min.y) / h) * 2.0;
    Vec2::new(x, y)
}

#[cfg(not(feature = "f64-math"))]
fn ndc_depth(depth: DepthRange, z01: f32) -> f32 {
    match depth {
        DepthRange::ZeroToOne => z01,
        DepthRange::NegOneToOne => z01 * 2.0 - 1.0,
    }
}

pub fn unproject_point(
    view_projection: Mat4,
    viewport: ViewportRect,
    screen: ScreenPoint,
    depth: DepthRange,
    z01: f32,
) -> Option<Vec3> {
    #[cfg(feature = "f64-math")]
    {
        return unproject_point_f64(view_projection, viewport, screen, depth, z01);
    }
    #[cfg(not(feature = "f64-math"))]
    {
        unproject_point_f32(view_projection, viewport, screen, depth, z01)
    }
}

#[cfg(not(feature = "f64-math"))]
fn unproject_point_f32(
    view_projection: Mat4,
    viewport: ViewportRect,
    screen: ScreenPoint,
    depth: DepthRange,
    z01: f32,
) -> Option<Vec3> {
    let inv = view_projection.inverse();
    if !inv.is_finite() {
        return None;
    }

    let ndc_xy = screen_to_ndc(viewport, screen);
    let ndc_z = ndc_depth(depth, z01);
    let mut p = inv * Vec4::new(ndc_xy.x, ndc_xy.y, ndc_z, 1.0);
    if p.w == 0.0 || !p.w.is_finite() {
        return None;
    }
    p /= p.w;
    Some(p.truncate())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3d {
    pub origin: Vec3,
    pub dir: Vec3,
}

pub fn ray_from_screen(
    view_projection: Mat4,
    viewport: ViewportRect,
    screen: ScreenPoint,
    depth: DepthRange,
) -> Option<Ray3d> {
    #[cfg(feature = "f64-math")]
    {
        return ray_from_screen_f64(view_projection, viewport, screen, depth);
    }
    #[cfg(not(feature = "f64-math"))]
    {
        ray_from_screen_f32(view_projection, viewport, screen, depth)
    }
}

#[cfg(not(feature = "f64-math"))]
fn ray_from_screen_f32(
    view_projection: Mat4,
    viewport: ViewportRect,
    screen: ScreenPoint,
    depth: DepthRange,
) -> Option<Ray3d> {
    let inv = view_projection.inverse();
    if !inv.is_finite() {
        return None;
    }

    let ndc_xy = screen_to_ndc(viewport, screen);
    let near = Vec4::new(ndc_xy.x, ndc_xy.y, ndc_depth(depth, 0.0), 1.0);
    let far = Vec4::new(ndc_xy.x, ndc_xy.y, ndc_depth(depth, 1.0), 1.0);

    let mut p0 = inv * near;
    let mut p1 = inv * far;
    if p0.w == 0.0 || p1.w == 0.0 {
        return None;
    }
    p0 /= p0.w;
    p1 /= p1.w;

    let origin = p0.truncate();
    let dir = (p1.truncate() - origin).normalize_or_zero();
    if dir.length_squared() == 0.0 {
        return None;
    }
    Some(Ray3d { origin, dir })
}

#[cfg(feature = "f64-math")]
fn project_point_f64(
    view_projection: Mat4,
    viewport: ViewportRect,
    world: Vec3,
    depth: DepthRange,
) -> Option<ProjectedPoint> {
    let view_projection = mat4_to_dmat4(view_projection);
    let world = vec3_to_dvec3(world);
    let viewport_min = vec2_to_dvec2(viewport.min);
    let viewport_size = vec2_to_dvec2(viewport.size);

    let clip: DVec4 = view_projection * world.extend(1.0);
    if !clip.w.is_finite() || clip.w <= 0.0 {
        return None;
    }
    if clip.w.abs() < 1e-12 {
        return None;
    }

    let ndc = clip.truncate() / clip.w;
    if !ndc.x.is_finite() || !ndc.y.is_finite() || !ndc.z.is_finite() {
        return None;
    }

    let ndc_z = ndc.z;
    let inside_clip = match depth {
        DepthRange::NegOneToOne => (-1.0..=1.0).contains(&ndc_z),
        DepthRange::ZeroToOne => (0.0..=1.0).contains(&ndc_z),
    } && (-1.0..=1.0).contains(&ndc.x)
        && (-1.0..=1.0).contains(&ndc.y);

    let w = viewport_size.x.max(1.0);
    let h = viewport_size.y.max(1.0);
    let x = viewport_min.x + (ndc.x + 1.0) * 0.5 * w;
    let y = viewport_min.y + (1.0 - (ndc.y + 1.0) * 0.5) * h;
    if !x.is_finite() || !y.is_finite() {
        return None;
    }

    let screen = Vec2::new(x as f32, y as f32);
    if !screen.x.is_finite() || !screen.y.is_finite() {
        return None;
    }

    Some(ProjectedPoint {
        screen,
        ndc_z: ndc_z as f32,
        w: clip.w as f32,
        inside_clip,
    })
}

#[cfg(feature = "f64-math")]
fn unproject_point_f64(
    view_projection: Mat4,
    viewport: ViewportRect,
    screen: ScreenPoint,
    depth: DepthRange,
    z01: f32,
) -> Option<Vec3> {
    let inv = mat4_to_dmat4(view_projection).inverse();
    if !inv.is_finite() {
        return None;
    }

    let ndc_xy = screen_to_ndc_f64(viewport, screen);
    let ndc_z = ndc_depth_f64(depth, z01 as f64);
    let mut p = inv * DVec4::new(ndc_xy.x, ndc_xy.y, ndc_z, 1.0);
    if p.w == 0.0 || !p.w.is_finite() {
        return None;
    }
    p /= p.w;

    let v = p.truncate();
    if !v.is_finite() {
        return None;
    }
    let out = Vec3::new(v.x as f32, v.y as f32, v.z as f32);
    out.is_finite().then_some(out)
}

#[cfg(feature = "f64-math")]
fn ray_from_screen_f64(
    view_projection: Mat4,
    viewport: ViewportRect,
    screen: ScreenPoint,
    depth: DepthRange,
) -> Option<Ray3d> {
    let inv = mat4_to_dmat4(view_projection).inverse();
    if !inv.is_finite() {
        return None;
    }

    let ndc_xy = screen_to_ndc_f64(viewport, screen);
    let near = DVec4::new(ndc_xy.x, ndc_xy.y, ndc_depth_f64(depth, 0.0), 1.0);
    let far = DVec4::new(ndc_xy.x, ndc_xy.y, ndc_depth_f64(depth, 1.0), 1.0);

    let mut p0 = inv * near;
    let mut p1 = inv * far;
    if p0.w == 0.0 || p1.w == 0.0 {
        return None;
    }
    p0 /= p0.w;
    p1 /= p1.w;

    let origin = p0.truncate();
    let dir = (p1.truncate() - origin).normalize_or_zero();
    if dir.length_squared() == 0.0 {
        return None;
    }

    if !origin.is_finite() || !dir.is_finite() {
        return None;
    }

    let origin_f32 = Vec3::new(origin.x as f32, origin.y as f32, origin.z as f32);
    let dir_f32 = Vec3::new(dir.x as f32, dir.y as f32, dir.z as f32);
    if !origin_f32.is_finite() || !dir_f32.is_finite() {
        return None;
    }

    Some(Ray3d {
        origin: origin_f32,
        dir: dir_f32,
    })
}

#[cfg(feature = "f64-math")]
fn mat4_to_dmat4(m: Mat4) -> DMat4 {
    let a = m.to_cols_array();
    DMat4::from_cols_array(&[
        a[0] as f64,
        a[1] as f64,
        a[2] as f64,
        a[3] as f64,
        a[4] as f64,
        a[5] as f64,
        a[6] as f64,
        a[7] as f64,
        a[8] as f64,
        a[9] as f64,
        a[10] as f64,
        a[11] as f64,
        a[12] as f64,
        a[13] as f64,
        a[14] as f64,
        a[15] as f64,
    ])
}

#[cfg(feature = "f64-math")]
fn vec2_to_dvec2(v: Vec2) -> DVec2 {
    DVec2::new(v.x as f64, v.y as f64)
}

#[cfg(feature = "f64-math")]
fn vec3_to_dvec3(v: Vec3) -> DVec3 {
    DVec3::new(v.x as f64, v.y as f64, v.z as f64)
}

#[cfg(feature = "f64-math")]
fn screen_to_ndc_f64(viewport: ViewportRect, screen: ScreenPoint) -> DVec2 {
    let viewport_min = vec2_to_dvec2(viewport.min);
    let viewport_size = vec2_to_dvec2(viewport.size);
    let screen = vec2_to_dvec2(screen);

    let w = viewport_size.x.max(1.0);
    let h = viewport_size.y.max(1.0);
    let x = ((screen.x - viewport_min.x) / w) * 2.0 - 1.0;
    let y = 1.0 - ((screen.y - viewport_min.y) / h) * 2.0;
    DVec2::new(x, y)
}

#[cfg(feature = "f64-math")]
fn ndc_depth_f64(depth: DepthRange, z01: f64) -> f64 {
    match depth {
        DepthRange::ZeroToOne => z01,
        DepthRange::NegOneToOne => z01 * 2.0 - 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_from_screen_large_z_translation_is_sensitive_to_precision() {
        let view_projection = Mat4::from_translation(Vec3::new(0.0, 0.0, -1.0e12));
        let viewport = ViewportRect::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let screen = Vec2::new(50.0, 50.0);
        let ray = ray_from_screen(view_projection, viewport, screen, DepthRange::ZeroToOne);

        #[cfg(feature = "f64-math")]
        assert!(
            ray.is_some(),
            "expected f64 math path to preserve near/far separation"
        );
        #[cfg(not(feature = "f64-math"))]
        assert!(
            ray.is_none(),
            "expected f32 path to lose precision under extreme z translation"
        );
    }
}

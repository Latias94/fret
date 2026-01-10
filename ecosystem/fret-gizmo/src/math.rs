use glam::{Mat4, Vec2, Vec3, Vec4};

/// Depth range convention used by the camera projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthRange {
    /// Normalized device coordinates Z is in `[-1, 1]` (OpenGL-style).
    NegOneToOne,
    /// Normalized device coordinates Z is in `[0, 1]` (wgpu/D3D/Vulkan-style).
    ZeroToOne,
}

impl Default for DepthRange {
    fn default() -> Self {
        Self::ZeroToOne
    }
}

/// Viewport rectangle in logical or physical pixels (caller-defined).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportRect {
    pub min: Vec2,
    pub size: Vec2,
}

impl ViewportRect {
    pub fn new(min: Vec2, size: Vec2) -> Self {
        Self { min, size }
    }

    pub fn max(self) -> Vec2 {
        self.min + self.size
    }
}

/// A 2D point in viewport coordinates (top-left origin).
pub type ScreenPoint = Vec2;

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
    let clip: Vec4 = view_projection * world.extend(1.0);
    if !clip.w.is_finite() || clip.w == 0.0 {
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

fn screen_to_ndc(viewport: ViewportRect, screen: ScreenPoint) -> Vec2 {
    let w = viewport.size.x.max(1.0);
    let h = viewport.size.y.max(1.0);
    let x = ((screen.x - viewport.min.x) / w) * 2.0 - 1.0;
    let y = 1.0 - ((screen.y - viewport.min.y) / h) * 2.0;
    Vec2::new(x, y)
}

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

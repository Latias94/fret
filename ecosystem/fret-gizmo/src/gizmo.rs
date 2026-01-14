use fret_core::Color;
use glam::{Mat4, Quat, Vec2, Vec3};

use crate::math::{
    DepthRange, Ray3d, ViewportRect, project_point, ray_from_screen, unproject_point,
};
use crate::picking::{
    PickCircle2d, PickConvexQuad2d, PickSegmentCapsule2d, distance_point_to_segment_px,
};
use crate::style::GizmoPartVisuals;

mod config;
mod runtime;
mod types;

pub use config::*;
pub use runtime::*;
pub use types::*;

mod handles;
use handles::*;

use pick::{MixedPickBand, PickHit};

#[derive(Debug, Default)]
pub struct Gizmo {
    pub config: GizmoConfig,
    pub state: GizmoState,
}

impl Gizmo {
    const UNIVERSAL_TRANSLATE_TIP_SCALE: f32 = 1.25;
    const ROTATE_VIEW_HANDLE: HandleId = pack_handle(HANDLE_GROUP_ROTATE, 8);
    const ROTATE_ARCBALL_HANDLE: HandleId = pack_handle(HANDLE_GROUP_ROTATE, 9);
    const BOUNDS_CORNER_BASE: u64 = 20;
    const BOUNDS_CORNER_END: u64 = 27;
    const BOUNDS_FACE_BASE: u64 = 30;
    const BOUNDS_FACE_END: u64 = 35;

    fn effective_ops(&self) -> GizmoOps {
        if let Some(mask) = self.config.operation_mask {
            return mask;
        }

        match self.config.mode {
            GizmoMode::Translate => GizmoOps::translate_all(),
            GizmoMode::Rotate => {
                let mut ops = GizmoOps::rotate_axis();
                if self.config.show_view_axis_ring {
                    ops |= GizmoOps::rotate_view();
                }
                if self.config.show_arcball {
                    ops |= GizmoOps::rotate_arcball();
                }
                ops
            }
            GizmoMode::Scale => {
                let mut ops =
                    GizmoOps::scale_axis() | GizmoOps::scale_plane() | GizmoOps::scale_uniform();
                if self.config.show_bounds {
                    ops |= GizmoOps::scale_bounds();
                }
                ops
            }
            GizmoMode::Universal => {
                let mut ops = GizmoOps::translate_all() | GizmoOps::rotate_axis();
                if self.config.universal_includes_rotate_view_ring {
                    ops |= GizmoOps::rotate_view();
                }
                if self.config.universal_includes_arcball {
                    ops |= GizmoOps::rotate_arcball();
                }
                if self.config.universal_includes_scale {
                    ops |= GizmoOps::scale_axis();
                }
                ops
            }
        }
    }

    fn translate_axis_tip_scale(&self) -> f32 {
        if self.config.mode == GizmoMode::Universal && self.config.universal_includes_scale {
            return Self::UNIVERSAL_TRANSLATE_TIP_SCALE;
        }
        if let Some(mask) = self.config.operation_mask {
            let translate_axis = mask.contains(GizmoOps::translate_axis());
            let scale_axis = mask.contains(GizmoOps::scale_axis());
            if translate_axis && scale_axis {
                return Self::UNIVERSAL_TRANSLATE_TIP_SCALE;
            }
        }
        1.0
    }

    fn handedness_rotation_sign(&self) -> f32 {
        match self.config.handedness {
            GizmoHandedness::RightHanded => 1.0,
            GizmoHandedness::LeftHanded => -1.0,
        }
    }

    fn pivot_origin(
        active_transform: Transform3d,
        targets: &[GizmoTarget3d],
        mode: GizmoPivotMode,
    ) -> Vec3 {
        match mode {
            GizmoPivotMode::Active => active_transform.translation,
            GizmoPivotMode::Center => {
                // Editor convention: "center" means the selection bounds center, not the average
                // of entity origins (which can drift for uneven distributions).
                if let Some(bounds) = Self::selection_world_aabb(targets) {
                    (bounds.min + bounds.max) * 0.5
                } else {
                    let sum = targets
                        .iter()
                        .fold(Vec3::ZERO, |acc, t| acc + t.transform.translation);
                    sum / (targets.len().max(1) as f32)
                }
            }
        }
    }

    fn selection_world_aabb(targets: &[GizmoTarget3d]) -> Option<Aabb3> {
        let mut min_v = Vec3::splat(f32::INFINITY);
        let mut max_v = Vec3::splat(f32::NEG_INFINITY);

        for t in targets {
            if let Some(aabb) = t.local_bounds {
                let aabb = aabb.normalized();
                let m = t.transform.to_mat4();
                for c in aabb.corners() {
                    let world = m.transform_point3(c);
                    if !world.is_finite() {
                        continue;
                    }
                    min_v = min_v.min(world);
                    max_v = max_v.max(world);
                }
            } else {
                let world = t.transform.translation;
                if !world.is_finite() {
                    continue;
                }
                min_v = min_v.min(world);
                max_v = max_v.max(world);
            }
        }

        if !min_v.is_finite() || !max_v.is_finite() {
            return None;
        }

        Some(
            Aabb3 {
                min: min_v,
                max: max_v,
            }
            .normalized(),
        )
    }

    fn clamp_size_length_world(&self, length_world: f32) -> f32 {
        let length_world = if length_world.is_finite() {
            length_world.max(0.0)
        } else {
            0.0
        };
        let Some((a, b)) = self.config.size_world_clamp else {
            return length_world;
        };
        if !a.is_finite() || !b.is_finite() {
            return length_world;
        }
        let (min_world, max_world) = if a <= b { (a, b) } else { (b, a) };
        length_world.clamp(min_world.max(0.0), max_world.max(0.0))
    }

    fn size_length_world(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        targets: &[GizmoTarget3d],
    ) -> Option<f32> {
        let length_world = match self.config.size_policy {
            GizmoSizePolicy::ConstantPixels => axis_length_world(
                view_projection,
                viewport,
                origin,
                self.config.depth_range,
                self.config.size_px,
            )?,
            GizmoSizePolicy::PixelsClampedBySelectionBounds {
                min_fraction_of_max_extent,
                max_fraction_of_max_extent,
            } => {
                let base = axis_length_world(
                    view_projection,
                    viewport,
                    origin,
                    self.config.depth_range,
                    self.config.size_px,
                )?;
                match Self::selection_world_aabb(targets) {
                    None => base,
                    Some(bounds) => {
                        let extent = (bounds.max - bounds.min).abs();
                        let max_extent = extent.max_element().max(1e-6);

                        let min_frac = if min_fraction_of_max_extent.is_finite() {
                            min_fraction_of_max_extent.clamp(0.0, 1000.0)
                        } else {
                            0.0
                        };
                        let max_frac = if max_fraction_of_max_extent.is_finite() {
                            max_fraction_of_max_extent.clamp(0.0, 1000.0)
                        } else {
                            0.0
                        };
                        let (min_frac, max_frac) = if min_frac <= max_frac {
                            (min_frac, max_frac)
                        } else {
                            (max_frac, min_frac)
                        };

                        let min_world = max_extent * min_frac;
                        let max_world = max_extent * max_frac;
                        if max_world <= 1e-6 {
                            base
                        } else {
                            base.clamp(min_world.max(0.0), max_world.max(min_world))
                        }
                    }
                }
            }
            GizmoSizePolicy::SelectionBounds {
                fraction_of_max_extent,
            } => {
                let fraction = if fraction_of_max_extent.is_finite() {
                    fraction_of_max_extent.clamp(0.01, 100.0)
                } else {
                    1.0
                };

                let bounds = Self::selection_world_aabb(targets);
                let len = bounds.map(|b| {
                    let extent = (b.max - b.min).abs();
                    extent.max_element().max(1e-6) * fraction
                });
                match len {
                    Some(v) if v.is_finite() && v > 1e-6 => v,
                    _ => axis_length_world(
                        view_projection,
                        viewport,
                        origin,
                        self.config.depth_range,
                        self.config.size_px,
                    )?,
                }
            }
        };

        Some(self.clamp_size_length_world(length_world))
    }

    fn size_length_world_or_one(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        targets: &[GizmoTarget3d],
    ) -> f32 {
        self.size_length_world(view_projection, viewport, origin, targets)
            .unwrap_or(1.0)
    }

    fn axis_is_masked(&self, axis_index: usize) -> bool {
        self.config
            .axis_mask
            .get(axis_index)
            .copied()
            .unwrap_or(false)
    }

    fn plane_allowed_by_mask(&self, plane_axes: (usize, usize)) -> bool {
        let (a, b) = plane_axes;
        if a == b || a > 2 || b > 2 {
            return false;
        }
        let masked = self.config.axis_mask;
        let masked_count = masked.iter().filter(|m| **m).count();
        if masked_count == 0 {
            return true;
        }
        if masked_count == 1 {
            // Show only the plane perpendicular to the masked axis.
            let perp = 3usize.saturating_sub(a + b);
            return perp <= 2 && masked[perp];
        }
        false
    }

    fn flip_axes_for_view(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> [Vec3; 3] {
        if !self.config.allow_axis_flip {
            return axes;
        }
        let length_world = size_length_world.max(0.0);

        let mut out = axes;
        for i in 0..3 {
            let axis = axes[i].normalize_or_zero();
            if axis.length_squared() == 0.0 {
                continue;
            }

            let len_plus = axis_segment_len_px(
                view_projection,
                viewport,
                origin,
                self.config.depth_range,
                axis,
                length_world,
            );
            let len_minus = axis_segment_len_px(
                view_projection,
                viewport,
                origin,
                self.config.depth_range,
                -axis,
                length_world,
            );

            out[i] = match (len_plus, len_minus) {
                (Some(a), Some(b)) => {
                    if b > a + 1e-3 {
                        -axis
                    } else {
                        axis
                    }
                }
                (None, Some(_)) => -axis,
                _ => axis,
            };
        }
        out
    }

    fn axis_visibility_alpha(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axis_dir: Vec3,
        axis_len_world: f32,
    ) -> f32 {
        let (lo, hi) = self.config.axis_fade_px;
        if !(lo.is_finite() && hi.is_finite()) {
            return 1.0;
        }
        let lo = lo.min(hi);
        let hi = hi.max(lo + 1e-3);
        let len_px = axis_segment_len_px(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            axis_dir,
            axis_len_world,
        )
        .unwrap_or(hi);
        ((len_px - lo) / (hi - lo)).clamp(0.0, 1.0)
    }

    fn rotate_ring_visibility_alpha(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axis_dir: Vec3,
    ) -> f32 {
        let (lo, hi) = self.config.rotate_ring_fade_dot;
        if !(lo.is_finite() && hi.is_finite()) {
            return 1.0;
        }
        let lo = lo.min(hi);
        let hi = hi.max(lo + 1e-3);

        let Some(view_dir) =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        else {
            return 1.0;
        };
        let axis = axis_dir.normalize_or_zero();
        let view = view_dir.normalize_or_zero();
        if axis.length_squared() == 0.0 || view.length_squared() == 0.0 {
            return 1.0;
        }

        let dot = view.dot(axis).abs().clamp(0.0, 1.0);
        let t = ((dot - lo) / (hi - lo)).clamp(0.0, 1.0);
        // smoothstep
        t * t * (3.0 - 2.0 * t)
    }

    fn plane_visibility_alpha(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        quad_world: [Vec3; 4],
    ) -> f32 {
        let (lo, hi) = self.config.plane_fade_px2;
        if !(lo.is_finite() && hi.is_finite()) {
            return 1.0;
        }
        let lo = lo.min(hi);
        let hi = hi.max(lo + 1e-3);
        let p = project_quad(
            view_projection,
            viewport,
            quad_world,
            self.config.depth_range,
        );
        let area = p.map(quad_area_px2).unwrap_or(hi);
        ((area - lo) / (hi - lo)).clamp(0.0, 1.0)
    }

    pub fn new(config: GizmoConfig) -> Self {
        Self {
            config,
            state: GizmoState::default(),
        }
    }

    pub fn set_part_visuals(&mut self, visuals: GizmoPartVisuals) {
        self.state.part_visuals = visuals;
    }

    pub fn part_visuals(&self) -> GizmoPartVisuals {
        self.state.part_visuals
    }

    pub fn draw(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> GizmoDrawList3d {
        if targets.is_empty() {
            return GizmoDrawList3d::default();
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

        if self.config.operation_mask.is_some() {
            let ops = self.effective_ops();
            let mut out = GizmoDrawList3d::default();

            let translate_axes = ops.contains(GizmoOps::translate_axis());
            let translate_planes = ops.contains(GizmoOps::translate_plane());
            let translate_screen = ops.contains(GizmoOps::translate_view());
            let translate_depth = ops.contains(GizmoOps::translate_depth());
            let rotate_any = ops.intersects(GizmoOps::rotate_all());
            let scale_axes = ops.contains(GizmoOps::scale_axis());
            let scale_planes = ops.contains(GizmoOps::scale_plane());
            let scale_uniform = ops.contains(GizmoOps::scale_uniform());
            let scale_bounds = ops.contains(GizmoOps::scale_bounds());

            if scale_bounds {
                let bounds_axes = [
                    axes_raw[0].normalize_or_zero(),
                    axes_raw[1].normalize_or_zero(),
                    axes_raw[2].normalize_or_zero(),
                ];
                self.draw_bounds(
                    &mut out,
                    view_projection,
                    viewport,
                    origin,
                    bounds_axes,
                    size_length_world,
                    targets,
                );
            }

            // When scale axes are present, skip the translate axis *lines* to reduce overlap.
            // The translate arrow tips remain as the explicit "grab" affordance.
            if translate_axes && !scale_axes {
                out.lines.extend(self.draw_translate_axes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
            }
            if translate_planes {
                out.lines.extend(self.draw_translate_planes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
            }
            if translate_screen {
                out.lines.extend(self.draw_translate_screen(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
            }
            if translate_depth {
                out.lines.extend(self.draw_translate_depth(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
            }

            if rotate_any {
                let rings = self.draw_rotate_rings(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                );
                out.lines.extend(rings.lines);
                out.triangles.extend(rings.triangles);
            }

            if scale_axes || scale_planes || scale_uniform {
                out.lines.extend(self.draw_scale_handles(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    scale_axes,
                    scale_uniform,
                    scale_planes,
                ));
            }

            let rotate_feedback =
                self.draw_rotate_feedback(view_projection, viewport, origin, size_length_world);
            let translate_feedback =
                self.draw_translate_feedback(view_projection, viewport, origin, size_length_world);
            let scale_feedback = self.draw_scale_feedback(view_projection, viewport, origin);
            out.lines.extend(rotate_feedback.lines);
            out.lines.extend(translate_feedback.lines);
            out.lines.extend(scale_feedback.lines);

            if translate_axes || translate_planes || translate_screen {
                out.triangles.extend(self.draw_translate_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    translate_axes,
                    translate_planes,
                    translate_screen,
                ));
            }
            if scale_axes || scale_planes || scale_uniform {
                out.triangles.extend(self.draw_scale_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    scale_axes,
                    scale_uniform,
                    scale_planes,
                ));
            }
            out.triangles.extend(rotate_feedback.triangles);

            return out;
        }

        match self.config.mode {
            GizmoMode::Translate => {
                let mut out = GizmoDrawList3d::default();
                out.lines.extend(self.draw_translate_axes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
                out.lines.extend(self.draw_translate_planes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
                out.lines.extend(self.draw_translate_screen(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
                out.lines.extend(self.draw_translate_depth(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
                let feedback = self.draw_translate_feedback(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                );
                out.lines.extend(feedback.lines);
                out.triangles.extend(self.draw_translate_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    true,
                    true,
                    true,
                ));
                out
            }
            GizmoMode::Rotate => {
                let mut out = GizmoDrawList3d::default();
                let rings = self.draw_rotate_rings(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                );
                out.lines.extend(rings.lines);
                out.triangles.extend(rings.triangles);
                let feedback =
                    self.draw_rotate_feedback(view_projection, viewport, origin, size_length_world);
                out.lines.extend(feedback.lines);
                out.triangles.extend(feedback.triangles);
                out
            }
            GizmoMode::Scale => {
                let mut out = GizmoDrawList3d::default();
                if self.config.show_bounds {
                    let bounds_axes = [
                        axes_raw[0].normalize_or_zero(),
                        axes_raw[1].normalize_or_zero(),
                        axes_raw[2].normalize_or_zero(),
                    ];
                    self.draw_bounds(
                        &mut out,
                        view_projection,
                        viewport,
                        origin,
                        bounds_axes,
                        size_length_world,
                        targets,
                    );
                }
                out.lines.extend(self.draw_scale_handles(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    true,
                    true,
                    true,
                ));
                let feedback = self.draw_scale_feedback(view_projection, viewport, origin);
                out.lines.extend(feedback.lines);
                out.triangles.extend(self.draw_scale_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    true,
                    true,
                    true,
                ));
                out
            }
            GizmoMode::Universal => {
                let mut out = GizmoDrawList3d::default();
                if !self.config.universal_includes_scale {
                    out.lines.extend(self.draw_translate_axes(
                        view_projection,
                        viewport,
                        origin,
                        axes,
                        size_length_world,
                    ));
                }
                out.lines.extend(self.draw_translate_planes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
                out.lines.extend(self.draw_translate_screen(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
                if self.config.universal_includes_translate_depth {
                    out.lines.extend(self.draw_translate_depth(
                        view_projection,
                        viewport,
                        origin,
                        size_length_world,
                    ));
                }
                let rings = self.draw_rotate_rings(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                );
                out.lines.extend(rings.lines);
                out.triangles.extend(rings.triangles);
                if self.config.universal_includes_scale {
                    out.lines.extend(self.draw_scale_handles(
                        view_projection,
                        viewport,
                        origin,
                        axes,
                        size_length_world,
                        true,
                        false,
                        false,
                    ));
                }
                let rotate_feedback =
                    self.draw_rotate_feedback(view_projection, viewport, origin, size_length_world);
                let translate_feedback = self.draw_translate_feedback(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                );
                let scale_feedback = self.draw_scale_feedback(view_projection, viewport, origin);
                out.lines.extend(rotate_feedback.lines);
                out.lines.extend(translate_feedback.lines);
                out.lines.extend(scale_feedback.lines);
                out.triangles.extend(self.draw_translate_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    true,
                    true,
                    true,
                ));
                if self.config.universal_includes_scale {
                    out.triangles.extend(self.draw_scale_solids(
                        view_projection,
                        viewport,
                        origin,
                        axes,
                        size_length_world,
                        true,
                        false,
                        false,
                    ));
                }
                out.triangles.extend(rotate_feedback.triangles);
                out
            }
        }
    }

    fn is_handle_highlighted(&self, kind: GizmoMode, handle: HandleId) -> bool {
        if self.state.active == Some(handle) {
            return self.state.drag_mode == kind;
        }
        self.state.hovered == Some(handle) && self.state.hovered_kind == Some(kind)
    }

    fn push_line(&self, out: &mut Vec<Line3d>, a: Vec3, b: Vec3, color: Color, depth: DepthMode) {
        match (depth, self.config.show_occluded) {
            (DepthMode::Test, true) => {
                out.push(Line3d {
                    a,
                    b,
                    color: mix_alpha(color, self.config.occluded_alpha),
                    depth: DepthMode::Ghost,
                });
                out.push(Line3d {
                    a,
                    b,
                    color,
                    depth: DepthMode::Test,
                });
            }
            _ => {
                out.push(Line3d { a, b, color, depth });
            }
        }
    }

    fn push_line_no_ghost(
        &self,
        out: &mut Vec<Line3d>,
        a: Vec3,
        b: Vec3,
        color: Color,
        depth: DepthMode,
    ) {
        out.push(Line3d { a, b, color, depth });
    }

    fn push_quad_outline(
        &self,
        out: &mut Vec<Line3d>,
        quad: [Vec3; 4],
        color: Color,
        depth: DepthMode,
        allow_ghost: bool,
    ) {
        for (a, b) in [
            (quad[0], quad[1]),
            (quad[1], quad[2]),
            (quad[2], quad[3]),
            (quad[3], quad[0]),
        ] {
            if allow_ghost {
                self.push_line(out, a, b, color, depth);
            } else {
                self.push_line_no_ghost(out, a, b, color, depth);
            }
        }
    }

    fn push_tri(
        &self,
        out: &mut Vec<Triangle3d>,
        a: Vec3,
        b: Vec3,
        c: Vec3,
        color: Color,
        depth: DepthMode,
    ) {
        match (depth, self.config.show_occluded) {
            (DepthMode::Test, true) => {
                out.push(Triangle3d {
                    a,
                    b,
                    c,
                    color: mix_alpha(color, self.config.occluded_alpha),
                    depth: DepthMode::Ghost,
                });
                out.push(Triangle3d {
                    a,
                    b,
                    c,
                    color,
                    depth: DepthMode::Test,
                });
            }
            _ => {
                out.push(Triangle3d {
                    a,
                    b,
                    c,
                    color,
                    depth,
                });
            }
        }
    }

    fn push_tri_no_ghost(
        &self,
        out: &mut Vec<Triangle3d>,
        a: Vec3,
        b: Vec3,
        c: Vec3,
        color: Color,
        depth: DepthMode,
    ) {
        out.push(Triangle3d {
            a,
            b,
            c,
            color,
            depth,
        });
    }

    fn push_quad_fill(
        &self,
        out: &mut Vec<Triangle3d>,
        quad: [Vec3; 4],
        color: Color,
        depth: DepthMode,
        allow_ghost: bool,
    ) {
        if allow_ghost {
            self.push_tri(out, quad[0], quad[1], quad[2], color, depth);
            self.push_tri(out, quad[0], quad[2], quad[3], color, depth);
        } else {
            self.push_tri_no_ghost(out, quad[0], quad[1], quad[2], color, depth);
            self.push_tri_no_ghost(out, quad[0], quad[2], quad[3], color, depth);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push_ring_band(
        &self,
        out: &mut GizmoDrawList3d,
        origin: Vec3,
        u: Vec3,
        v: Vec3,
        inner_r: f32,
        outer_r: f32,
        fill: Color,
        edge: Color,
        depth: DepthMode,
        allow_ghost: bool,
        segments: usize,
    ) {
        let point =
            |theta: f32, r: f32| -> Vec3 { origin + (u * theta.cos() + v * theta.sin()) * r };

        let step = std::f32::consts::TAU / (segments as f32);
        let mut prev_outer = point(0.0, outer_r);
        for i in 0..segments {
            let t0 = step * (i as f32);
            let t1 = step * ((i + 1) as f32);
            let o0 = point(t0, outer_r);
            let i0 = point(t0, inner_r);
            let o1 = point(t1, outer_r);
            let i1 = point(t1, inner_r);

            self.push_quad_fill(
                &mut out.triangles,
                [o0, i0, i1, o1],
                fill,
                depth,
                allow_ghost,
            );
            if allow_ghost {
                self.push_line(&mut out.lines, prev_outer, o1, edge, depth);
            } else {
                self.push_line_no_ghost(&mut out.lines, prev_outer, o1, edge, depth);
            }
            prev_outer = o1;
        }
    }

    fn axis_dirs(&self, target: &Transform3d) -> [Vec3; 3] {
        match self.config.orientation {
            GizmoOrientation::World => [Vec3::X, Vec3::Y, Vec3::Z],
            GizmoOrientation::Local => [
                target.rotation * Vec3::X,
                target.rotation * Vec3::Y,
                target.rotation * Vec3::Z,
            ],
        }
    }

    fn bounds_min_max_local(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        basis: [Vec3; 3],
        size_length_world: f32,
        targets: &[GizmoTarget3d],
    ) -> (Vec3, Vec3) {
        let mut min_v = Vec3::splat(f32::INFINITY);
        let mut max_v = Vec3::splat(f32::NEG_INFINITY);

        for t in targets {
            if let Some(aabb) = t.local_bounds {
                let aabb = aabb.normalized();
                let m = t.transform.to_mat4();
                for c in aabb.corners() {
                    let world = m.transform_point3(c);
                    if !world.is_finite() {
                        continue;
                    }
                    let p = world - origin;
                    let v = Vec3::new(p.dot(basis[0]), p.dot(basis[1]), p.dot(basis[2]));
                    min_v = min_v.min(v);
                    max_v = max_v.max(v);
                }
            } else {
                let p = t.transform.translation - origin;
                let v = Vec3::new(p.dot(basis[0]), p.dot(basis[1]), p.dot(basis[2]));
                min_v = min_v.min(v);
                max_v = max_v.max(v);
            }
        }

        let _ = (view_projection, viewport);
        let min_extent = size_length_world.max(1e-6) * 0.25;

        if !min_v.is_finite() || !max_v.is_finite() {
            let half = Vec3::splat(min_extent.max(1e-6) * 0.5);
            return (-half, half);
        }

        let center = (min_v + max_v) * 0.5;
        let extent = (max_v - min_v).max(Vec3::splat(min_extent));
        (center - extent * 0.5, center + extent * 0.5)
    }

    fn bounds_corner_id(x_max: bool, y_max: bool, z_max: bool) -> HandleId {
        let bits = (x_max as u64) | ((y_max as u64) << 1) | ((z_max as u64) << 2);
        pack_handle(HANDLE_GROUP_SCALE, (Self::BOUNDS_CORNER_BASE + bits) as u32)
    }

    fn bounds_face_id(axis: usize, max_side: bool) -> HandleId {
        let axis = axis.min(2) as u64;
        let side = if max_side { 1u64 } else { 0u64 };
        pack_handle(
            HANDLE_GROUP_SCALE,
            (Self::BOUNDS_FACE_BASE + axis * 2 + side) as u32,
        )
    }

    fn bounds_handle_from_id(handle: HandleId) -> Option<BoundsHandle> {
        if handle_group(handle) != HANDLE_GROUP_SCALE {
            return None;
        }
        match handle_sub_id(handle) as u64 {
            Self::BOUNDS_CORNER_BASE..=Self::BOUNDS_CORNER_END => {
                let bits = (handle_sub_id(handle) as u64) - Self::BOUNDS_CORNER_BASE;
                Some(BoundsHandle::Corner {
                    x_max: (bits & 1) != 0,
                    y_max: (bits & 2) != 0,
                    z_max: (bits & 4) != 0,
                })
            }
            Self::BOUNDS_FACE_BASE..=Self::BOUNDS_FACE_END => {
                let v = (handle_sub_id(handle) as u64) - Self::BOUNDS_FACE_BASE;
                let axis = (v / 2) as usize;
                let max_side = (v % 2) == 1;
                Some(BoundsHandle::Face { axis, max_side })
            }
            _ => None,
        }
    }

    fn arcball_vector_world(&self, cursor_px: Vec2) -> Option<Vec3> {
        let r = self.state.drag_arcball_radius_px;
        if !r.is_finite() || r <= 1e-3 {
            return None;
        }
        let p = (cursor_px - self.state.drag_arcball_center_px) / r;
        if !p.x.is_finite() || !p.y.is_finite() {
            return None;
        }

        // Note: screen Y is down, but arcball math expects Y up.
        let mut x = p.x;
        let mut y = -p.y;
        let d2 = x * x + y * y;
        let z = if d2 <= 1.0 {
            (1.0 - d2).sqrt()
        } else {
            let inv = d2.sqrt().recip();
            x *= inv;
            y *= inv;
            0.0
        };

        let u = self.state.drag_basis_u.normalize_or_zero();
        let v = self.state.drag_basis_v.normalize_or_zero();
        let n = self.state.drag_plane_normal.normalize_or_zero();
        if u.length_squared() == 0.0 || v.length_squared() == 0.0 || n.length_squared() == 0.0 {
            return None;
        }

        let w = (u * x + v * y + n * z).normalize_or_zero();
        (w.length_squared() > 0.0).then_some(w)
    }

    fn tick_perp_dir(
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        depth: DepthRange,
        dir: Vec3,
    ) -> Vec3 {
        let dir = dir.normalize_or_zero();
        if dir.length_squared() == 0.0 {
            return Vec3::X;
        }
        if let Some(view_dir) = view_dir_at_origin(view_projection, viewport, origin, depth) {
            let perp = dir.cross(view_dir).normalize_or_zero();
            if perp.length_squared() > 0.0 {
                return perp;
            }
        }
        plane_basis(dir).0.normalize_or_zero()
    }

    #[allow(clippy::too_many_arguments)]
    fn mixed_translate_axis_tip_intent(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        hit: PickHit,
    ) -> bool {
        if handle_group(hit.handle) != HANDLE_GROUP_TRANSLATE {
            return false;
        }
        let sub = handle_sub_id(hit.handle);
        if !(1..=3).contains(&sub) {
            return false;
        }
        let axis_dir = axes[(sub.saturating_sub(1)) as usize].normalize_or_zero();
        if axis_dir.length_squared() == 0.0 {
            return false;
        }

        let axis_tip_len = size_length_world * self.translate_axis_tip_scale();
        let tip_world = origin + axis_dir * axis_tip_len;
        let Some(tip) = project_point(
            view_projection,
            viewport,
            tip_world,
            self.config.depth_range,
        ) else {
            return false;
        };
        let d = (cursor - tip.screen).length();
        if !d.is_finite() {
            return false;
        }
        let r = self.config.pick_radius_px.max(6.0)
            * self.config.pick_policy.translate_axis_tip_radius_scale;
        d <= r
    }

    #[allow(clippy::too_many_arguments)]
    fn mixed_pick_band(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        rotate_present: bool,
        allow_translate_axis_tip_intent: bool,
        hit: PickHit,
        kind: GizmoMode,
    ) -> MixedPickBand {
        match kind {
            GizmoMode::Translate => {
                if hit.handle == TranslateHandle::Screen.id() {
                    return MixedPickBand::TranslateCenter;
                }
                if handle_group(hit.handle) == HANDLE_GROUP_TRANSLATE
                    && (4..=6).contains(&handle_sub_id(hit.handle))
                    && hit.score <= self.config.pick_policy.translate_plane_inside_score_max
                {
                    return MixedPickBand::TranslatePlaneInside;
                }
                if rotate_present
                    && allow_translate_axis_tip_intent
                    && self.mixed_translate_axis_tip_intent(
                        view_projection,
                        viewport,
                        origin,
                        cursor,
                        axes,
                        size_length_world,
                        hit,
                    )
                {
                    return MixedPickBand::TranslateAxisTipIntent;
                }
                MixedPickBand::Default
            }
            GizmoMode::Scale => {
                if hit.score <= self.config.pick_policy.scale_solid_inside_score_max {
                    return MixedPickBand::ScaleSolidInside;
                }
                if Self::bounds_handle_from_id(hit.handle).is_some()
                    && hit.score <= self.config.pick_policy.bounds_inside_score_max
                {
                    return MixedPickBand::ScaleSolidInside;
                }
                MixedPickBand::Default
            }
            GizmoMode::Rotate | GizmoMode::Universal => MixedPickBand::Default,
        }
    }
}

mod helpers;
use helpers::*;

mod draw_bounds;
mod draw_rotate;
mod draw_scale;
mod draw_translate;
mod pick;
mod update;

#[cfg(test)]
mod tests;

use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, SceneOp};
use fret_core::text::{FontWeight, TextConstraints, TextOverflow, TextStyle, TextWrap};
use fret_core::{
    AppWindowId, Event, RenderTargetId, ViewportFit, ViewportInputEvent, ViewportInputKind,
};
use fret_gizmo::{
    Aabb3, DepthMode, DepthRange, Gizmo, GizmoConfig, GizmoDrawList3d, GizmoInput, GizmoMode,
    GizmoOps, GizmoOrientation, GizmoPhase, GizmoPivotMode, GizmoResult, GizmoSizePolicy,
    GizmoTarget3d, GizmoTargetId, Grid3d, HandleId, Transform3d, ViewGizmo, ViewGizmoAnchor,
    ViewGizmoConfig, ViewGizmoInput, ViewGizmoProjection, ViewGizmoUpdate, ViewportRect,
};
use fret_launch::{
    EngineFrameUpdate, ViewportOverlay3dHooks, ViewportOverlay3dHooksService, WinitAppDriver,
    WinitCommandContext, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    record_viewport_overlay_3d,
};
use fret_plot3d::retained::{Plot3dCanvas, Plot3dModel, Plot3dStyle, Plot3dViewport};
use fret_render::viewport_overlay::{
    Overlay3dCache, Overlay3dCpuBuilder, Overlay3dLineVertex, Overlay3dPipelines,
    Overlay3dUniforms, Overlay3dVertex, ViewportOverlay3dContext, push_thick_line_quad,
    push_triangle,
};
use fret_render::{RenderTargetColorSpace, RenderTargetDescriptor, Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui::{Theme, ThemeConfig};
use fret_undo::{CoalesceKey, DocumentId, UndoRecord, UndoService, ValueTx};
use glam::{Mat4, Quat, Vec2, Vec3};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GizmoOpMaskPreset {
    Translate,
    Rotate,
    Scale,
    Universal,
    UniversalArcball,
    BoundsOnly,
}

impl GizmoOpMaskPreset {
    const ALL: [Self; 6] = [
        Self::Translate,
        Self::Rotate,
        Self::Scale,
        Self::Universal,
        Self::UniversalArcball,
        Self::BoundsOnly,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Translate => "Translate (axis+plane+view)",
            Self::Rotate => "Rotate (axis+view+arcball)",
            Self::Scale => "Scale (axis+plane+uniform+bounds)",
            Self::Universal => "Universal (t + r + s-axis)",
            Self::UniversalArcball => "Universal (t + r + arcball + s-axis)",
            Self::BoundsOnly => "Bounds (box scaling only)",
        }
    }

    fn mask(self) -> GizmoOps {
        match self {
            Self::Translate => GizmoOps::translate_all(),
            Self::Rotate => GizmoOps::rotate_all(),
            Self::Scale => GizmoOps::scale_all(),
            Self::Universal => {
                GizmoOps::translate_all()
                    | GizmoOps::rotate_axis()
                    | GizmoOps::rotate_view()
                    | GizmoOps::scale_axis()
            }
            Self::UniversalArcball => {
                GizmoOps::translate_all()
                    | GizmoOps::rotate_axis()
                    | GizmoOps::rotate_view()
                    | GizmoOps::rotate_arcball()
                    | GizmoOps::scale_axis()
            }
            Self::BoundsOnly => GizmoOps::scale_bounds(),
        }
    }
}

#[derive(Debug, Clone)]
struct OverlayTextCache {
    last_text: String,
    last_scale_bits: u32,
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<fret_core::text::TextMetrics>,
}

impl Default for OverlayTextCache {
    fn default() -> Self {
        Self {
            last_text: String::new(),
            last_scale_bits: 0,
            blob: None,
            metrics: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct ViewGizmoLabelCache {
    last_scale_bits: u32,
    blob_x: Option<fret_core::TextBlobId>,
    blob_y: Option<fret_core::TextBlobId>,
    blob_z: Option<fret_core::TextBlobId>,
    blob_p: Option<fret_core::TextBlobId>,
    blob_o: Option<fret_core::TextBlobId>,
    metrics_x: Option<fret_core::text::TextMetrics>,
    metrics_y: Option<fret_core::text::TextMetrics>,
    metrics_z: Option<fret_core::text::TextMetrics>,
    metrics_p: Option<fret_core::text::TextMetrics>,
    metrics_o: Option<fret_core::text::TextMetrics>,
}

impl ViewGizmoLabelCache {
    fn release_all(&mut self, services: &mut dyn fret_core::UiServices) {
        for blob in [
            self.blob_x.take(),
            self.blob_y.take(),
            self.blob_z.take(),
            self.blob_p.take(),
            self.blob_o.take(),
        ]
        .into_iter()
        .flatten()
        {
            services.text().release(blob);
        }
        self.metrics_x = None;
        self.metrics_y = None;
        self.metrics_z = None;
        self.metrics_p = None;
        self.metrics_o = None;
    }

    fn ensure(&mut self, services: &mut dyn fret_core::UiServices, scale_factor: f32) {
        let scale_bits = scale_factor.to_bits();
        if self.last_scale_bits != scale_bits {
            self.release_all(services);
            self.last_scale_bits = scale_bits;
        }

        let style = TextStyle {
            font: fret_core::FontId::default(),
            size: Px(12.0),
            weight: FontWeight::BOLD,
            slant: fret_core::text::TextSlant::Normal,
            line_height: Some(Px(14.0)),
            letter_spacing_em: None,
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor,
        };

        let mut prepare =
            |text: &'static str,
             blob: &mut Option<fret_core::TextBlobId>,
             metrics: &mut Option<fret_core::text::TextMetrics>| {
                if blob.is_some() && metrics.is_some() {
                    return;
                }
                let (b, m) = services.text().prepare(text, &style, constraints);
                *blob = Some(b);
                *metrics = Some(m);
            };

        prepare("X", &mut self.blob_x, &mut self.metrics_x);
        prepare("Y", &mut self.blob_y, &mut self.metrics_y);
        prepare("Z", &mut self.blob_z, &mut self.metrics_z);
        prepare("P", &mut self.blob_p, &mut self.metrics_p);
        prepare("O", &mut self.blob_o, &mut self.metrics_o);
    }

    fn blob_and_metrics(
        &self,
        text: &'static str,
    ) -> Option<(fret_core::TextBlobId, fret_core::text::TextMetrics)> {
        match text {
            "X" => self.blob_x.zip(self.metrics_x),
            "Y" => self.blob_y.zip(self.metrics_y),
            "Z" => self.blob_z.zip(self.metrics_z),
            "P" => self.blob_p.zip(self.metrics_p),
            "O" => self.blob_o.zip(self.metrics_o),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct GizmoHudCache {
    last_text: String,
    last_scale_bits: u32,
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<fret_core::text::TextMetrics>,
}

#[derive(Debug, Clone, Copy)]
struct GizmoHudLastUpdate {
    phase: GizmoPhase,
    active: HandleId,
    result: GizmoResult,
}

#[derive(Debug, Default, Clone, Copy)]
struct GizmoHudState {
    hovered: Option<HandleId>,
    hovered_kind: Option<GizmoMode>,
    active: Option<HandleId>,
    last: Option<GizmoHudLastUpdate>,
    snap: bool,
}

fn gizmo_handle_label(handle: HandleId) -> String {
    match handle.0 {
        1 => "X".to_string(),
        2 => "Y".to_string(),
        3 => "Z".to_string(),
        4 => "Plane XY".to_string(),
        5 => "Plane XZ".to_string(),
        6 => "Plane YZ".to_string(),
        7 => "Uniform".to_string(),
        8 => "View ring".to_string(),
        9 => "Arcball".to_string(),
        10 => "Screen".to_string(),
        14 => "Scale plane XY".to_string(),
        15 => "Scale plane XZ".to_string(),
        16 => "Scale plane YZ".to_string(),
        20..=27 => {
            let bits = (handle.0 - 20) as u32;
            let sx = if (bits & 1) != 0 { "+" } else { "-" };
            let sy = if (bits & 2) != 0 { "+" } else { "-" };
            let sz = if (bits & 4) != 0 { "+" } else { "-" };
            format!("Bounds corner (X{sx} Y{sy} Z{sz})")
        }
        30..=35 => {
            let v = (handle.0 - 30) as u32;
            let axis = (v / 2) as usize;
            let max_side = (v % 2) == 1;
            let sign = if max_side { "+" } else { "-" };
            let axis_name = match axis {
                0 => "X",
                1 => "Y",
                _ => "Z",
            };
            format!("Bounds face ({axis_name}{sign})")
        }
        _ => format!("Handle {}", handle.0),
    }
}

fn gizmo_hud_text(state: GizmoHudState, config: GizmoConfig) -> Option<String> {
    let show = state.active.is_some() || state.hovered.is_some();
    if !show {
        return None;
    }

    let mut out = String::new();

    if let Some(active) = state.active {
        let _ = writeln!(&mut out, "Active: {}", gizmo_handle_label(active));
    } else if let Some(hovered) = state.hovered {
        let kind = state
            .hovered_kind
            .map(|k| format!("{k:?} "))
            .unwrap_or_default();
        let _ = writeln!(&mut out, "Hover: {kind}{}", gizmo_handle_label(hovered));
    }

    let snap = if state.snap { "ON" } else { "OFF" };
    let _ = write!(&mut out, "Snap: {snap}");
    if state.snap {
        if let Some(last) = state.last {
            match last.result {
                GizmoResult::Translation { .. } => {
                    if let Some(step) = config.translate_snap_step {
                        let _ = write!(&mut out, " (step={step:.3})");
                    }
                }
                GizmoResult::Rotation { .. } => {
                    if let Some(step) = config.rotate_snap_step_radians {
                        let _ = write!(&mut out, " (step={:.1}°)", step.to_degrees());
                    }
                }
                GizmoResult::Arcball { .. } => {
                    if let Some(step) = config.rotate_snap_step_radians {
                        let _ = write!(&mut out, " (step={:.1}°)", step.to_degrees());
                    }
                }
                GizmoResult::Scale { .. } => {
                    if last.active.0 >= 20 && last.active.0 <= 35 {
                        if let Some(step) = config.bounds_snap_step {
                            let _ = write!(
                                &mut out,
                                " (bounds_step=({:.2}, {:.2}, {:.2}))",
                                step.x, step.y, step.z
                            );
                        }
                    } else if let Some(step) = config.scale_snap_step {
                        let _ = write!(&mut out, " (step={step:.3})");
                    }
                }
            }
        }
    }
    out.push('\n');

    if let Some(last) = state.last {
        match last.result {
            GizmoResult::Translation { delta, total } => {
                let _ = writeln!(
                    &mut out,
                    "Δt=({:.3}, {:.3}, {:.3})   Σt=({:.3}, {:.3}, {:.3})",
                    delta.x, delta.y, delta.z, total.x, total.y, total.z
                );
            }
            GizmoResult::Rotation {
                axis,
                delta_radians,
                total_radians,
            } => {
                let _ = writeln!(
                    &mut out,
                    "Δr={:.1}°   Σr={:.1}°   axis=({:.2}, {:.2}, {:.2})",
                    delta_radians.to_degrees(),
                    total_radians.to_degrees(),
                    axis.x,
                    axis.y,
                    axis.z
                );
            }
            GizmoResult::Arcball { delta, total } => {
                let (_axis_d, angle_d) = delta.to_axis_angle();
                let (_axis_t, angle_t) = total.to_axis_angle();
                let _ = writeln!(
                    &mut out,
                    "Δr={:.1}°   Σr={:.1}°   (arcball)",
                    angle_d.to_degrees(),
                    angle_t.to_degrees()
                );
            }
            GizmoResult::Scale { delta, total } => {
                let _ = writeln!(
                    &mut out,
                    "Δs=({:.3}, {:.3}, {:.3})   Σs=({:.3}, {:.3}, {:.3})",
                    delta.x, delta.y, delta.z, total.x, total.y, total.z
                );
            }
        }

        let phase = match last.phase {
            GizmoPhase::Begin => "Begin",
            GizmoPhase::Update => "Update",
            GizmoPhase::Commit => "Commit",
            GizmoPhase::Cancel => "Cancel",
        };
        let _ = writeln!(&mut out, "Phase: {phase}");
    }

    Some(out)
}

#[derive(Debug, Clone, Copy)]
struct FrameAnim {
    target: Vec3,
    distance: f32,
    yaw_radians: f32,
    pitch_radians: f32,
    ortho_half_height: f32,
    target_velocity: Vec3,
    distance_velocity: f32,
    yaw_velocity: f32,
    pitch_velocity: f32,
    ortho_half_height_velocity: f32,
    smooth_time_s: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrbitProjection {
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone, Copy)]
struct OrbitCamera {
    target: Vec3,
    yaw_radians: f32,
    pitch_radians: f32,
    distance: f32,
    ortho_half_height: f32,
    projection: OrbitProjection,
    orbiting: bool,
    panning: bool,
    last_cursor_px: Vec2,
    frame_anim: Option<FrameAnim>,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        // Roughly matches the previous hard-coded view: eye = (1.6, 1.2, 2.2), target = (0,0,0).
        let distance = 2.95;
        Self {
            target: Vec3::ZERO,
            yaw_radians: 0.94,
            pitch_radians: 0.42,
            distance,
            ortho_half_height: distance_to_ortho_half_height(distance),
            projection: OrbitProjection::Perspective,
            orbiting: false,
            panning: false,
            last_cursor_px: Vec2::ZERO,
            frame_anim: None,
        }
    }
}

type Vertex = Overlay3dVertex;
type LineVertex = Overlay3dLineVertex;

const CAMERA_NEAR: f32 = 0.05;
const CAMERA_FAR: f32 = 50.0;
const CAMERA_FOV_Y_RADIANS: f32 = 55.0_f32.to_radians();

fn distance_to_ortho_half_height(distance: f32) -> f32 {
    (distance.max(0.0) * (CAMERA_FOV_Y_RADIANS * 0.5).tan()).max(0.01)
}

fn ortho_half_height_to_distance(ortho_half_height: f32) -> f32 {
    (ortho_half_height.max(0.0) / (CAMERA_FOV_Y_RADIANS * 0.5).tan()).max(0.05)
}

type Uniforms = Overlay3dUniforms;

struct Gizmo3dDemoTarget {
    id: RenderTargetId,
    size: (u32, u32),
    color: wgpu::Texture,
    depth: wgpu::Texture,
}

#[derive(Debug, Clone, Copy)]
enum SelectionOp {
    Replace,
    Add,
    Subtract,
    Toggle,
}

fn selection_op(modifiers: &fret_core::Modifiers) -> SelectionOp {
    if modifiers.alt || modifiers.alt_gr {
        SelectionOp::Subtract
    } else if modifiers.ctrl || modifiers.meta {
        SelectionOp::Toggle
    } else if modifiers.shift {
        SelectionOp::Add
    } else {
        SelectionOp::Replace
    }
}

fn apply_click_selection_op(
    selection: &mut Vec<GizmoTargetId>,
    active_target: &mut GizmoTargetId,
    hit: Option<GizmoTargetId>,
    op: SelectionOp,
) {
    match op {
        SelectionOp::Replace => match hit {
            Some(id) => {
                selection.clear();
                selection.push(id);
                *active_target = id;
            }
            None => {
                selection.clear();
            }
        },
        SelectionOp::Add => {
            let Some(id) = hit else { return };
            if !selection.contains(&id) {
                selection.push(id);
            }
            *active_target = id;
        }
        SelectionOp::Subtract => {
            let Some(id) = hit else { return };
            if let Some(pos) = selection.iter().position(|v| *v == id) {
                selection.remove(pos);
                if *active_target == id {
                    if let Some(next) = selection.first().copied() {
                        *active_target = next;
                    }
                }
            }
        }
        SelectionOp::Toggle => {
            let Some(id) = hit else { return };
            if let Some(pos) = selection.iter().position(|v| *v == id) {
                selection.remove(pos);
                if *active_target == id {
                    if let Some(next) = selection.first().copied() {
                        *active_target = next;
                    }
                }
            } else {
                selection.push(id);
                *active_target = id;
            }
        }
    }
}

fn apply_marquee_selection_op(
    base: &[GizmoTargetId],
    hits: &[(GizmoTargetId, f32)],
    op: SelectionOp,
) -> (Vec<GizmoTargetId>, Option<(GizmoTargetId, f32)>) {
    match op {
        SelectionOp::Replace => {
            let result: Vec<GizmoTargetId> = hits.iter().map(|(id, _z01)| *id).collect();
            let nearest = hits.iter().min_by(|a, b| a.1.total_cmp(&b.1)).copied();
            (result, nearest)
        }
        SelectionOp::Add => {
            let mut result = base.to_vec();
            for (id, _z01) in hits {
                if !result.contains(id) {
                    result.push(*id);
                }
            }
            let nearest = hits.iter().min_by(|a, b| a.1.total_cmp(&b.1)).copied();
            (result, nearest)
        }
        SelectionOp::Subtract => {
            let mut result: Vec<GizmoTargetId> = base.to_vec();
            for (id, _z01) in hits {
                if let Some(pos) = result.iter().position(|v| *v == *id) {
                    result.remove(pos);
                }
            }
            (result, None)
        }
        SelectionOp::Toggle => {
            let mut result: Vec<GizmoTargetId> = base.to_vec();
            let mut nearest_added: Option<(GizmoTargetId, f32)> = None;
            for (id, z01) in hits {
                if let Some(pos) = result.iter().position(|v| *v == *id) {
                    result.remove(pos);
                } else {
                    result.push(*id);
                    if nearest_added
                        .as_ref()
                        .is_none_or(|(_best_id, best_z01)| *z01 < *best_z01)
                    {
                        nearest_added = Some((*id, *z01));
                    }
                }
            }
            (result, nearest_added)
        }
    }
}

fn unit_cube_world_aabb(transform: Transform3d) -> Option<(Vec3, Vec3)> {
    if !transform.translation.is_finite()
        || !transform.rotation.is_finite()
        || !transform.scale.is_finite()
    {
        return None;
    }

    let half = 0.4;
    let corners = [
        Vec3::new(-half, -half, -half),
        Vec3::new(half, -half, -half),
        Vec3::new(-half, half, -half),
        Vec3::new(half, half, -half),
        Vec3::new(-half, -half, half),
        Vec3::new(half, -half, half),
        Vec3::new(-half, half, half),
        Vec3::new(half, half, half),
    ];

    let mut any = false;
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for local in corners {
        let world = transform.rotation * (local * transform.scale) + transform.translation;
        if !world.is_finite() {
            continue;
        }
        any = true;
        min = min.min(world);
        max = max.max(world);
    }

    any.then_some((min, max))
}

fn targets_world_aabb(targets: &[GizmoTarget3d]) -> Option<(Vec3, Vec3)> {
    let mut any = false;
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for t in targets {
        let Some((tmin, tmax)) = unit_cube_world_aabb(t.transform) else {
            continue;
        };
        any = true;
        min = min.min(tmin);
        max = max.max(tmax);
    }

    any.then_some((min, max))
}

fn smooth_damp_f32(
    current: f32,
    target: f32,
    current_velocity: &mut f32,
    smooth_time_s: f32,
    dt_seconds: f32,
) -> f32 {
    let smooth_time_s = smooth_time_s.max(1e-4);
    let omega = 2.0 / smooth_time_s;
    let x = omega * dt_seconds;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    let change = current - target;
    let temp = (*current_velocity + omega * change) * dt_seconds;
    *current_velocity = (*current_velocity - omega * temp) * exp;

    let mut output = target + (change + temp) * exp;

    // Prevent overshoot.
    if (target - current > 0.0) == (output > target) {
        output = target;
        *current_velocity = 0.0;
    }

    output
}

fn smooth_damp_vec3(
    current: Vec3,
    target: Vec3,
    current_velocity: &mut Vec3,
    smooth_time_s: f32,
    dt_seconds: f32,
) -> Vec3 {
    let mut vx = current_velocity.x;
    let mut vy = current_velocity.y;
    let mut vz = current_velocity.z;

    let x = smooth_damp_f32(current.x, target.x, &mut vx, smooth_time_s, dt_seconds);
    let y = smooth_damp_f32(current.y, target.y, &mut vy, smooth_time_s, dt_seconds);
    let z = smooth_damp_f32(current.z, target.z, &mut vz, smooth_time_s, dt_seconds);

    *current_velocity = Vec3::new(vx, vy, vz);
    Vec3::new(x, y, z)
}

fn wrap_angle_pi(radians: f32) -> f32 {
    let two_pi = std::f32::consts::PI * 2.0;
    let mut x = radians % two_pi;
    if x > std::f32::consts::PI {
        x -= two_pi;
    } else if x < -std::f32::consts::PI {
        x += two_pi;
    }
    x
}

fn smooth_damp_angle(
    current: f32,
    target: f32,
    current_velocity: &mut f32,
    smooth_time_s: f32,
    dt_seconds: f32,
) -> f32 {
    let adjusted_target = current + wrap_angle_pi(target - current);
    smooth_damp_f32(
        current,
        adjusted_target,
        current_velocity,
        smooth_time_s,
        dt_seconds,
    )
}

fn step_frame_anim(camera: &mut OrbitCamera, dt_seconds: f32) -> bool {
    if camera.orbiting || camera.panning {
        camera.frame_anim = None;
        return false;
    }

    let Some(mut anim) = camera.frame_anim else {
        return false;
    };

    let dt_seconds = dt_seconds.clamp(0.0, 0.1);
    if dt_seconds <= 0.0 {
        camera.frame_anim = Some(anim);
        return true;
    }

    camera.target = smooth_damp_vec3(
        camera.target,
        anim.target,
        &mut anim.target_velocity,
        anim.smooth_time_s,
        dt_seconds,
    );
    camera.distance = smooth_damp_f32(
        camera.distance,
        anim.distance,
        &mut anim.distance_velocity,
        anim.smooth_time_s,
        dt_seconds,
    )
    .clamp(0.2, 25.0);

    camera.yaw_radians = smooth_damp_angle(
        camera.yaw_radians,
        anim.yaw_radians,
        &mut anim.yaw_velocity,
        anim.smooth_time_s,
        dt_seconds,
    );
    camera.pitch_radians = smooth_damp_f32(
        camera.pitch_radians,
        anim.pitch_radians,
        &mut anim.pitch_velocity,
        anim.smooth_time_s,
        dt_seconds,
    )
    .clamp(-1.55, 1.55);

    camera.ortho_half_height = smooth_damp_f32(
        camera.ortho_half_height,
        anim.ortho_half_height,
        &mut anim.ortho_half_height_velocity,
        anim.smooth_time_s,
        dt_seconds,
    )
    .clamp(0.01, 1000.0);

    let done = (camera.target - anim.target).length() <= 1e-3
        && (camera.distance - anim.distance).abs() <= 1e-3
        && wrap_angle_pi(camera.yaw_radians - anim.yaw_radians).abs() <= 1e-3
        && (camera.pitch_radians - anim.pitch_radians).abs() <= 1e-3
        && (camera.ortho_half_height - anim.ortho_half_height).abs() <= 1e-3
        && anim.target_velocity.length() <= 1e-3
        && anim.distance_velocity.abs() <= 1e-3
        && anim.ortho_half_height_velocity.abs() <= 1e-3;

    if done {
        camera.target = anim.target;
        camera.distance = anim.distance.clamp(0.2, 25.0);
        camera.yaw_radians = anim.yaw_radians;
        camera.pitch_radians = anim.pitch_radians.clamp(-1.55, 1.55);
        camera.ortho_half_height = anim.ortho_half_height.clamp(0.01, 1000.0);
        camera.frame_anim = None;
        false
    } else {
        camera.frame_anim = Some(anim);
        true
    }
}

fn frame_aabb(
    camera: &mut OrbitCamera,
    viewport_px: (u32, u32),
    min: Vec3,
    max: Vec3,
    smooth_time_s: f32,
) {
    let center = (min + max) * 0.5;
    let radius = ((max - min).length() * 0.5).max(0.001);

    let (w, h) = viewport_px;
    let aspect = (w.max(1) as f32) / (h.max(1) as f32);

    let margin = 1.25;
    let (dist, ortho_half_height) = match camera.projection {
        OrbitProjection::Perspective => {
            let fov_x = 2.0 * ((CAMERA_FOV_Y_RADIANS * 0.5).tan() * aspect).atan();
            let fov = CAMERA_FOV_Y_RADIANS.min(fov_x).max(0.001);
            let dist = (radius * margin) / (fov * 0.5).tan();
            (dist, camera.ortho_half_height)
        }
        OrbitProjection::Orthographic => {
            let half_h = (radius * margin / aspect.min(1.0)).max(0.01);
            let dist = camera.distance.max(radius * margin * 2.0);
            (dist, half_h)
        }
    };

    camera.frame_anim = Some(FrameAnim {
        target: center,
        distance: dist.clamp(0.2, 25.0),
        yaw_radians: camera.yaw_radians,
        pitch_radians: camera.pitch_radians,
        ortho_half_height: ortho_half_height.clamp(0.01, 1000.0),
        target_velocity: Vec3::ZERO,
        distance_velocity: 0.0,
        yaw_velocity: 0.0,
        pitch_velocity: 0.0,
        ortho_half_height_velocity: 0.0,
        smooth_time_s: smooth_time_s.max(1e-4),
    });
}
#[derive(Debug, Clone, Copy)]
struct PendingSelection {
    start_cursor_px: Vec2,
    click_count: u8,
}

#[derive(Debug, Clone, Copy)]
struct MarqueeSelection {
    start_cursor_px: Vec2,
    cursor_px: Vec2,
    op: SelectionOp,
}

#[derive(Debug)]
struct Gizmo3dDemoModel {
    viewport_target: RenderTargetId,
    viewport_px: (u32, u32),
    gizmo: Gizmo,
    view_gizmo: ViewGizmo,
    theme_preset_index: usize,
    op_mask_enabled: bool,
    op_mask_preset_index: usize,
    show_help: bool,
    targets: Vec<GizmoTarget3d>,
    selection: Vec<GizmoTargetId>,
    marquee_preview: Vec<GizmoTargetId>,
    active_target: GizmoTargetId,
    selection_before_select: Option<Vec<GizmoTargetId>>,
    active_before_select: Option<GizmoTargetId>,
    drag_start_targets: Option<Vec<GizmoTarget3d>>,
    pending_selection: Option<PendingSelection>,
    marquee: Option<MarqueeSelection>,
    input: GizmoInput,
    camera: OrbitCamera,
    last_frame_instant: Option<Instant>,
    hud: GizmoHudState,
}

impl Gizmo3dDemoModel {
    fn is_busy(&self) -> bool {
        self.input.dragging
            || self.gizmo.state.active.is_some()
            || self.pending_selection.is_some()
            || self.marquee.is_some()
    }

    fn op_mask_preset(&self) -> GizmoOpMaskPreset {
        let idx = self.op_mask_preset_index % GizmoOpMaskPreset::ALL.len();
        GizmoOpMaskPreset::ALL[idx]
    }

    fn set_op_mask_preset(&mut self, preset: GizmoOpMaskPreset) {
        let idx = GizmoOpMaskPreset::ALL
            .iter()
            .position(|p| *p == preset)
            .unwrap_or(0);
        self.op_mask_preset_index = idx;
        self.apply_op_mask();
    }

    fn apply_op_mask(&mut self) {
        if self.op_mask_enabled {
            let preset = self.op_mask_preset();
            self.gizmo.config.operation_mask = Some(preset.mask());
        } else {
            self.gizmo.config.operation_mask = None;
        }
    }

    fn overlay_text(&self) -> String {
        let mut out = String::new();
        out.push_str("Gizmo3D Demo\n");
        out.push_str("Controls:\n");
        out.push_str("  T/R/S/U: translate/rotate/scale/universal\n");
        out.push_str("  L: local/world   P: pivot active/center\n");
        out.push_str("  M: toggle op mask   [ / ]: prev/next preset\n");
        out.push_str("  V: cycle size policy (pixels/clamped/bounds)\n");
        out.push_str("  O: toggle depth mode (depth test / on top)\n");
        out.push_str("  Y: cycle theme (Fret/Godot/HardHacker)\n");
        out.push_str("  ; / ': bounds adjust (Shift: bigger step)\n");
        out.push_str("  -/=: gizmo size   ,/.: thickness + pick radius (Shift: bigger step)\n");
        out.push_str("  H: toggle help\n");
        out.push_str("  Esc: cancel drag / selection\n");
        out.push_str("  Ctrl+A: select all (Shift: clear)\n");
        out.push('\n');

        out.push_str(&format!(
            "Mode: {:?}   Orientation: {:?}   Pivot: {:?}\n",
            self.gizmo.config.mode, self.gizmo.config.orientation, self.gizmo.config.pivot_mode
        ));
        out.push_str(&format!(
            "Gizmo: size_px={:.0}   thickness_px={:.0}   pick_radius_px={:.0}\n",
            self.gizmo.config.size_px,
            self.gizmo.config.line_thickness_px,
            self.gizmo.config.pick_radius_px
        ));
        out.push_str(&format!(
            "Gizmo: size_policy={:?}\n",
            self.gizmo.config.size_policy
        ));
        out.push_str(&format!(
            "Gizmo: depth_mode={:?}\n",
            self.gizmo.config.depth_mode
        ));
        out.push_str(&format!(
            "Theme preset: {}\n",
            DEMO_THEME_PRESETS[self.theme_preset_index % DEMO_THEME_PRESETS.len()].0
        ));

        if self.op_mask_enabled {
            let preset = self.op_mask_preset();
            out.push_str(&format!("Op mask: ON   Preset: {}\n", preset.name()));
            out.push_str(&format!(
                "  mask={:?}\n",
                self.gizmo
                    .config
                    .operation_mask
                    .unwrap_or_else(GizmoOps::empty)
            ));
        } else {
            out.push_str("Op mask: OFF\n");
        }

        out.push_str(&format!(
            "Selection: {}   Active: {}\n",
            self.selection.len(),
            self.active_target.0
        ));
        out
    }
}

impl Default for Gizmo3dDemoModel {
    fn default() -> Self {
        let mut gizmo_cfg = GizmoConfig::editor_default();
        gizmo_cfg.translate_snap_step = Some(0.25);
        gizmo_cfg.bounds_snap_step = Some(Vec3::splat(0.5));
        gizmo_cfg.show_bounds = true;
        let targets = vec![
            GizmoTarget3d {
                id: GizmoTargetId(1),
                transform: Transform3d {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                local_bounds: Some(Aabb3 {
                    min: Vec3::splat(-0.5),
                    max: Vec3::splat(0.5),
                }),
            },
            GizmoTarget3d {
                id: GizmoTargetId(2),
                transform: Transform3d {
                    translation: Vec3::new(1.25, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                local_bounds: Some(Aabb3 {
                    min: Vec3::splat(-0.5),
                    max: Vec3::splat(0.5),
                }),
            },
            GizmoTarget3d {
                id: GizmoTargetId(3),
                transform: Transform3d {
                    translation: Vec3::new(-1.0, 0.0, -0.75),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                local_bounds: Some(Aabb3 {
                    min: Vec3::splat(-0.5),
                    max: Vec3::splat(0.5),
                }),
            },
        ];
        let mut view_gizmo_cfg = ViewGizmoConfig::default();
        view_gizmo_cfg.depth_range = gizmo_cfg.depth_range;
        view_gizmo_cfg.anchor = ViewGizmoAnchor::TopRight;
        let view_gizmo = ViewGizmo::new(view_gizmo_cfg);
        Self {
            viewport_target: RenderTargetId::default(),
            viewport_px: (960, 540),
            gizmo: Gizmo::new(gizmo_cfg),
            view_gizmo,
            theme_preset_index: 0,
            op_mask_enabled: false,
            op_mask_preset_index: 0,
            show_help: true,
            targets,
            selection: vec![GizmoTargetId(1)],
            marquee_preview: Vec::new(),
            active_target: GizmoTargetId(1),
            selection_before_select: None,
            active_before_select: None,
            drag_start_targets: None,
            pending_selection: None,
            marquee: None,
            input: GizmoInput {
                cursor_px: Vec2::ZERO,
                hovered: true,
                drag_started: false,
                dragging: false,
                snap: false,
                cancel: false,
            },
            camera: OrbitCamera::default(),
            last_frame_instant: None,
            hud: GizmoHudState::default(),
        }
    }
}

const DEMO_THEME_PRESETS: [(&str, &str); 3] = [
    ("Fret Default", "themes/fret-default-dark.json"),
    ("Godot Default", "themes/godot-default-dark.json"),
    ("HardHacker", "themes/hardhacker-dark.json"),
];

fn apply_viewport_gizmo_theme(theme: &Theme, model: &mut Gizmo3dDemoModel) {
    model.gizmo.config.x_color = theme.color_required("color.viewport.gizmo.x");
    model.gizmo.config.y_color = theme.color_required("color.viewport.gizmo.y");
    model.gizmo.config.z_color = theme.color_required("color.viewport.gizmo.z");
    model.gizmo.config.hover_color = theme.color_required("color.viewport.gizmo.hover");

    model.view_gizmo.config.x_color = model.gizmo.config.x_color;
    model.view_gizmo.config.y_color = model.gizmo.config.y_color;
    model.view_gizmo.config.z_color = model.gizmo.config.z_color;
    model.view_gizmo.config.hover_color = model.gizmo.config.hover_color;
}

#[derive(Default)]
struct Gizmo3dDemoService {
    per_window: HashMap<AppWindowId, fret_runtime::Model<Gizmo3dDemoModel>>,
}

struct Gizmo3dDemoViewportOverlayService {
    cache: Overlay3dCache<(AppWindowId, RenderTargetId)>,
}

impl Default for Gizmo3dDemoViewportOverlayService {
    fn default() -> Self {
        Self {
            cache: Overlay3dCache::new(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                wgpu::TextureFormat::Depth24Plus,
            ),
        }
    }
}

impl Gizmo3dDemoViewportOverlayService {
    fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: AppWindowId,
        target: RenderTargetId,
        uniforms: Uniforms,
        cpu: &Overlay3dCpuBuilder,
    ) -> Overlay3dPipelines {
        let entry = self.cache.ensure(device, (window, target));
        entry.update_uniform(queue, &uniforms);
        entry.batch.upload(
            device,
            queue,
            cpu.solid_test(),
            cpu.solid_ghost(),
            cpu.solid_always(),
            cpu.line_test(),
            cpu.line_ghost(),
            cpu.line_always(),
        );
        entry.overlay.clone()
    }

    fn record(&self, window: AppWindowId, target: RenderTargetId, pass: &mut wgpu::RenderPass<'_>) {
        let Some(entry) = self.cache.get(&(window, target)) else {
            return;
        };
        entry.batch.record(&entry.overlay, pass);
    }
}

struct Gizmo3dDemoViewportOverlayHooks;

impl ViewportOverlay3dHooks for Gizmo3dDemoViewportOverlayHooks {
    fn record(
        &self,
        app: &mut App,
        window: AppWindowId,
        target: RenderTargetId,
        pass: &mut wgpu::RenderPass<'_>,
        _ctx: &ViewportOverlay3dContext,
    ) {
        let Some(svc) = app.global::<Gizmo3dDemoViewportOverlayService>() else {
            return;
        };
        svc.record(window, target, pass);
    }
}

struct Gizmo3dDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<Plot3dModel>,
    demo: fret_runtime::Model<Gizmo3dDemoModel>,
    overlay: OverlayTextCache,
    view_gizmo_labels: ViewGizmoLabelCache,
    hud: GizmoHudCache,
    overlay_cpu: Overlay3dCpuBuilder,
    target: Option<Gizmo3dDemoTarget>,
    doc: DocumentId,
    warmup_frames_remaining: u8,
}

#[derive(Default)]
struct Gizmo3dDemoDriver;

impl Gizmo3dDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> Gizmo3dDemoWindowState {
        let plot = app.models_mut().insert(Plot3dModel {
            viewport: Plot3dViewport {
                target: RenderTargetId::default(),
                target_px_size: (960, 540),
                fit: ViewportFit::Contain,
                opacity: 1.0,
            },
        });

        let demo = app.models_mut().insert(Gizmo3dDemoModel::default());
        let theme = Theme::global(&*app).clone();
        let _ = demo.update(app, |m, _cx| {
            apply_viewport_gizmo_theme(&theme, m);
        });

        app.with_global_mut(Gizmo3dDemoService::default, |svc, _app| {
            svc.per_window.insert(window, demo.clone());
        });

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let doc: DocumentId = "gizmo3d_demo.scene".into();
        app.with_global_mut(
            || UndoService::<ValueTx<Vec<GizmoTarget3d>>>::with_limit(256),
            |undo, _app| {
                undo.set_active_document(window, doc.clone());
            },
        );

        Gizmo3dDemoWindowState {
            ui,
            root: None,
            plot,
            demo,
            overlay: OverlayTextCache::default(),
            view_gizmo_labels: ViewGizmoLabelCache::default(),
            hud: GizmoHudCache::default(),
            overlay_cpu: Overlay3dCpuBuilder::default(),
            target: None,
            doc,
            warmup_frames_remaining: 3,
        }
    }

    fn ensure_target(
        app: &mut App,
        window: AppWindowId,
        state: &mut Gizmo3dDemoWindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
    ) -> (
        RenderTargetId,
        wgpu::TextureView,
        wgpu::TextureView,
        (u32, u32),
    ) {
        let desired_size = state
            .plot
            .read(app, |_app, m| m.viewport.target_px_size)
            .unwrap_or((960, 540));

        let needs_new = state.target.as_ref().is_none_or(|t| t.size != desired_size);

        if needs_new {
            let (w, h) = desired_size;
            let w = w.max(1);
            let h = h.max(1);
            let size = (w, h);

            let color = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("gizmo3d demo color target"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let depth = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("gizmo3d demo depth target"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            let view_for_registry = color.create_view(&wgpu::TextureViewDescriptor::default());

            let id = if let Some(prev) = state.target.take() {
                renderer.update_render_target(
                    prev.id,
                    RenderTargetDescriptor {
                        view: view_for_registry,
                        size,
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        color_space: RenderTargetColorSpace::Srgb,
                    },
                );
                prev.id
            } else {
                renderer.register_render_target(RenderTargetDescriptor {
                    view: view_for_registry,
                    size,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    color_space: RenderTargetColorSpace::Srgb,
                })
            };

            state.target = Some(Gizmo3dDemoTarget {
                id,
                size,
                color,
                depth,
            });

            let _ = state.plot.update(app, |m, _cx| {
                m.viewport.target = id;
                m.viewport.target_px_size = size;
            });
            let _ = state.demo.update(app, |m, _cx| {
                m.viewport_target = id;
                m.viewport_px = size;
            });

            app.request_redraw(window);
        }

        let target = state.target.as_ref().expect("target ensured");
        let color_view = target
            .color
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = target
            .depth
            .create_view(&wgpu::TextureViewDescriptor::default());
        (target.id, color_view, depth_view, target.size)
    }

    #[cfg(any())]
    fn ensure_gpu(state: &mut Gizmo3dDemoWindowState, context: &WgpuContext) {
        if state.gpu.is_some() {
            return;
        }

        state.gpu = Some(Gizmo3dDemoGpu {
            overlay: Overlay3dPipelines::new(
                &context.device,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                wgpu::TextureFormat::Depth24Plus,
            ),
        });

        #[cfg(any())]
        {
            let shader = context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("gizmo3d demo shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        r#"
struct Globals {
  view_proj: mat4x4f,
  viewport_and_thickness: vec4f,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VsIn {
  @location(0) pos: vec3f,
  @location(1) color: vec4f,
};

struct VsOut {
  @builtin(position) pos: vec4f,
  @location(0) color: vec4f,
};

@vertex
fn vs_main_tri(in: VsIn) -> VsOut {
  var out: VsOut;
  out.pos = globals.view_proj * vec4f(in.pos, 1.0);
  out.color = in.color;
  return out;
}

struct LineVsIn {
  @location(0) a: vec3f,
  @location(1) b: vec3f,
  @location(2) t: f32,
  @location(3) side: f32,
  @location(4) color: vec4f,
};

@vertex
fn vs_main_thick_line(in: LineVsIn) -> VsOut {
  let clip_a = globals.view_proj * vec4f(in.a, 1.0);
  let clip_b = globals.view_proj * vec4f(in.b, 1.0);

  let viewport = globals.viewport_and_thickness.xy;
  let thickness_px = globals.viewport_and_thickness.z;

  let ndc_a = clip_a.xy / clip_a.w;
  let ndc_b = clip_b.xy / clip_b.w;
  let dir_px = (ndc_b - ndc_a) * viewport;

  var offset_ndc = vec2f(0.0, 0.0);
  if dot(dir_px, dir_px) > 1e-8 && thickness_px > 0.0 {
    let dir_px_norm = normalize(dir_px);
    let normal_px = vec2f(-dir_px_norm.y, dir_px_norm.x);
    offset_ndc = normal_px * (thickness_px / viewport) * 0.5;
  }

  let clip = mix(clip_a, clip_b, in.t);
  let ndc = clip.xy / clip.w;
  let ndc_out = ndc + offset_ndc * in.side;

  var out: VsOut;
  out.pos = vec4f(ndc_out * clip.w, clip.z, clip.w);
  out.color = in.color;
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  return in.color;
}
"#
                        .into(),
                    ),
                });

            let uniform = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("gizmo3d demo view_proj uniform"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group_layout =
                context
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("gizmo3d demo bgl"),
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    });

            let bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("gizmo3d demo bind group"),
                    layout: &bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    }],
                });

            let pipeline_layout =
                context
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("gizmo3d demo pipeline layout"),
                        bind_group_layouts: &[&bind_group_layout],
                        immediate_size: 0,
                    });

            let vertex_layout = wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
            };

            let line_vertex_layout = wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<LineVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![
                    0 => Float32x3, // a
                    1 => Float32x3, // b
                    2 => Float32,   // t
                    3 => Float32,   // side
                    4 => Float32x4  // color
                ],
            };

            let depth_state = wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            };

            let tri_pipeline =
                context
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("gizmo3d demo tri pipeline"),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main_tri"),
                            buffers: &[vertex_layout.clone()],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            ..Default::default()
                        },
                        depth_stencil: Some(depth_state.clone()),
                        multisample: wgpu::MultisampleState::default(),
                        multiview_mask: None,
                        cache: None,
                    });

            let gizmo_solid_depth_pipeline =
                context
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("gizmo3d demo gizmo solid depth pipeline"),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main_tri"),
                            buffers: &[vertex_layout.clone()],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            ..Default::default()
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                            depth_write_enabled: false,
                            bias: wgpu::DepthBiasState {
                                constant: -2,
                                slope_scale: -1.0,
                                clamp: 0.0,
                            },
                            ..depth_state.clone()
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        multiview_mask: None,
                        cache: None,
                    });

            let gizmo_solid_always_pipeline =
                context
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("gizmo3d demo gizmo solid always pipeline"),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main_tri"),
                            buffers: &[vertex_layout.clone()],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            ..Default::default()
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: wgpu::TextureFormat::Depth24Plus,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Always,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        multiview_mask: None,
                        cache: None,
                    });

            let thick_line_depth_pipeline =
                context
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("gizmo3d demo thick line depth pipeline"),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main_thick_line"),
                            buffers: &[line_vertex_layout.clone()],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            ..Default::default()
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                            depth_write_enabled: false,
                            // Pull gizmo slightly toward the camera to reduce z-fighting with scene geometry.
                            bias: wgpu::DepthBiasState {
                                constant: -2,
                                slope_scale: -1.0,
                                clamp: 0.0,
                            },
                            ..depth_state.clone()
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        multiview_mask: None,
                        cache: None,
                    });

            let thick_line_always_pipeline =
                context
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("gizmo3d demo thick line always pipeline"),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main_thick_line"),
                            buffers: &[line_vertex_layout],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            ..Default::default()
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: wgpu::TextureFormat::Depth24Plus,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Always,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        multiview_mask: None,
                        cache: None,
                    });

            let _ = (
                uniform,
                bind_group,
                tri_pipeline,
                gizmo_solid_depth_pipeline,
                gizmo_solid_always_pipeline,
                thick_line_depth_pipeline,
                thick_line_always_pipeline,
            );
        }
    }

    fn handle_undo_redo_shortcut(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Gizmo3dDemoWindowState,
        undo: bool,
    ) -> bool {
        let mut did_apply = false;

        // Always cancel any in-progress gizmo interaction before applying undo/redo.
        let _ = state.demo.update(app, |m, _cx| {
            let is_dragging = m.input.dragging || m.gizmo.state.active.is_some();
            if is_dragging {
                let view_projection = camera_view_projection(m.viewport_px, m.camera);
                let viewport = ViewportRect::new(
                    Vec2::ZERO,
                    Vec2::new(m.viewport_px.0 as f32, m.viewport_px.1 as f32),
                );
                let mut input = m.input;
                input.drag_started = false;
                input.dragging = false;
                input.cancel = true;

                let selected: Vec<GizmoTarget3d> = m
                    .targets
                    .iter()
                    .copied()
                    .filter(|t| m.selection.contains(&t.id))
                    .collect();
                if let Some(update) =
                    m.gizmo
                        .update(view_projection, viewport, input, m.active_target, &selected)
                {
                    if update.phase == GizmoPhase::Cancel {
                        if let Some(start) = m.drag_start_targets.take() {
                            for updated in start {
                                if let Some(target) =
                                    m.targets.iter_mut().find(|t| t.id == updated.id)
                                {
                                    target.transform = updated.transform;
                                }
                            }
                        }
                    }
                }
                m.drag_start_targets = None;
                m.input.cancel = false;
                m.input.dragging = false;
                m.input.drag_started = false;
            }
        });

        let _ = app.with_global_mut(
            || UndoService::<ValueTx<Vec<GizmoTarget3d>>>::with_limit(256),
            |undo_svc, app| {
                // Ensure the window routes edit.undo/edit.redo to this viewport document.
                undo_svc.set_active_document(window, state.doc.clone());

                let applied = if undo {
                    undo_svc
                        .undo_active_invertible(window, |rec| {
                            let _ = state.demo.update(app, |m, _cx| {
                                for updated in &rec.tx.after {
                                    if let Some(target) =
                                        m.targets.iter_mut().find(|t| t.id == updated.id)
                                    {
                                        target.transform = updated.transform;
                                    }
                                }
                            });
                            Ok::<(), ()>(())
                        })
                        .unwrap_or(false)
                } else {
                    undo_svc
                        .redo_active_invertible(window, |rec| {
                            let _ = state.demo.update(app, |m, _cx| {
                                for updated in &rec.tx.after {
                                    if let Some(target) =
                                        m.targets.iter_mut().find(|t| t.id == updated.id)
                                    {
                                        target.transform = updated.transform;
                                    }
                                }
                            });
                            Ok::<(), ()>(())
                        })
                        .unwrap_or(false)
                };
                did_apply |= applied;
            },
        );

        if did_apply {
            app.request_redraw(window);
        }
        did_apply
    }
}

fn append_cube_triangles(out: &mut Vec<Vertex>, transform: Transform3d, color: [f32; 4]) {
    let verts = [
        Vec3::new(-0.4, -0.4, 0.4),
        Vec3::new(0.4, -0.4, 0.4),
        Vec3::new(0.4, 0.4, 0.4),
        Vec3::new(-0.4, 0.4, 0.4),
        Vec3::new(-0.4, -0.4, -0.4),
        Vec3::new(0.4, -0.4, -0.4),
        Vec3::new(0.4, 0.4, -0.4),
        Vec3::new(-0.4, 0.4, -0.4),
    ];

    let idx: [usize; 36] = [
        0, 1, 2, 0, 2, 3, // front
        1, 5, 6, 1, 6, 2, // right
        5, 4, 7, 5, 7, 6, // back
        4, 0, 3, 4, 3, 7, // left
        3, 2, 6, 3, 6, 7, // top
        4, 5, 1, 4, 1, 0, // bottom
    ];

    for &i in &idx {
        let p = verts[i];
        let p = transform.rotation * (p * transform.scale) + transform.translation;
        out.push(Vertex {
            pos: p.to_array(),
            color,
        });
    }
}

fn pick_unit_cube_t(ray: fret_gizmo::Ray3d, transform: Transform3d) -> Option<f32> {
    let inv_rot = transform.rotation.inverse();
    let scale = transform.scale;
    if !scale.is_finite() {
        return None;
    }
    if scale.x.abs() < 1e-6 || scale.y.abs() < 1e-6 || scale.z.abs() < 1e-6 {
        return None;
    }

    let origin_local = inv_rot * (ray.origin - transform.translation);
    let dir_local = inv_rot * ray.dir;
    let origin_local = origin_local / scale;
    let dir_local = dir_local / scale;

    let min = Vec3::splat(-0.4);
    let max = Vec3::splat(0.4);

    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for axis in 0..3 {
        let o = origin_local[axis];
        let d = dir_local[axis];
        if d.abs() < 1e-8 {
            if o < min[axis] || o > max[axis] {
                return None;
            }
            continue;
        }

        let mut t1 = (min[axis] - o) / d;
        let mut t2 = (max[axis] - o) / d;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_max < t_min {
            return None;
        }
    }

    let t = if t_min >= 0.0 { t_min } else { t_max };
    (t.is_finite() && t >= 0.0).then_some(t)
}

fn pick_target_id(ray: fret_gizmo::Ray3d, targets: &[GizmoTarget3d]) -> Option<GizmoTargetId> {
    let mut best_id: Option<GizmoTargetId> = None;
    let mut best_t = f32::INFINITY;
    for t in targets {
        let Some(hit_t) = pick_unit_cube_t(ray, t.transform) else {
            continue;
        };
        if hit_t < best_t {
            best_t = hit_t;
            best_id = Some(t.id);
        }
    }
    best_id
}

fn marquee_rect(a: Vec2, b: Vec2) -> (Vec2, Vec2) {
    let min = Vec2::new(a.x.min(b.x), a.y.min(b.y));
    let max = Vec2::new(a.x.max(b.x), a.y.max(b.y));
    (min, max)
}

fn ndc_z_to_z01(depth: DepthRange, ndc_z: f32) -> f32 {
    match depth {
        DepthRange::ZeroToOne => ndc_z,
        DepthRange::NegOneToOne => (ndc_z + 1.0) * 0.5,
    }
    .clamp(0.0, 1.0)
}

fn project_target_screen_aabb(
    view_projection: Mat4,
    viewport: ViewportRect,
    depth: DepthRange,
    target: GizmoTarget3d,
) -> Option<(Vec2, Vec2, f32)> {
    let transform = target.transform;
    if !transform.translation.is_finite()
        || !transform.rotation.is_finite()
        || !transform.scale.is_finite()
    {
        return None;
    }

    let half = 0.4;
    let corners = [
        Vec3::new(-half, -half, -half),
        Vec3::new(half, -half, -half),
        Vec3::new(-half, half, -half),
        Vec3::new(half, half, -half),
        Vec3::new(-half, -half, half),
        Vec3::new(half, -half, half),
        Vec3::new(-half, half, half),
        Vec3::new(half, half, half),
    ];

    let mut any = false;
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    let mut best_z01 = f32::INFINITY;

    for local in corners {
        let world = transform.rotation * (local * transform.scale) + transform.translation;
        let Some(p) = fret_gizmo::project_point(view_projection, viewport, world, depth) else {
            continue;
        };
        if !p.screen.is_finite() || !p.w.is_finite() || p.w <= 0.0 {
            continue;
        }
        any = true;
        min = min.min(p.screen);
        max = max.max(p.screen);
        best_z01 = best_z01.min(ndc_z_to_z01(depth, p.ndc_z));
    }

    any.then_some((min, max, best_z01))
}

fn marquee_hits(
    view_projection: Mat4,
    viewport: ViewportRect,
    depth: DepthRange,
    targets: &[GizmoTarget3d],
    rect_min: Vec2,
    rect_max: Vec2,
) -> Vec<(GizmoTargetId, f32)> {
    let mut hits: Vec<(GizmoTargetId, f32)> = Vec::new();
    for t in targets {
        let Some((min, max, best_z01)) =
            project_target_screen_aabb(view_projection, viewport, depth, *t)
        else {
            continue;
        };

        if max.x < rect_min.x || min.x > rect_max.x || max.y < rect_min.y || min.y > rect_max.y {
            continue;
        }

        hits.push((t.id, best_z01));
    }
    hits
}

fn camera_view_projection(size: (u32, u32), camera: OrbitCamera) -> Mat4 {
    let (w, h) = size;
    let aspect = (w.max(1) as f32) / (h.max(1) as f32);
    let pitch = camera.pitch_radians.clamp(-1.55, 1.55);
    let yaw = camera.yaw_radians;
    let distance = camera.distance.max(0.05);
    let dir = Vec3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    );
    let eye = camera.target + dir * distance;
    let view = Mat4::look_at_rh(eye, camera.target, Vec3::Y);
    let proj = match camera.projection {
        OrbitProjection::Perspective => {
            Mat4::perspective_rh(CAMERA_FOV_Y_RADIANS, aspect, CAMERA_NEAR, CAMERA_FAR)
        }
        OrbitProjection::Orthographic => {
            let half_h = camera.ortho_half_height.max(0.01);
            let half_w = half_h * aspect.max(1e-6);
            Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, CAMERA_NEAR, CAMERA_FAR)
        }
    };
    proj * view
}

impl WinitAppDriver for Gizmo3dDemoDriver {
    type WindowState = Gizmo3dDemoWindowState;

    fn init(&mut self, app: &mut App, _main_window: AppWindowId) {
        app.with_global_mut(ViewportOverlay3dHooksService::default, |svc, _app| {
            svc.set(Arc::new(Gizmo3dDemoViewportOverlayHooks));
        });
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        let state = Self::build_ui(app, window);
        // Ensure we render at least one frame; otherwise the viewport surface can remain blank until
        // the first input event happens to request a redraw.
        app.request_redraw(window);
        app.push_effect(Effect::RequestAnimationFrame(window));
        state
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
        } = context;

        // Prefer focused-surface command handling (e.g. local widget histories) before falling
        // back to the window's active document undo stack (ADR 0136, ADR 0020).
        if state.ui.dispatch_command(app, services, &command) {
            return;
        }

        match command.as_str() {
            "edit.undo" => {
                let _ = self.handle_undo_redo_shortcut(app, window, state, true);
            }
            "edit.redo" => {
                let _ = self.handle_undo_redo_shortcut(app, window, state, false);
            }
            _ => {}
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        match event {
            Event::WindowCloseRequested => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                let mut did_cancel = false;
                let _ = state.demo.update(app, |m, _cx| {
                    let is_gizmo_dragging = m.input.dragging || m.gizmo.state.active.is_some();
                    let is_selecting = m.pending_selection.is_some() || m.marquee.is_some();

                    if !is_gizmo_dragging && !is_selecting {
                        return;
                    }

                    if is_selecting {
                        m.pending_selection = None;
                        m.marquee = None;
                        m.marquee_preview.clear();
                        if let Some(sel) = m.selection_before_select.take() {
                            m.selection = sel;
                        }
                        if let Some(active) = m.active_before_select.take() {
                            m.active_target = active;
                        }
                        did_cancel = true;
                        return;
                    }

                    let view_projection = camera_view_projection(m.viewport_px, m.camera);
                    let viewport = ViewportRect::new(
                        Vec2::ZERO,
                        Vec2::new(m.viewport_px.0 as f32, m.viewport_px.1 as f32),
                    );

                    let mut input = m.input;
                    input.drag_started = false;
                    input.dragging = false;
                    input.cancel = true;

                    let selected: Vec<GizmoTarget3d> = m
                        .targets
                        .iter()
                        .copied()
                        .filter(|t| m.selection.contains(&t.id))
                        .collect();

                    if let Some(update) =
                        m.gizmo
                            .update(view_projection, viewport, input, m.active_target, &selected)
                    {
                        if update.phase == GizmoPhase::Cancel {
                            if let Some(start) = m.drag_start_targets.take() {
                                for updated in start {
                                    if let Some(target) =
                                        m.targets.iter_mut().find(|t| t.id == updated.id)
                                    {
                                        target.transform = updated.transform;
                                    }
                                }
                            }
                            did_cancel = true;
                        }
                    }

                    m.input.cancel = false;
                    m.input.dragging = false;
                    m.input.drag_started = false;
                    m.selection_before_select = None;
                    m.active_before_select = None;
                });

                if did_cancel {
                    app.request_redraw(window);
                } else {
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                }
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyR,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    if m.op_mask_enabled {
                        m.set_op_mask_preset(GizmoOpMaskPreset::Rotate);
                    } else {
                        m.gizmo.config.mode = GizmoMode::Rotate;
                    }
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyS,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    if m.op_mask_enabled {
                        m.set_op_mask_preset(GizmoOpMaskPreset::Scale);
                    } else {
                        m.gizmo.config.mode = GizmoMode::Scale;
                    }
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyT,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    if m.op_mask_enabled {
                        m.set_op_mask_preset(GizmoOpMaskPreset::Translate);
                    } else {
                        m.gizmo.config.mode = GizmoMode::Translate;
                    }
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyU,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    if m.op_mask_enabled {
                        m.set_op_mask_preset(GizmoOpMaskPreset::Universal);
                    } else {
                        m.gizmo.config.mode = GizmoMode::Universal;
                    }
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyH,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    m.show_help = !m.show_help;
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyM,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    m.op_mask_enabled = !m.op_mask_enabled;
                    if m.op_mask_enabled {
                        // Pick a reasonable starting preset based on the current coarse mode.
                        let preset = match m.gizmo.config.mode {
                            GizmoMode::Translate => GizmoOpMaskPreset::Translate,
                            GizmoMode::Rotate => GizmoOpMaskPreset::Rotate,
                            GizmoMode::Scale => GizmoOpMaskPreset::Scale,
                            GizmoMode::Universal => GizmoOpMaskPreset::Universal,
                        };
                        m.set_op_mask_preset(preset);
                    } else {
                        m.apply_op_mask();
                    }
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyO,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    m.gizmo.config.depth_mode = match m.gizmo.config.depth_mode {
                        DepthMode::Test => DepthMode::Always,
                        DepthMode::Ghost | DepthMode::Always => DepthMode::Test,
                    };
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyY,
                repeat: false,
                ..
            } => {
                let mut next_index: Option<usize> = None;
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    let idx = (m.theme_preset_index + 1) % DEMO_THEME_PRESETS.len();
                    next_index = Some(idx);
                });

                let Some(next_index) = next_index else {
                    return;
                };
                let (_name, path) = DEMO_THEME_PRESETS[next_index];

                let Some(bytes) = fs::read(path).ok() else {
                    return;
                };
                let Ok(cfg) = ThemeConfig::from_slice(&bytes) else {
                    return;
                };

                Theme::with_global_mut(app, |theme| theme.apply_config(&cfg));

                let theme = Theme::global(&*app).clone();
                let _ = state.demo.update(app, |m, _cx| {
                    m.theme_preset_index = next_index;
                    apply_viewport_gizmo_theme(&theme, m);
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyV,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    m.gizmo.config.size_policy = match m.gizmo.config.size_policy {
                        GizmoSizePolicy::ConstantPixels => {
                            GizmoSizePolicy::PixelsClampedBySelectionBounds {
                                min_fraction_of_max_extent: 0.0,
                                max_fraction_of_max_extent: 1.50,
                            }
                        }
                        GizmoSizePolicy::PixelsClampedBySelectionBounds { .. } => {
                            GizmoSizePolicy::SelectionBounds {
                                fraction_of_max_extent: 1.2,
                            }
                        }
                        GizmoSizePolicy::SelectionBounds { .. } => GizmoSizePolicy::ConstantPixels,
                    };
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Semicolon,
                modifiers,
                repeat: false,
                ..
            } => {
                let step = if modifiers.shift { 0.25 } else { 0.05 };
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    match m.gizmo.config.size_policy {
                        GizmoSizePolicy::SelectionBounds {
                            ref mut fraction_of_max_extent,
                        } => {
                            *fraction_of_max_extent =
                                (*fraction_of_max_extent - step).clamp(0.05, 5.0);
                        }
                        GizmoSizePolicy::PixelsClampedBySelectionBounds {
                            ref mut min_fraction_of_max_extent,
                            max_fraction_of_max_extent,
                        } => {
                            *min_fraction_of_max_extent = (*min_fraction_of_max_extent - step)
                                .clamp(0.0, max_fraction_of_max_extent);
                        }
                        GizmoSizePolicy::ConstantPixels => {}
                    }
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Quote,
                modifiers,
                repeat: false,
                ..
            } => {
                let step = if modifiers.shift { 0.25 } else { 0.05 };
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    match m.gizmo.config.size_policy {
                        GizmoSizePolicy::SelectionBounds {
                            ref mut fraction_of_max_extent,
                        } => {
                            *fraction_of_max_extent =
                                (*fraction_of_max_extent + step).clamp(0.05, 5.0);
                        }
                        GizmoSizePolicy::PixelsClampedBySelectionBounds {
                            min_fraction_of_max_extent,
                            ref mut max_fraction_of_max_extent,
                        } => {
                            *max_fraction_of_max_extent = (*max_fraction_of_max_extent + step)
                                .clamp(min_fraction_of_max_extent, 5.0);
                        }
                        GizmoSizePolicy::ConstantPixels => {}
                    }
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Minus,
                modifiers,
                repeat: false,
                ..
            } => {
                let step = if modifiers.shift { 16.0 } else { 4.0 };
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    m.gizmo.config.size_px = (m.gizmo.config.size_px - step).clamp(24.0, 256.0);
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Equal,
                modifiers,
                repeat: false,
                ..
            } => {
                let step = if modifiers.shift { 16.0 } else { 4.0 };
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    m.gizmo.config.size_px = (m.gizmo.config.size_px + step).clamp(24.0, 256.0);
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Comma,
                modifiers,
                repeat: false,
                ..
            } => {
                let step = if modifiers.shift { 2.0 } else { 1.0 };
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    m.gizmo.config.line_thickness_px =
                        (m.gizmo.config.line_thickness_px - step).clamp(1.0, 24.0);
                    m.gizmo.config.pick_radius_px =
                        (m.gizmo.config.pick_radius_px - step).clamp(4.0, 32.0);
                    m.gizmo.config.bounds_handle_size_px =
                        (m.gizmo.config.bounds_handle_size_px - step).clamp(6.0, 32.0);
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Period,
                modifiers,
                repeat: false,
                ..
            } => {
                let step = if modifiers.shift { 2.0 } else { 1.0 };
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() {
                        return;
                    }
                    m.gizmo.config.line_thickness_px =
                        (m.gizmo.config.line_thickness_px + step).clamp(1.0, 24.0);
                    m.gizmo.config.pick_radius_px =
                        (m.gizmo.config.pick_radius_px + step).clamp(4.0, 32.0);
                    m.gizmo.config.bounds_handle_size_px =
                        (m.gizmo.config.bounds_handle_size_px + step).clamp(6.0, 32.0);
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::BracketLeft,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() || !m.op_mask_enabled {
                        return;
                    }
                    let n = GizmoOpMaskPreset::ALL.len();
                    m.op_mask_preset_index = (m.op_mask_preset_index + n - 1) % n;
                    m.apply_op_mask();
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::BracketRight,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.is_busy() || !m.op_mask_enabled {
                        return;
                    }
                    let n = GizmoOpMaskPreset::ALL.len();
                    m.op_mask_preset_index = (m.op_mask_preset_index + 1) % n;
                    m.apply_op_mask();
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyL,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.input.dragging || m.gizmo.state.active.is_some() {
                        return;
                    }
                    m.gizmo.config.orientation = match m.gizmo.config.orientation {
                        GizmoOrientation::World => GizmoOrientation::Local,
                        GizmoOrientation::Local => GizmoOrientation::World,
                    };
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyP,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.input.dragging || m.gizmo.state.active.is_some() {
                        return;
                    }
                    m.gizmo.config.pivot_mode = match m.gizmo.config.pivot_mode {
                        GizmoPivotMode::Active => GizmoPivotMode::Center,
                        GizmoPivotMode::Center => GizmoPivotMode::Active,
                    };
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyN,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.input.dragging || m.gizmo.state.active.is_some() || m.selection.is_empty()
                    {
                        return;
                    }
                    let Some(i) = m.selection.iter().position(|id| *id == m.active_target) else {
                        m.active_target = m.selection[0];
                        return;
                    };
                    m.active_target = m.selection[(i + 1) % m.selection.len()];
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyB,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    if m.input.dragging || m.gizmo.state.active.is_some() || m.selection.is_empty()
                    {
                        return;
                    }
                    let Some(i) = m.selection.iter().position(|id| *id == m.active_target) else {
                        m.active_target = m.selection[0];
                        return;
                    };
                    m.active_target = m.selection[(i + m.selection.len() - 1) % m.selection.len()];
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyF,
                modifiers,
                repeat: false,
            } => {
                let frame_all = modifiers.shift;
                let smooth_time_s = if frame_all { 0.32 } else { 0.18 };
                let _ = state.demo.update(app, |m, _cx| {
                    if m.input.dragging || m.gizmo.state.active.is_some() {
                        return;
                    }

                    let targets: Vec<GizmoTarget3d> = if frame_all || m.selection.is_empty() {
                        m.targets.clone()
                    } else {
                        m.targets
                            .iter()
                            .copied()
                            .filter(|t| m.selection.contains(&t.id))
                            .collect()
                    };

                    let Some((min, max)) = targets_world_aabb(&targets) else {
                        return;
                    };
                    frame_aabb(&mut m.camera, m.viewport_px, min, max, smooth_time_s);
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyA,
                modifiers,
                repeat: false,
            } if modifiers.ctrl || modifiers.meta => {
                let clear = modifiers.shift;
                let _ = state.demo.update(app, |m, _cx| {
                    let is_busy = m.input.dragging
                        || m.gizmo.state.active.is_some()
                        || m.pending_selection.is_some()
                        || m.marquee.is_some();
                    if is_busy {
                        return;
                    }

                    if clear {
                        m.selection.clear();
                        m.marquee_preview.clear();
                    } else {
                        m.selection = m.targets.iter().map(|t| t.id).collect();
                        if !m.selection.contains(&m.active_target) {
                            if let Some(id) = m.selection.first().copied() {
                                m.active_target = id;
                            }
                        }
                    }
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Digit1,
                modifiers,
                repeat: false,
            }
            | Event::KeyDown {
                key: fret_core::KeyCode::Digit2,
                modifiers,
                repeat: false,
            }
            | Event::KeyDown {
                key: fret_core::KeyCode::Digit3,
                modifiers,
                repeat: false,
            } => {
                let id = match event {
                    Event::KeyDown {
                        key: fret_core::KeyCode::Digit1,
                        ..
                    } => GizmoTargetId(1),
                    Event::KeyDown {
                        key: fret_core::KeyCode::Digit2,
                        ..
                    } => GizmoTargetId(2),
                    Event::KeyDown {
                        key: fret_core::KeyCode::Digit3,
                        ..
                    } => GizmoTargetId(3),
                    _ => return,
                };
                let op = selection_op(modifiers);
                let _ = state.demo.update(app, |m, _cx| {
                    if m.input.dragging || m.gizmo.state.active.is_some() {
                        return;
                    }
                    apply_click_selection_op(&mut m.selection, &mut m.active_target, Some(id), op);
                });
                app.request_redraw(window);
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
            }
        }
    }

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        let model = app.with_global_mut(Gizmo3dDemoService::default, |svc, _app| {
            svc.per_window.get(&event.window).cloned()
        });
        let Some(model) = model else {
            return;
        };

        let rec_to_record = model.update(app, |m, _cx| {
            if m.viewport_target != event.target {
                return None;
            }

            // Use UV instead of integer target pixels to avoid cursor quantization.
            let cursor_px = Vec2::new(
                event.uv.0 * m.viewport_px.0 as f32,
                event.uv.1 * m.viewport_px.1 as f32,
            );

            let mut rec_to_record: Option<UndoRecord<ValueTx<Vec<GizmoTarget3d>>>> = None;

            match event.kind {
                ViewportInputKind::PointerDown {
                    button: fret_core::MouseButton::Right,
                    ..
                } => {
                    m.camera.frame_anim = None;
                    m.camera.orbiting = true;
                    m.camera.panning = false;
                    m.camera.last_cursor_px = cursor_px;
                }
                ViewportInputKind::PointerDown {
                    button: fret_core::MouseButton::Middle,
                    ..
                } => {
                    m.camera.frame_anim = None;
                    m.camera.panning = true;
                    m.camera.orbiting = false;
                    m.camera.last_cursor_px = cursor_px;
                }
                ViewportInputKind::PointerUp {
                    button: fret_core::MouseButton::Right,
                    ..
                } => {
                    m.camera.orbiting = false;
                }
                ViewportInputKind::PointerUp {
                    button: fret_core::MouseButton::Middle,
                    ..
                } => {
                    m.camera.panning = false;
                }
                ViewportInputKind::PointerMove { buttons, .. } => {
                    // Some platforms can produce inconsistent "buttons" state for move events.
                    // Prefer to keep orbit/pan latched until an explicit PointerUp arrives, but
                    // still allow the move buttons state to end navigation if it becomes false.
                    if m.camera.orbiting && !buttons.right {
                        m.camera.orbiting = false;
                    }
                    if m.camera.panning && !buttons.middle {
                        m.camera.panning = false;
                    }

                    if m.camera.orbiting || m.camera.panning {
                        let delta = cursor_px - m.camera.last_cursor_px;
                        m.camera.last_cursor_px = cursor_px;

                        if m.camera.orbiting {
                            let orbit_sensitivity = 0.008;
                            m.camera.yaw_radians -= delta.x * orbit_sensitivity;
                            m.camera.pitch_radians = (m.camera.pitch_radians
                                - delta.y * orbit_sensitivity)
                                .clamp(-1.55, 1.55);
                        }

                        if m.camera.panning {
                            let pan_sensitivity = 0.002;
                            let pitch = m.camera.pitch_radians.clamp(-1.55, 1.55);
                            let yaw = m.camera.yaw_radians;
                            let distance = m.camera.distance.max(0.05);
                            let pan_scale = match m.camera.projection {
                                OrbitProjection::Perspective => distance,
                                OrbitProjection::Orthographic => {
                                    m.camera.ortho_half_height.max(0.05)
                                }
                            };

                            let dir = Vec3::new(
                                yaw.cos() * pitch.cos(),
                                pitch.sin(),
                                yaw.sin() * pitch.cos(),
                            );
                            let eye = m.camera.target + dir * distance;
                            let forward = (m.camera.target - eye).normalize_or_zero();
                            let right = forward.cross(Vec3::Y).normalize_or_zero();
                            let up = right.cross(forward).normalize_or_zero();

                            if right.length_squared() > 0.0 && up.length_squared() > 0.0 {
                                let pan = (-right * delta.x + up * delta.y)
                                    * (pan_scale * pan_sensitivity);
                                m.camera.target += pan;
                            }
                        }
                    }
                }
                ViewportInputKind::Wheel { delta, .. } => {
                    m.camera.frame_anim = None;
                    // Positive wheel delta.y typically scrolls up; treat that as "zoom in".
                    let zoom_sensitivity = 0.0015;
                    let scroll = delta.y.0;
                    let factor = (-scroll * zoom_sensitivity).exp();
                    match m.camera.projection {
                        OrbitProjection::Perspective => {
                            m.camera.distance = (m.camera.distance * factor).clamp(0.2, 25.0);
                        }
                        OrbitProjection::Orthographic => {
                            m.camera.ortho_half_height =
                                (m.camera.ortho_half_height * factor).clamp(0.05, 1000.0);
                        }
                    }
                }
                _ => {}
            };

            let (drag_started, dragging) = match event.kind {
                ViewportInputKind::PointerDown {
                    button: fret_core::MouseButton::Left,
                    ..
                } => (false, false),
                ViewportInputKind::PointerMove { .. } => (false, m.input.dragging),
                ViewportInputKind::PointerUp {
                    button: fret_core::MouseButton::Left,
                    ..
                } => (false, false),
                _ => (false, m.input.dragging),
            };

            let snap = match event.kind {
                ViewportInputKind::PointerMove { modifiers, .. } => modifiers.shift,
                ViewportInputKind::PointerDown { modifiers, .. } => modifiers.shift,
                ViewportInputKind::PointerUp { modifiers, .. } => modifiers.shift,
                ViewportInputKind::Wheel { modifiers, .. } => modifiers.shift,
            };

            let is_navigating = m.camera.orbiting || m.camera.panning;
            let hovered = !is_navigating;

            let (mut drag_started, mut dragging) = if is_navigating {
                (false, false)
            } else {
                (drag_started, dragging)
            };

            let view_projection = camera_view_projection(m.viewport_px, m.camera);
            let viewport = ViewportRect::new(
                Vec2::ZERO,
                Vec2::new(m.viewport_px.0 as f32, m.viewport_px.1 as f32),
            );

            let (view_gizmo_drag_started, view_gizmo_dragging) = match event.kind {
                ViewportInputKind::PointerDown {
                    button: fret_core::MouseButton::Left,
                    ..
                } => (true, true),
                ViewportInputKind::PointerMove { buttons, .. } => (false, buttons.left),
                ViewportInputKind::PointerUp {
                    button: fret_core::MouseButton::Left,
                    ..
                } => (false, false),
                _ => (false, false),
            };
            let view_gizmo_input = ViewGizmoInput {
                cursor_px,
                hovered,
                drag_started: view_gizmo_drag_started,
                dragging: view_gizmo_dragging,
            };
            let view_gizmo_update =
                m.view_gizmo
                    .update(view_projection, viewport, view_gizmo_input);

            let clear_other_interactions = |m: &mut Gizmo3dDemoModel| {
                m.gizmo.state.hovered = None;
                m.pending_selection = None;
                m.marquee = None;
                m.marquee_preview.clear();
                m.selection_before_select = None;
                m.active_before_select = None;
                m.input = GizmoInput {
                    cursor_px,
                    hovered: false,
                    drag_started: false,
                    dragging: false,
                    snap,
                    cancel: false,
                };
            };

            // If the left press starts on the view gizmo, consume the interaction so it doesn't
            // become a selection click or a transform gizmo drag.
            if view_gizmo_drag_started && m.view_gizmo.state.drag_active && !m.is_busy() {
                clear_other_interactions(m);
                return rec_to_record;
            }

            if !m.is_busy() {
                match view_gizmo_update {
                    Some(ViewGizmoUpdate::OrbitDelta {
                        delta_yaw_radians,
                        delta_pitch_radians,
                    }) => {
                        clear_other_interactions(m);
                        m.camera.frame_anim = None;
                        m.camera.yaw_radians += delta_yaw_radians;
                        m.camera.pitch_radians =
                            (m.camera.pitch_radians + delta_pitch_radians).clamp(-1.55, 1.55);
                        return rec_to_record;
                    }
                    Some(ViewGizmoUpdate::ToggleProjection) => {
                        clear_other_interactions(m);
                        let target = m.camera.target;
                        let yaw_radians = m.camera.yaw_radians;
                        let pitch_radians = m.camera.pitch_radians;
                        let smooth_time_s = 0.12;

                        match m.camera.projection {
                            OrbitProjection::Perspective => {
                                let ortho_half_height =
                                    distance_to_ortho_half_height(m.camera.distance);
                                m.camera.projection = OrbitProjection::Orthographic;
                                m.camera.frame_anim = Some(FrameAnim {
                                    target,
                                    distance: m.camera.distance,
                                    yaw_radians,
                                    pitch_radians,
                                    ortho_half_height,
                                    target_velocity: Vec3::ZERO,
                                    distance_velocity: 0.0,
                                    yaw_velocity: 0.0,
                                    pitch_velocity: 0.0,
                                    ortho_half_height_velocity: 0.0,
                                    smooth_time_s,
                                });
                            }
                            OrbitProjection::Orthographic => {
                                let distance =
                                    ortho_half_height_to_distance(m.camera.ortho_half_height);
                                m.camera.projection = OrbitProjection::Perspective;
                                m.camera.frame_anim = Some(FrameAnim {
                                    target,
                                    distance,
                                    yaw_radians,
                                    pitch_radians,
                                    ortho_half_height: m.camera.ortho_half_height,
                                    target_velocity: Vec3::ZERO,
                                    distance_velocity: 0.0,
                                    yaw_velocity: 0.0,
                                    pitch_velocity: 0.0,
                                    ortho_half_height_velocity: 0.0,
                                    smooth_time_s,
                                });
                            }
                        }
                        return rec_to_record;
                    }
                    Some(ViewGizmoUpdate::SnapView {
                        snap: _,
                        view_dir,
                        up: _,
                    }) => {
                        clear_other_interactions(m);
                        let pivot = if m.selection.is_empty() {
                            m.camera.target
                        } else {
                            let selected: Vec<GizmoTarget3d> = m
                                .targets
                                .iter()
                                .copied()
                                .filter(|t| m.selection.contains(&t.id))
                                .collect();
                            targets_world_aabb(&selected)
                                .map(|(min, max)| (min + max) * 0.5)
                                .unwrap_or(m.camera.target)
                        };

                        let desired_eye_dir = (-view_dir).normalize_or_zero();
                        if desired_eye_dir.length_squared() > 0.0 {
                            let (yaw_radians, pitch_radians) =
                                if desired_eye_dir.dot(Vec3::Y).abs() > 0.98 {
                                    (m.camera.yaw_radians, desired_eye_dir.y.signum() * 1.55)
                                } else {
                                    (
                                        desired_eye_dir.z.atan2(desired_eye_dir.x),
                                        desired_eye_dir.y.asin(),
                                    )
                                };

                            m.camera.frame_anim = Some(FrameAnim {
                                target: pivot,
                                distance: m.camera.distance,
                                yaw_radians,
                                pitch_radians: pitch_radians.clamp(-1.55, 1.55),
                                ortho_half_height: m.camera.ortho_half_height,
                                target_velocity: Vec3::ZERO,
                                distance_velocity: 0.0,
                                yaw_velocity: 0.0,
                                pitch_velocity: 0.0,
                                ortho_half_height_velocity: 0.0,
                                smooth_time_s: 0.12,
                            });
                        }

                        return rec_to_record;
                    }
                    _ => {}
                }
            }

            let over_view_gizmo = m.view_gizmo.state.drag_active
                || m.view_gizmo.state.hovered.is_some()
                || m.view_gizmo.state.hovered_center_button;
            let scene_hovered = hovered && !over_view_gizmo;

            if scene_hovered {
                match event.kind {
                    ViewportInputKind::PointerDown {
                        button: fret_core::MouseButton::Left,
                        modifiers,
                        click_count,
                        ..
                    } => {
                        let selected: Vec<GizmoTarget3d> = m
                            .targets
                            .iter()
                            .copied()
                            .filter(|t| m.selection.contains(&t.id))
                            .collect();

                        let hover_input = GizmoInput {
                            cursor_px,
                            hovered: true,
                            drag_started: false,
                            dragging: false,
                            snap: modifiers.shift,
                            cancel: false,
                        };
                        let _ = m.gizmo.update(
                            view_projection,
                            viewport,
                            hover_input,
                            m.active_target,
                            &selected,
                        );

                        let over_handle = m.gizmo.state.hovered.is_some();
                        if over_handle {
                            m.pending_selection = None;
                            m.marquee = None;
                            m.marquee_preview.clear();
                            m.selection_before_select = None;
                            m.active_before_select = None;
                            drag_started = true;
                            dragging = true;
                        } else {
                            m.selection_before_select = Some(m.selection.clone());
                            m.active_before_select = Some(m.active_target);
                            m.pending_selection = Some(PendingSelection {
                                start_cursor_px: cursor_px,
                                click_count,
                            });
                            m.marquee = None;
                            m.marquee_preview.clear();
                            drag_started = false;
                            dragging = false;
                        }
                    }
                    ViewportInputKind::PointerMove { buttons, modifiers } => {
                        const MARQUEE_THRESHOLD_PX: f32 = 4.0;
                        let threshold_sq = MARQUEE_THRESHOLD_PX * MARQUEE_THRESHOLD_PX;

                        if buttons.left {
                            if let Some(pending) = m.pending_selection {
                                if (cursor_px - pending.start_cursor_px).length_squared()
                                    >= threshold_sq
                                {
                                    m.pending_selection = None;
                                    m.marquee = Some(MarqueeSelection {
                                        start_cursor_px: pending.start_cursor_px,
                                        cursor_px,
                                        op: selection_op(&modifiers),
                                    });
                                }
                            }

                            if let Some(mut marquee) = m.marquee {
                                marquee.cursor_px = cursor_px;
                                marquee.op = selection_op(&modifiers);
                                m.marquee = Some(marquee);
                            }

                            if let Some(marquee) = m.marquee {
                                let (rect_min, rect_max) =
                                    marquee_rect(marquee.start_cursor_px, marquee.cursor_px);
                                let hits = marquee_hits(
                                    view_projection,
                                    viewport,
                                    m.gizmo.config.depth_range,
                                    &m.targets,
                                    rect_min,
                                    rect_max,
                                );

                                let (preview, _nearest) =
                                    apply_marquee_selection_op(&m.selection, &hits, marquee.op);
                                m.marquee_preview = preview;
                            } else {
                                m.marquee_preview.clear();
                            }

                            if m.pending_selection.is_some() || m.marquee.is_some() {
                                drag_started = false;
                                dragging = false;
                            }
                        }
                    }
                    ViewportInputKind::PointerUp {
                        button: fret_core::MouseButton::Left,
                        modifiers,
                        click_count: _click_count,
                        ..
                    } => {
                        let is_gizmo_dragging = m.gizmo.state.active.is_some() || m.input.dragging;
                        if is_gizmo_dragging {
                            m.pending_selection = None;
                            m.marquee = None;
                            m.marquee_preview.clear();
                            m.selection_before_select = None;
                            m.active_before_select = None;
                        } else {
                            if let Some(marquee) = m.marquee.take() {
                                let op = selection_op(&modifiers);
                                let (rect_min, rect_max) =
                                    marquee_rect(marquee.start_cursor_px, marquee.cursor_px);
                                let hits = marquee_hits(
                                    view_projection,
                                    viewport,
                                    m.gizmo.config.depth_range,
                                    &m.targets,
                                    rect_min,
                                    rect_max,
                                );

                                let (selection, nearest) =
                                    apply_marquee_selection_op(&m.selection, &hits, op);
                                m.selection = selection;

                                if !m.selection.contains(&m.active_target) {
                                    if let Some((id, _z01)) = nearest {
                                        m.active_target = id;
                                    } else if let Some(id) = m.selection.first().copied() {
                                        m.active_target = id;
                                    }
                                }

                                m.pending_selection = None;
                                m.marquee_preview.clear();
                                m.selection_before_select = None;
                                m.active_before_select = None;
                            } else if let Some(pending) = m.pending_selection.take() {
                                let op = selection_op(&modifiers);
                                if let Some(ray) = fret_gizmo::ray_from_screen(
                                    view_projection,
                                    viewport,
                                    cursor_px,
                                    m.gizmo.config.depth_range,
                                ) {
                                    let hit = pick_target_id(ray, &m.targets);
                                    apply_click_selection_op(
                                        &mut m.selection,
                                        &mut m.active_target,
                                        hit,
                                        op,
                                    );

                                    if pending.click_count >= 2 && !m.selection.is_empty() {
                                        let targets: Vec<GizmoTarget3d> = m
                                            .targets
                                            .iter()
                                            .copied()
                                            .filter(|t| m.selection.contains(&t.id))
                                            .collect();
                                        if let Some((min, max)) = targets_world_aabb(&targets) {
                                            frame_aabb(
                                                &mut m.camera,
                                                m.viewport_px,
                                                min,
                                                max,
                                                0.18,
                                            );
                                        }
                                    }
                                } else if matches!(op, SelectionOp::Replace) {
                                    m.selection.clear();
                                }
                                m.marquee_preview.clear();
                                m.selection_before_select = None;
                                m.active_before_select = None;
                            }

                            m.pending_selection = None;
                            m.marquee = None;
                            drag_started = false;
                            dragging = false;
                        }
                    }
                    _ => {}
                }
            }

            m.input = GizmoInput {
                cursor_px,
                hovered: scene_hovered,
                drag_started,
                dragging,
                snap,
                cancel: false,
            };

            let selected: Vec<GizmoTarget3d> = m
                .targets
                .iter()
                .copied()
                .filter(|t| m.selection.contains(&t.id))
                .collect();

            let apply_updated_targets =
                |targets: &mut Vec<GizmoTarget3d>, updated: &[GizmoTarget3d]| {
                    for u in updated {
                        if let Some(t) = targets.iter_mut().find(|t| t.id == u.id) {
                            t.transform = u.transform;
                        }
                    }
                };

            if let Some(update) = m.gizmo.update(
                view_projection,
                viewport,
                m.input,
                m.active_target,
                &selected,
            ) {
                m.hud.last = Some(GizmoHudLastUpdate {
                    phase: update.phase,
                    active: update.active,
                    result: update.result,
                });
                match update.phase {
                    GizmoPhase::Begin => {
                        m.drag_start_targets = Some(selected.clone());
                        apply_updated_targets(&mut m.targets, &update.updated_targets);
                    }
                    GizmoPhase::Update => {
                        apply_updated_targets(&mut m.targets, &update.updated_targets);
                    }
                    GizmoPhase::Commit => {
                        let Some(before) = m.drag_start_targets.take() else {
                            return rec_to_record;
                        };
                        let mut after: Vec<GizmoTarget3d> = Vec::with_capacity(before.len());
                        for t in &before {
                            if let Some(now) = m.targets.iter().find(|v| v.id == t.id) {
                                after.push(*now);
                            }
                        }

                        if before != after {
                            let tool = match update.result {
                                fret_gizmo::GizmoResult::Translation { .. } => "gizmo.translate",
                                fret_gizmo::GizmoResult::Rotation { .. } => "gizmo.rotate",
                                fret_gizmo::GizmoResult::Arcball { .. } => "gizmo.arcball",
                                fret_gizmo::GizmoResult::Scale { .. } => "gizmo.scale",
                            };

                            let mut sel = m.selection.clone();
                            sel.sort_by_key(|id| id.0);
                            let sel_key = sel
                                .iter()
                                .map(|id| id.0.to_string())
                                .collect::<Vec<_>>()
                                .join(",");
                            let coalesce_key =
                                format!("{tool}:active={}:sel={sel_key}", m.active_target.0);

                            let rec = UndoRecord::new(ValueTx::new(before, after))
                                .label("Transform")
                                .coalesce_key(CoalesceKey::from(coalesce_key));
                            rec_to_record = Some(rec);
                        }
                    }
                    GizmoPhase::Cancel => {
                        if let Some(start) = m.drag_start_targets.take() {
                            apply_updated_targets(&mut m.targets, &start);
                        }
                    }
                }
            }

            m.hud.hovered = m.gizmo.state.hovered;
            m.hud.hovered_kind = m.gizmo.state.hovered_kind;
            m.hud.active = m.gizmo.state.active;
            m.hud.snap = m.input.snap;

            rec_to_record
        });

        if let Ok(Some(rec)) = rec_to_record {
            let _ = app.with_global_mut(
                || UndoService::<ValueTx<Vec<GizmoTarget3d>>>::with_limit(256),
                |undo_svc, _app| {
                    undo_svc.record_or_coalesce_active(event.window, rec);
                },
            );
        }

        app.request_redraw(event.window);
    }

    fn record_engine_frame(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
        _scale_factor: f32,
        _tick_id: fret_runtime::TickId,
        _frame_id: fret_runtime::FrameId,
    ) -> EngineFrameUpdate {
        let (target_id, color_view, depth_view, size) =
            Self::ensure_target(app, window, state, context, renderer);

        let animating = state
            .demo
            .update(app, |m, _cx| {
                let now = Instant::now();
                let dt = m
                    .last_frame_instant
                    .and_then(|prev| now.checked_duration_since(prev))
                    .unwrap_or_default();
                m.last_frame_instant = Some(now);
                step_frame_anim(&mut m.camera, dt.as_secs_f32())
            })
            .unwrap_or(false);
        if animating {
            app.request_redraw(window);
        }

        let (
            scene_targets,
            selection,
            active_target,
            draw,
            thickness_px,
            view_proj,
            marquee,
            depth,
        ) = state
            .demo
            .read(app, |_app, m| {
                let view_proj = camera_view_projection(size, m.camera);

                let marquee = m.marquee;
                let selection = if marquee.is_some() {
                    m.marquee_preview.clone()
                } else {
                    m.selection.clone()
                };
                let active_target = if selection.contains(&m.active_target) {
                    m.active_target
                } else {
                    selection.first().copied().unwrap_or(m.active_target)
                };

                let viewport =
                    ViewportRect::new(Vec2::ZERO, Vec2::new(size.0 as f32, size.1 as f32));
                let mut draw = if marquee.is_some() {
                    GizmoDrawList3d::default()
                } else {
                    let gizmo_targets: Vec<GizmoTarget3d> = m
                        .targets
                        .iter()
                        .copied()
                        .filter(|t| selection.contains(&t.id))
                        .collect();
                    m.gizmo
                        .draw(view_proj, viewport, active_target, &gizmo_targets)
                };
                if marquee.is_none() {
                    let projection = match m.camera.projection {
                        OrbitProjection::Perspective => ViewGizmoProjection::Perspective,
                        OrbitProjection::Orthographic => ViewGizmoProjection::Orthographic,
                    };
                    let view_draw = m
                        .view_gizmo
                        .draw_with_projection(view_proj, viewport, projection);
                    draw.lines.extend(view_draw.lines);
                    draw.triangles.extend(view_draw.triangles);
                }

                let grid = Grid3d::default().draw();
                draw.lines.extend(grid.lines);
                draw.triangles.extend(grid.triangles);

                let thickness_px = m.gizmo.config.line_thickness_px;
                let depth = m.gizmo.config.depth_range;

                (
                    m.targets.clone(),
                    selection,
                    active_target,
                    draw,
                    thickness_px,
                    view_proj,
                    marquee,
                    depth,
                )
            })
            .unwrap_or_else(|_| {
                (
                    Vec::new(),
                    Vec::new(),
                    GizmoTargetId(0),
                    GizmoDrawList3d::default(),
                    6.0,
                    Mat4::IDENTITY,
                    None,
                    DepthRange::ZeroToOne,
                )
            });

        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            viewport_and_thickness: [size.0 as f32, size.1 as f32, thickness_px, 0.0],
        };
        let cpu = &mut state.overlay_cpu;
        cpu.clear();

        let mut cube_verts: Vec<Vertex> = Vec::new();
        for t in &scene_targets {
            let is_selected = selection.contains(&t.id);
            let color = if t.id == active_target {
                [1.0, 0.85, 0.2, 1.0]
            } else if is_selected {
                [0.25, 0.85, 0.35, 1.0]
            } else {
                [0.55, 0.58, 0.62, 1.0]
            };
            append_cube_triangles(&mut cube_verts, t.transform, color);
        }

        let cube_vb = (!cube_verts.is_empty()).then(|| {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmo3d demo cubes vb"),
                    contents: bytemuck::cast_slice(&cube_verts),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });

        for tri in draw.triangles {
            let a = tri.a.to_array();
            let b = tri.b.to_array();
            let c = tri.c.to_array();
            let color = [tri.color.r, tri.color.g, tri.color.b, tri.color.a];
            match tri.depth {
                DepthMode::Test => push_triangle(cpu.solid_test_mut(), a, b, c, color),
                DepthMode::Ghost => push_triangle(cpu.solid_ghost_mut(), a, b, c, color),
                DepthMode::Always => push_triangle(cpu.solid_always_mut(), a, b, c, color),
            }
        }

        for line in draw.lines {
            let a = line.a.to_array();
            let b = line.b.to_array();
            let color = [line.color.r, line.color.g, line.color.b, line.color.a];
            match line.depth {
                DepthMode::Test => push_thick_line_quad(cpu.line_test_mut(), a, b, color),
                DepthMode::Ghost => push_thick_line_quad(cpu.line_ghost_mut(), a, b, color),
                DepthMode::Always => push_thick_line_quad(cpu.line_always_mut(), a, b, color),
            }
        }

        if let Some(marquee) = marquee {
            let viewport = ViewportRect::new(Vec2::ZERO, Vec2::new(size.0 as f32, size.1 as f32));
            let (mut rect_min, mut rect_max) =
                marquee_rect(marquee.start_cursor_px, marquee.cursor_px);
            rect_min.x = rect_min.x.clamp(viewport.min.x, viewport.max().x);
            rect_min.y = rect_min.y.clamp(viewport.min.y, viewport.max().y);
            rect_max.x = rect_max.x.clamp(viewport.min.x, viewport.max().x);
            rect_max.y = rect_max.y.clamp(viewport.min.y, viewport.max().y);

            let corners = [
                Vec2::new(rect_min.x, rect_min.y),
                Vec2::new(rect_max.x, rect_min.y),
                Vec2::new(rect_max.x, rect_max.y),
                Vec2::new(rect_min.x, rect_max.y),
            ];

            let z01 = 0.001;
            let mut w = [Vec3::ZERO; 4];
            let mut ok = true;
            for (i, s) in corners.iter().enumerate() {
                if let Some(p) = fret_gizmo::unproject_point(view_proj, viewport, *s, depth, z01) {
                    w[i] = p;
                } else {
                    ok = false;
                    break;
                }
            }

            if ok {
                let (fill, border) = match marquee.op {
                    SelectionOp::Replace => ([0.25, 0.60, 1.00, 0.10], [0.25, 0.60, 1.00, 0.90]),
                    SelectionOp::Add => ([0.25, 0.85, 0.35, 0.10], [0.25, 0.85, 0.35, 0.90]),
                    SelectionOp::Subtract => ([1.00, 0.25, 0.25, 0.10], [1.00, 0.25, 0.25, 0.90]),
                    SelectionOp::Toggle => ([1.00, 0.85, 0.20, 0.10], [1.00, 0.85, 0.20, 0.90]),
                };

                push_triangle(
                    cpu.solid_always_mut(),
                    w[0].to_array(),
                    w[1].to_array(),
                    w[2].to_array(),
                    fill,
                );
                push_triangle(
                    cpu.solid_always_mut(),
                    w[0].to_array(),
                    w[2].to_array(),
                    w[3].to_array(),
                    fill,
                );

                let edges = [(0, 1), (1, 2), (2, 3), (3, 0)];
                for (a, b) in edges {
                    push_thick_line_quad(
                        cpu.line_always_mut(),
                        w[a].to_array(),
                        w[b].to_array(),
                        border,
                    );
                }
            }
        }

        let overlay =
            app.with_global_mut(Gizmo3dDemoViewportOverlayService::default, |svc, _app| {
                svc.upload(
                    &context.device,
                    &context.queue,
                    window,
                    target_id,
                    uniforms,
                    cpu,
                )
            });

        let clear = wgpu::Color {
            r: 0.08,
            g: 0.08,
            b: 0.10,
            a: 1.0,
        };

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gizmo3d demo encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gizmo3d demo pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if let Some(cube_vb) = &cube_vb {
                pass.set_bind_group(0, &overlay.bind_group, &[]);
                pass.set_pipeline(&overlay.tri_pipeline);
                pass.set_vertex_buffer(0, cube_vb.slice(..));
                pass.draw(0..(cube_verts.len().min(u32::MAX as usize) as u32), 0..1);
            }

            let ctx = ViewportOverlay3dContext {
                view_proj,
                viewport_px: size,
            };

            record_viewport_overlay_3d(app, window, target_id, &mut pass, &ctx);

            let _ = _frame_id;
        }

        EngineFrameUpdate {
            target_updates: Vec::new(),
            command_buffers: vec![encoder.finish()],
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        let root = state.root.get_or_insert_with(|| {
            let style = Plot3dStyle::default();
            let canvas = Plot3dCanvas::new(state.plot.clone()).style(style);
            let node = Plot3dCanvas::create_node(&mut state.ui, canvas);
            state.ui.set_root(node);
            node
        });

        state.ui.set_root(*root);
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);

        let (show_help, overlay_text) = state
            .demo
            .read(app, |_app, m| (m.show_help, m.overlay_text()))
            .unwrap_or((false, String::new()));

        if show_help {
            let scale_bits = scale_factor.to_bits();
            if state.overlay.last_text != overlay_text
                || state.overlay.last_scale_bits != scale_bits
            {
                if let Some(blob) = state.overlay.blob.take() {
                    services.text().release(blob);
                }

                let style = TextStyle {
                    font: fret_core::FontId::default(),
                    size: Px(13.0),
                    weight: FontWeight::MEDIUM,
                    slant: fret_core::text::TextSlant::Normal,
                    line_height: Some(Px(16.0)),
                    letter_spacing_em: None,
                };
                let constraints = TextConstraints {
                    max_width: Some(Px(bounds.size.width.0 - 24.0)),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                    scale_factor,
                };

                let (blob, metrics) = services.text().prepare(&overlay_text, &style, constraints);
                state.overlay.last_text = overlay_text;
                state.overlay.last_scale_bits = scale_bits;
                state.overlay.blob = Some(blob);
                state.overlay.metrics = Some(metrics);
            }

            if let (Some(blob), Some(metrics)) = (state.overlay.blob, state.overlay.metrics) {
                let pad = Px(10.0);
                let outer_pad = Px(12.0);

                let bg_rect = Rect::new(
                    Point::new(
                        Px(bounds.origin.x.0 + outer_pad.0),
                        Px(bounds.origin.y.0 + outer_pad.0),
                    ),
                    Size::new(
                        Px(metrics.size.width.0 + pad.0 * 2.0),
                        Px(metrics.size.height.0 + pad.0 * 2.0),
                    ),
                );
                scene.push(SceneOp::Quad {
                    order: DrawOrder(50_000),
                    rect: bg_rect,
                    background: Color {
                        r: 0.08,
                        g: 0.08,
                        b: 0.09,
                        a: 0.78,
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Color {
                        r: 0.35,
                        g: 0.35,
                        b: 0.40,
                        a: 0.85,
                    },
                    corner_radii: Corners::all(Px(12.0)),
                });
                scene.push(SceneOp::Text {
                    order: DrawOrder(50_010),
                    origin: Point::new(
                        Px(bg_rect.origin.x.0 + pad.0),
                        Px(bg_rect.origin.y.0 + pad.0),
                    ),
                    text: blob,
                    color: Color {
                        r: 0.92,
                        g: 0.92,
                        b: 0.94,
                        a: 0.95,
                    },
                });
            }
        }

        // View gizmo labels (X/Y/Z + P/O).
        let viewport = state
            .plot
            .read(app, |_app, m| m.viewport)
            .unwrap_or_default();
        let (viewport_px, camera, view_gizmo, gizmo_cfg, hud_state) = state
            .demo
            .read(app, |_app, m| {
                (
                    m.viewport_px,
                    m.camera,
                    m.view_gizmo.clone(),
                    m.gizmo.config,
                    m.hud,
                )
            })
            .unwrap_or((
                (1, 1),
                OrbitCamera::default(),
                ViewGizmo::new(ViewGizmoConfig::default()),
                GizmoConfig::default(),
                GizmoHudState::default(),
            ));

        let draw_rect = viewport.draw_rect(bounds);
        let scale_x = draw_rect.size.width.0 / (viewport_px.0.max(1) as f32);
        let scale_y = draw_rect.size.height.0 / (viewport_px.1.max(1) as f32);
        let scale = scale_x.min(scale_y).max(1e-6);

        let view_proj = camera_view_projection(viewport_px, camera);
        let viewport_rect = ViewportRect::new(
            Vec2::ZERO,
            Vec2::new(viewport_px.0 as f32, viewport_px.1 as f32),
        );
        let projection = match camera.projection {
            OrbitProjection::Perspective => ViewGizmoProjection::Perspective,
            OrbitProjection::Orthographic => ViewGizmoProjection::Orthographic,
        };

        let labels = view_gizmo.labels(view_proj, viewport_rect, projection);
        if !labels.is_empty() {
            state.view_gizmo_labels.ensure(services, scale_factor);
        }

        for label in labels {
            let Some((blob, metrics)) = state.view_gizmo_labels.blob_and_metrics(label.text) else {
                continue;
            };

            let x = draw_rect.origin.x.0 + label.screen_px.x * scale;
            let y = draw_rect.origin.y.0 + label.screen_px.y * scale;

            let pad = Px(3.0);
            let bg = Rect::new(
                Point::new(
                    Px(x - metrics.size.width.0 * 0.5 - pad.0),
                    Px(y - metrics.size.height.0 * 0.5 - pad.0),
                ),
                Size::new(
                    Px(metrics.size.width.0 + pad.0 * 2.0),
                    Px(metrics.size.height.0 + pad.0 * 2.0),
                ),
            );

            scene.push(SceneOp::Quad {
                order: DrawOrder(49_000),
                rect: bg,
                background: Color {
                    r: 0.06,
                    g: 0.06,
                    b: 0.07,
                    a: 0.55,
                },
                border: Edges::all(Px(1.0)),
                border_color: Color {
                    r: label.color.r,
                    g: label.color.g,
                    b: label.color.b,
                    a: 0.85,
                },
                corner_radii: Corners::all(Px(8.0)),
            });

            scene.push(SceneOp::Text {
                order: DrawOrder(49_010),
                origin: Point::new(
                    Px(x - metrics.size.width.0 * 0.5),
                    Px(y - metrics.size.height.0 * 0.5),
                ),
                text: blob,
                color: Color {
                    r: label.color.r,
                    g: label.color.g,
                    b: label.color.b,
                    a: label.color.a,
                },
            });
        }

        if let Some(text) = gizmo_hud_text(hud_state, gizmo_cfg) {
            let scale_bits = scale_factor.to_bits();
            if state.hud.last_text != text || state.hud.last_scale_bits != scale_bits {
                if let Some(blob) = state.hud.blob.take() {
                    services.text().release(blob);
                }

                let style = TextStyle {
                    font: fret_core::FontId::default(),
                    size: Px(12.0),
                    weight: FontWeight::MEDIUM,
                    slant: fret_core::text::TextSlant::Normal,
                    line_height: Some(Px(14.0)),
                    letter_spacing_em: None,
                };
                let constraints = TextConstraints {
                    max_width: Some(Px(340.0)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor,
                };

                let (blob, metrics) = services.text().prepare(&text, &style, constraints);
                state.hud.last_text = text;
                state.hud.last_scale_bits = scale_bits;
                state.hud.blob = Some(blob);
                state.hud.metrics = Some(metrics);
            }

            if let (Some(blob), Some(metrics)) = (state.hud.blob, state.hud.metrics) {
                let pad = Px(10.0);
                let outer_pad = Px(12.0);

                let origin = Point::new(
                    Px(draw_rect.origin.x.0 + outer_pad.0),
                    Px(draw_rect.origin.y.0 + draw_rect.size.height.0 - outer_pad.0),
                );

                let bg_rect = Rect::new(
                    Point::new(
                        Px(origin.x.0),
                        Px(origin.y.0 - (metrics.size.height.0 + pad.0 * 2.0)),
                    ),
                    Size::new(
                        Px(metrics.size.width.0 + pad.0 * 2.0),
                        Px(metrics.size.height.0 + pad.0 * 2.0),
                    ),
                );

                scene.push(SceneOp::Quad {
                    order: DrawOrder(49_200),
                    rect: bg_rect,
                    background: Color {
                        r: 0.06,
                        g: 0.06,
                        b: 0.07,
                        a: 0.62,
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Color {
                        r: 0.35,
                        g: 0.35,
                        b: 0.40,
                        a: 0.85,
                    },
                    corner_radii: Corners::all(Px(12.0)),
                });
                scene.push(SceneOp::Text {
                    order: DrawOrder(49_210),
                    origin: Point::new(
                        Px(bg_rect.origin.x.0 + pad.0),
                        Px(bg_rect.origin.y.0 + pad.0),
                    ),
                    text: blob,
                    color: Color {
                        r: 0.92,
                        g: 0.92,
                        b: 0.94,
                        a: 0.95,
                    },
                });
            }
        }

        if state.warmup_frames_remaining > 0 {
            state.warmup_frames_remaining = state.warmup_frames_remaining.saturating_sub(1);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo gizmo3d_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    Gizmo3dDemoDriver
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    crate::run_native_demo(config, app, driver).context("run gizmo3d_demo app")
}

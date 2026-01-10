use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_core::{
    AppWindowId, Event, RenderTargetId, ViewportFit, ViewportInputEvent, ViewportInputKind,
};
use fret_gizmo::{
    Aabb3, DepthMode, DepthRange, Gizmo, GizmoConfig, GizmoDrawList3d, GizmoInput, GizmoMode,
    GizmoOrientation, GizmoPhase, GizmoPivotMode, GizmoTarget3d, GizmoTargetId, Transform3d,
    ViewportRect,
};
use fret_launch::{
    EngineFrameUpdate, ViewportOverlay3dHooks, ViewportOverlay3dHooksService, WinitAppDriver,
    WinitCommandContext, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    record_viewport_overlay_3d,
};
use fret_plot3d::retained::{Plot3dCanvas, Plot3dModel, Plot3dStyle, Plot3dViewport};
use fret_render::viewport_overlay::ViewportOverlay3dContext;
use fret_render::{RenderTargetColorSpace, RenderTargetDescriptor, Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_undo::{CoalesceKey, DocumentId, UndoRecord, UndoService, ValueTx};
use glam::{Mat4, Quat, Vec2, Vec3};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt as _;

#[derive(Debug, Clone, Copy)]
struct FrameAnim {
    target: Vec3,
    distance: f32,
    target_velocity: Vec3,
    distance_velocity: f32,
    smooth_time_s: f32,
}

#[derive(Debug, Clone, Copy)]
struct OrbitCamera {
    target: Vec3,
    yaw_radians: f32,
    pitch_radians: f32,
    distance: f32,
    orbiting: bool,
    panning: bool,
    last_cursor_px: Vec2,
    frame_anim: Option<FrameAnim>,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        // Roughly matches the previous hard-coded view: eye = (1.6, 1.2, 2.2), target = (0,0,0).
        Self {
            target: Vec3::ZERO,
            yaw_radians: 0.94,
            pitch_radians: 0.42,
            distance: 2.95,
            orbiting: false,
            panning: false,
            last_cursor_px: Vec2::ZERO,
            frame_anim: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct LineVertex {
    a: [f32; 3],
    b: [f32; 3],
    t: f32,
    side: f32,
    color: [f32; 4],
}

unsafe impl bytemuck::Zeroable for LineVertex {}
unsafe impl bytemuck::Pod for LineVertex {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    /// x = viewport_w_px, y = viewport_h_px, z = line_thickness_px, w = unused
    viewport_and_thickness: [f32; 4],
}

unsafe impl bytemuck::Zeroable for Uniforms {}
unsafe impl bytemuck::Pod for Uniforms {}

fn push_thick_line_quad(out: &mut Vec<LineVertex>, a: [f32; 3], b: [f32; 3], color: [f32; 4]) {
    // Two triangles (6 vertices) for a screen-space thick line quad.
    out.extend_from_slice(&[
        LineVertex {
            a,
            b,
            t: 0.0,
            side: -1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 0.0,
            side: 1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 1.0,
            side: 1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 0.0,
            side: -1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 1.0,
            side: 1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 1.0,
            side: -1.0,
            color,
        },
    ]);
}

struct Gizmo3dDemoTarget {
    id: RenderTargetId,
    size: (u32, u32),
    color: wgpu::Texture,
    depth: wgpu::Texture,
}

struct Gizmo3dDemoGpu {
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    tri_pipeline: wgpu::RenderPipeline,
    gizmo_solid_depth_pipeline: wgpu::RenderPipeline,
    gizmo_solid_always_pipeline: wgpu::RenderPipeline,
    thick_line_depth_pipeline: wgpu::RenderPipeline,
    thick_line_always_pipeline: wgpu::RenderPipeline,
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

    let done = (camera.target - anim.target).length() <= 1e-3
        && (camera.distance - anim.distance).abs() <= 1e-3
        && anim.target_velocity.length() <= 1e-3
        && anim.distance_velocity.abs() <= 1e-3;

    if done {
        camera.target = anim.target;
        camera.distance = anim.distance.clamp(0.2, 25.0);
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

    let fov_y = 55.0_f32.to_radians();
    let fov_x = 2.0 * ((fov_y * 0.5).tan() * aspect).atan();
    let fov = fov_y.min(fov_x).max(0.001);

    let margin = 1.25;
    let dist = (radius * margin) / (fov * 0.5).tan();

    camera.frame_anim = Some(FrameAnim {
        target: center,
        distance: dist.clamp(0.2, 25.0),
        target_velocity: Vec3::ZERO,
        distance_velocity: 0.0,
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
}

impl Default for Gizmo3dDemoModel {
    fn default() -> Self {
        let mut gizmo_cfg = GizmoConfig::default();
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
        Self {
            viewport_target: RenderTargetId::default(),
            viewport_px: (960, 540),
            gizmo: Gizmo::new(gizmo_cfg),
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
        }
    }
}

#[derive(Default)]
struct Gizmo3dDemoService {
    per_window: HashMap<AppWindowId, fret_runtime::Model<Gizmo3dDemoModel>>,
}

#[derive(Clone)]
struct OverlayDrawBuffer {
    buffer: wgpu::Buffer,
    vertex_count: u32,
}

#[derive(Clone)]
struct Gizmo3dDemoViewportOverlayWindow {
    bind_group: wgpu::BindGroup,
    gizmo_solid_depth_pipeline: wgpu::RenderPipeline,
    gizmo_solid_always_pipeline: wgpu::RenderPipeline,
    thick_line_depth_pipeline: wgpu::RenderPipeline,
    thick_line_always_pipeline: wgpu::RenderPipeline,
    solid_test: Option<OverlayDrawBuffer>,
    solid_ghost: Option<OverlayDrawBuffer>,
    solid_always: Option<OverlayDrawBuffer>,
    line_test: Option<OverlayDrawBuffer>,
    line_ghost: Option<OverlayDrawBuffer>,
    line_always: Option<OverlayDrawBuffer>,
}

#[derive(Default)]
struct Gizmo3dDemoViewportOverlayService {
    per_viewport: HashMap<(AppWindowId, RenderTargetId), Gizmo3dDemoViewportOverlayWindow>,
}

impl Gizmo3dDemoViewportOverlayService {
    fn ensure_entry(
        &mut self,
        window: AppWindowId,
        target: RenderTargetId,
        gpu: &Gizmo3dDemoGpu,
    ) -> &mut Gizmo3dDemoViewportOverlayWindow {
        self.per_viewport
            .entry((window, target))
            .or_insert_with(|| Gizmo3dDemoViewportOverlayWindow {
                bind_group: gpu.bind_group.clone(),
                gizmo_solid_depth_pipeline: gpu.gizmo_solid_depth_pipeline.clone(),
                gizmo_solid_always_pipeline: gpu.gizmo_solid_always_pipeline.clone(),
                thick_line_depth_pipeline: gpu.thick_line_depth_pipeline.clone(),
                thick_line_always_pipeline: gpu.thick_line_always_pipeline.clone(),
                solid_test: None,
                solid_ghost: None,
                solid_always: None,
                line_test: None,
                line_ghost: None,
                line_always: None,
            })
    }

    fn update_buffers(
        &mut self,
        window: AppWindowId,
        target: RenderTargetId,
        gpu: &Gizmo3dDemoGpu,
        solid_vb_test: Option<wgpu::Buffer>,
        solid_count_test: u32,
        solid_vb_ghost: Option<wgpu::Buffer>,
        solid_count_ghost: u32,
        solid_vb_always: Option<wgpu::Buffer>,
        solid_count_always: u32,
        line_vb_test: Option<wgpu::Buffer>,
        line_count_test: u32,
        line_vb_ghost: Option<wgpu::Buffer>,
        line_count_ghost: u32,
        line_vb_always: Option<wgpu::Buffer>,
        line_count_always: u32,
    ) {
        let entry = self.ensure_entry(window, target, gpu);
        entry.solid_test = solid_vb_test.map(|buffer| OverlayDrawBuffer {
            buffer,
            vertex_count: solid_count_test,
        });
        entry.solid_ghost = solid_vb_ghost.map(|buffer| OverlayDrawBuffer {
            buffer,
            vertex_count: solid_count_ghost,
        });
        entry.solid_always = solid_vb_always.map(|buffer| OverlayDrawBuffer {
            buffer,
            vertex_count: solid_count_always,
        });
        entry.line_test = line_vb_test.map(|buffer| OverlayDrawBuffer {
            buffer,
            vertex_count: line_count_test,
        });
        entry.line_ghost = line_vb_ghost.map(|buffer| OverlayDrawBuffer {
            buffer,
            vertex_count: line_count_ghost,
        });
        entry.line_always = line_vb_always.map(|buffer| OverlayDrawBuffer {
            buffer,
            vertex_count: line_count_always,
        });
    }

    fn record(&self, window: AppWindowId, target: RenderTargetId, pass: &mut wgpu::RenderPass<'_>) {
        let Some(overlays) = self.per_viewport.get(&(window, target)) else {
            return;
        };

        pass.set_bind_group(0, &overlays.bind_group, &[]);

        if let Some(buf) = &overlays.solid_ghost {
            pass.set_pipeline(&overlays.gizmo_solid_always_pipeline);
            pass.set_vertex_buffer(0, buf.buffer.slice(..));
            pass.draw(0..buf.vertex_count, 0..1);
        }
        if let Some(buf) = &overlays.line_ghost {
            pass.set_pipeline(&overlays.thick_line_always_pipeline);
            pass.set_vertex_buffer(0, buf.buffer.slice(..));
            pass.draw(0..buf.vertex_count, 0..1);
        }

        if let Some(buf) = &overlays.solid_test {
            pass.set_pipeline(&overlays.gizmo_solid_depth_pipeline);
            pass.set_vertex_buffer(0, buf.buffer.slice(..));
            pass.draw(0..buf.vertex_count, 0..1);
        }
        if let Some(buf) = &overlays.line_test {
            pass.set_pipeline(&overlays.thick_line_depth_pipeline);
            pass.set_vertex_buffer(0, buf.buffer.slice(..));
            pass.draw(0..buf.vertex_count, 0..1);
        }

        if let Some(buf) = &overlays.solid_always {
            pass.set_pipeline(&overlays.gizmo_solid_always_pipeline);
            pass.set_vertex_buffer(0, buf.buffer.slice(..));
            pass.draw(0..buf.vertex_count, 0..1);
        }
        if let Some(buf) = &overlays.line_always {
            pass.set_pipeline(&overlays.thick_line_always_pipeline);
            pass.set_vertex_buffer(0, buf.buffer.slice(..));
            pass.draw(0..buf.vertex_count, 0..1);
        }
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
    target: Option<Gizmo3dDemoTarget>,
    gpu: Option<Gizmo3dDemoGpu>,
    doc: DocumentId,
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
            target: None,
            gpu: None,
            doc,
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

    fn ensure_gpu(state: &mut Gizmo3dDemoWindowState, context: &WgpuContext) {
        if state.gpu.is_some() {
            return;
        }

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

        let tri_pipeline = context
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

        state.gpu = Some(Gizmo3dDemoGpu {
            uniform,
            bind_group,
            tri_pipeline,
            gizmo_solid_depth_pipeline,
            gizmo_solid_always_pipeline,
            thick_line_depth_pipeline,
            thick_line_always_pipeline,
        });
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
    let proj = Mat4::perspective_rh(55.0_f32.to_radians(), aspect, 0.05, 50.0);
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
                    m.gizmo.config.mode = GizmoMode::Rotate;
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyS,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    m.gizmo.config.mode = GizmoMode::Scale;
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyT,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    m.gizmo.config.mode = GizmoMode::Translate;
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyU,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    m.gizmo.config.mode = GizmoMode::Universal;
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
                                    * (distance * pan_sensitivity);
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
                    m.camera.distance = (m.camera.distance * factor).clamp(0.2, 25.0);
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

            if hovered {
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
                hovered,
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
        Self::ensure_gpu(state, context);

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

        let gpu = state.gpu.as_ref().expect("gpu ensured");

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

                let draw = if marquee.is_some() {
                    GizmoDrawList3d::default()
                } else {
                    let gizmo_targets: Vec<GizmoTarget3d> = m
                        .targets
                        .iter()
                        .copied()
                        .filter(|t| selection.contains(&t.id))
                        .collect();
                    m.gizmo.draw(
                        view_proj,
                        ViewportRect::new(Vec2::ZERO, Vec2::new(size.0 as f32, size.1 as f32)),
                        active_target,
                        &gizmo_targets,
                    )
                };
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
        context
            .queue
            .write_buffer(&gpu.uniform, 0, bytemuck::bytes_of(&uniforms));

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

        let mut solid_verts_test: Vec<Vertex> = Vec::new();
        let mut solid_verts_ghost: Vec<Vertex> = Vec::new();
        let mut solid_verts_always: Vec<Vertex> = Vec::new();

        for tri in draw.triangles {
            let a = tri.a.to_array();
            let b = tri.b.to_array();
            let c = tri.c.to_array();
            let color = [tri.color.r, tri.color.g, tri.color.b, tri.color.a];
            let push = |out: &mut Vec<Vertex>| {
                out.push(Vertex { pos: a, color });
                out.push(Vertex { pos: b, color });
                out.push(Vertex { pos: c, color });
            };
            match tri.depth {
                DepthMode::Test => push(&mut solid_verts_test),
                DepthMode::Ghost => push(&mut solid_verts_ghost),
                DepthMode::Always => push(&mut solid_verts_always),
            }
        }

        let mut line_verts_test: Vec<LineVertex> = Vec::new();
        let mut line_verts_ghost: Vec<LineVertex> = Vec::new();
        let mut line_verts_always: Vec<LineVertex> = Vec::new();

        for line in draw.lines {
            let a = line.a.to_array();
            let b = line.b.to_array();
            let color = [line.color.r, line.color.g, line.color.b, line.color.a];
            match line.depth {
                DepthMode::Test => push_thick_line_quad(&mut line_verts_test, a, b, color),
                DepthMode::Ghost => push_thick_line_quad(&mut line_verts_ghost, a, b, color),
                DepthMode::Always => push_thick_line_quad(&mut line_verts_always, a, b, color),
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

                solid_verts_always.push(Vertex {
                    pos: w[0].to_array(),
                    color: fill,
                });
                solid_verts_always.push(Vertex {
                    pos: w[1].to_array(),
                    color: fill,
                });
                solid_verts_always.push(Vertex {
                    pos: w[2].to_array(),
                    color: fill,
                });
                solid_verts_always.push(Vertex {
                    pos: w[0].to_array(),
                    color: fill,
                });
                solid_verts_always.push(Vertex {
                    pos: w[2].to_array(),
                    color: fill,
                });
                solid_verts_always.push(Vertex {
                    pos: w[3].to_array(),
                    color: fill,
                });

                let edges = [(0, 1), (1, 2), (2, 3), (3, 0)];
                for (a, b) in edges {
                    push_thick_line_quad(
                        &mut line_verts_always,
                        w[a].to_array(),
                        w[b].to_array(),
                        border,
                    );
                }
            }
        }

        let solid_vb_test = (!solid_verts_test.is_empty()).then(|| {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmo3d demo gizmo solid vb (test)"),
                    contents: bytemuck::cast_slice(&solid_verts_test),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let solid_vb_ghost = (!solid_verts_ghost.is_empty()).then(|| {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmo3d demo gizmo solid vb (ghost)"),
                    contents: bytemuck::cast_slice(&solid_verts_ghost),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let solid_vb_always = (!solid_verts_always.is_empty()).then(|| {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmo3d demo gizmo solid vb (always)"),
                    contents: bytemuck::cast_slice(&solid_verts_always),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });

        let line_vb_test = (!line_verts_test.is_empty()).then(|| {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmo3d demo thick line vb (test)"),
                    contents: bytemuck::cast_slice(&line_verts_test),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let line_vb_ghost = (!line_verts_ghost.is_empty()).then(|| {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmo3d demo thick line vb (ghost)"),
                    contents: bytemuck::cast_slice(&line_verts_ghost),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let line_vb_always = (!line_verts_always.is_empty()).then(|| {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmo3d demo thick line vb (always)"),
                    contents: bytemuck::cast_slice(&line_verts_always),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });

        let solid_count_test = solid_verts_test.len().min(u32::MAX as usize) as u32;
        let solid_count_ghost = solid_verts_ghost.len().min(u32::MAX as usize) as u32;
        let solid_count_always = solid_verts_always.len().min(u32::MAX as usize) as u32;
        let line_count_test = line_verts_test.len().min(u32::MAX as usize) as u32;
        let line_count_ghost = line_verts_ghost.len().min(u32::MAX as usize) as u32;
        let line_count_always = line_verts_always.len().min(u32::MAX as usize) as u32;

        app.with_global_mut(Gizmo3dDemoViewportOverlayService::default, |svc, _app| {
            svc.update_buffers(
                window,
                target_id,
                gpu,
                solid_vb_test.clone(),
                solid_count_test,
                solid_vb_ghost.clone(),
                solid_count_ghost,
                solid_vb_always.clone(),
                solid_count_always,
                line_vb_test.clone(),
                line_count_test,
                line_vb_ghost.clone(),
                line_count_ghost,
                line_vb_always.clone(),
                line_count_always,
            );
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
                pass.set_bind_group(0, &gpu.bind_group, &[]);
                pass.set_pipeline(&gpu.tri_pipeline);
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

use fret::app::prelude::*;
use fret::app::{LocalState, RenderContextAccess as _};
use fret::canvas::{
    self, CanvasPaint, Color, Corners, DrawOrder, Edges, PathCommand, PathStyle, Point, Rect,
    StrokeCapV1, StrokeJoinV1, StrokeStyleV2,
};
use fret::pointer::{self, CursorIcon, MouseButton, PointerRegion};
use fret::semantics::{SemanticsDecoration, SemanticsRole};
use fret::{shadcn, style::Space};
use fret_gizmo::{Aabb3, DepthMode, Gizmo, GizmoConfig, GizmoInput, GizmoState, GizmoTarget3d};
use fret_gizmo::{DepthRange, GizmoTargetId, Transform3d, ViewportRect, project_point};

use glam::{Mat4, Quat, Vec2, Vec3};

mod act {
    fret::actions!([
        Reset = "cookbook.gizmo.reset",
        ToggleSnap = "cookbook.gizmo.toggle_snap"
    ]);
}

const ROOT_NAME: &str = "cookbook-gizmo-basics";

const TEST_ID_ROOT: &str = "cookbook.gizmo_basics.root";
const TEST_ID_VIEWPORT: &str = "cookbook.gizmo_basics.viewport";
const TEST_ID_RESET: &str = "cookbook.gizmo_basics.reset";
const TEST_ID_TOGGLE_SNAP: &str = "cookbook.gizmo_basics.toggle_snap";

const TEST_ID_POS_X: &str = "cookbook.gizmo_basics.pos_x";
const TEST_ID_POS_Y: &str = "cookbook.gizmo_basics.pos_y";
const TEST_ID_POS_Z: &str = "cookbook.gizmo_basics.pos_z";
const TEST_ID_POS_LEN: &str = "cookbook.gizmo_basics.pos_len";

const CAMERA_FOV_Y_RADIANS: f32 = 45.0_f32.to_radians();
const CAMERA_NEAR: f32 = 0.01;
const CAMERA_FAR: f32 = 500.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct OrbitCamera {
    yaw_radians: f32,
    pitch_radians: f32,
    distance: f32,
    target: Vec3,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw_radians: 0.55,
            pitch_radians: 0.45,
            distance: 6.0,
            target: Vec3::ZERO,
        }
    }
}

fn camera_view_projection(viewport_px_size: Vec2, camera: OrbitCamera) -> Mat4 {
    let aspect = (viewport_px_size.x.max(1.0)) / (viewport_px_size.y.max(1.0));
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
    let proj = Mat4::perspective_rh(CAMERA_FOV_Y_RADIANS, aspect, CAMERA_NEAR, CAMERA_FAR);
    proj * view
}

fn viewport_rect_from_bounds(bounds: Rect, pixels_per_point: f32) -> ViewportRect {
    let w = (bounds.size.width.0 * pixels_per_point).max(1.0);
    let h = (bounds.size.height.0 * pixels_per_point).max(1.0);
    ViewportRect {
        min: Vec2::ZERO,
        size: Vec2::new(w, h),
    }
}

fn color_mul_alpha(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, PartialEq)]
struct GizmoBasicsModel {
    camera: OrbitCamera,
    gizmo_config: GizmoConfig,
    gizmo_state: GizmoState,
    active_target: GizmoTargetId,
    transform: Transform3d,
    snap: bool,
    dragging: bool,
}

impl Default for GizmoBasicsModel {
    fn default() -> Self {
        let mut cfg = GizmoConfig::default();
        cfg.drag_start_threshold_px = 0.0;
        cfg.translate_snap_step = Some(0.25);
        cfg.show_bounds = false;

        Self {
            camera: OrbitCamera::default(),
            gizmo_config: cfg,
            gizmo_state: GizmoState::default(),
            active_target: GizmoTargetId(1),
            transform: Transform3d {
                translation: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            snap: false,
            dragging: false,
        }
    }
}

struct GizmoBasicsView {
    model: LocalState<GizmoBasicsModel>,
}

fn gizmo_targets(active_target: GizmoTargetId, transform: Transform3d) -> [GizmoTarget3d; 1] {
    let bounds = Aabb3 {
        min: Vec3::splat(-0.5),
        max: Vec3::splat(0.5),
    };
    [GizmoTarget3d {
        id: active_target,
        transform,
        local_bounds: Some(bounds),
    }]
}

fn paint_cube_wireframe(
    painter: &mut canvas::AppCanvasPainter<'_, '_>,
    view_projection: Mat4,
    viewport: ViewportRect,
    depth_range: DepthRange,
    scale_factor: f32,
    transform: Transform3d,
    color: Color,
) {
    let m = transform.to_mat4();
    let bounds = Aabb3 {
        min: Vec3::splat(-0.5),
        max: Vec3::splat(0.5),
    };
    let c = bounds.corners();
    let w = c.map(|p| m.transform_point3(p));
    let edges: [(usize, usize); 12] = [
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
    ];

    let width = Px(1.5);
    let style = PathStyle::StrokeV2(StrokeStyleV2 {
        width,
        join: StrokeJoinV1::Round,
        cap: StrokeCapV1::Round,
        ..Default::default()
    });

    let scope = painter.key_scope(&"cookbook.gizmo_basics.cube");
    for (i, (a, b)) in edges.iter().enumerate() {
        let Some(pa) = project_point(view_projection, viewport, w[*a], depth_range) else {
            continue;
        };
        let Some(pb) = project_point(view_projection, viewport, w[*b], depth_range) else {
            continue;
        };
        let a = Point::new(
            Px(pa.screen.x / scale_factor),
            Px(pa.screen.y / scale_factor),
        );
        let b = Point::new(
            Px(pb.screen.x / scale_factor),
            Px(pb.screen.y / scale_factor),
        );
        let cmds = [PathCommand::MoveTo(a), PathCommand::LineTo(b)];
        let key: u64 = painter.child_key(scope, &i).into();
        painter.path(
            key,
            DrawOrder(10),
            Point::new(Px(0.0), Px(0.0)),
            &cmds,
            style,
            color,
            scale_factor,
        );
    }
}

fn paint_gizmo(
    painter: &mut canvas::AppCanvasPainter<'_, '_>,
    view_projection: Mat4,
    viewport: ViewportRect,
    scale_factor: f32,
    model: &GizmoBasicsModel,
) {
    let targets = gizmo_targets(model.active_target, model.transform);

    let gizmo = Gizmo {
        config: model.gizmo_config,
        state: model.gizmo_state.clone(),
    };

    let draw = gizmo.draw(view_projection, viewport, model.active_target, &targets);

    let thickness = Px((model.gizmo_config.line_thickness_px / scale_factor).max(0.75));
    let stroke = PathStyle::StrokeV2(StrokeStyleV2 {
        width: thickness,
        join: StrokeJoinV1::Round,
        cap: StrokeCapV1::Round,
        ..Default::default()
    });

    let fill = PathStyle::Fill(Default::default());

    let scope_lines = painter.key_scope(&"cookbook.gizmo_basics.gizmo.lines");
    let scope_tris = painter.key_scope(&"cookbook.gizmo_basics.gizmo.tris");

    let project = |world: Vec3| -> Option<Point> {
        let p = project_point(
            view_projection,
            viewport,
            world,
            model.gizmo_config.depth_range,
        )?;
        Some(Point::new(
            Px(p.screen.x / scale_factor),
            Px(p.screen.y / scale_factor),
        ))
    };

    for (i, line) in draw.lines.iter().enumerate() {
        let Some(a) = project(line.a) else { continue };
        let Some(b) = project(line.b) else { continue };

        let (order, c) = match line.depth {
            DepthMode::Ghost => (DrawOrder(20), color_mul_alpha(line.color, 0.25)),
            DepthMode::Test => (DrawOrder(30), line.color),
            DepthMode::Always => (DrawOrder(40), line.color),
        };

        let cmds = [PathCommand::MoveTo(a), PathCommand::LineTo(b)];
        let key: u64 = painter.child_key(scope_lines, &i).into();
        painter.path(
            key,
            order,
            Point::new(Px(0.0), Px(0.0)),
            &cmds,
            stroke,
            c,
            scale_factor,
        );
    }

    for (i, tri) in draw.triangles.iter().enumerate() {
        let Some(a) = project(tri.a) else { continue };
        let Some(b) = project(tri.b) else { continue };
        let Some(c2) = project(tri.c) else { continue };

        let (order, c) = match tri.depth {
            DepthMode::Ghost => (DrawOrder(21), color_mul_alpha(tri.color, 0.25)),
            DepthMode::Test => (DrawOrder(31), tri.color),
            DepthMode::Always => (DrawOrder(41), tri.color),
        };

        let cmds = [
            PathCommand::MoveTo(a),
            PathCommand::LineTo(b),
            PathCommand::LineTo(c2),
            PathCommand::Close,
        ];
        let key: u64 = painter.child_key(scope_tris, &i).into();
        painter.path(
            key,
            order,
            Point::new(Px(0.0), Px(0.0)),
            &cmds,
            fill,
            c,
            scale_factor,
        );
    }
}

fn apply_gizmo_input(
    m: &mut GizmoBasicsModel,
    viewport: ViewportRect,
    cursor_px: Vec2,
    drag_started: bool,
    dragging: bool,
) -> bool {
    let view_projection = camera_view_projection(viewport.size, m.camera);
    let mut gizmo = Gizmo {
        config: m.gizmo_config,
        state: std::mem::take(&mut m.gizmo_state),
    };
    let targets = gizmo_targets(m.active_target, m.transform);
    let input = GizmoInput {
        cursor_px,
        hovered: true,
        drag_started,
        dragging,
        snap: m.snap,
        cancel: false,
        precision: 1.0,
    };

    if let Some(update) = gizmo.update(view_projection, viewport, input, m.active_target, &targets)
    {
        if let Some(t) = update
            .updated_targets
            .iter()
            .find(|t| t.id == m.active_target)
        {
            m.transform = t.transform;
        }
    }

    let is_over = gizmo.state.is_over();
    m.gizmo_state = gizmo.state;
    is_over
}

impl View for GizmoBasicsView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        Self {
            model: app.local_state(GizmoBasicsModel::default()),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let model = self.model.paint_value(cx);

        cx.actions().local(&self.model).update::<act::Reset>(|m| {
            m.transform.translation = Vec3::ZERO;
            m.transform.rotation = Quat::IDENTITY;
            m.transform.scale = Vec3::ONE;
            m.gizmo_state = GizmoState::default();
            m.dragging = false;
        });
        cx.actions()
            .local(&self.model)
            .update::<act::ToggleSnap>(|m| {
                m.snap = !m.snap;
            });

        let pos = model.transform.translation;
        let pos_len = pos.length() as f64;

        let snap_label = if model.snap { "Snap: on" } else { "Snap: off" };

        let header = shadcn::card_header(|cx| {
            ui::children![
                cx;
                shadcn::card_title("Gizmo basics"),
                shadcn::card_description(
                    "A minimal editor-style gizmo loop: pointer input -> fret-gizmo update -> app-owned transform -> paint.",
                ),
            ]
        });

        let pos_badges = ui::h_flex(|_cx| {
            let badge = |label: String, test_id: &'static str, value: f64| {
                shadcn::Badge::new(label)
                    .variant(shadcn::BadgeVariant::Secondary)
                    .a11y(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Meter)
                            .test_id(test_id)
                            .numeric_value(value),
                    )
            };

            [
                badge(format!("X: {:.2}", pos.x), TEST_ID_POS_X, pos.x as f64),
                badge(format!("Y: {:.2}", pos.y), TEST_ID_POS_Y, pos.y as f64),
                badge(format!("Z: {:.2}", pos.z), TEST_ID_POS_Z, pos.z as f64),
                badge(
                    format!("|pos|: {:.2}", pos.length()),
                    TEST_ID_POS_LEN,
                    pos_len,
                ),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let toolbar = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::Reset)
                    .test_id(TEST_ID_RESET),
                shadcn::Button::new(snap_label)
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::ToggleSnap)
                    .test_id(TEST_ID_TOGGLE_SNAP),
                pos_badges,
            ]
        })
        .gap(Space::N2)
        .items_center();

        let hint = shadcn::Alert::new(ui::children![
            cx;
            shadcn::AlertTitle::new("Try it"),
            shadcn::AlertDescription::new(
                "Left-drag inside the viewport. Dragging from the center should pick the view-plane translation handle, which is easy to script for regression gates.",
            ),
        ])
        .ui();

        let viewport = {
            let model_down = self.model.clone();
            let on_pointer_down =
                move |cx: &mut pointer::PointerActionCx<'_>, down: pointer::PointerDown| {
                    if down.button != MouseButton::Left {
                        return false;
                    }

                    cx.prevent_focus_on_pointer_down();
                    cx.capture_pointer();
                    cx.set_cursor_icon(CursorIcon::Pointer);

                    let bounds = cx.bounds();
                    let viewport = viewport_rect_from_bounds(bounds, down.pixels_per_point);
                    let cursor_px = Vec2::new(
                        down.position_local.x.0 * down.pixels_per_point,
                        down.position_local.y.0 * down.pixels_per_point,
                    );

                    cx.update_local(&model_down, |m| {
                        apply_gizmo_input(m, viewport, cursor_px, true, true);
                        m.dragging = true;
                    });
                    cx.invalidate_paint();
                    true
                };

            let model_move = self.model.clone();
            let on_pointer_move =
                move |cx: &mut pointer::PointerActionCx<'_>, mv: pointer::PointerMove| {
                    let bounds = cx.bounds();
                    let viewport = viewport_rect_from_bounds(bounds, mv.pixels_per_point);
                    let cursor_px = Vec2::new(
                        mv.position_local.x.0 * mv.pixels_per_point,
                        mv.position_local.y.0 * mv.pixels_per_point,
                    );

                    let mut cursor = CursorIcon::Default;
                    cx.update_local(&model_move, |m| {
                        let is_over = apply_gizmo_input(m, viewport, cursor_px, false, m.dragging);
                        cursor = if is_over {
                            CursorIcon::Pointer
                        } else {
                            CursorIcon::Default
                        };
                    });
                    cx.set_cursor_icon(cursor);
                    cx.invalidate_paint();
                    true
                };

            let model_up = self.model.clone();
            let on_pointer_up = move |cx: &mut pointer::PointerActionCx<'_>,
                                      up: pointer::PointerUp| {
                if up.button != MouseButton::Left {
                    return false;
                }

                cx.release_pointer_capture();
                cx.set_cursor_icon(CursorIcon::Default);

                let bounds = cx.bounds();
                let viewport = viewport_rect_from_bounds(bounds, up.pixels_per_point);
                let cursor_px = Vec2::new(
                    up.position_local.x.0 * up.pixels_per_point,
                    up.position_local.y.0 * up.pixels_per_point,
                );

                cx.update_local(&model_up, |m| {
                    apply_gizmo_input(m, viewport, cursor_px, false, false);
                    m.dragging = false;
                });
                cx.invalidate_paint();
                true
            };

            let model_wheel = self.model.clone();
            let on_wheel = move |cx: &mut pointer::PointerActionCx<'_>, wheel: pointer::Wheel| {
                let dy = wheel.delta.y.0;
                if !dy.is_finite() || dy.abs() < 1e-3 {
                    return false;
                }

                cx.update_local(&model_wheel, |m| {
                    let k = 1.0 + dy * 0.002;
                    let k = k.clamp(0.1, 10.0);
                    m.camera.distance = (m.camera.distance * k).clamp(1.5, 30.0);
                });
                cx.invalidate_paint();
                true
            };

            let region = PointerRegion::new().size_full();
            let paint_model = model.clone();
            cx.pointer_region(region, |cx| {
                cx.on_pointer_down(on_pointer_down);
                cx.on_pointer_move(on_pointer_move);
                cx.on_pointer_up(on_pointer_up);
                cx.on_wheel(on_wheel);

                [canvas::Canvas::new()
                    .size_full()
                    .paint(move |painter| {
                        let theme = painter.theme_snapshot();
                        let bounds = painter.bounds();
                        let sf = painter.scale_factor();
                        let viewport = viewport_rect_from_bounds(bounds, sf);
                        let view_projection =
                            camera_view_projection(viewport.size, paint_model.camera);

                        painter.quad(
                            DrawOrder(0),
                            bounds,
                            CanvasPaint::Solid(theme.color_token("card")),
                            Edges::all(Px(1.0)),
                            CanvasPaint::Solid(theme.color_token("border")),
                            Corners::all(Px(0.0)),
                        );

                        paint_cube_wireframe(
                            painter,
                            view_projection,
                            viewport,
                            paint_model.gizmo_config.depth_range,
                            sf,
                            paint_model.transform,
                            Color::from_srgb_hex_rgb(0x94A3B8),
                        );

                        paint_gizmo(painter, view_projection, viewport, sf, &paint_model);
                    })
                    .test_id(TEST_ID_VIEWPORT)]
            })
        };

        let viewport = ui::container(|_cx| [viewport])
            .w_full()
            .h_full()
            .min_h(Px(480.0));

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                header,
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| ui::children![cx; toolbar, hint, viewport])
                            .gap(Space::N3)
                            .w_full()
                            .h_full()
                            .min_w_0(),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .h_full()
        .max_w(Px(1100.0))
        .a11y_role(SemanticsRole::Group);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    let builder = FretApp::new(ROOT_NAME)
        .window("cookbook-gizmo-basics", (1120.0, 820.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)
        .view::<GizmoBasicsView>()?;

    #[cfg(feature = "cookbook-diag")]
    let builder = builder.with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}

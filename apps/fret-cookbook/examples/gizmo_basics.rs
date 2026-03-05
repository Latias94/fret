use std::sync::Arc;

use fret::prelude::*;
use fret_app::{CommandMeta, CommandScope};
use fret_bootstrap::ui_app_driver::ViewElements;
use fret_core::scene::Paint;
use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, MouseButton, Px, Rect, SemanticsRole,
};
use fret_gizmo::{Aabb3, DepthMode, Gizmo, GizmoConfig, GizmoInput, GizmoState, GizmoTarget3d};
use fret_gizmo::{DepthRange, GizmoTargetId, Transform3d, ViewportRect, project_point};
use fret_runtime::{CommandId, DefaultAction};
use fret_ui::Invalidation;
use fret_ui::action::{OnPointerDown, OnPointerMove, OnPointerUp, OnWheel};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{
    CanvasProps, Length, PointerRegionProps, SemanticsDecoration, SemanticsProps,
};

use glam::{Mat4, Quat, Vec2, Vec3};

const ROOT_NAME: &str = "cookbook-gizmo-basics";

const TEST_ID_ROOT: &str = "cookbook.gizmo_basics.root";
const TEST_ID_VIEWPORT: &str = "cookbook.gizmo_basics.viewport";
const TEST_ID_RESET: &str = "cookbook.gizmo_basics.reset";
const TEST_ID_TOGGLE_SNAP: &str = "cookbook.gizmo_basics.toggle_snap";

const TEST_ID_POS_X: &str = "cookbook.gizmo_basics.pos_x";
const TEST_ID_POS_Y: &str = "cookbook.gizmo_basics.pos_y";
const TEST_ID_POS_Z: &str = "cookbook.gizmo_basics.pos_z";
const TEST_ID_POS_LEN: &str = "cookbook.gizmo_basics.pos_len";

const CMD_RESET: &str = "cookbook.gizmo.reset";
const CMD_TOGGLE_SNAP: &str = "cookbook.gizmo.toggle_snap";

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

struct GizmoBasicsWindowState {
    model: Model<GizmoBasicsModel>,
}

fn install_commands(app: &mut App) {
    let scope = CommandScope::Widget;

    app.commands_mut().register(
        CommandId::from(CMD_RESET),
        CommandMeta::new("Reset gizmo target")
            .with_description("Reset the target transform and gizmo interaction state.")
            .with_category("Gizmo")
            .with_scope(scope),
    );

    app.commands_mut().register(
        CommandId::from(CMD_TOGGLE_SNAP),
        CommandMeta::new("Toggle snapping")
            .with_description("Toggle translation snapping for the active gizmo.")
            .with_category("Gizmo")
            .with_scope(scope),
    );
}

fn init_window(app: &mut App, _window: AppWindowId) -> GizmoBasicsWindowState {
    GizmoBasicsWindowState {
        model: app.models_mut().insert(GizmoBasicsModel::default()),
    }
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
    painter: &mut CanvasPainter<'_>,
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
    let style = fret_core::PathStyle::StrokeV2(fret_core::StrokeStyleV2 {
        width,
        join: fret_core::StrokeJoinV1::Round,
        cap: fret_core::StrokeCapV1::Round,
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
        let a = fret_core::Point::new(
            Px(pa.screen.x / scale_factor),
            Px(pa.screen.y / scale_factor),
        );
        let b = fret_core::Point::new(
            Px(pb.screen.x / scale_factor),
            Px(pb.screen.y / scale_factor),
        );
        let cmds = [
            fret_core::PathCommand::MoveTo(a),
            fret_core::PathCommand::LineTo(b),
        ];
        let key: u64 = painter.child_key(scope, &i).into();
        painter.path(
            key,
            DrawOrder(10),
            fret_core::Point::new(Px(0.0), Px(0.0)),
            &cmds,
            style,
            color,
            scale_factor,
        );
    }
}

fn paint_gizmo(
    painter: &mut CanvasPainter<'_>,
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
    let stroke = fret_core::PathStyle::StrokeV2(fret_core::StrokeStyleV2 {
        width: thickness,
        join: fret_core::StrokeJoinV1::Round,
        cap: fret_core::StrokeCapV1::Round,
        ..Default::default()
    });

    let fill = fret_core::PathStyle::Fill(Default::default());

    let scope_lines = painter.key_scope(&"cookbook.gizmo_basics.gizmo.lines");
    let scope_tris = painter.key_scope(&"cookbook.gizmo_basics.gizmo.tris");

    let project = |world: Vec3| -> Option<fret_core::Point> {
        let p = project_point(
            view_projection,
            viewport,
            world,
            model.gizmo_config.depth_range,
        )?;
        Some(fret_core::Point::new(
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

        let cmds = [
            fret_core::PathCommand::MoveTo(a),
            fret_core::PathCommand::LineTo(b),
        ];
        let key: u64 = painter.child_key(scope_lines, &i).into();
        painter.path(
            key,
            order,
            fret_core::Point::new(Px(0.0), Px(0.0)),
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
            fret_core::PathCommand::MoveTo(a),
            fret_core::PathCommand::LineTo(b),
            fret_core::PathCommand::LineTo(c2),
            fret_core::PathCommand::Close,
        ];
        let key: u64 = painter.child_key(scope_tris, &i).into();
        painter.path(
            key,
            order,
            fret_core::Point::new(Px(0.0), Px(0.0)),
            &cmds,
            fill,
            c,
            scale_factor,
        );
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut GizmoBasicsWindowState) -> ViewElements {
    let model = cx.watch_model(&st.model).paint().cloned_or_default();

    let pos = model.transform.translation;
    let pos_len = pos.length() as f64;

    let snap_label = if model.snap { "Snap: on" } else { "Snap: off" };

    let header = shadcn::CardHeader::new(vec![
        shadcn::CardTitle::new("Gizmo basics").into_element(cx),
        shadcn::CardDescription::new(
            "A minimal editor-style gizmo loop: pointer input -> fret-gizmo update -> app-owned transform -> paint.",
        )
        .into_element(cx),
    ])
    .into_element(cx);

    let pos_badges = ui::h_flex(|cx| {
        let mut badge = |label: String, test_id: &'static str, value: f64| {
            shadcn::Badge::new(label)
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx)
                .attach_semantics(
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
    .items_center()
    .into_element(cx);

    let toolbar = ui::h_flex(|cx| {
        [
            shadcn::Button::new("Reset")
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(CMD_RESET)
                .test_id(TEST_ID_RESET)
                .into_element(cx),
            shadcn::Button::new(snap_label)
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_TOGGLE_SNAP)
                .test_id(TEST_ID_TOGGLE_SNAP)
                .into_element(cx),
            pos_badges,
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let hint = shadcn::Alert::new([
        shadcn::AlertTitle::new("Try it").into_element(cx),
        shadcn::AlertDescription::new(
            "Left-drag inside the viewport. Dragging from the center should pick the view-plane translation handle, which is easy to script for regression gates.",
        )
        .into_element(cx),
    ])
    .ui()
    .into_element(cx);

    let viewport = {
        let model_handle = st.model.clone();

        let on_pointer_down: OnPointerDown = Arc::new(move |host, action_cx, down| {
            if down.button != MouseButton::Left {
                return false;
            }

            host.prevent_default(DefaultAction::FocusOnPointerDown);
            host.capture_pointer();
            host.set_cursor_icon(CursorIcon::Pointer);

            let bounds = host.bounds();
            let viewport = viewport_rect_from_bounds(bounds, down.pixels_per_point);

            let mut should_redraw = false;
            let _ = host.models_mut().update(&model_handle, |m| {
                let view_projection = camera_view_projection(viewport.size, m.camera);
                let cursor_px = Vec2::new(
                    down.position_local.x.0 * down.pixels_per_point,
                    down.position_local.y.0 * down.pixels_per_point,
                );

                let mut gizmo = Gizmo {
                    config: m.gizmo_config,
                    state: std::mem::take(&mut m.gizmo_state),
                };

                let targets = gizmo_targets(m.active_target, m.transform);
                let input = GizmoInput {
                    cursor_px,
                    hovered: true,
                    drag_started: true,
                    dragging: true,
                    snap: m.snap,
                    cancel: false,
                    precision: 1.0,
                };
                if let Some(update) =
                    gizmo.update(view_projection, viewport, input, m.active_target, &targets)
                {
                    if let Some(t) = update
                        .updated_targets
                        .iter()
                        .find(|t| t.id == m.active_target)
                    {
                        m.transform = t.transform;
                    }
                }

                m.gizmo_state = gizmo.state;
                m.dragging = true;
                should_redraw = true;
            });

            if should_redraw {
                host.invalidate(Invalidation::Paint);
                host.request_redraw(action_cx.window);
            }
            true
        });

        let model_handle_move = st.model.clone();
        let on_pointer_move: OnPointerMove = Arc::new(move |host, action_cx, mv| {
            let bounds = host.bounds();
            let viewport = viewport_rect_from_bounds(bounds, mv.pixels_per_point);

            let mut should_redraw = false;
            let mut cursor = CursorIcon::Default;
            let _ = host.models_mut().update(&model_handle_move, |m| {
                let view_projection = camera_view_projection(viewport.size, m.camera);
                let cursor_px = Vec2::new(
                    mv.position_local.x.0 * mv.pixels_per_point,
                    mv.position_local.y.0 * mv.pixels_per_point,
                );

                let mut gizmo = Gizmo {
                    config: m.gizmo_config,
                    state: std::mem::take(&mut m.gizmo_state),
                };
                let targets = gizmo_targets(m.active_target, m.transform);
                let input = GizmoInput {
                    cursor_px,
                    hovered: true,
                    drag_started: false,
                    dragging: m.dragging,
                    snap: m.snap,
                    cancel: false,
                    precision: 1.0,
                };

                if let Some(update) =
                    gizmo.update(view_projection, viewport, input, m.active_target, &targets)
                {
                    if let Some(t) = update
                        .updated_targets
                        .iter()
                        .find(|t| t.id == m.active_target)
                    {
                        m.transform = t.transform;
                    }
                }

                cursor = if gizmo.state.is_over() {
                    CursorIcon::Pointer
                } else {
                    CursorIcon::Default
                };

                m.gizmo_state = gizmo.state;
                should_redraw = true;
            });

            if should_redraw {
                host.set_cursor_icon(cursor);
                host.invalidate(Invalidation::Paint);
                host.request_redraw(action_cx.window);
            }
            true
        });

        let model_handle_up = st.model.clone();
        let on_pointer_up: OnPointerUp = Arc::new(move |host, action_cx, up| {
            if up.button != MouseButton::Left {
                return false;
            }

            host.release_pointer_capture();
            host.set_cursor_icon(CursorIcon::Default);

            let bounds = host.bounds();
            let viewport = viewport_rect_from_bounds(bounds, up.pixels_per_point);

            let mut should_redraw = false;
            let _ = host.models_mut().update(&model_handle_up, |m| {
                let view_projection = camera_view_projection(viewport.size, m.camera);
                let cursor_px = Vec2::new(
                    up.position_local.x.0 * up.pixels_per_point,
                    up.position_local.y.0 * up.pixels_per_point,
                );

                let mut gizmo = Gizmo {
                    config: m.gizmo_config,
                    state: std::mem::take(&mut m.gizmo_state),
                };
                let targets = gizmo_targets(m.active_target, m.transform);
                let input = GizmoInput {
                    cursor_px,
                    hovered: true,
                    drag_started: false,
                    dragging: false,
                    snap: m.snap,
                    cancel: false,
                    precision: 1.0,
                };

                if let Some(update) =
                    gizmo.update(view_projection, viewport, input, m.active_target, &targets)
                {
                    if let Some(t) = update
                        .updated_targets
                        .iter()
                        .find(|t| t.id == m.active_target)
                    {
                        m.transform = t.transform;
                    }
                }

                m.gizmo_state = gizmo.state;
                m.dragging = false;
                should_redraw = true;
            });

            if should_redraw {
                host.invalidate(Invalidation::Paint);
                host.request_redraw(action_cx.window);
            }
            true
        });

        let model_handle_wheel = st.model.clone();
        let on_wheel: OnWheel = Arc::new(move |host, action_cx, wheel| {
            let dy = wheel.delta.y.0;
            if !dy.is_finite() || dy.abs() < 1e-3 {
                return false;
            }

            let _ = host.models_mut().update(&model_handle_wheel, |m| {
                let k = 1.0 + dy * 0.002;
                let k = k.clamp(0.1, 10.0);
                m.camera.distance = (m.camera.distance * k).clamp(1.5, 30.0);
            });

            host.invalidate(Invalidation::Paint);
            host.request_redraw(action_cx.window);
            true
        });

        let mut pointer = PointerRegionProps::default();
        pointer.layout.size.width = Length::Fill;
        pointer.layout.size.height = Length::Fill;

        cx.pointer_region(pointer, |cx| {
            cx.pointer_region_on_pointer_down(on_pointer_down);
            cx.pointer_region_on_pointer_move(on_pointer_move);
            cx.pointer_region_on_pointer_up(on_pointer_up);
            cx.pointer_region_on_wheel(on_wheel);

            let mut canvas = CanvasProps::default();
            canvas.layout.size.width = Length::Fill;
            canvas.layout.size.height = Length::Fill;

            let paint_model = model;
            vec![
                cx.canvas(canvas, move |painter| {
                    let theme = painter.theme().clone();
                    let bounds = painter.bounds();
                    let sf = painter.scale_factor();
                    let viewport = viewport_rect_from_bounds(bounds, sf);
                    let view_projection = camera_view_projection(viewport.size, paint_model.camera);

                    painter.scene().push(fret_core::SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: bounds,
                        background: Paint::Solid(theme.color_token("card")).into(),
                        border: Edges::all(Px(1.0)),
                        border_paint: Paint::Solid(theme.color_token("border")).into(),
                        corner_radii: Corners::all(Px(0.0)),
                    });

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
                .test_id(TEST_ID_VIEWPORT),
            ]
        })
    };

    let viewport = ui::container(|_cx| vec![viewport])
        .w_full()
        .h_full()
        .min_h(Px(480.0))
        .into_element(cx);

    let content = ui::v_flex(|_cx| vec![toolbar, hint, viewport])
        .gap(Space::N3)
        .w_full()
        .h_full()
        .min_w_0()
        .into_element(cx);

    let card = shadcn::Card::new(vec![
        header,
        shadcn::CardContent::new(vec![content]).into_element(cx),
    ])
    .ui()
    .w_full()
    .h_full()
    .max_w(Px(1100.0))
    .into_element(cx);

    let root = fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card);

    vec![cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Group,
            test_id: None,
            ..Default::default()
        },
        |_cx| vec![root],
    )]
    .into()
}

fn on_command(
    app: &mut App,
    _services: &mut dyn fret_core::UiServices,
    _window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut GizmoBasicsWindowState,
    command: &CommandId,
) {
    let cmd = command.as_str();

    if cmd == CMD_RESET {
        let _ = app.models_mut().update(&st.model, |m| {
            m.transform.translation = Vec3::ZERO;
            m.transform.rotation = Quat::IDENTITY;
            m.transform.scale = Vec3::ONE;
            m.gizmo_state = GizmoState::default();
            m.dragging = false;
        });
        return;
    }

    if cmd == CMD_TOGGLE_SNAP {
        let _ = app.models_mut().update(&st.model, |m| {
            m.snap = !m.snap;
        });
    }
}

fn configure_driver(
    driver: fret_bootstrap::ui_app_driver::UiAppDriver<GizmoBasicsWindowState>,
) -> fret_bootstrap::ui_app_driver::UiAppDriver<GizmoBasicsWindowState> {
    driver.on_command(on_command)
}

fn main() -> anyhow::Result<()> {
    let builder = fret_bootstrap::ui_app_with_hooks(ROOT_NAME, init_window, view, configure_driver)
        .with_main_window("cookbook-gizmo-basics", (1120.0, 820.0))
        .with_command_default_keybindings()
        .install_app(install_commands)
        .install_app(shadcn::install_app)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)
        .with_lucide_icons()
        .with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}

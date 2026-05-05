use std::sync::{Arc, Mutex};

use fret_core::{
    Axis, Color, ColorSpace, Corners, DrawOrder, Edges, FillStyle, GradientStop, MAX_STOPS,
    MouseButton, Paint, PathCommand, PathStyle, Point, Px, StrokeCapV1, StrokeJoinV1, StrokeStyle,
    StrokeStyleV2, SweepGradient, TileMode,
};
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, PressablePointerDownResult, PressablePointerUpResult, UiPointerActionHost,
};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, GridProps,
    GridTrackSizing, LayoutStyle, Length, MainAlign, Overflow, PressableA11y, PressableProps,
    SizeStyle, SpacingLength, StackProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::input_group::derived_test_id;

use super::super::model::{
    HsvColor, HueWheelDragTarget, color_from_rgb_preserving_alpha, format_hex, hsv_from_color,
    hsv_to_color_preserving_alpha, hsv_with_hue_wheel_position, hsv_with_sv_from_local_position,
    hue_from_local_y, hue_percent_text, hue_wheel_geometry, hue_wheel_rotated_triangle,
    hue_wheel_sv_cursor_position, hue_wheel_target_from_local_position, sv_picker_a11y_text,
    unit_from_step,
};
use super::super::{ALPHA_BAR_STEPS, HUE_BAR_STEPS, SV_PICKER_STEPS};
use super::preview::{checkerboard_grid, fill_preview_layout};

const HSV_PICKER_SIZE: Px = Px(120.0);
const HUE_WHEEL_PICKER_WIDTH: Px = Px(138.0);
const HUE_WHEEL_TRIANGLE_STEPS: usize = 12;

pub(super) fn hsv_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    show_alpha_bar: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let sv_test_id = derived_test_id(test_id.as_ref(), "sv");
    let hue_test_id = derived_test_id(test_id.as_ref(), "hue");
    let alpha_test_id = derived_test_id(test_id.as_ref(), "alpha");
    let sv = sv_picker(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        show_alpha,
        enabled,
        sv_test_id,
    );
    let hue = hue_bar(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        show_alpha,
        enabled,
        hue_test_id,
    );
    let alpha = show_alpha_bar
        .then(|| vertical_alpha_bar(cx, current, model, draft, error, enabled, alpha_test_id));

    let mut picker = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |_cx| {
            let mut out = vec![sv, hue];
            if let Some(alpha) = alpha {
                out.push(alpha);
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker
}

pub(super) fn hsv_hue_wheel_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    show_alpha_bar: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let wheel_test_id = derived_test_id(test_id.as_ref(), "wheel");
    let alpha_test_id = derived_test_id(test_id.as_ref(), "alpha");
    let wheel = hue_wheel_picker(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        show_alpha,
        enabled,
        wheel_test_id,
    );
    let alpha = show_alpha_bar
        .then(|| vertical_alpha_bar(cx, current, model, draft, error, enabled, alpha_test_id));

    let mut picker = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |_cx| {
            let mut out = vec![wheel];
            if let Some(alpha) = alpha {
                out.push(alpha);
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker
}

fn hue_wheel_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let hsv = hsv_from_color(current);
    let value = Arc::from(format!(
        "Hue {}%, S {}%, V {}%",
        (hsv.hue.clamp(0.0, 1.0) * 100.0).round() as u8,
        (hsv.saturation.clamp(0.0, 1.0) * 100.0).round() as u8,
        (hsv.value.clamp(0.0, 1.0) * 100.0).round() as u8
    ));

    let drag_target = Arc::new(Mutex::new(None::<HueWheelDragTarget>));
    let target_for_down = Arc::clone(&drag_target);
    let target_for_move = Arc::clone(&drag_target);
    let target_for_up = Arc::clone(&drag_target);

    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    let mut picker = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(HUE_WHEEL_PICKER_WIDTH),
                    height: Length::Px(HSV_PICKER_SIZE),
                    min_width: Some(Length::Px(HUE_WHEEL_PICKER_WIDTH)),
                    min_height: Some(Length::Px(HSV_PICKER_SIZE)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Hue wheel and saturation/value triangle")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }

                let current = host
                    .models_mut()
                    .get_copied(&model_for_down)
                    .unwrap_or(Color::TRANSPARENT);
                let hsv = hsv_from_color(current);
                let bounds = host.bounds();
                let target = hue_wheel_target_from_local_position(
                    hsv,
                    down.position_local.x.0,
                    down.position_local.y.0,
                    bounds.size.width.0,
                    bounds.size.height.0,
                );
                let Some(target) = target else {
                    return PressablePointerDownResult::Continue;
                };

                if let Ok(mut slot) = target_for_down.lock() {
                    *slot = Some(target);
                }
                apply_hue_wheel_position(
                    host,
                    action_cx,
                    &model_for_down,
                    &draft_for_down,
                    &error_for_down,
                    show_alpha,
                    target,
                    down.position_local.x.0,
                    down.position_local.y.0,
                );
                host.capture_pointer();
                PressablePointerDownResult::Continue
            }));

            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                if !mv.buttons.left {
                    if let Ok(mut slot) = target_for_move.lock() {
                        *slot = None;
                    }
                    host.release_pointer_capture();
                    return false;
                }

                let target = target_for_move.lock().ok().and_then(|slot| *slot);
                let Some(target) = target else {
                    return false;
                };
                apply_hue_wheel_position(
                    host,
                    action_cx,
                    &model_for_move,
                    &draft_for_move,
                    &error_for_move,
                    show_alpha,
                    target,
                    mv.position_local.x.0,
                    mv.position_local.y.0,
                );
                true
            }));
            cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
                if let Ok(mut slot) = target_for_up.lock() {
                    *slot = None;
                }
                host.release_pointer_capture();
                PressablePointerUpResult::Continue
            }));

            let (border, ring) = {
                let theme = Theme::global(&*cx.app);
                let border = theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_token("border"));
                let ring = theme
                    .color_by_key("ring")
                    .unwrap_or_else(|| theme.color_token("primary"));
                (border, ring)
            };
            let border_color = if st.focused { ring } else { border };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        overflow: Overflow::Clip,
                        ..fill_preview_layout()
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(5.0)),
                    ..Default::default()
                },
                move |cx| vec![hue_wheel_canvas(cx, hsv)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker.a11y_value(value)
}

fn hue_wheel_canvas<H: UiHost>(cx: &mut ElementContext<'_, H>, hsv: HsvColor) -> AnyElement {
    cx.canvas(
        CanvasProps {
            layout: fill_preview_layout(),
            ..Default::default()
        },
        move |painter| paint_hue_wheel_canvas(painter, hsv),
    )
}

fn paint_hue_wheel_canvas(painter: &mut CanvasPainter<'_>, hsv: HsvColor) {
    let bounds = painter.bounds();
    let geometry = hue_wheel_geometry(bounds.size.width.0, bounds.size.height.0);
    if geometry.wheel_r_outer <= f32::EPSILON || geometry.wheel_thickness <= f32::EPSILON {
        return;
    }

    let scale = painter.scale_factor().max(1.0);
    let origin = bounds.origin;
    let base = painter.key_scope(&"fret-ui-editor.color_edit.hue_wheel");
    paint_hue_wheel_ring(painter, base, origin, geometry, scale);
    paint_hue_wheel_triangle(painter, base, origin, geometry, hsv, scale);
    paint_hue_wheel_cursors(painter, base, origin, geometry, hsv, scale);
}

fn paint_hue_wheel_ring(
    painter: &mut CanvasPainter<'_>,
    base: fret_ui::canvas::CanvasKey,
    origin: Point,
    geometry: super::super::model::HueWheelGeometry,
    scale: f32,
) {
    let center = absolute_point(origin, (geometry.center_x, geometry.center_y));
    let radius = (geometry.wheel_r_inner + geometry.wheel_r_outer) * 0.5;
    let path = circle_path(center, radius);
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, Color::from_srgb_hex_rgb(0xff_00_00));
    stops[1] = GradientStop::new(1.0 / 6.0, Color::from_srgb_hex_rgb(0xff_ff_00));
    stops[2] = GradientStop::new(2.0 / 6.0, Color::from_srgb_hex_rgb(0x00_ff_00));
    stops[3] = GradientStop::new(3.0 / 6.0, Color::from_srgb_hex_rgb(0x00_ff_ff));
    stops[4] = GradientStop::new(4.0 / 6.0, Color::from_srgb_hex_rgb(0x00_00_ff));
    stops[5] = GradientStop::new(5.0 / 6.0, Color::from_srgb_hex_rgb(0xff_00_ff));
    stops[6] = GradientStop::new(1.0, Color::from_srgb_hex_rgb(0xff_00_00));

    painter.path_paint(
        u64::from(painter.child_key(base, &"ring")),
        DrawOrder(0),
        Point::new(Px(0.0), Px(0.0)),
        &path,
        PathStyle::StrokeV2(StrokeStyleV2 {
            width: Px(geometry.wheel_thickness),
            join: StrokeJoinV1::Round,
            cap: StrokeCapV1::Butt,
            ..Default::default()
        }),
        Paint::SweepGradient(SweepGradient {
            center,
            start_angle_turns: 0.0,
            end_angle_turns: 1.0,
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count: 7,
            stops,
        })
        .into(),
        scale,
    );
}

fn paint_hue_wheel_triangle(
    painter: &mut CanvasPainter<'_>,
    base: fret_ui::canvas::CanvasKey,
    origin: Point,
    geometry: super::super::model::HueWheelGeometry,
    hsv: HsvColor,
    scale: f32,
) {
    let triangle = hue_wheel_rotated_triangle(geometry, hsv.hue);
    let mut order = 10u32;
    for i in 0..HUE_WHEEL_TRIANGLE_STEPS {
        for j in 0..(HUE_WHEEL_TRIANGLE_STEPS - i) {
            let p0 = triangle_grid_barycentric(i, j);
            let p1 = triangle_grid_barycentric(i + 1, j);
            let p2 = triangle_grid_barycentric(i, j + 1);
            paint_hue_wheel_triangle_cell(
                painter, base, origin, triangle, hsv.hue, p0, p1, p2, order, scale,
            );
            order += 1;

            if j < HUE_WHEEL_TRIANGLE_STEPS - i - 1 {
                let p3 = triangle_grid_barycentric(i + 1, j + 1);
                paint_hue_wheel_triangle_cell(
                    painter, base, origin, triangle, hsv.hue, p1, p3, p2, order, scale,
                );
                order += 1;
            }
        }
    }

    let border_path = triangle_path(
        absolute_point(origin, triangle.hue),
        absolute_point(origin, triangle.black),
        absolute_point(origin, triangle.white),
    );
    painter.path(
        u64::from(painter.child_key(base, &"triangle.border")),
        DrawOrder(order),
        Point::new(Px(0.0), Px(0.0)),
        &border_path,
        PathStyle::Stroke(StrokeStyle { width: Px(1.5) }),
        Color::from_srgb_hex_rgb(0x80_80_80),
        scale,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_hue_wheel_triangle_cell(
    painter: &mut CanvasPainter<'_>,
    base: fret_ui::canvas::CanvasKey,
    origin: Point,
    triangle: super::super::model::HueWheelTriangle,
    hue: f32,
    a: (f32, f32, f32),
    b: (f32, f32, f32),
    c: (f32, f32, f32),
    order: u32,
    scale: f32,
) {
    let centroid = (
        (a.0 + b.0 + c.0) / 3.0,
        (a.1 + b.1 + c.1) / 3.0,
        (a.2 + b.2 + c.2) / 3.0,
    );
    let value = (1.0 - centroid.1).clamp(0.0, 1.0);
    let saturation = if value <= f32::EPSILON {
        0.0
    } else {
        (centroid.0 / value).clamp(0.0, 1.0)
    };
    let color = hsv_to_color_preserving_alpha(
        HsvColor {
            hue,
            saturation,
            value,
        },
        1.0,
    );
    let path = triangle_path(
        absolute_point(origin, point_from_triangle_barycentric(triangle, a)),
        absolute_point(origin, point_from_triangle_barycentric(triangle, b)),
        absolute_point(origin, point_from_triangle_barycentric(triangle, c)),
    );
    painter.path(
        u64::from(painter.child_key(base, &("triangle.cell", order))),
        DrawOrder(order),
        Point::new(Px(0.0), Px(0.0)),
        &path,
        PathStyle::Fill(FillStyle::default()),
        color,
        scale,
    );
}

fn paint_hue_wheel_cursors(
    painter: &mut CanvasPainter<'_>,
    base: fret_ui::canvas::CanvasKey,
    origin: Point,
    geometry: super::super::model::HueWheelGeometry,
    hsv: HsvColor,
    scale: f32,
) {
    let hue_angle = hsv.hue.rem_euclid(1.0) * std::f32::consts::PI * 2.0;
    let hue_radius = (geometry.wheel_r_inner + geometry.wheel_r_outer) * 0.5;
    let hue_cursor = absolute_point(
        origin,
        (
            geometry.center_x + hue_angle.cos() * hue_radius,
            geometry.center_y + hue_angle.sin() * hue_radius,
        ),
    );
    let hue_color = hsv_to_color_preserving_alpha(
        HsvColor {
            hue: hsv.hue,
            saturation: 1.0,
            value: 1.0,
        },
        1.0,
    );
    paint_cursor_circle(
        painter,
        base,
        "hue.cursor",
        DrawOrder(320),
        hue_cursor,
        geometry.wheel_thickness * 0.55,
        hue_color,
        scale,
    );

    let sv_cursor = absolute_point(
        origin,
        hue_wheel_sv_cursor_position(hsv, geometry.center_x * 2.0, geometry.center_y * 2.0),
    );
    paint_cursor_circle(
        painter,
        base,
        "sv.cursor",
        DrawOrder(324),
        sv_cursor,
        geometry.wheel_thickness * 0.40,
        hsv_to_color_preserving_alpha(hsv, 1.0),
        scale,
    );
}

fn paint_cursor_circle(
    painter: &mut CanvasPainter<'_>,
    base: fret_ui::canvas::CanvasKey,
    key: &'static str,
    order: DrawOrder,
    center: Point,
    radius: f32,
    color: Color,
    scale: f32,
) {
    let fill = circle_path(center, radius.max(1.0));
    painter.path(
        u64::from(painter.child_key(base, &(key, "fill"))),
        order,
        Point::new(Px(0.0), Px(0.0)),
        &fill,
        PathStyle::Fill(FillStyle::default()),
        color,
        scale,
    );
    let outer = circle_path(center, (radius + 1.0).max(1.0));
    painter.path(
        u64::from(painter.child_key(base, &(key, "outer"))),
        DrawOrder(order.0 + 1),
        Point::new(Px(0.0), Px(0.0)),
        &outer,
        PathStyle::Stroke(StrokeStyle { width: Px(1.0) }),
        Color::from_srgb_hex_rgb(0x80_80_80),
        scale,
    );
    let inner = circle_path(center, radius.max(1.0));
    painter.path(
        u64::from(painter.child_key(base, &(key, "inner"))),
        DrawOrder(order.0 + 2),
        Point::new(Px(0.0), Px(0.0)),
        &inner,
        PathStyle::Stroke(StrokeStyle { width: Px(1.0) }),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        scale,
    );
}

fn triangle_grid_barycentric(i: usize, j: usize) -> (f32, f32, f32) {
    let n = HUE_WHEEL_TRIANGLE_STEPS as f32;
    let u = i as f32 / n;
    let v = j as f32 / n;
    (u, v, (1.0 - u - v).max(0.0))
}

fn point_from_triangle_barycentric(
    triangle: super::super::model::HueWheelTriangle,
    barycentric: (f32, f32, f32),
) -> (f32, f32) {
    (
        triangle.hue.0 * barycentric.0
            + triangle.black.0 * barycentric.1
            + triangle.white.0 * barycentric.2,
        triangle.hue.1 * barycentric.0
            + triangle.black.1 * barycentric.1
            + triangle.white.1 * barycentric.2,
    )
}

fn absolute_point(origin: Point, local: (f32, f32)) -> Point {
    Point::new(Px(origin.x.0 + local.0), Px(origin.y.0 + local.1))
}

fn triangle_path(a: Point, b: Point, c: Point) -> [PathCommand; 4] {
    [
        PathCommand::MoveTo(a),
        PathCommand::LineTo(b),
        PathCommand::LineTo(c),
        PathCommand::Close,
    ]
}

fn circle_path(center: Point, radius: f32) -> [PathCommand; 6] {
    let r = radius.max(0.0);
    let k = 0.552_284_8_f32 * r;
    let cx = center.x.0;
    let cy = center.y.0;
    [
        PathCommand::MoveTo(Point::new(Px(cx + r), Px(cy))),
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + r), Px(cy + k)),
            ctrl2: Point::new(Px(cx + k), Px(cy + r)),
            to: Point::new(Px(cx), Px(cy + r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - k), Px(cy + r)),
            ctrl2: Point::new(Px(cx - r), Px(cy + k)),
            to: Point::new(Px(cx - r), Px(cy)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - r), Px(cy - k)),
            ctrl2: Point::new(Px(cx - k), Px(cy - r)),
            to: Point::new(Px(cx), Px(cy - r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + k), Px(cy - r)),
            ctrl2: Point::new(Px(cx + r), Px(cy - k)),
            to: Point::new(Px(cx + r), Px(cy)),
        },
        PathCommand::Close,
    ]
}

fn sv_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let hsv = hsv_from_color(current);
    let value = sv_picker_a11y_text(hsv);

    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    let mut picker = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(HSV_PICKER_SIZE),
                    height: Length::Px(HSV_PICKER_SIZE),
                    min_width: Some(Length::Px(HSV_PICKER_SIZE)),
                    min_height: Some(Length::Px(HSV_PICKER_SIZE)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Saturation and value")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }
                apply_sv_picker_position(
                    host,
                    action_cx,
                    &model_for_down,
                    &draft_for_down,
                    &error_for_down,
                    show_alpha,
                    down.position_local.x.0,
                    down.position_local.y.0,
                );
                host.capture_pointer();
                PressablePointerDownResult::Continue
            }));

            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                if !mv.buttons.left {
                    host.release_pointer_capture();
                    return false;
                }
                apply_sv_picker_position(
                    host,
                    action_cx,
                    &model_for_move,
                    &draft_for_move,
                    &error_for_move,
                    show_alpha,
                    mv.position_local.x.0,
                    mv.position_local.y.0,
                );
                true
            }));
            cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
                host.release_pointer_capture();
                PressablePointerUpResult::Continue
            }));

            let (border, ring) = {
                let theme = Theme::global(&*cx.app);
                let border = theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_token("border"));
                let ring = theme
                    .color_by_key("ring")
                    .unwrap_or_else(|| theme.color_token("primary"));
                (border, ring)
            };
            let border_color = if st.focused { ring } else { border };

            vec![cx.container(
                ContainerProps {
                    layout: fill_preview_layout(),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(5.0)),
                    ..Default::default()
                },
                move |cx| vec![sv_picker_preview_stack(cx, hsv)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker.a11y_value(value)
}

fn sv_picker_preview_stack<H: UiHost>(cx: &mut ElementContext<'_, H>, hsv: HsvColor) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                sv_picker_grid(cx, hsv.hue),
                sv_picker_thumb_overlay(cx, hsv.saturation, hsv.value),
            ]
        },
    )
}

fn sv_picker_grid<H: UiHost>(cx: &mut ElementContext<'_, H>, hue: f32) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: SV_PICKER_STEPS as u16,
            rows: Some(SV_PICKER_STEPS as u16),
            template_columns: Some(
                (0..SV_PICKER_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            template_rows: Some(
                (0..SV_PICKER_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        move |cx| {
            (0..SV_PICKER_STEPS * SV_PICKER_STEPS)
                .map(|idx| {
                    let row = idx / SV_PICKER_STEPS;
                    let col = idx % SV_PICKER_STEPS;
                    let saturation = unit_from_step(col, SV_PICKER_STEPS);
                    let value = 1.0 - unit_from_step(row, SV_PICKER_STEPS);
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(hsv_to_color_preserving_alpha(
                                HsvColor {
                                    hue,
                                    saturation,
                                    value,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn sv_picker_thumb_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    saturation: f32,
    value: f32,
) -> AnyElement {
    let left_grow = saturation.clamp(0.0, 1.0);
    let right_grow = (1.0 - left_grow).max(0.0);
    let top_grow = (1.0 - value.clamp(0.0, 1.0)).max(0.0);
    let bottom_grow = value.clamp(0.0, 1.0);

    cx.flex(
        FlexProps {
            layout: fill_preview_layout(),
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                sv_thumb_vertical_spacer(cx, top_grow),
                cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(9.0)),
                                ..Default::default()
                            },
                            flex: FlexItemStyle {
                                grow: 0.0,
                                shrink: 0.0,
                                basis: Length::Px(Px(9.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Horizontal,
                        gap: SpacingLength::Px(Px(0.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![
                            horizontal_bar_thumb_spacer(cx, left_grow),
                            cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Px(Px(9.0)),
                                            height: Length::Px(Px(9.0)),
                                            ..Default::default()
                                        },
                                        flex: FlexItemStyle {
                                            grow: 0.0,
                                            shrink: 0.0,
                                            basis: Length::Px(Px(9.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    background: Some(Color::TRANSPARENT),
                                    border: Edges::all(Px(2.0)),
                                    border_color: Some(Color::from_srgb_hex_rgb(0xff_ff_ff)),
                                    corner_radii: Corners::all(Px(10.0)),
                                    ..Default::default()
                                },
                                |_cx| vec![],
                            ),
                            horizontal_bar_thumb_spacer(cx, right_grow),
                        ]
                    },
                ),
                sv_thumb_vertical_spacer(cx, bottom_grow),
            ]
        },
    )
}

fn sv_thumb_vertical_spacer<H: UiHost>(cx: &mut ElementContext<'_, H>, grow: f32) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow,
                    shrink: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn hue_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let hsv = hsv_from_color(current);
    let value = hue_percent_text(hsv.hue);

    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    let mut bar = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(18.0)),
                    height: Length::Px(HSV_PICKER_SIZE),
                    min_height: Some(Length::Px(HSV_PICKER_SIZE)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Hue")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }
                apply_hue_bar_position(
                    host,
                    action_cx,
                    &model_for_down,
                    &draft_for_down,
                    &error_for_down,
                    show_alpha,
                    down.position_local.y.0,
                );
                host.capture_pointer();
                PressablePointerDownResult::Continue
            }));

            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                if !mv.buttons.left {
                    host.release_pointer_capture();
                    return false;
                }
                apply_hue_bar_position(
                    host,
                    action_cx,
                    &model_for_move,
                    &draft_for_move,
                    &error_for_move,
                    show_alpha,
                    mv.position_local.y.0,
                );
                true
            }));
            cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
                host.release_pointer_capture();
                PressablePointerUpResult::Continue
            }));

            let (border, ring) = {
                let theme = Theme::global(&*cx.app);
                let border = theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_token("border"));
                let ring = theme
                    .color_by_key("ring")
                    .unwrap_or_else(|| theme.color_token("primary"));
                (border, ring)
            };
            let border_color = if st.focused { ring } else { border };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(4.0)),
                    padding: Edges::all(Px(1.0)).into(),
                    ..Default::default()
                },
                move |cx| vec![hue_bar_preview_stack(cx, hsv.hue)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        bar = bar.test_id(test_id);
    }
    bar.a11y_value(value)
}

fn hue_bar_preview_stack<H: UiHost>(cx: &mut ElementContext<'_, H>, hue: f32) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                vertical_hue_gradient_overlay(cx),
                vertical_bar_thumb_overlay(cx, hue),
            ]
        },
    )
}

fn vertical_hue_gradient_overlay<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: 1,
            rows: Some(HUE_BAR_STEPS as u16),
            template_columns: Some(vec![GridTrackSizing::Flex(1.0)]),
            template_rows: Some(
                (0..HUE_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..HUE_BAR_STEPS)
                .map(|idx| {
                    let hue = idx as f32 / HUE_BAR_STEPS as f32;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(hsv_to_color_preserving_alpha(
                                HsvColor {
                                    hue,
                                    saturation: 1.0,
                                    value: 1.0,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn vertical_bar_thumb_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    position: f32,
) -> AnyElement {
    let top_grow = position.clamp(0.0, 1.0);
    let bottom_grow = (1.0 - top_grow).max(0.0);
    cx.flex(
        FlexProps {
            layout: fill_preview_layout(),
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                vertical_bar_thumb_spacer(cx, top_grow),
                cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(3.0)),
                                ..Default::default()
                            },
                            flex: FlexItemStyle {
                                grow: 0.0,
                                shrink: 0.0,
                                basis: Length::Px(Px(3.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        background: Some(Color::from_srgb_hex_rgb(0xff_ff_ff)),
                        border: Edges::all(Px(1.0)),
                        border_color: Some(Color::from_srgb_hex_rgb(0x1f_29_37)),
                        corner_radii: Corners::all(Px(2.0)),
                        ..Default::default()
                    },
                    |_cx| vec![],
                ),
                vertical_bar_thumb_spacer(cx, bottom_grow),
            ]
        },
    )
}

fn vertical_bar_thumb_spacer<H: UiHost>(cx: &mut ElementContext<'_, H>, grow: f32) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow,
                    shrink: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn vertical_alpha_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let rgb = fret_ui_kit::colors::hex_rgb_from_linear(current);
    let alpha = current.a.clamp(0.0, 1.0);
    let value = alpha_percent_text(alpha);

    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    let mut bar = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(18.0)),
                    height: Length::Px(HSV_PICKER_SIZE),
                    min_height: Some(Length::Px(HSV_PICKER_SIZE)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Alpha")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }
                apply_vertical_alpha_bar_position(
                    host,
                    action_cx,
                    &model_for_down,
                    &draft_for_down,
                    &error_for_down,
                    down.position_local.y.0,
                );
                host.capture_pointer();
                PressablePointerDownResult::Continue
            }));

            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                if !mv.buttons.left {
                    host.release_pointer_capture();
                    return false;
                }
                apply_vertical_alpha_bar_position(
                    host,
                    action_cx,
                    &model_for_move,
                    &draft_for_move,
                    &error_for_move,
                    mv.position_local.y.0,
                );
                true
            }));
            cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
                host.release_pointer_capture();
                PressablePointerUpResult::Continue
            }));

            let (border, ring) = {
                let theme = Theme::global(&*cx.app);
                let border = theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_token("border"));
                let ring = theme
                    .color_by_key("ring")
                    .unwrap_or_else(|| theme.color_token("primary"));
                (border, ring)
            };
            let border_color = if st.focused { ring } else { border };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(4.0)),
                    padding: Edges::all(Px(1.0)).into(),
                    ..Default::default()
                },
                move |cx| vec![vertical_alpha_bar_preview_stack(cx, rgb, alpha)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        bar = bar.test_id(test_id);
    }
    bar.a11y_value(value)
}

fn vertical_alpha_bar_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rgb: u32,
    alpha: f32,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                checkerboard_grid(cx),
                vertical_alpha_gradient_overlay(cx, rgb),
                vertical_bar_thumb_overlay(cx, 1.0 - alpha.clamp(0.0, 1.0)),
            ]
        },
    )
}

fn vertical_alpha_gradient_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rgb: u32,
) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: 1,
            rows: Some(ALPHA_BAR_STEPS as u16),
            template_columns: Some(vec![GridTrackSizing::Flex(1.0)]),
            template_rows: Some(
                (0..ALPHA_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..ALPHA_BAR_STEPS)
                .map(|idx| {
                    let alpha = 1.0 - unit_from_step(idx, ALPHA_BAR_STEPS);
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(color_from_rgb_preserving_alpha(rgb, alpha)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

pub(super) fn alpha_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let rgb = fret_ui_kit::colors::hex_rgb_from_linear(current);
    let alpha = current.a.clamp(0.0, 1.0);
    let value = alpha_percent_text(alpha);

    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    let mut bar = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(18.0)),
                    min_height: Some(Length::Px(Px(18.0))),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Alpha")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }
                apply_alpha_bar_position(
                    host,
                    action_cx,
                    &model_for_down,
                    &draft_for_down,
                    &error_for_down,
                    down.position_local.x.0,
                );
                host.capture_pointer();
                PressablePointerDownResult::Continue
            }));

            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                if !mv.buttons.left {
                    host.release_pointer_capture();
                    return false;
                }
                apply_alpha_bar_position(
                    host,
                    action_cx,
                    &model_for_move,
                    &draft_for_move,
                    &error_for_move,
                    mv.position_local.x.0,
                );
                true
            }));
            cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
                host.release_pointer_capture();
                PressablePointerUpResult::Continue
            }));

            let (border, ring) = {
                let theme = Theme::global(&*cx.app);
                let border = theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_token("border"));
                let ring = theme
                    .color_by_key("ring")
                    .unwrap_or_else(|| theme.color_token("primary"));
                (border, ring)
            };
            let border_color = if st.focused { ring } else { border };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(4.0)),
                    padding: Edges::all(Px(1.0)).into(),
                    ..Default::default()
                },
                move |cx| vec![alpha_bar_preview_stack(cx, rgb, alpha)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        bar = bar.test_id(test_id);
    }
    bar.a11y_value(value)
}

fn alpha_bar_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rgb: u32,
    alpha: f32,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                checkerboard_grid(cx),
                alpha_gradient_overlay(cx, rgb),
                horizontal_bar_thumb_overlay(cx, alpha),
            ]
        },
    )
}

fn alpha_gradient_overlay<H: UiHost>(cx: &mut ElementContext<'_, H>, rgb: u32) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: ALPHA_BAR_STEPS as u16,
            rows: Some(1),
            template_columns: Some(
                (0..ALPHA_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            template_rows: Some(vec![GridTrackSizing::Flex(1.0)]),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..ALPHA_BAR_STEPS)
                .map(|idx| {
                    let alpha = (idx + 1) as f32 / ALPHA_BAR_STEPS as f32;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(color_from_rgb_preserving_alpha(rgb, alpha)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn horizontal_bar_thumb_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    position: f32,
) -> AnyElement {
    let left_grow = position.clamp(0.0, 1.0);
    let right_grow = (1.0 - left_grow).max(0.0);
    cx.flex(
        FlexProps {
            layout: fill_preview_layout(),
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                horizontal_bar_thumb_spacer(cx, left_grow),
                cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(Px(3.0)),
                                height: Length::Fill,
                                ..Default::default()
                            },
                            flex: FlexItemStyle {
                                grow: 0.0,
                                shrink: 0.0,
                                basis: Length::Px(Px(3.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        background: Some(Color::from_srgb_hex_rgb(0xff_ff_ff)),
                        border: Edges::all(Px(1.0)),
                        border_color: Some(Color::from_srgb_hex_rgb(0x1f_29_37)),
                        corner_radii: Corners::all(Px(2.0)),
                        ..Default::default()
                    },
                    |_cx| vec![],
                ),
                horizontal_bar_thumb_spacer(cx, right_grow),
            ]
        },
    )
}

fn horizontal_bar_thumb_spacer<H: UiHost>(cx: &mut ElementContext<'_, H>, grow: f32) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Fill,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow,
                    shrink: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn apply_alpha_bar_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    x: f32,
) {
    let width = host.bounds().size.width.0;
    let alpha = alpha_from_local_x(x, width);
    let mut next = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    next.a = alpha;
    let formatted = format_hex(next, true);

    let _ = host.models_mut().update(model, |c| *c = next);
    let _ = host
        .models_mut()
        .update(draft, |s| *s = formatted.as_ref().to_string());
    let _ = host.models_mut().update(error, |e| *e = None);
    host.request_redraw(action_cx.window);
}

fn apply_vertical_alpha_bar_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    y: f32,
) {
    let height = host.bounds().size.height.0;
    let alpha = alpha_from_local_y(y, height);
    let mut next = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    next.a = alpha;
    let formatted = format_hex(next, true);

    let _ = host.models_mut().update(model, |c| *c = next);
    let _ = host
        .models_mut()
        .update(draft, |s| *s = formatted.as_ref().to_string());
    let _ = host.models_mut().update(error, |e| *e = None);
    host.request_redraw(action_cx.window);
}

pub(in crate::controls::color_edit) fn alpha_from_local_x(x: f32, width: f32) -> f32 {
    if !width.is_finite() || width <= f32::EPSILON {
        return 0.0;
    }
    (x / width).clamp(0.0, 1.0)
}

pub(in crate::controls::color_edit) fn alpha_from_local_y(y: f32, height: f32) -> f32 {
    if !height.is_finite() || height <= f32::EPSILON {
        return 1.0;
    }
    (1.0 - y / height).clamp(0.0, 1.0)
}

pub(in crate::controls::color_edit) fn alpha_percent_text(alpha: f32) -> Arc<str> {
    Arc::from(format!(
        "{}%",
        (alpha.clamp(0.0, 1.0) * 100.0).round() as u8
    ))
}

fn apply_sv_picker_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    x: f32,
    y: f32,
) {
    let bounds = host.bounds();
    let current = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    let current_hsv = hsv_from_color(current);
    let next_hsv = hsv_with_sv_from_local_position(
        current_hsv,
        x,
        y,
        bounds.size.width.0,
        bounds.size.height.0,
    );
    apply_hsv_color(
        host, action_cx, model, draft, error, show_alpha, current, next_hsv,
    );
}

fn apply_hue_bar_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    y: f32,
) {
    let height = host.bounds().size.height.0;
    let current = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    let mut next_hsv = hsv_from_color(current);
    next_hsv.hue = hue_from_local_y(y, height);
    apply_hsv_color(
        host, action_cx, model, draft, error, show_alpha, current, next_hsv,
    );
}

#[allow(clippy::too_many_arguments)]
fn apply_hue_wheel_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    target: HueWheelDragTarget,
    x: f32,
    y: f32,
) {
    let bounds = host.bounds();
    let current = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    let current_hsv = hsv_from_color(current);
    let next_hsv = hsv_with_hue_wheel_position(
        current_hsv,
        x,
        y,
        bounds.size.width.0,
        bounds.size.height.0,
        target,
    );
    apply_hsv_color(
        host, action_cx, model, draft, error, show_alpha, current, next_hsv,
    );
}

fn apply_hsv_color(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    current: Color,
    next_hsv: HsvColor,
) {
    let next = hsv_to_color_preserving_alpha(next_hsv, current.a);
    let formatted = format_hex(next, show_alpha);

    let _ = host.models_mut().update(model, |c| *c = next);
    let _ = host
        .models_mut()
        .update(draft, |s| *s = formatted.as_ref().to_string());
    let _ = host.models_mut().update(error, |e| *e = None);
    host.request_redraw(action_cx.window);
}

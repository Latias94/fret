use std::sync::{Arc, Mutex};

use fret_core::{Axis, Color, Corners, Edges, MouseButton, Px};
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, PressablePointerDownResult, PressablePointerUpResult, UiPointerActionHost,
};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, GridProps, GridTrackSizing,
    LayoutStyle, Length, MainAlign, Overflow, PressableA11y, PressableProps, SizeStyle,
    SpacingLength, StackProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_border, editor_focus_ring};
use crate::primitives::input_group::derived_test_id;

use super::super::SV_PICKER_STEPS;
use super::super::model::{
    HsvColor, HueWheelDragTarget, format_hex, hsv_from_color, hsv_to_color_preserving_alpha,
    hsv_with_hue_wheel_position, hsv_with_sv_from_local_position,
    hue_wheel_target_from_local_position, sv_picker_a11y_text, unit_from_step,
};
use super::preview::fill_preview_layout;

pub(in crate::controls::color_edit) mod alpha;
mod hue_bar;
mod hue_wheel;

pub(super) use alpha::alpha_bar;
use alpha::vertical_alpha_bar;
use hue_bar::hue_bar;
pub(super) use hue_bar::hue_bar_preview_stack;
pub(super) use hue_wheel::hue_wheel_canvas;

const HSV_PICKER_SIZE: Px = Px(120.0);
const HUE_WHEEL_PICKER_WIDTH: Px = Px(138.0);

fn picker_border_and_ring(theme: &Theme) -> (Color, Color) {
    (editor_border(theme), editor_focus_ring(theme))
}

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

            let theme = Theme::global(&*cx.app);
            let (border, ring) = picker_border_and_ring(theme);
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

            let theme = Theme::global(&*cx.app);
            let (border, ring) = picker_border_and_ring(theme);
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

pub(super) fn sv_picker_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hsv: HsvColor,
) -> AnyElement {
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

use std::sync::Arc;

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

use crate::primitives::input_group::derived_test_id;

use super::super::model::{
    HsvColor, color_from_rgb_preserving_alpha, format_hex, hsv_from_color,
    hsv_to_color_preserving_alpha, hsv_with_sv_from_local_position, hue_from_local_x,
    hue_percent_text, sv_picker_a11y_text, unit_from_step,
};
use super::super::{ALPHA_BAR_STEPS, HUE_BAR_STEPS, SV_PICKER_STEPS};
use super::{checkerboard_grid, fill_preview_layout};

pub(super) fn hsv_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let sv_test_id = derived_test_id(test_id.as_ref(), "sv");
    let hue_test_id = derived_test_id(test_id.as_ref(), "hue");
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
        model,
        draft,
        error,
        show_alpha,
        enabled,
        hue_test_id,
    );

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
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |_cx| vec![sv, hue],
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker
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
                    width: Length::Fill,
                    height: Length::Px(Px(96.0)),
                    min_height: Some(Length::Px(Px(96.0))),
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
                apply_hue_bar_position(
                    host,
                    action_cx,
                    &model_for_move,
                    &draft_for_move,
                    &error_for_move,
                    show_alpha,
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
                hue_gradient_overlay(cx),
                horizontal_bar_thumb_overlay(cx, hue),
            ]
        },
    )
}

fn hue_gradient_overlay<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: HUE_BAR_STEPS as u16,
            rows: Some(1),
            template_columns: Some(
                (0..HUE_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            template_rows: Some(vec![GridTrackSizing::Flex(1.0)]),
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

pub(in crate::controls::color_edit) fn alpha_from_local_x(x: f32, width: f32) -> f32 {
    if !width.is_finite() || width <= f32::EPSILON {
        return 0.0;
    }
    (x / width).clamp(0.0, 1.0)
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
    x: f32,
) {
    let width = host.bounds().size.width.0;
    let current = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    let mut next_hsv = hsv_from_color(current);
    next_hsv.hue = hue_from_local_x(x, width);
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

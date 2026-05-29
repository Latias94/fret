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

use super::super::super::HUE_BAR_STEPS;
use super::super::super::model::{
    HsvColor, hsv_from_color, hsv_to_color_preserving_alpha, hue_from_local_y, hue_percent_text,
};
use super::super::preview::fill_preview_layout;
use super::{HSV_PICKER_SIZE, apply_hsv_color, picker_border_and_ring};

pub(super) fn hue_bar<H: UiHost>(
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

            let theme = Theme::global(&*cx.app);
            let (border, ring) = picker_border_and_ring(theme);
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

pub(in crate::controls::color_edit::popup) fn hue_bar_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hue: f32,
) -> AnyElement {
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

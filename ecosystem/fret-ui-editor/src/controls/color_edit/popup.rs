use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, MouseButton, Px, SemanticsInvalid, TextAlign, TextStyle,
};
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, ActivateReason, OnActivate, OnCloseAutoFocus, PressablePointerDownResult,
    PressablePointerUpResult, UiPointerActionHost,
};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, GridProps, GridTrackSizing,
    InsetStyle, LayoutStyle, Length, MainAlign, Overflow, PointerRegionProps, PositionStyle,
    PressableA11y, PressableProps, SizeStyle, SpacingLength, StackProps, TextInputProps, TextProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, OverlayController, OverlayPresence, OverlayRequest, Size};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;

use super::model::{
    ColorNumericInputMode, HsvColor, color_from_rgb_preserving_alpha, color_numeric_input_modes,
    color_numeric_text, format_hex, hsv_from_color, hsv_numeric_text,
    hsv_to_color_preserving_alpha, hsv_with_sv_from_local_position, hue_from_local_x,
    hue_percent_text, parse_color_numeric_input, rgb_numeric_text, sv_picker_a11y_text,
    unit_from_step,
};
use super::{
    ALPHA_BAR_STEPS, CHECKERBOARD_DARK_RGB, CHECKERBOARD_LIGHT_RGB, COLOR_PRESETS,
    ColorEditPopupNumericInputs, ColorEditPopupOptions, ColorEditPopupPicker, HUE_BAR_STEPS,
    SV_PICKER_STEPS, draft_model, error_model,
};

pub(super) fn request_popup_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    swatch_id: fret_ui::elements::GlobalElementId,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    popup_options: ColorEditPopupOptions,
    popup_padding: Px,
    popup_test_id: Option<Arc<str>>,
) {
    if !popup_options.has_visible_content(show_alpha) {
        return;
    }

    let rgb_draft = draft_model(cx);
    let hsv_draft = draft_model(cx);
    let numeric_error = error_model(cx);
    let overlay_id = cx
        .named("color_edit.popup", |cx| cx.spacer(Default::default()))
        .id;
    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let presence = OverlayPresence::instant(is_open);

    let close_focus: OnCloseAutoFocus = Arc::new(move |host, _cx, req| {
        req.prevent_default();
        host.request_focus(swatch_id);
    });

    let placement = popper::PopperContentPlacement::new(
        popper::LayoutDirection::Ltr,
        Side::Bottom,
        Align::Start,
        Px(4.0),
    )
    .with_collision_padding(Edges::all(Px(8.0)));

    let open_for_content = open.clone();
    let popup = cx.anchored_props(
        fret_ui::element::AnchoredProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            outer_margin: Edges::all(Px(0.0)),
            anchor_element: Some(swatch_id.0),
            side: placement.side,
            align: placement.align,
            side_offset: placement.side_offset,
            options: placement.options(),
            ..Default::default()
        },
        move |cx| {
            let popup_chrome = {
                let theme = Theme::global(&*cx.app);
                resolve_editor_popup_surface_chrome(theme, true)
            };
            let current = cx
                .get_model_copied(&model, Invalidation::Paint)
                .unwrap_or(Color::TRANSPARENT);
            let current_rgb = fret_ui_kit::colors::hex_rgb_from_linear(current);
            let picker = match popup_options.picker {
                ColorEditPopupPicker::HsvHueBar => Some(hsv_picker(
                    cx,
                    current,
                    model.clone(),
                    draft.clone(),
                    error.clone(),
                    show_alpha,
                    enabled,
                    derived_test_id(popup_test_id.as_ref(), "hsv"),
                )),
                ColorEditPopupPicker::Hidden => None,
            };
            let numbers = (popup_options.numeric_inputs != ColorEditPopupNumericInputs::Hidden)
                .then(|| {
                    color_numeric_inputs(
                        cx,
                        current,
                        model.clone(),
                        draft.clone(),
                        rgb_draft.clone(),
                        hsv_draft.clone(),
                        numeric_error.clone(),
                        popup_options.numeric_inputs,
                        show_alpha,
                        enabled,
                        derived_test_id(popup_test_id.as_ref(), "numbers"),
                    )
                });
            let popup_test_id_for_swatches = popup_test_id.clone();
            let model_for_swatches = model.clone();
            let draft_for_swatches = draft.clone();
            let error_for_swatches = error.clone();
            let swatches = popup_options.presets.then(|| {
                cx.flex(
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
                        align: CrossAlign::Center,
                        wrap: true,
                    },
                    move |cx| {
                        COLOR_PRESETS
                            .iter()
                            .enumerate()
                            .map(|(idx, (name, rgb))| {
                                preset_swatch(
                                    cx,
                                    *name,
                                    *rgb,
                                    current_rgb == *rgb,
                                    current.a,
                                    model_for_swatches.clone(),
                                    draft_for_swatches.clone(),
                                    error_for_swatches.clone(),
                                    open_for_content.clone(),
                                    show_alpha,
                                    enabled,
                                    derived_test_id(
                                        popup_test_id_for_swatches.as_ref(),
                                        format!("preset.{idx}").as_str(),
                                    ),
                                )
                            })
                            .collect::<Vec<_>>()
                    },
                )
            });
            let alpha_bar = if popup_options.shows_alpha_bar(show_alpha) {
                Some(alpha_bar(
                    cx,
                    current,
                    model.clone(),
                    draft.clone(),
                    error.clone(),
                    enabled,
                    derived_test_id(popup_test_id.as_ref(), "alpha"),
                ))
            } else {
                None
            };
            let content = cx.flex(
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
                    gap: SpacingLength::Px(Px(8.0)),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                },
                move |_cx| {
                    let mut out = Vec::new();
                    if let Some(picker) = picker {
                        out.push(picker);
                    }
                    if let Some(numbers) = numbers {
                        out.push(numbers);
                    }
                    if let Some(swatches) = swatches {
                        out.push(swatches);
                    }
                    if let Some(alpha_bar) = alpha_bar {
                        out.push(alpha_bar);
                    }
                    out
                },
            );
            let popup = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(Px(216.0)),
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::all(popup_padding).into(),
                    background: Some(popup_chrome.bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(popup_chrome.border),
                    corner_radii: Corners::all(popup_chrome.radius),
                    shadow: popup_chrome.shadow,
                    ..Default::default()
                },
                move |_cx| vec![content],
            );

            let popup = if let Some(test_id) = popup_test_id.as_ref() {
                popup.test_id(test_id.clone())
            } else {
                popup
            };

            vec![popup]
        },
    );

    let mut request = OverlayRequest::dismissible_menu(
        overlay_id,
        swatch_id,
        open,
        presence,
        vec![cx.pointer_region(
            PointerRegionProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                enabled: true,
                capture_phase_pointer_moves: false,
            },
            move |_cx| vec![popup],
        )],
    );
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.on_close_auto_focus = Some(close_focus);

    OverlayController::request(cx, request);
}

pub(super) fn color_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: fill_preview_layout(),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        move |cx| {
            vec![cx.stack_props(
                StackProps {
                    layout: fill_preview_layout(),
                },
                move |cx| {
                    let checkerboard = checkerboard_grid(cx);
                    let overlay = cx.container(
                        ContainerProps {
                            layout: fill_absolute_preview_layout(),
                            background: Some(color),
                            corner_radii: Corners::all(radius),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    );
                    vec![checkerboard, overlay]
                },
            )]
        },
    )
}

fn checkerboard_grid<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: 2,
            rows: Some(2),
            template_columns: Some(vec![GridTrackSizing::Flex(1.0), GridTrackSizing::Flex(1.0)]),
            template_rows: Some(vec![GridTrackSizing::Flex(1.0), GridTrackSizing::Flex(1.0)]),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..4)
                .map(|idx| {
                    let row = idx / 2;
                    let col = idx % 2;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(checkerboard_cell_color(row, col)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn fill_preview_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        overflow: Overflow::Clip,
        ..Default::default()
    }
}

fn fill_absolute_preview_layout() -> LayoutStyle {
    let mut layout = fill_preview_layout();
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        right: Some(Px(0.0)).into(),
        bottom: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
    };
    layout
}

pub(super) fn checkerboard_cell_color(row: usize, col: usize) -> Color {
    let rgb = if (row + col).is_multiple_of(2) {
        CHECKERBOARD_LIGHT_RGB
    } else {
        CHECKERBOARD_DARK_RGB
    };
    Color::from_srgb_hex_rgb(rgb)
}

fn hsv_picker<H: UiHost>(
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

fn color_numeric_inputs<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    hex_draft: Model<String>,
    rgb_draft: Model<String>,
    hsv_draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    numeric_inputs: ColorEditPopupNumericInputs,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let rgb = rgb_numeric_text(current, show_alpha);
    let hsv = hsv_numeric_text(current);
    let error_msg = cx
        .get_model_cloned(&error, Invalidation::Paint)
        .unwrap_or(None);
    let (chrome, text_style, error_color, row_height) = {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);
        let (chrome, text_style) =
            resolve_editor_text_field_style(theme, Size::default(), &ChromeRefinement::default());
        (
            chrome,
            typography::as_control_text(TextStyle {
                size: Px(10.0),
                line_height: Some(density.row_height),
                ..text_style
            }),
            theme.color_token("destructive"),
            density.row_height,
        )
    };
    let rgb_test_id = derived_test_id(test_id.as_ref(), ColorNumericInputMode::Rgb.test_suffix());
    let hsv_test_id = derived_test_id(test_id.as_ref(), ColorNumericInputMode::Hsv.test_suffix());

    let mut inputs = cx.flex(
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
            gap: SpacingLength::Px(Px(2.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            let mut out = Vec::new();
            for mode in color_numeric_input_modes(numeric_inputs) {
                let (draft, display_text, test_id) = match *mode {
                    ColorNumericInputMode::Rgb => {
                        (rgb_draft.clone(), rgb.clone(), rgb_test_id.clone())
                    }
                    ColorNumericInputMode::Hsv => {
                        (hsv_draft.clone(), hsv.clone(), hsv_test_id.clone())
                    }
                };
                out.push(color_numeric_input_field(
                    cx,
                    *mode,
                    model.clone(),
                    hex_draft.clone(),
                    draft,
                    error.clone(),
                    display_text,
                    show_alpha,
                    enabled,
                    chrome.clone(),
                    text_style.clone(),
                    error_msg.is_some(),
                    test_id,
                ));
            }
            if let Some(msg) = error_msg.clone() {
                out.push(color_numeric_error_line(cx, msg, error_color, row_height));
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        inputs = inputs.test_id(test_id);
    }
    inputs
}

fn color_numeric_input_field<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mode: ColorNumericInputMode,
    model: Model<Color>,
    hex_draft: Model<String>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    display_text: Arc<str>,
    show_alpha: bool,
    enabled: bool,
    chrome: fret_ui::TextInputStyle,
    text_style: TextStyle,
    has_error: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut props = TextInputProps::new(draft.clone());
    props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            min_height: Some(Length::Px(row_height_from_style(&text_style))),
            ..Default::default()
        },
        ..Default::default()
    };
    props.enabled = enabled;
    props.focusable = enabled;
    props.test_id = test_id;
    props.placeholder = Some(color_numeric_placeholder(mode, show_alpha));
    props.a11y_label = Some(mode.a11y_label());
    props.a11y_invalid = has_error.then_some(SemanticsInvalid::True);
    props.chrome = chrome;
    props.text_style = text_style;

    let input = cx.text_input(props);
    let input_id = input.id;
    let is_focused = cx.is_focused_element(input_id);

    if !is_focused {
        let _ = cx
            .app
            .models_mut()
            .update(&draft, |s| *s = display_text.as_ref().to_string());
    }

    let model_for_key = model;
    let hex_draft_for_key = hex_draft;
    let draft_for_key = draft;
    let error_for_key = error;
    cx.key_add_on_key_down_capture_for(
        input_id,
        Arc::new(move |host, action_cx: ActionCx, down| match down.key {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                let text = host
                    .models_mut()
                    .read(&draft_for_key, |s| s.clone())
                    .unwrap_or_default();
                let current = host
                    .models_mut()
                    .get_copied(&model_for_key)
                    .unwrap_or(Color::TRANSPARENT);
                if let Some(next) = parse_color_numeric_input(mode, &text, show_alpha, current) {
                    let _ = host.models_mut().update(&model_for_key, |c| *c = next);
                    let formatted = format_hex(next, show_alpha);
                    let numeric = color_numeric_text(next, show_alpha, mode);
                    let _ = host
                        .models_mut()
                        .update(&hex_draft_for_key, |s| *s = formatted.as_ref().to_string());
                    let _ = host
                        .models_mut()
                        .update(&draft_for_key, |s| *s = numeric.as_ref().to_string());
                    let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                } else {
                    let message = mode.invalid_message();
                    let _ = host
                        .models_mut()
                        .update(&error_for_key, |e| *e = Some(message));
                }
                host.request_redraw(action_cx.window);
                true
            }
            KeyCode::Escape => {
                let current = host
                    .models_mut()
                    .get_copied(&model_for_key)
                    .unwrap_or(Color::TRANSPARENT);
                let numeric = color_numeric_text(current, show_alpha, mode);
                let _ = host
                    .models_mut()
                    .update(&draft_for_key, |s| *s = numeric.as_ref().to_string());
                let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                host.request_redraw(action_cx.window);
                true
            }
            _ => false,
        }),
    );

    input
}

fn color_numeric_error_line<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    color: Color,
    row_height: Px,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: Px(10.0),
            line_height: Some(row_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    })
}

fn row_height_from_style(style: &TextStyle) -> Px {
    style.line_height.unwrap_or(style.size)
}

fn color_numeric_placeholder(mode: ColorNumericInputMode, show_alpha: bool) -> Arc<str> {
    match (mode, show_alpha) {
        (ColorNumericInputMode::Rgb, true) => Arc::from("RGB 255 255 255 | A 100%"),
        (ColorNumericInputMode::Rgb, false) => Arc::from("RGB 255 255 255"),
        (ColorNumericInputMode::Hsv, _) => Arc::from("HSV 0deg | S 100% | V 100%"),
    }
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

fn alpha_bar<H: UiHost>(
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

pub(super) fn alpha_from_local_x(x: f32, width: f32) -> f32 {
    if !width.is_finite() || width <= f32::EPSILON {
        return 0.0;
    }
    (x / width).clamp(0.0, 1.0)
}

pub(super) fn alpha_percent_text(alpha: f32) -> Arc<str> {
    Arc::from(format!(
        "{}%",
        (alpha.clamp(0.0, 1.0) * 100.0).round() as u8
    ))
}

fn preset_swatch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    name: &'static str,
    rgb: u32,
    selected: bool,
    current_alpha: f32,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let color = color_from_rgb_preserving_alpha(rgb, current_alpha);
    let on_activate: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let current = host.models_mut().get_copied(&model).unwrap_or(color);
            let color = color_from_rgb_preserving_alpha(rgb, current.a);
            let formatted = format_hex(color, show_alpha);
            let _ = host.models_mut().update(&model, |c| *c = color);
            let _ = host
                .models_mut()
                .update(&draft, |s| *s = formatted.as_ref().to_string());
            let _ = host.models_mut().update(&error, |e| *e = None);
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        });

    let (border_color, ring) = {
        let theme = Theme::global(&*cx.app);
        let ring = theme
            .color_by_key("ring")
            .unwrap_or_else(|| theme.color_token("primary"));
        let border_color = if selected {
            ring
        } else {
            theme
                .color_by_key("border")
                .unwrap_or_else(|| theme.color_token("border"))
        };
        (border_color, ring)
    };

    let mut swatch = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(28.0)),
                    height: Length::Px(Px(28.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Button),
                label: Some(Arc::from(format!("{name} color preset"))),
                ..Default::default()
            },
            focus_ring: Some(fret_ui::element::RingStyle {
                placement: fret_ui::element::RingPlacement::Outset,
                width: Px(2.0),
                offset: Px(1.0),
                color: ring,
                offset_color: None,
                corner_radii: Corners::all(Px(5.0)),
            }),
            ..Default::default()
        },
        move |cx, _st| {
            cx.pressable_add_on_activate(on_activate.clone());
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
                    border: Edges::all(if selected { Px(2.0) } else { Px(1.0) }),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(5.0)),
                    padding: Edges::all(if selected { Px(2.0) } else { Px(1.0) }).into(),
                    ..Default::default()
                },
                move |cx| vec![color_preview_stack(cx, color, Px(5.0))],
            )]
        },
    );

    if let Some(test_id) = test_id {
        swatch = swatch.test_id(test_id);
    }
    swatch.a11y_value(format_hex(color, show_alpha))
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

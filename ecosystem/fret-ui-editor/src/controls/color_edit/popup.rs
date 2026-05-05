use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, OnCloseAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, GridProps, GridTrackSizing, InsetStyle,
    LayoutStyle, Length, MainAlign, Overflow, PointerRegionProps, PositionStyle, PressableA11y,
    PressableProps, SizeStyle, SpacingLength, StackProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use crate::primitives::input_group::derived_test_id;
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;

use super::model::{color_from_rgb_preserving_alpha, format_hex};
use super::{
    CHECKERBOARD_DARK_RGB, CHECKERBOARD_LIGHT_RGB, COLOR_PRESETS, ColorEditPopupNumericInputs,
    ColorEditPopupOptions, ColorEditPopupPicker, draft_model, error_model,
};

mod numeric;
pub(super) mod picker;

use self::numeric::color_numeric_inputs;
use self::picker::{alpha_bar, hsv_picker};

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

pub(super) fn checkerboard_grid<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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

pub(super) fn fill_preview_layout() -> LayoutStyle {
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

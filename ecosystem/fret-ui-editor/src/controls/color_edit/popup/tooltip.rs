use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::DismissReason;
use fret_ui::element::{
    AnchoredProps, AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length,
    MainAlign, SemanticsDecoration, SizeStyle, SpacingLength,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::OverlayPresence;
use fret_ui_kit::primitives::{popper, tooltip as radix_tooltip};

use super::super::model::{format_hex, hsv_numeric_text, rgb_numeric_text};
use super::super::{ColorEditAlphaPreview, ColorEditTooltipOptions};
use super::preview::{color_preview_stack, preview_color_for_alpha_visibility};
use crate::primitives::colors::{editor_foreground, editor_popup_background, editor_popup_border};
use crate::primitives::readout::editor_tooltip_readout_text_props;

pub(in crate::controls::color_edit) fn request_color_tooltip_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    swatch_id: fret_ui::elements::GlobalElementId,
    open: Model<bool>,
    current: Color,
    show_alpha: bool,
    alpha_preview: ColorEditAlphaPreview,
    tooltip_options: ColorEditTooltipOptions,
    test_id: Option<Arc<str>>,
) {
    if !tooltip_options.enabled {
        return;
    }

    let is_open = cx
        .get_model_copied(&open, Invalidation::Paint)
        .unwrap_or(false);
    if !is_open {
        return;
    }

    let tooltip_id = cx
        .named("color_edit.tooltip", |cx| cx.spacer(Default::default()))
        .id;
    let placement = popper::PopperContentPlacement::new(
        popper::LayoutDirection::Ltr,
        Side::Top,
        Align::Center,
        Px(6.0),
    )
    .with_collision_padding(Edges::all(Px(8.0)))
    .with_shift_cross_axis(true);

    let tooltip = cx.anchored_props(
        AnchoredProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            anchor_element: Some(swatch_id.0),
            side: placement.side,
            align: placement.align,
            side_offset: placement.side_offset,
            options: placement.options(),
            ..Default::default()
        },
        move |cx| {
            vec![color_tooltip_panel(
                cx,
                current,
                show_alpha,
                alpha_preview,
                test_id.clone(),
            )]
        },
    );

    let mut request = radix_tooltip::tooltip_request(
        tooltip_id,
        open.clone(),
        OverlayPresence::instant(true),
        vec![tooltip],
    );
    request.trigger = Some(swatch_id);
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.dismissible_on_dismiss_request =
        Some(Arc::new(move |host, action_cx, req| match req.reason {
            DismissReason::Escape | DismissReason::OutsidePress { .. } => {
                let _ = host.models_mut().update(&open, |value| *value = false);
                host.request_redraw(action_cx.window);
            }
            _ => req.prevent_default(),
        }));

    radix_tooltip::request_tooltip(cx, request);
}

pub(in crate::controls::color_edit) fn color_tooltip_lines(
    color: Color,
    show_alpha: bool,
) -> Vec<Arc<str>> {
    vec![
        format_hex(color, show_alpha),
        rgb_numeric_text(color, show_alpha),
        hsv_numeric_text(color),
    ]
}

fn color_tooltip_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    show_alpha: bool,
    alpha_preview: ColorEditAlphaPreview,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let lines = color_tooltip_lines(color, show_alpha);
    let preview_color = preview_color_for_alpha_visibility(color, show_alpha);
    let theme = Theme::global(&*cx.app);
    let bg = editor_popup_background(theme);
    let fg = editor_foreground(theme);
    let border = editor_popup_border(theme);
    let radius = Px(5.0);

    let panel = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(6.0)).into(),
            background: Some(bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(border),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        {
            let lines_for_text = lines.clone();
            move |cx| {
                vec![cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Auto,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Horizontal,
                        gap: SpacingLength::Px(Px(8.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![
                            cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Px(Px(44.0)),
                                            height: Length::Px(Px(44.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    corner_radii: Corners::all(radius),
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![color_preview_stack(
                                        cx,
                                        preview_color,
                                        radius,
                                        alpha_preview,
                                    )]
                                },
                            ),
                            cx.flex(
                                FlexProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Auto,
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
                                    lines_for_text
                                        .iter()
                                        .cloned()
                                        .map(|line| {
                                            cx.text_props(editor_tooltip_readout_text_props(
                                                line, fg,
                                            ))
                                        })
                                        .collect::<Vec<_>>()
                                },
                            ),
                        ]
                    },
                )]
            }
        },
    );

    let semantics_value = lines
        .iter()
        .map(|line| line.as_ref())
        .collect::<Vec<_>>()
        .join(" ");
    let mut semantics = SemanticsDecoration::default()
        .role(SemanticsRole::Tooltip)
        .value(Arc::from(semantics_value));
    if let Some(test_id) = test_id {
        semantics = semantics.test_id(test_id);
    }
    panel.attach_semantics(semantics)
}

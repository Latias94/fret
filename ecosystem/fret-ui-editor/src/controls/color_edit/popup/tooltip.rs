mod panel;

use std::sync::Arc;

use fret_core::{Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnchoredProps, LayoutStyle, Length, SizeStyle};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::OverlayPresence;
use fret_ui_kit::primitives::dismissable_layer::DismissReason;
use fret_ui_kit::primitives::{popper, tooltip as radix_tooltip};

use panel::color_tooltip_panel;

use super::super::model::{format_hex, hsv_numeric_text, rgb_numeric_text};
use super::super::{ColorEditAlphaPreview, ColorEditTooltipOptions};

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

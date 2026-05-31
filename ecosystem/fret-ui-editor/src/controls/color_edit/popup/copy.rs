mod entries;
mod panel;
mod row;

use std::sync::Arc;

use fret_core::{Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::OnCloseAutoFocus;
use fret_ui::element::{AnchoredProps, LayoutStyle, Length, PointerRegionProps, SizeStyle};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use super::super::ColorEditCopyOptions;
#[cfg(test)]
pub(in crate::controls::color_edit) use entries::ColorEditCopyFormat;
pub(in crate::controls::color_edit) use entries::{ColorEditCopyEntry, color_copy_entries};
use panel::color_copy_menu_panel;

pub(in crate::controls::color_edit) fn request_color_copy_menu_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    swatch_id: fret_ui::elements::GlobalElementId,
    open: Model<bool>,
    current: Color,
    show_alpha: bool,
    copy_options: ColorEditCopyOptions,
    test_id: Option<Arc<str>>,
) {
    if !copy_options.enabled {
        return;
    }

    let overlay_id = cx
        .named("color_edit.copy_menu", |cx| cx.spacer(Default::default()))
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
        Side::Right,
        Align::Start,
        Px(4.0),
    )
    .with_collision_padding(Edges::all(Px(8.0)))
    .with_shift_cross_axis(true);

    let open_for_content = open.clone();
    let copy_menu = cx.anchored_props(
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
            vec![color_copy_menu_panel(
                cx,
                open_for_content.clone(),
                current,
                show_alpha,
                test_id.clone(),
            )]
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
            move |_cx| vec![copy_menu],
        )],
    );
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.on_close_auto_focus = Some(close_focus);

    OverlayController::request(cx, request);
}

mod entries;
mod row;

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::OnCloseAutoFocus;
use fret_ui::element::{
    AnchoredProps, AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length,
    MainAlign, PointerRegionProps, SemanticsDecoration, SizeStyle, SpacingLength,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use crate::primitives::EditorDensity;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::popup_list::editor_popup_list_row_gap;
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;

use super::super::ColorEditCopyOptions;
#[cfg(test)]
pub(in crate::controls::color_edit) use entries::ColorEditCopyFormat;
pub(in crate::controls::color_edit) use entries::{ColorEditCopyEntry, color_copy_entries};
use row::color_copy_menu_row;

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

fn color_copy_menu_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    color: Color,
    show_alpha: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let density = {
        let theme = Theme::global(&*cx.app);
        EditorDensity::resolve(theme)
    };
    let popup_chrome = {
        let theme = Theme::global(&*cx.app);
        resolve_editor_popup_surface_chrome(theme, true)
    };
    let entries = Arc::<[ColorEditCopyEntry]>::from(color_copy_entries(color, show_alpha));
    let row_test_id_prefix = test_id.clone();

    let mut panel = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(196.0)),
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(4.0)).into(),
            background: Some(popup_chrome.bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(popup_chrome.border),
            corner_radii: Corners::all(popup_chrome.radius),
            shadow: popup_chrome.shadow,
            ..Default::default()
        },
        {
            let entries = entries.clone();
            let row_test_id_prefix = row_test_id_prefix.clone();
            move |cx| {
                vec![cx.flex(
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
                        gap: SpacingLength::Px(editor_popup_list_row_gap()),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        wrap: false,
                    },
                    {
                        let entries = entries.clone();
                        let row_test_id_prefix = row_test_id_prefix.clone();
                        move |cx| {
                            entries
                                .iter()
                                .map(|entry| {
                                    color_copy_menu_row(
                                        cx,
                                        entry.clone(),
                                        open.clone(),
                                        density.row_height,
                                        derived_test_id(
                                            row_test_id_prefix.as_ref(),
                                            entry.format.test_suffix(),
                                        ),
                                    )
                                })
                                .collect::<Vec<_>>()
                        }
                    },
                )]
            }
        },
    );

    let mut semantics = SemanticsDecoration::default().role(SemanticsRole::Menu);
    if let Some(test_id) = test_id {
        semantics = semantics.test_id(test_id);
    }
    panel = panel.attach_semantics(semantics);
    panel
}

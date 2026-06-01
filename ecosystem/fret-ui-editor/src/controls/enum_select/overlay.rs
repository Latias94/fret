use std::sync::{Arc, Mutex};

use fret_core::{Axis, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    ScrollAxis, ScrollProps, SizeStyle, SpacingLength,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::{active_descendant as active_desc, combobox as kit_combobox, popper};
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use crate::controls::MiniSearchBox;
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::popup_list::{
    editor_popup_list_row_gap, editor_popup_side_offset, editor_popup_window_margin,
};
use crate::primitives::popup_surface::EditorPopupSurfaceChrome;
use crate::primitives::readout::editor_popup_empty_text_props;
use crate::primitives::{EditorDensity, EditorTokenKeys};

use super::{EnumSelectItem, EnumSelectOptions, row};

mod filter;

use filter::filter_enum_select_items;

#[cfg(test)]
mod tests;

pub(super) fn request_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: fret_ui::elements::GlobalElementId,
    model: Model<Option<Arc<str>>>,
    items: Arc<[EnumSelectItem]>,
    open: Model<bool>,
    filter: Model<String>,
    open_change_reason: Model<Option<kit_combobox::ComboboxOpenChangeReason>>,
    focus_restore_target: Arc<Mutex<Option<GlobalElementId>>>,
    options: EnumSelectOptions,
    density: EditorDensity,
    popup_chrome: EditorPopupSurfaceChrome,
) {
    let model_for_list = model.clone();
    let open_for_list = open.clone();
    let open_for_dismiss = open.clone();
    let query_for_list = filter.clone();
    let open_change_reason_for_list = open_change_reason.clone();
    let open_change_reason_for_dismiss = open_change_reason.clone();
    let list_test_id = options.list_test_id.clone();
    let list_viewport_test_id = list_test_id
        .as_ref()
        .map(|test_id| enum_select_viewport_test_id(test_id.as_ref()));
    let item_test_id_prefix = list_test_id.clone();
    let search_test_id = options.search_test_id.clone();
    let scroll_handle = cx.slot_state(ScrollHandle::default, |handle| handle.clone());
    let pending_selected_reveal = cx.local_model_keyed("pending_selected_reveal", || false);

    let overlay_id = cx
        .named("enum_select.overlay", |cx| cx.spacer(Default::default()))
        .id;

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let presence = OverlayPresence::instant(is_open);
    let close_focus = kit_combobox::on_close_auto_focus_with_reason(
        open_change_reason.clone(),
        focus_restore_target,
        enum_select_close_auto_focus_policy(),
    );

    let max_h = {
        let theme = Theme::global(&*cx.app);
        options
            .max_list_height
            .or_else(|| theme.metric_by_key(EditorTokenKeys::ENUM_SELECT_MAX_LIST_HEIGHT))
            .unwrap_or(Px(240.0))
    };

    let filter_text = cx
        .get_model_cloned(&filter, Invalidation::Paint)
        .unwrap_or_default();
    let filtered = filter_enum_select_items(items.as_ref(), filter_text.as_ref());
    let queue_selected_reveal = cx.slot_state(
        || false,
        |was_open| {
            let queue = !*was_open && is_open;
            *was_open = is_open;
            queue
        },
    );
    if queue_selected_reveal {
        let _ = cx
            .app
            .models_mut()
            .update(&pending_selected_reveal, |pending| *pending = true);
    } else if !is_open {
        let _ = cx
            .app
            .models_mut()
            .update(&pending_selected_reveal, |pending| *pending = false);
    }
    let should_reveal_selected = cx
        .get_model_copied(&pending_selected_reveal, Invalidation::Layout)
        .unwrap_or(false);

    let placement = popper::PopperContentPlacement::new(
        popper::LayoutDirection::Ltr,
        Side::Bottom,
        Align::Start,
        editor_popup_side_offset(),
    )
    .with_collision_padding(Edges::all(editor_popup_window_margin()));

    let list = cx.anchored_props(
        fret_ui::element::AnchoredProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            outer_margin: Edges::all(Px(0.0)),
            anchor_element: Some(trigger_id.0),
            side: placement.side,
            align: placement.align,
            side_offset: placement.side_offset,
            options: placement.options(),
            ..Default::default()
        },
        move |cx| {
            let filtered = filtered.clone();
            let panel = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(Px(260.0)),
                            height: Length::Auto,
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    padding: Edges::all(Px(8.0)).into(),
                    background: Some(popup_chrome.bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(popup_chrome.border),
                    corner_radii: Corners::all(popup_chrome.radius),
                    shadow: popup_chrome.shadow,
                    ..Default::default()
                },
                move |cx| {
                    // `Container` does not imply vertical flow layout. Use an explicit column so
                    // the search box and the list do not overlap.
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
                            gap: SpacingLength::Px(Px(6.0)),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();

                            let mut search = MiniSearchBox::new(filter.clone()).into_element(cx);
                            if let Some(test_id) = search_test_id.as_ref() {
                                search = search.test_id(test_id.clone());
                            }
                            out.push(search);

                            let scroll_handle_for_list = scroll_handle.clone();
                            let pending_selected_reveal_for_list = pending_selected_reveal.clone();
                            let selected_row_element_out = Arc::new(Mutex::new(None));
                            let selected_row_element_out_for_rows =
                                selected_row_element_out.clone();
                            let scroll = cx.scroll(
                                ScrollProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    axis: ScrollAxis::Y,
                                    scroll_handle: Some(scroll_handle.clone()),
                                    ..Default::default()
                                },
                                move |cx| {
                                    let filtered = filtered.clone();
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
                                            gap: editor_popup_list_row_gap().into(),
                                            padding: Edges::all(Px(0.0)).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Stretch,
                                            wrap: false,
                                        },
                                        move |cx| {
                                            if filtered.is_empty() {
                                                let theme = Theme::global(&*cx.app);
                                                return vec![cx.text_props(
                                                    editor_popup_empty_text_props(
                                                        Arc::from("No matches"),
                                                        editor_muted_foreground(theme),
                                                        density.row_height,
                                                    ),
                                                )];
                                            }

                                            let item_test_id_prefix = item_test_id_prefix.clone();
                                            let mut rows = Vec::with_capacity(filtered.len());
                                            for (idx, it) in filtered.iter().enumerate() {
                                                let (row, row_id, row_selected) =
                                                    row::enum_select_row(
                                                        cx,
                                                        idx,
                                                        filtered.len(),
                                                        model_for_list.clone(),
                                                        open_for_list.clone(),
                                                        query_for_list.clone(),
                                                        open_change_reason_for_list.clone(),
                                                        it.clone(),
                                                        density,
                                                        item_test_id_prefix.clone(),
                                                    );
                                                if row_selected {
                                                    *selected_row_element_out_for_rows
                                                        .lock()
                                                        .unwrap_or_else(|e| e.into_inner()) =
                                                        Some(row_id);
                                                }
                                                rows.push(row);
                                            }
                                            rows
                                        },
                                    )]
                                },
                            );
                            let viewport = cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Px(max_h),
                                            ..Default::default()
                                        },
                                        overflow: Overflow::Clip,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                move |_cx| vec![scroll],
                            );
                            let viewport = if let Some(test_id) = list_viewport_test_id.as_ref() {
                                viewport.test_id(test_id.clone())
                            } else {
                                viewport
                            };
                            let viewport_id = viewport.id;
                            if should_reveal_selected {
                                let selected_row_element = *selected_row_element_out
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                if let Some(selected_row_element) = selected_row_element {
                                    let did_reveal = active_desc::scroll_active_element_into_view_y(
                                        cx,
                                        &scroll_handle_for_list,
                                        viewport_id,
                                        selected_row_element,
                                    );
                                    let already_visible = element_visible_within_viewport_y(
                                        cx,
                                        viewport_id,
                                        selected_row_element,
                                    )
                                    .unwrap_or(false);
                                    if did_reveal || already_visible {
                                        let _ =
                                            cx.app.models_mut().update(
                                                &pending_selected_reveal_for_list,
                                                |pending| *pending = false,
                                            );
                                    }
                                } else {
                                    let _ = cx
                                        .app
                                        .models_mut()
                                        .update(&pending_selected_reveal_for_list, |pending| {
                                            *pending = false
                                        });
                                }
                            }
                            out.push(viewport);
                            out
                        },
                    )]
                },
            );

            vec![panel]
        },
    );

    let list = if let Some(test_id) = list_test_id.as_ref() {
        list.test_id(test_id.clone())
    } else {
        list
    };

    // For editor selects, we want menu-like outside press dismissal that does not "click through"
    // (outside press closes the overlay without activating the underlay), but we do not need
    // Radix-style `disableOutsidePointerEvents` occlusion. Keeping occlusion off improves
    // reliability when other layers temporarily hold pointer capture.
    let mut request =
        OverlayRequest::dismissible_popover(overlay_id, trigger_id, open, presence, vec![list]);
    request.consume_outside_pointer_events = true;
    request.disable_outside_pointer_events = false;
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.on_close_auto_focus = Some(close_focus);
    let set_reason_on_dismiss =
        kit_combobox::set_open_change_reason_on_dismiss_request(open_change_reason_for_dismiss);
    request.dismissible_on_dismiss_request =
        Some(Arc::new(move |host, action_cx: ActionCx, req| {
            set_reason_on_dismiss(host, action_cx, req);
            let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
            host.request_redraw(action_cx.window);
        }));

    OverlayController::request(cx, request);
}

fn enum_select_close_auto_focus_policy() -> kit_combobox::ComboboxCloseAutoFocusPolicy {
    kit_combobox::ComboboxCloseAutoFocusPolicy::default()
}

fn enum_select_viewport_test_id(list_test_id: &str) -> Arc<str> {
    Arc::from(format!("{list_test_id}.viewport"))
}

fn element_visible_within_viewport_y<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    viewport_element: GlobalElementId,
    child_element: GlobalElementId,
) -> Option<bool> {
    let viewport = cx.last_bounds_for_element(viewport_element)?;
    let child = cx.last_bounds_for_element(child_element)?;
    Some(rect_visible_within_viewport_y(viewport, child))
}

fn rect_visible_within_viewport_y(viewport: fret_core::Rect, child: fret_core::Rect) -> bool {
    let viewport_h = viewport.size.height.0.max(0.0);
    if viewport_h <= 0.0 {
        return false;
    }

    let view_top = viewport.origin.y.0;
    let view_bottom = view_top + viewport_h;
    let child_top = child.origin.y.0;
    let child_h = child.size.height.0.max(0.0);
    let child_bottom = child_top + child_h;

    if child_h >= viewport_h - 0.01 {
        child_top >= view_top - 0.01
    } else {
        child_top >= view_top - 0.01 && child_bottom <= view_bottom + 0.01
    }
}

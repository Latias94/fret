//! Enum-select overlay list viewport owner.

use std::sync::{Arc, Mutex};

use fret_core::{Axis, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    ScrollAxis, ScrollProps, SizeStyle,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::primitives::combobox as kit_combobox;

use super::super::{EnumSelectItem, row};
use super::empty::enum_select_empty_row;
use super::reveal::reveal_selected_row_if_needed;
use crate::primitives::EditorDensity;
use crate::primitives::popup_list::editor_popup_list_row_gap;

pub(in crate::controls::enum_select::overlay) struct EnumSelectListViewportInput {
    pub(in crate::controls::enum_select::overlay) filtered: Arc<[EnumSelectItem]>,
    pub(in crate::controls::enum_select::overlay) max_height: Px,
    pub(in crate::controls::enum_select::overlay) density: EditorDensity,
    pub(in crate::controls::enum_select::overlay) list_viewport_test_id: Option<Arc<str>>,
    pub(in crate::controls::enum_select::overlay) item_test_id_prefix: Option<Arc<str>>,
    pub(in crate::controls::enum_select::overlay) model: Model<Option<Arc<str>>>,
    pub(in crate::controls::enum_select::overlay) open: Model<bool>,
    pub(in crate::controls::enum_select::overlay) query: Model<String>,
    pub(in crate::controls::enum_select::overlay) open_change_reason:
        Model<Option<kit_combobox::ComboboxOpenChangeReason>>,
    pub(in crate::controls::enum_select::overlay) scroll_handle: ScrollHandle,
    pub(in crate::controls::enum_select::overlay) pending_selected_reveal: Model<bool>,
    pub(in crate::controls::enum_select::overlay) should_reveal_selected: bool,
}

pub(in crate::controls::enum_select::overlay) fn enum_select_list_viewport<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: EnumSelectListViewportInput,
) -> AnyElement {
    let EnumSelectListViewportInput {
        filtered,
        max_height,
        density,
        list_viewport_test_id,
        item_test_id_prefix,
        model,
        open,
        query,
        open_change_reason,
        scroll_handle,
        pending_selected_reveal,
        should_reveal_selected,
    } = input;

    let scroll_handle_for_list = scroll_handle.clone();
    let pending_selected_reveal_for_list = pending_selected_reveal.clone();
    let selected_row_element_out = Arc::new(Mutex::new(None));
    let selected_row_element_out_for_rows = selected_row_element_out.clone();
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
                        return vec![enum_select_empty_row(cx, density.row_height)];
                    }

                    let item_test_id_prefix = item_test_id_prefix.clone();
                    let mut rows = Vec::with_capacity(filtered.len());
                    for (idx, it) in filtered.iter().enumerate() {
                        let (row, row_id, row_selected) = row::enum_select_row(
                            cx,
                            idx,
                            filtered.len(),
                            model.clone(),
                            open.clone(),
                            query.clone(),
                            open_change_reason.clone(),
                            it.clone(),
                            density,
                            item_test_id_prefix.clone(),
                        );
                        if row_selected {
                            *selected_row_element_out_for_rows
                                .lock()
                                .unwrap_or_else(|e| e.into_inner()) = Some(row_id);
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
                    height: Length::Px(max_height),
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
        reveal_selected_row_if_needed(
            cx,
            &scroll_handle_for_list,
            viewport_id,
            selected_row_element,
            &pending_selected_reveal_for_list,
        );
    }
    viewport
}

//! Enum-select overlay anchored panel owner.

use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnchoredProps, AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, SizeStyle, SpacingLength,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::primitives::{combobox as kit_combobox, popper};

use crate::controls::MiniSearchBox;
use crate::primitives::EditorDensity;
use crate::primitives::popup_surface::EditorPopupSurfaceChrome;

use super::super::EnumSelectItem;
use super::list::{EnumSelectListViewportInput, enum_select_list_viewport};
use super::reveal::enum_select_viewport_test_id;

pub(in crate::controls::enum_select::overlay) struct EnumSelectOverlayPanelInput {
    pub(in crate::controls::enum_select::overlay) trigger_id: GlobalElementId,
    pub(in crate::controls::enum_select::overlay) placement: popper::PopperContentPlacement,
    pub(in crate::controls::enum_select::overlay) popup_chrome: EditorPopupSurfaceChrome,
    pub(in crate::controls::enum_select::overlay) filtered: Arc<[EnumSelectItem]>,
    pub(in crate::controls::enum_select::overlay) max_height: Px,
    pub(in crate::controls::enum_select::overlay) density: EditorDensity,
    pub(in crate::controls::enum_select::overlay) list_test_id: Option<Arc<str>>,
    pub(in crate::controls::enum_select::overlay) search_test_id: Option<Arc<str>>,
    pub(in crate::controls::enum_select::overlay) model: Model<Option<Arc<str>>>,
    pub(in crate::controls::enum_select::overlay) open: Model<bool>,
    pub(in crate::controls::enum_select::overlay) query: Model<String>,
    pub(in crate::controls::enum_select::overlay) open_change_reason:
        Model<Option<kit_combobox::ComboboxOpenChangeReason>>,
    pub(in crate::controls::enum_select::overlay) scroll_handle: ScrollHandle,
    pub(in crate::controls::enum_select::overlay) pending_selected_reveal: Model<bool>,
    pub(in crate::controls::enum_select::overlay) should_reveal_selected: bool,
}

pub(in crate::controls::enum_select::overlay) fn enum_select_overlay_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: EnumSelectOverlayPanelInput,
) -> AnyElement {
    let EnumSelectOverlayPanelInput {
        trigger_id,
        placement,
        popup_chrome,
        filtered,
        max_height,
        density,
        list_test_id,
        search_test_id,
        model,
        open,
        query,
        open_change_reason,
        scroll_handle,
        pending_selected_reveal,
        should_reveal_selected,
    } = input;

    let list_viewport_test_id = list_test_id
        .as_ref()
        .map(|test_id| enum_select_viewport_test_id(test_id.as_ref()));
    let item_test_id_prefix = list_test_id.clone();

    let list = cx.anchored_props(
        AnchoredProps {
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

                            let mut search = MiniSearchBox::new(query.clone()).into_element(cx);
                            if let Some(test_id) = search_test_id.as_ref() {
                                search = search.test_id(test_id.clone());
                            }
                            out.push(search);

                            out.push(enum_select_list_viewport(
                                cx,
                                EnumSelectListViewportInput {
                                    filtered: filtered.clone(),
                                    max_height,
                                    density,
                                    list_viewport_test_id: list_viewport_test_id.clone(),
                                    item_test_id_prefix: item_test_id_prefix.clone(),
                                    model: model.clone(),
                                    open: open.clone(),
                                    query: query.clone(),
                                    open_change_reason: open_change_reason.clone(),
                                    scroll_handle: scroll_handle.clone(),
                                    pending_selected_reveal: pending_selected_reveal.clone(),
                                    should_reveal_selected,
                                },
                            ));
                            out
                        },
                    )]
                },
            );

            vec![panel]
        },
    );

    if let Some(test_id) = list_test_id.as_ref() {
        list.test_id(test_id.clone())
    } else {
        list
    }
}

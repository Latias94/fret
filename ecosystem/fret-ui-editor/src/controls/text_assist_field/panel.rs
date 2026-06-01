//! Text-assist suggestion panel rendering owner.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow, ScrollAxis,
    ScrollProps, SizeStyle,
};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};
use fret_ui_kit::headless::text_assist::TextAssistController;

mod row;

use super::model::RenderedTextAssistPanel;
use super::{
    OnTextAssistFieldAccept, TextAssistFieldOptions, TextAssistFieldSurface,
    text_assist_max_content_height,
};
use crate::primitives::popup_list::{
    editor_popup_list_content_height, editor_popup_list_row_gap, editor_popup_list_surface_padding,
};
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;
use crate::primitives::style::EditorStyle;
use row::{TextAssistOptionRowInput, text_assist_option_row};

pub(super) fn render_text_assist_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controller: &TextAssistController,
    expanded: bool,
    options: &TextAssistFieldOptions,
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    active_item_id_model: Model<Option<Arc<str>>>,
    on_accept: Option<OnTextAssistFieldAccept>,
) -> Option<RenderedTextAssistPanel> {
    if !expanded {
        return None;
    }

    let is_overlay_surface = matches!(options.surface, TextAssistFieldSurface::AnchoredOverlay);
    let (density, popup_chrome) = {
        let theme = Theme::global(&*cx.app);
        let style = EditorStyle::resolve(theme);
        (
            style.density,
            resolve_editor_popup_surface_chrome(theme, is_overlay_surface),
        )
    };

    let content_height =
        editor_popup_list_content_height(density.row_height, controller.visible().len());
    let max_content_height = text_assist_max_content_height(
        options.surface,
        options.max_list_height,
        density.row_height,
    );
    let viewport_height = max_content_height
        .map(|max_height| Px(content_height.0.min(max_height.0)))
        .unwrap_or(content_height);
    let surface_height = Px(viewport_height.0 + editor_popup_list_surface_padding().0 * 2.0);
    let item_test_id_prefix = options
        .item_test_id_prefix
        .clone()
        .or_else(|| options.list_test_id.clone());

    let mut option_elements = Vec::new();
    let total = controller.visible().len();
    let option_rows: Vec<_> = controller
        .visible()
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_active = controller
                .active_item_id()
                .is_some_and(|active| active == &entry.item_id);

            let (row, option_id) = text_assist_option_row(
                cx,
                TextAssistOptionRowInput {
                    index: idx,
                    total,
                    entry: entry.clone(),
                    is_active,
                    item_test_id_prefix: item_test_id_prefix.clone(),
                    row_height: density.row_height,
                    padding_x: density.padding_x,
                    query_model: query_model.clone(),
                    dismissed_query_model: dismissed_query_model.clone(),
                    active_item_id_model: active_item_id_model.clone(),
                    on_accept: on_accept.clone(),
                },
            );
            option_elements.push(option_id);
            row
        })
        .collect();

    let listbox_id_out = Rc::new(Cell::new(None::<GlobalElementId>));
    let listbox_label = options.list_label.clone();
    let list_test_id = options.list_test_id.clone();
    let panel_layout = if is_overlay_surface {
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            },
            overflow: Overflow::Clip,
            ..Default::default()
        }
    } else {
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            overflow: Overflow::Clip,
            ..Default::default()
        }
    };
    let panel = {
        let listbox_id_out = listbox_id_out.clone();
        cx.semantics_with_id(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::ListBox,
                label: Some(listbox_label),
                test_id: list_test_id,
                ..Default::default()
            },
            move |cx, id| {
                listbox_id_out.set(Some(id));

                let list_content = cx.flex(
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
                    move |_cx| option_rows,
                );

                let body = if viewport_height != content_height {
                    cx.scroll(
                        ScrollProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Px(viewport_height),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            axis: ScrollAxis::Y,
                            ..Default::default()
                        },
                        move |_cx| vec![list_content],
                    )
                } else {
                    list_content
                };

                vec![cx.container(
                    ContainerProps {
                        layout: panel_layout,
                        padding: Edges::all(editor_popup_list_surface_padding()).into(),
                        background: Some(popup_chrome.bg),
                        border: Edges::all(Px(1.0)),
                        border_color: Some(popup_chrome.border),
                        corner_radii: Corners::all(popup_chrome.radius),
                        shadow: popup_chrome.shadow,
                        ..Default::default()
                    },
                    move |_cx| vec![body],
                )]
            },
        )
    };

    Some(RenderedTextAssistPanel {
        panel,
        listbox_id: listbox_id_out.get(),
        option_elements,
        surface_height,
    })
}

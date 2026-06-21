use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::readout::editor_inspector_panel_title_text_props;

use super::super::InspectorPanelSearchAssistOptions;
use super::search::inspector_panel_search_element;

pub(super) struct InspectorPanelHeaderInput {
    pub(super) title: Option<Arc<str>>,
    pub(super) toolbar: Vec<AnyElement>,
    pub(super) search: Option<Model<String>>,
    pub(super) enabled: bool,
    pub(super) search_test_id: Option<Arc<str>>,
    pub(super) search_clear_test_id: Option<Arc<str>>,
    pub(super) search_assist: Option<InspectorPanelSearchAssistOptions>,
    pub(super) toolbar_test_id: Option<Arc<str>>,
    pub(super) header_test_id: Option<Arc<str>>,
    pub(super) density: EditorDensity,
    pub(super) header_gap: Px,
    pub(super) header_bg: Color,
    pub(super) header_border: Color,
    pub(super) radius: Px,
    pub(super) header_fg: Color,
}

pub(super) fn inspector_panel_header_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: InspectorPanelHeaderInput,
) -> Option<AnyElement> {
    let InspectorPanelHeaderInput {
        title,
        mut toolbar,
        search,
        enabled,
        search_test_id,
        search_clear_test_id,
        search_assist,
        toolbar_test_id,
        header_test_id,
        density,
        header_gap,
        header_bg,
        header_border,
        radius,
        header_fg,
    } = input;

    if title.is_none() && toolbar.is_empty() && search.is_none() {
        return None;
    }

    let title_only = title.is_some() && toolbar.is_empty() && search.is_none();
    let title_only_row = if title_only {
        Some({
            let title = title
                .clone()
                .expect("title should exist for the title-only branch");
            let mut row = cx.text_props(editor_inspector_panel_title_text_props(
                title,
                header_fg,
                density.row_height,
            ));
            if let Some(test_id) = toolbar_test_id.as_ref() {
                row = row.test_id(test_id.clone());
            }
            row
        })
    } else {
        None
    };

    let mut out = Vec::new();
    if let Some(row) = title_only_row {
        out.push(row);
    } else {
        if let Some(title) = title {
            let mut row = {
                let toolbar = std::mem::take(&mut toolbar);
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
                        wrap: false,
                    },
                    move |cx| {
                        let mut row = Vec::new();
                        row.push(cx.text_props(editor_inspector_panel_title_text_props(
                            title.clone(),
                            header_fg,
                            density.row_height,
                        )));
                        row.extend(toolbar);
                        row
                    },
                )
            };

            if let Some(test_id) = toolbar_test_id.as_ref() {
                row = row.test_id(test_id.clone());
            }

            out.push(row);
        } else if !toolbar.is_empty() {
            let toolbar = std::mem::take(&mut toolbar);
            let mut row = cx.flex(
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
                    justify: MainAlign::End,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| toolbar,
            );

            if let Some(test_id) = toolbar_test_id.as_ref() {
                row = row.test_id(test_id.clone());
            }

            out.push(row);
        }

        if let Some(search) = search {
            let search_el = inspector_panel_search_element(
                cx,
                search,
                enabled,
                search_test_id,
                search_clear_test_id,
                search_assist,
            );

            out.push(search_el);
        }
    }

    let mut header = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges {
                top: Px(density.padding_y.0 + 3.0),
                right: density.padding_x,
                bottom: Px(density.padding_y.0 + 4.0),
                left: density.padding_x,
            }
            .into(),
            background: Some(header_bg),
            corner_radii: Corners {
                top_left: radius,
                top_right: radius,
                bottom_right: Px(0.0),
                bottom_left: Px(0.0),
            },
            border: Edges {
                top: Px(0.0),
                right: Px(0.0),
                bottom: Px(1.0),
                left: Px(0.0),
            },
            border_color: Some(header_border),
            ..Default::default()
        },
        move |cx| {
            if title_only {
                return out;
            }

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
                    gap: SpacingLength::Px(header_gap),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                },
                move |_cx| out,
            )]
        },
    );

    if let Some(test_id) = header_test_id.as_ref() {
        header = header.test_id(test_id.clone());
    }

    Some(header)
}

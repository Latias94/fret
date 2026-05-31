use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::controls::{
    EditorTextCancelBehavior, EditorTextSelectionBehavior, MiniSearchBox, MiniSearchBoxOptions,
    TextAssistField, TextAssistFieldOptions, TextAssistFieldSurface, TextFieldOptions,
};
use crate::primitives::EditorTokenKeys;
use crate::primitives::colors::{
    editor_panel_background, editor_panel_border, editor_panel_header_background,
    editor_panel_header_border, editor_property_header_foreground,
};
use crate::primitives::inspector_layout::InspectorLayoutMetrics;
use crate::primitives::readout::editor_inspector_panel_title_text_props;

use super::{InspectorPanelCx, InspectorPanelOptions, InspectorPanelSearchAssistOptions};

pub(super) fn inspector_panel_element<H, Toolbar, Contents>(
    cx: &mut ElementContext<'_, H>,
    search: Option<Model<String>>,
    options: InspectorPanelOptions,
    toolbar: Toolbar,
    contents: Contents,
) -> AnyElement
where
    H: UiHost,
    Toolbar: FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>,
    Contents: FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>,
{
    cx.scope(|cx| {
        let (
            density,
            gap,
            header_gap,
            padding,
            header_bg,
            header_border,
            panel_bg,
            panel_border,
            radius,
            header_fg,
        ) = {
            let theme = Theme::global(&*cx.app);
            let metrics = InspectorLayoutMetrics::resolve(theme);
            let density = metrics.density;
            let gap = options.gap.unwrap_or(metrics.panel_gap);
            let header_gap = options.header_gap.unwrap_or(metrics.panel_header_gap);
            let padding = options.padding.unwrap_or_else(|| Edges::all(Px(0.0)));
            let header_bg = editor_panel_header_background(theme);
            let header_border = editor_panel_header_border(theme);
            let panel_bg = editor_panel_background(theme);
            let panel_border = editor_panel_border(theme);
            let radius = theme
                .metric_by_key(EditorTokenKeys::PROPERTY_PANEL_RADIUS)
                .unwrap_or_else(|| theme.metric_token("metric.radius.sm"));
            let header_fg = editor_property_header_foreground(theme);
            (
                density,
                gap,
                header_gap,
                padding,
                header_bg,
                header_border,
                panel_bg,
                panel_border,
                radius,
                header_fg,
            )
        };

        let query = search
            .as_ref()
            .and_then(|m| {
                cx.get_model_cloned(m, Invalidation::Layout)
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_default();
        let query_lower = query.to_lowercase();

        let panel_cx = InspectorPanelCx {
            density,
            query: Arc::from(query),
            query_lower: Arc::from(query_lower),
        };

        let title = options.title.clone();

        let mut toolbar = toolbar(cx, &panel_cx);
        let has_header = title.is_some() || !toolbar.is_empty() || search.is_some();

        let header = has_header.then(|| {
            let mut out = Vec::new();
            if let Some(title) = title.clone() {
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

                if let Some(test_id) = options.toolbar_test_id.as_ref() {
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

                if let Some(test_id) = options.toolbar_test_id.as_ref() {
                    row = row.test_id(test_id.clone());
                }

                out.push(row);
            }

            if let Some(search) = search.clone() {
                let search_el = inspector_panel_search_element(
                    cx,
                    search,
                    options.enabled,
                    options.search_test_id.clone(),
                    options.search_clear_test_id.clone(),
                    options.search_assist.clone(),
                );

                out.push(search_el);
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

            if let Some(test_id) = options.header_test_id.as_ref() {
                header = header.test_id(test_id.clone());
            }
            header
        });

        let mut content = cx.flex(
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
                gap: SpacingLength::Px(gap),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |cx| contents(cx, &panel_cx),
        );

        if let Some(test_id) = options.content_test_id.as_ref() {
            content = content.test_id(test_id.clone());
        }

        let mut root = cx.container(
            ContainerProps {
                layout: options.layout,
                padding: padding.into(),
                background: Some(panel_bg),
                border: Edges::all(Px(1.0)),
                border_color: Some(panel_border),
                corner_radii: Corners::all(radius),
                ..Default::default()
            },
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
                        gap: SpacingLength::Px(gap),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        wrap: false,
                    },
                    move |_cx| {
                        let mut out = Vec::new();
                        if let Some(header) = header {
                            out.push(header);
                        }
                        out.push(content);
                        out
                    },
                )]
            },
        );

        if let Some(test_id) = options.test_id.as_ref() {
            root = root.test_id(test_id.clone());
        }

        root
    })
}

fn inspector_panel_search_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    search: Model<String>,
    enabled: bool,
    search_test_id: Option<Arc<str>>,
    search_clear_test_id: Option<Arc<str>>,
    search_assist: Option<InspectorPanelSearchAssistOptions>,
) -> AnyElement {
    if let Some(search_assist) = search_assist {
        return TextAssistField::new(
            search,
            search_assist.dismissed_query_model,
            search_assist.active_item_id_model,
            search_assist.items,
        )
        .options(TextAssistFieldOptions {
            field: TextFieldOptions {
                enabled,
                focusable: enabled,
                placeholder: Some(Arc::from("Search…")),
                clear_button: true,
                buffered: false,
                selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
                cancel_behavior: EditorTextCancelBehavior::Clear,
                test_id: search_test_id,
                clear_test_id: search_clear_test_id,
                ..Default::default()
            },
            surface: TextAssistFieldSurface::AnchoredOverlay,
            list_label: search_assist.list_label,
            empty_label: search_assist.empty_label,
            key_options: search_assist.key_options,
            list_test_id: search_assist.list_test_id,
            item_test_id_prefix: search_assist.item_test_id_prefix,
            empty_test_id: search_assist.empty_test_id,
            max_list_height: search_assist.max_list_height,
        })
        .into_element(cx);
    }

    MiniSearchBox::new(search)
        .options(MiniSearchBoxOptions {
            enabled,
            focusable: enabled,
            test_id: search_test_id,
            clear_test_id: search_clear_test_id,
            ..Default::default()
        })
        .into_element(cx)
}

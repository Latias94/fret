use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::EditorTokenKeys;
use crate::primitives::colors::{
    editor_panel_background, editor_panel_border, editor_panel_header_background,
    editor_panel_header_border, editor_property_header_foreground,
};
use crate::primitives::inspector_layout::InspectorLayoutMetrics;

mod header;
mod search;

use header::{InspectorPanelHeaderInput, inspector_panel_header_element};

use super::{InspectorPanelCx, InspectorPanelOptions};

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

        let toolbar_elements = toolbar(cx, &panel_cx);
        let header = inspector_panel_header_element(
            cx,
            InspectorPanelHeaderInput {
                title: options.title.clone(),
                toolbar: toolbar_elements,
                search: search.clone(),
                enabled: options.enabled,
                search_test_id: options.search_test_id.clone(),
                search_clear_test_id: options.search_clear_test_id.clone(),
                search_assist: options.search_assist.clone(),
                toolbar_test_id: options.toolbar_test_id.clone(),
                header_test_id: options.header_test_id.clone(),
                density,
                header_gap,
                header_bg,
                header_border,
                radius,
                header_fg,
            },
        );

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

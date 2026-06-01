//! PropertyGroup element assembly owner.

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
    editor_panel_background, editor_property_group_border, editor_property_header_background,
    editor_property_header_border, editor_property_header_foreground,
};
use crate::primitives::inspector_layout::InspectorLayoutMetrics;

mod header;

use header::{PropertyGroupHeaderElementOptions, property_group_header_element};

use super::{OnPropertyGroupToggle, PropertyGroupOptions};

pub(super) fn property_group_element<H, HeaderActions, Contents>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    options: PropertyGroupOptions,
    on_toggle: Option<OnPropertyGroupToggle>,
    header_actions: HeaderActions,
    contents: Contents,
) -> AnyElement
where
    H: UiHost,
    HeaderActions: FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
    Contents: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    cx.scope(|cx| {
        let (
            metrics,
            header_height,
            header_bg,
            header_border,
            panel_bg,
            group_border,
            radius,
            header_fg,
        ) = {
            let theme = Theme::global(&*cx.app);
            let metrics = InspectorLayoutMetrics::resolve(theme);
            let header_height = options.header_height.unwrap_or(metrics.group_header_height);
            let header_bg = editor_property_header_background(theme);
            let header_border = editor_property_header_border(theme);
            let panel_bg = editor_panel_background(theme);
            let group_border = editor_property_group_border(theme);
            let radius = theme
                .metric_by_key(EditorTokenKeys::PROPERTY_PANEL_RADIUS)
                .unwrap_or_else(|| theme.metric_token("metric.radius.sm"));
            let header_fg = editor_property_header_foreground(theme);
            (
                metrics,
                header_height,
                header_bg,
                header_border,
                panel_bg,
                group_border,
                radius,
                header_fg,
            )
        };

        let density = metrics.density;
        let gap = options.gap.unwrap_or(metrics.group_content_gap);

        let collapsed_model = options
            .collapsed
            .clone()
            .unwrap_or_else(|| collapsed_model(cx, options.default_collapsed));
        let collapsed = cx
            .get_model_copied(&collapsed_model, Invalidation::Layout)
            .unwrap_or(options.default_collapsed);

        let header = property_group_header_element(
            cx,
            PropertyGroupHeaderElementOptions {
                label: label.clone(),
                enabled: options.enabled,
                collapsible: options.collapsible,
                collapsed,
                collapsed_model: collapsed_model.clone(),
                on_toggle,
                header_height,
                density,
                header_bg,
                header_border,
                radius,
                header_fg,
                test_id: options.header_test_id.clone(),
            },
            header_actions,
        );

        let mut out = Vec::new();
        out.push(header);

        if !collapsed || !options.collapsible {
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
                    padding: Edges {
                        top: Px(density.padding_y.0 + 2.0),
                        right: density.padding_x,
                        bottom: Px(density.padding_y.0 + 4.0),
                        left: density.padding_x,
                    }
                    .into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                },
                contents,
            );
            if let Some(test_id) = options.content_test_id.as_ref() {
                content = content.test_id(test_id.clone());
            }
            out.push(content);
        }

        let mut root = cx.flex(
            FlexProps {
                layout: options.layout,
                direction: Axis::Vertical,
                gap: SpacingLength::Px(Px(0.0)),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| out,
        );

        if let Some(test_id) = options.test_id.as_ref() {
            root = root.test_id(test_id.clone());
        }

        cx.container(
            ContainerProps {
                layout: options.layout,
                padding: Edges::all(Px(0.0)).into(),
                background: Some(panel_bg),
                border: Edges::all(Px(1.0)),
                border_color: Some(group_border),
                corner_radii: Corners::all(radius),
                ..Default::default()
            },
            move |_cx| vec![root],
        )
    })
}

#[track_caller]
fn collapsed_model<H: UiHost>(cx: &mut ElementContext<'_, H>, default: bool) -> Model<bool> {
    cx.local_model(move || default)
}

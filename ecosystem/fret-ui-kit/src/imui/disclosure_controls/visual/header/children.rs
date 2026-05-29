use std::sync::Arc;

use fret_core::{Axis, Color, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacerProps, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

pub(super) struct HeaderChildrenRequest {
    pub(super) label: Arc<str>,
    pub(super) indicator: Option<Arc<str>>,
    pub(super) foreground: Color,
}

pub(super) fn header_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    request: HeaderChildrenRequest,
) -> Vec<AnyElement> {
    let HeaderChildrenRequest {
        label,
        indicator,
        foreground,
    } = request;

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
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(4.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
            ..Default::default()
        },
        move |cx| {
            let mut out = Vec::new();
            out.push(indicator_slot(cx, indicator, foreground));
            out.push(disclosure_label_text(cx, label, foreground));
            out.push(cx.spacer(SpacerProps::default()));
            out
        },
    )]
}

fn indicator_slot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    indicator: Option<Arc<str>>,
    foreground: Color,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(12.0)),
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx| {
            indicator
                .as_ref()
                .map(|indicator| {
                    vec![
                        crate::declarative::text::text_chrome_glyph(cx, indicator.clone())
                            .inherit_foreground(foreground),
                    ]
                })
                .unwrap_or_default()
        },
    )
}

fn disclosure_label_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    foreground: Color,
) -> AnyElement {
    crate::declarative::text::text_list_row_label(cx, label).inherit_foreground(foreground)
}

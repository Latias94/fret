use std::sync::Arc;

use fret_core::{Axis, Color, Px};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacerProps,
    SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

mod indicator;

use indicator::indicator_slot;

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
            vec![
                indicator_slot(cx, indicator, foreground),
                disclosure_label_text(cx, label, foreground),
                cx.spacer(SpacerProps::default()),
            ]
        },
    )]
}

fn disclosure_label_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    foreground: Color,
) -> AnyElement {
    crate::declarative::text::text_list_row_label(cx, label).inherit_foreground(foreground)
}

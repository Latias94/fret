use std::sync::Arc;

use fret_core::{Color, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, UiHost};

pub(super) fn indicator_slot<H: UiHost>(
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

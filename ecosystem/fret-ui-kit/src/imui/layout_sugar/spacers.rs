use fret_core::{Px, Size};
use fret_ui::element::{AnyElement, Length, SemanticsDecoration, SpacerProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::style::MetricFallback;

use super::super::{DummyOptions, SpacingOptions};

pub(in crate::imui) fn dummy_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: Size,
    options: DummyOptions,
) -> AnyElement {
    let mut props = SpacerProps::default();
    props.layout.size.width = Length::Px(size.width);
    props.layout.size.height = Length::Px(size.height);
    props.layout.size.min_width = Some(Length::Px(size.width));
    props.layout.size.min_height = Some(Length::Px(size.height));
    props.layout.flex.grow = 0.0;
    props.layout.flex.shrink = 0.0;
    props.layout.flex.basis = Length::Px(size.height);
    props.min = size.height;

    let mut element = cx.spacer(props);
    if let Some(test_id) = options.test_id {
        element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }
    element
}

pub(in crate::imui) fn spacing_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: SpacingOptions,
) -> AnyElement {
    let size = options.size.unwrap_or_else(|| {
        let theme = Theme::global(&*cx.app);
        Size::new(
            Px(0.0),
            crate::MetricRef::Token {
                key: "component.imui.item_spacing_y_px",
                fallback: MetricFallback::Px(Px(4.0)),
            }
            .resolve(&*theme),
        )
    });
    dummy_element(
        cx,
        size,
        DummyOptions {
            test_id: options.test_id,
        },
    )
}

//! Stable Material recipe test-id helpers.

use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::element::{
    AnyElement, InsetEdge, InsetStyle, LayoutStyle, Length, MarginEdge, PositionStyle,
    SemanticsProps,
};
use fret_ui::elements::ElementContext;

pub(crate) fn part_test_id(base: &Arc<str>, part: &str) -> Arc<str> {
    Arc::from(format!("{base}.{part}"))
}

pub(crate) fn chrome_part_test_id(base: &Arc<str>) -> Arc<str> {
    part_test_id(base, "chrome")
}

pub(crate) fn optional_part_test_id(base: Option<&Arc<str>>, part: &str) -> Option<Arc<str>> {
    base.map(|id| part_test_id(id, part))
}

pub(crate) fn optional_chrome_part_test_id(base: Option<&Arc<str>>) -> Option<Arc<str>> {
    optional_part_test_id(base, "chrome")
}

pub(crate) fn absolute_region_layout(
    left: InsetEdge,
    top: InsetEdge,
    width: Length,
    height: Length,
) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top,
        right: InsetEdge::Auto,
        bottom: InsetEdge::Auto,
        left,
    };
    layout.size.width = width;
    layout.size.height = height;
    layout
}

pub(crate) fn centered_absolute_region_layout(
    center_left: InsetEdge,
    top: InsetEdge,
    width: Px,
    height: Px,
) -> LayoutStyle {
    let mut layout =
        absolute_region_layout(center_left, top, Length::Px(width), Length::Px(height));
    layout.margin.left = MarginEdge::Px(Px(width.0 * -0.5));
    layout
}

pub(crate) fn diagnostic_anchor<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: Arc<str>,
    layout: LayoutStyle,
) -> AnyElement {
    cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Generic,
            test_id: Some(test_id),
            hidden: true,
            focusable: false,
            layout,
            ..Default::default()
        },
        |_cx| Vec::<AnyElement>::new(),
    )
}

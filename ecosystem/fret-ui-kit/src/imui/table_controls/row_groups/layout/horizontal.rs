//! Shared table row-group horizontal flex chrome.

use fret_core::Px;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, MainAlign, SpacingEdges, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

pub(super) fn table_h_flex<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
    gap: crate::MetricRef,
    layout: LayoutStyle,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    cx.flex(
        FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap: SpacingLength::Px(gap.resolve(theme)),
            padding: SpacingEdges::all(SpacingLength::Px(Px(0.0))),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        |_cx| children,
    )
}

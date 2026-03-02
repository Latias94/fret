// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap_x(Space::N4)
            .items_center()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::extras::Ticker::new("AAPL")
                    .price("$199.18")
                    .change("+1.01%")
                    .change_kind(shadcn::extras::TickerChangeKind::Up)
                    .into_element(cx)
                    .test_id("ui-gallery-shadcn-extras-ticker-aapl"),
                shadcn::extras::Ticker::new("TSLA")
                    .price("$187.42")
                    .change("-2.31%")
                    .change_kind(shadcn::extras::TickerChangeKind::Down)
                    .into_element(cx)
                    .test_id("ui-gallery-shadcn-extras-ticker-tsla"),
            ]
        },
    )
    .test_id("ui-gallery-shadcn-extras-ticker-row")
}
// endregion: example


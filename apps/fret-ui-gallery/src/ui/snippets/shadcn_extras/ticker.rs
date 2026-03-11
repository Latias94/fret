pub const SOURCE: &str = include_str!("ticker.rs");

// region: example
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_flex(|cx| {
        vec![
            fret_ui_shadcn::extras::Ticker::new("AAPL")
                .price("$199.18")
                .change("+1.01%")
                .change_kind(fret_ui_shadcn::extras::TickerChangeKind::Up)
                .into_element(cx)
                .test_id("ui-gallery-shadcn-extras-ticker-aapl"),
            fret_ui_shadcn::extras::Ticker::new("TSLA")
                .price("$187.42")
                .change("-2.31%")
                .change_kind(fret_ui_shadcn::extras::TickerChangeKind::Down)
                .into_element(cx)
                .test_id("ui-gallery-shadcn-extras-ticker-tsla"),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-ticker-row")
}
// endregion: example

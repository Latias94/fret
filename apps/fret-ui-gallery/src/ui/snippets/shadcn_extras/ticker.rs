pub const SOURCE: &str = include_str!("ticker.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::h_flex(|cx| {
        vec![
            shadcn::raw::extras::Ticker::new("AAPL")
                .price("$199.18")
                .change("+1.01%")
                .change_kind(shadcn::raw::extras::TickerChangeKind::Up)
                .into_element(cx)
                .test_id("ui-gallery-shadcn-extras-ticker-aapl"),
            shadcn::raw::extras::Ticker::new("TSLA")
                .price("$187.42")
                .change("-2.31%")
                .change_kind(shadcn::raw::extras::TickerChangeKind::Down)
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

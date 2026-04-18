pub const SOURCE: &str = include_str!("h1.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::h1("Taxing Laughter: The Joke Tax Chronicles")
                .into_element(cx),
        ]
    })
    .items_center()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}
// endregion: example

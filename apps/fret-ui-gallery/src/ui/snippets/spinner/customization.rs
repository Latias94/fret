pub const SOURCE: &str = include_str!("customization.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::h_flex(|cx| {
        vec![
            shadcn::Spinner::new().into_element(cx),
            shadcn::Spinner::new()
                .icon(fret_icons::ids::ui::SETTINGS)
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-spinner-customization")
}
// endregion: example

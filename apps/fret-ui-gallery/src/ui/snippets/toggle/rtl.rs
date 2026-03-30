pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::toggle_uncontrolled(cx, false, |cx| {
            ui::children![
                cx;
                shadcn::raw::icon::icon(cx, IconId::new_static("lucide.bookmark")),
                ui::text("إشارة مرجعية")
            ]
        })
        .variant(shadcn::ToggleVariant::Outline)
        .size(shadcn::ToggleSize::Sm)
        .a11y_label("Toggle bookmark")
        .into_element(cx)
    })
    .test_id("ui-gallery-toggle-rtl")
}
// endregion: example

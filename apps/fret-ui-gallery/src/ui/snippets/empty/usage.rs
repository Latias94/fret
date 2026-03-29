pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let icon = icon::icon(cx, IconId::new_static("lucide.folder-code"));

    shadcn::Empty::new([
        shadcn::EmptyHeader::new([
            shadcn::EmptyMedia::new([icon])
                .variant(shadcn::EmptyMediaVariant::Icon)
                .into_element(cx),
            shadcn::EmptyTitle::new("No data").into_element(cx),
            shadcn::EmptyDescription::new("No data found.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::EmptyContent::new([shadcn::Button::new("Add data")
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx)])
        .into_element(cx),
    ])
    .into_element(cx)
}
// endregion: example

pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let icon = shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.folder-search"));

    shadcn::Empty::new([
        shadcn::empty::EmptyHeader::new([
            shadcn::empty::EmptyMedia::new([icon])
                .variant(shadcn::empty::EmptyMediaVariant::Icon)
                .into_element(cx),
            shadcn::empty::EmptyTitle::new("No data").into_element(cx),
            shadcn::empty::EmptyDescription::new("No data found.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::empty::EmptyContent::new([shadcn::Button::new("Add data").into_element(cx)])
            .into_element(cx),
    ])
    .into_element(cx)
}
// endregion: example

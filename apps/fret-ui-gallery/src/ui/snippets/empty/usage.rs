pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let icon = fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.folder-search"));

    shadcn::Empty::new([
        fret_ui_shadcn::empty::EmptyHeader::new([
            fret_ui_shadcn::empty::EmptyMedia::new([icon])
                .variant(fret_ui_shadcn::empty::EmptyMediaVariant::Icon)
                .into_element(cx),
            fret_ui_shadcn::empty::EmptyTitle::new("No data").into_element(cx),
            fret_ui_shadcn::empty::EmptyDescription::new("No data found.").into_element(cx),
        ])
        .into_element(cx),
        fret_ui_shadcn::empty::EmptyContent::new([shadcn::Button::new("Add data").into_element(cx)])
            .into_element(cx),
    ])
    .into_element(cx)
}
// endregion: example

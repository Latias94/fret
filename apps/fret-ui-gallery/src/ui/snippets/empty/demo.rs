pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let icon =
        fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.folder-search"));

    shadcn::Empty::new([
        fret_ui_shadcn::empty::EmptyHeader::new([
            fret_ui_shadcn::empty::EmptyMedia::new([icon])
                .variant(fret_ui_shadcn::empty::EmptyMediaVariant::Icon)
                .into_element(cx),
            fret_ui_shadcn::empty::EmptyTitle::new("No Projects Yet")
                .into_element(cx)
                .test_id("ui-gallery-empty-demo-title"),
            fret_ui_shadcn::empty::EmptyDescription::new(
                "You haven't created any projects yet. Get started by creating your first project.",
            )
            .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-empty-demo-header"),
        fret_ui_shadcn::empty::EmptyContent::new([
            shadcn::Button::new("Create Project").into_element(cx),
            shadcn::Button::new("Import Project")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx),
        shadcn::Button::new("Learn more")
            .variant(shadcn::ButtonVariant::Link)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-demo")
}
// endregion: example

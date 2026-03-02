// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let actions = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Create project").into_element(cx),
                shadcn::Button::new("Import project")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            ]
        },
    );

    let learn_more = shadcn::Button::new("Learn more")
        .variant(shadcn::ButtonVariant::Link)
        .trailing_icon(fret_icons::IconId::new_static("lucide.arrow-right"))
        .into_element(cx);

    shadcn::Empty::new([
        shadcn::empty::EmptyHeader::new([
            shadcn::empty::EmptyMedia::new([shadcn::Spinner::new().into_element(cx)])
                .variant(shadcn::empty::EmptyMediaVariant::Icon)
                .into_element(cx),
            shadcn::empty::EmptyTitle::new("No projects yet").into_element(cx),
            shadcn::empty::EmptyDescription::new(
                "You haven't created any projects yet. Get started by creating your first project.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::empty::EmptyContent::new([actions])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        learn_more,
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-spinner-empty")
}

// endregion: example


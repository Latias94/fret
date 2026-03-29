pub const SOURCE: &str = include_str!("spinner.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let searching = cx.local_model_keyed("searching", String::new);
    let processing = cx.local_model_keyed("processing", String::new);
    let saving = cx.local_model_keyed("saving", String::new);
    let refreshing = cx.local_model_keyed("refreshing", String::new);

    let max_w = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let searching_group = shadcn::InputGroup::new(searching)
        .a11y_label("Searching")
        .placeholder("Searching...")
        .disabled(true)
        .trailing([shadcn::Spinner::new().into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    let processing_group = shadcn::InputGroup::new(processing)
        .a11y_label("Processing")
        .placeholder("Processing...")
        .disabled(true)
        .leading([shadcn::Spinner::new().into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    let saving_group = shadcn::InputGroup::new(saving)
        .a11y_label("Saving changes")
        .placeholder("Saving changes...")
        .disabled(true)
        .trailing([
            shadcn::InputGroupText::new("Saving...").into_element(cx),
            shadcn::Spinner::new().into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    let refreshing_group = shadcn::InputGroup::new(refreshing)
        .a11y_label("Refreshing data")
        .placeholder("Refreshing data...")
        .disabled(true)
        .leading([shadcn::Spinner::new().into_element(cx)])
        .trailing([shadcn::InputGroupText::new("Please wait...").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    ui::v_stack(move |_cx| {
        vec![
            searching_group,
            processing_group,
            saving_group,
            refreshing_group,
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(max_w)
    .into_element(cx)
    .test_id("ui-gallery-input-group-spinner")
}
// endregion: example

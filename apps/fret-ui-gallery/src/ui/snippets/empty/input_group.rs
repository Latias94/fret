pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let query = cx.local_model_keyed("query", String::new);
    let search = shadcn::InputGroup::new(query)
        .a11y_label("Search pages")
        .leading([shadcn::InputGroupText::new("Search").into_element(cx)])
        .trailing([shadcn::InputGroupText::new("/").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
        .test_id("ui-gallery-empty-input-group-search")
        .into_element(cx);

    shadcn::empty(|cx| {
        ui::children![
            cx;
            shadcn::empty_header(|cx| {
                ui::children![
                    cx;
                    shadcn::empty_title("404 - Not Found"),
                    shadcn::empty_description(
                        "The page you are looking for doesn't exist. Try searching below.",
                    ),
                ]
            }),
            shadcn::empty_content(|cx| {
                ui::children![
                    cx;
                    search,
                    shadcn::empty_description("Need help? Contact support."),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-input-group")
}
// endregion: example
